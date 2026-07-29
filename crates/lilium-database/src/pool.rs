// Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 database/async_engine.py database/database_url.py

use percent_encoding::{AsciiSet, CONTROLS, percent_decode_str, utf8_percent_encode};
use sqlx::PgPool;
use std::fmt::Write as _;
use url::ParseError;

#[derive(Debug, Clone)]
/// Internal wrapper around the SQLx Postgres pool used by [`crate::Database`].
///
/// This type exists so the database crate can keep raw SQLx pool access behind
/// an explicit infrastructure boundary. Service code should use
/// [`crate::Database::orm`] and SeaORM instead of depending on this pool.
pub struct DbPool {
    inner: PgPool,
}

/// Characters the `url` crate rejects inside an opaque (non-special scheme)
/// host. We percent-encode these when lifting a libpq `host=` query parameter
/// into the URL authority so the result still parses; SQLx percent-decodes the
/// host back before use.
const HOST_AUTHORITY_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'#')
    .add(b'/')
    .add(b':')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'|');

/// Normalize Postgres URLs for SQLx/SeaORM connection constructors.
///
/// The Python side and deployment configuration may use `postgresql://`; SQLx
/// accepts `postgres://` consistently, so the database layer normalizes it once.
///
/// The shared deployment `.env` also uses the libpq convention of placing the
/// real host (often a PgBouncer Unix socket) in a `host=` query parameter with
/// an empty authority, e.g. `postgres://user:pass@/db?host=/run/pgbouncer`.
/// libpq and the Python stack accept that, but the `url` crate — which SQLx
/// uses to parse connection URLs — rejects it with `EmptyHost` whenever
/// userinfo is present without an authority host. To keep that shared
/// configuration working, such URLs are rewritten into the authority-host form
/// SQLx expects: `postgres://user:pass@%2Frun%2Fpgbouncer:6432/db`.
pub(crate) fn normalize_database_url(url: &str) -> String {
    let url = url.trim();
    let normalized = if let Some(rest) = url.strip_prefix("postgresql://") {
        format!("postgres://{rest}")
    } else {
        url.to_string()
    };

    match normalized.parse::<url::Url>() {
        // Already valid for the `url` crate / SQLx.
        Ok(_) => normalized,
        // libpq-style empty authority host with the real host in a query param.
        Err(ParseError::EmptyHost) => rewrite_libpq_host_query(&normalized).unwrap_or(normalized),
        // Any other parse error is surfaced by SQLx downstream; pass it through.
        Err(_) => normalized,
    }
}

/// Rewrite a libpq-style URL (empty authority host, `host=`/`port=` query
/// parameters) into the authority-host form the `url` crate accepts.
///
/// Returns `None` when the input does not match the expected shape, leaving the
/// original URL to be parsed (and errored on) by SQLx as-is.
fn rewrite_libpq_host_query(input: &str) -> Option<String> {
    // Peel off the fragment, then the query, so authority/path parsing sees a
    // clean `scheme://authority/path` prefix.
    let (before_fragment, fragment) = match input.split_once('#') {
        Some((before, frag)) => (before, Some(frag)),
        None => (input, None),
    };
    let (authority_and_path, query) = match before_fragment.split_once('?') {
        Some((prefix, q)) => (prefix, Some(q)),
        None => (before_fragment, None),
    };

    let mut host: Option<String> = None;
    let mut port: Option<String> = None;
    let mut remaining: Vec<(String, String)> = Vec::new();
    if let Some(q) = query {
        for pair in q.split('&') {
            if pair.is_empty() {
                continue;
            }
            let (key, value) = match pair.split_once('=') {
                Some((k, v)) => (k, v),
                None => (pair, ""),
            };
            match key {
                "host" => host = Some(decode_query_value(value)),
                "port" => port = Some(decode_query_value(value)),
                _ => remaining.push((key.to_string(), decode_query_value(value))),
            }
        }
    }

    // Without a `host=` query parameter there is nothing to lift into the
    // authority; this is a genuinely malformed URL.
    let host = host?;

    let scheme_end = input.find("://")? + 3;
    let authority_end = authority_and_path[scheme_end..]
        .find('/')
        .map(|offset| scheme_end + offset)
        .unwrap_or(authority_and_path.len());
    let authority = &authority_and_path[scheme_end..authority_end];
    let path = &authority_and_path[authority_end..];

    let userinfo = authority.rsplit_once('@').map(|(user, _)| user);

    let encoded_host = utf8_percent_encode(&host, HOST_AUTHORITY_ENCODE_SET);

    let mut out = String::from(&input[..scheme_end]);
    if let Some(user) = userinfo {
        out.push_str(user);
        out.push('@');
    }
    // `PercentEncode` is `Display`; write it directly to avoid allocating an
    // intermediate `String` for the host.
    write!(out, "{}", encoded_host).unwrap();
    if let Some(port) = port {
        out.push(':');
        out.push_str(&port);
    }
    out.push_str(path);
    if !remaining.is_empty() {
        out.push('?');
        let mut first = true;
        for (key, value) in &remaining {
            if !first {
                out.push('&');
            }
            first = false;
            write!(
                out,
                "{}={}",
                utf8_percent_encode(key, HOST_AUTHORITY_ENCODE_SET),
                utf8_percent_encode(value, HOST_AUTHORITY_ENCODE_SET)
            )
            .unwrap();
        }
    }
    if let Some(fragment) = fragment {
        out.push('#');
        out.push_str(fragment);
    }

    Some(out)
}

