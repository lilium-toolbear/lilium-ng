use anyhow::{Context, Result};
use chrono::{DateTime, Duration, TimeZone, Utc};
use lilium_database::DbSession;

pub async fn reset_database(session: &mut DbSession) -> Result<()> {
    let table_names = public_table_names(session).await?;
    if !table_names.is_empty() {
        let truncate_sql = format!(
            "TRUNCATE TABLE {} RESTART IDENTITY CASCADE",
            table_names.join(", ")
        );
        sqlx::query(&truncate_sql)
            .execute(session.as_mut())
            .await
            .context("truncate public tables")?;
    }

    ensure_time_partitions(session).await?;

    sqlx::query("SELECT pg_advisory_unlock_all()")
        .execute(session.as_mut())
        .await
        .context("unlock advisory locks")?;

    Ok(())
}

async fn public_table_names(session: &mut DbSession) -> Result<Vec<String>> {
    let names = sqlx::query_scalar::<_, String>(
        r#"
        SELECT format('%I.%I', schemaname, tablename)
        FROM pg_tables
        WHERE schemaname = 'public'
          AND tablename <> 'sqlx_migrations'
          AND NOT EXISTS (
              SELECT 1
              FROM pg_inherits
              WHERE inhrelid = format('%I.%I', schemaname, tablename)::regclass
          )
        ORDER BY tablename
        "#,
    )
    .fetch_all(session.as_mut())
    .await
    .context("list public tables")?;

    Ok(names)
}

async fn ensure_time_partitions(session: &mut DbSession) -> Result<()> {
    let now = Utc::now();
    let messages_anchors = [
        timestamp(2024, 1, 1, 0, 0, 0),
        timestamp(2026, 2, 28, 23, 59, 0),
        timestamp(2026, 3, 1, 0, 1, 0),
        now - Duration::days(40),
        now,
        now + Duration::days(40),
    ];
    let websocket_anchors = [
        timestamp(2024, 1, 1, 0, 0, 0),
        timestamp(2026, 3, 15, 0, 0, 0),
        timestamp(2026, 3, 16, 0, 1, 0),
        now - Duration::days(8),
        now,
        now + Duration::days(8),
    ];

    ensure_partitions_for_table(session, "messages", &messages_anchors).await?;
    ensure_partitions_for_table(session, "websocket_events", &websocket_anchors).await?;

    Ok(())
}

fn timestamp(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month, day, hour, minute, second)
        .single()
        .expect("valid test partition timestamp")
}

async fn ensure_partitions_for_table(
    session: &mut DbSession,
    table_name: &str,
    anchors: &[DateTime<Utc>],
) -> Result<()> {
    for anchor in anchors {
        let _: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT child_name
            FROM ensure_time_partitions(
                p_table_name => $1,
                p_anchor => $2,
                p_apply => true
            )
            "#,
        )
        .bind(table_name)
        .bind(anchor)
        .fetch_all(session.as_mut())
        .await
        .with_context(|| format!("ensure {table_name} partitions"))?;
    }

    Ok(())
}
