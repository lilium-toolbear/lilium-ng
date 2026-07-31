use chrono::{TimeZone, Utc};
use lilium_api_client::clearance::{
    ClearanceAgentClient, ClearanceProvider, ClearanceRefreshReason, ClearanceSnapshot,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

fn snapshot_json(expires_at: &str) -> serde_json::Value {
    serde_json::json!({
        "generation": 7,
        "user_agent": "Mozilla/5.0 exact-browser-identity",
        "cookies": [
            {
                "name": "cf_clearance",
                "value": "fresh-clearance",
                "domain": ".dzmm.ai",
                "path": "/",
                "expires": 4070908800.0
            },
            {
                "name": "__cf_bm",
                "value": "browser-management",
                "domain": ".dzmm.ai",
                "path": "/",
                "expires": 4070908800.0
            }
        ],
        "expires_at": expires_at,
        "verified_at": "2026-07-31T04:00:00Z"
    })
}

#[test]
fn snapshot_parsing_validates_expiry() {
    let now = Utc.with_ymd_and_hms(2026, 7, 31, 5, 0, 0).unwrap();
    let current: ClearanceSnapshot =
        serde_json::from_value(snapshot_json("2026-07-31T06:00:00Z")).unwrap();
    let expired: ClearanceSnapshot =
        serde_json::from_value(snapshot_json("2026-07-31T04:30:00Z")).unwrap();

    current
        .validate_at(now)
        .expect("current snapshot is usable");
    let error = expired
        .validate_at(now)
        .expect_err("expired snapshot is rejected");
    assert!(error.to_string().contains("expired"));
}

#[test]
fn snapshot_rejects_an_expired_clearance_cookie_even_when_snapshot_expiry_is_future() {
    let now = Utc.with_ymd_and_hms(2026, 7, 31, 5, 0, 0).unwrap();
    let mut payload = snapshot_json("2026-07-31T06:00:00Z");
    payload["cookies"][0]["expires"] = serde_json::json!(now.timestamp() - 1);
    let snapshot: ClearanceSnapshot = serde_json::from_value(payload).unwrap();

    let error = snapshot
        .validate_at(now)
        .expect_err("expired cf_clearance cookie is rejected");

    assert!(error.to_string().contains("cf_clearance cookie expired"));
}

#[test]
fn cloudflare_cookies_override_stale_account_values_without_mutating_account_cookie_input() {
    let now = Utc.with_ymd_and_hms(2026, 7, 31, 5, 0, 0).unwrap();
    let snapshot: ClearanceSnapshot =
        serde_json::from_value(snapshot_json("2026-07-31T06:00:00Z")).unwrap();
    snapshot.validate_at(now).unwrap();
    let account_cookie_header = "session=account; cf_clearance=stale";

    let merged = snapshot.merge_cookie_header(Some(account_cookie_header));

    let pairs: Vec<_> = merged.split("; ").collect();
    assert!(pairs.contains(&"cf_clearance=fresh-clearance"));
    assert!(pairs.contains(&"__cf_bm=browser-management"));
    assert!(pairs.contains(&"session=account"));
    assert_eq!(
        pairs
            .iter()
            .filter(|pair| pair.starts_with("cf_clearance="))
            .count(),
        1
    );
    assert!(!pairs.contains(&"cf_clearance=stale"));
    assert_eq!(
        account_cookie_header, "session=account; cf_clearance=stale",
        "merge must not mutate the account cookie store"
    );
}

#[tokio::test]
async fn agent_client_uses_the_typed_snapshot_and_refresh_contract() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let base_url = format!("http://{}", listener.local_addr().unwrap());
    let handle = tokio::spawn(async move {
        let mut requests = Vec::new();
        for generation in [1, 2] {
            let (mut stream, _) = listener.accept().await.unwrap();
            let request = read_http_request(&mut stream).await;
            requests.push(request);

            let body = snapshot_json("2099-01-01T00:00:00Z")
                .as_object()
                .map(|snapshot| {
                    let mut snapshot = snapshot.clone();
                    snapshot.insert("generation".to_string(), serde_json::json!(generation));
                    serde_json::Value::Object(snapshot)
                })
                .unwrap()
                .to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        }
        requests
    });

    let client = ClearanceAgentClient::new(&base_url).unwrap();
    let first = client.snapshot().await.unwrap();
    let refreshed = client
        .refresh(1, ClearanceRefreshReason::CfMitigated)
        .await
        .unwrap();
    let requests = handle.await.unwrap();

    assert_eq!(first.generation, 1);
    assert_eq!(refreshed.generation, 2);
    assert!(requests[0].starts_with("GET /v1/snapshot HTTP/1.1\r\n"));
    assert!(requests[1].starts_with("POST /v1/refresh HTTP/1.1\r\n"));
    assert!(requests[1].contains(r#""observed_generation":1"#));
    assert!(requests[1].contains(r#""reason":"cf-mitigated""#));
}

async fn read_http_request(stream: &mut tokio::net::TcpStream) -> String {
    let mut data = Vec::new();
    let mut buffer = [0_u8; 1024];
    let header_end = loop {
        let count = stream.read(&mut buffer).await.unwrap();
        assert_ne!(count, 0, "connection closed before request headers");
        data.extend_from_slice(&buffer[..count]);
        if let Some(position) = data.windows(4).position(|window| window == b"\r\n\r\n") {
            break position + 4;
        }
    };

    let headers = String::from_utf8_lossy(&data[..header_end]);
    let content_length = headers
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, value)| value.trim().parse::<usize>().ok())
        .unwrap_or(0);
    while data.len() < header_end + content_length {
        let count = stream.read(&mut buffer).await.unwrap();
        assert_ne!(count, 0, "connection closed before request body");
        data.extend_from_slice(&buffer[..count]);
    }
    String::from_utf8(data[..header_end + content_length].to_vec()).unwrap()
}