fn decode_query_value(input: &str) -> String {
    percent_decode_str(input).decode_utf8_lossy().into_owned()
}

impl DbPool {
    /// Wrap an existing SQLx Postgres pool for internal database runtime use.
    pub(crate) fn from_pg_pool(inner: PgPool) -> Self {
        Self { inner }
    }

    /// Return the underlying SQLx pool for infrastructure-only helpers.
    ///
    /// Do not expose this to services for ordinary table CRUD. Prefer
    /// `ConnectionTrait` and SeaORM query builders.
    pub fn inner(&self) -> &PgPool {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_database_url;

    #[test]
    fn normalizes_postgresql_scheme() {
        assert_eq!(
            normalize_database_url("  postgresql://localhost/db  "),
            "postgres://localhost/db"
        );
    }

    #[test]
    fn leaves_valid_authority_url_untouched() {
        assert_eq!(
            normalize_database_url("postgres://user:pass@localhost:5432/db"),
            "postgres://user:pass@localhost:5432/db"
        );
    }

    #[test]
    fn rewrites_libpq_unix_socket_host_query() {
        // Mirrors the shared deployment `.env` form.
        let out = normalize_database_url(
            "postgresql://dzmm:Eipei2luthuCej0B@/dzmm?host=/run/pgbouncer&port=6432",
        );
        assert_eq!(
            out,
            "postgres://dzmm:Eipei2luthuCej0B@%2Frun%2Fpgbouncer:6432/dzmm"
        );
        // The rewritten URL must parse for the `url` crate / SQLx.
        assert!(out.parse::<url::Url>().is_ok());
    }

    #[test]
    fn rewrites_libpq_tcp_host_query() {
        let out = normalize_database_url("postgres://user:pass@/db?host=db.example.com&port=6543");
        assert_eq!(out, "postgres://user:pass@db.example.com:6543/db");
        assert!(out.parse::<url::Url>().is_ok());
    }

    #[test]
    fn rewrites_libpq_query_without_port() {
        let out = normalize_database_url("postgres://user@/db?host=/var/run/postgresql");
        assert_eq!(out, "postgres://user@%2Fvar%2Frun%2Fpostgresql/db");
        assert!(out.parse::<url::Url>().is_ok());
    }

    #[test]
    fn preserves_other_query_pairs_during_rewrite() {
        let out = normalize_database_url(
            "postgres://user:pass@/db?host=/run/pgbouncer&sslmode=disable&port=6432",
        );
        assert_eq!(
            out,
            "postgres://user:pass@%2Frun%2Fpgbouncer:6432/db?sslmode=disable"
        );
        assert!(out.parse::<url::Url>().is_ok());
    }

    #[test]
    fn passes_through_empty_host_without_host_query() {
        // Genuinely malformed (no host anywhere) — leave it for SQLx to report.
        let out = normalize_database_url("postgres://user:pass@/db");
        assert_eq!(out, "postgres://user:pass@/db");
    }
}
