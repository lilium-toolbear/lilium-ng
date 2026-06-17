// Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 tests/conftest.py, database/partitioning.py

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, TimeZone, Utc};
use sea_orm::{ConnectionTrait, Statement};

pub async fn reset_database<C: ConnectionTrait>(db: &C) -> Result<()> {
    let table_names = public_table_names(db).await?;
    if !table_names.is_empty() {
        let truncate_sql = format!(
            "TRUNCATE TABLE {} RESTART IDENTITY CASCADE",
            table_names.join(", ")
        );
        db.execute(Statement::from_string(
            db.get_database_backend(),
            truncate_sql,
        ))
        .await
        .context("truncate public tables")?;
    }

    ensure_time_partitions(db).await?;

    db.execute(Statement::from_string(
        db.get_database_backend(),
        "SELECT pg_advisory_unlock_all()".to_owned(),
    ))
    .await
    .context("unlock advisory locks")?;

    Ok(())
}

async fn public_table_names<C: ConnectionTrait>(db: &C) -> Result<Vec<String>> {
    let rows = db
        .query_all(Statement::from_string(
            db.get_database_backend(),
            r#"
            SELECT format('%I.%I', schemaname, tablename) AS table_name
            FROM pg_tables
            WHERE schemaname = 'public'
              AND tablename <> 'sqlx_migrations'
              AND NOT EXISTS (
                  SELECT 1
                  FROM pg_inherits
                  WHERE inhrelid = format('%I.%I', schemaname, tablename)::regclass
              )
            ORDER BY tablename
            "#
            .to_owned(),
        ))
        .await
        .context("list public tables")?;

    rows.into_iter()
        .map(|row| {
            row.try_get("", "table_name")
                .context("read public table name")
        })
        .collect()
}

async fn ensure_time_partitions<C: ConnectionTrait>(db: &C) -> Result<()> {
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

    ensure_partitions_for_table(db, "messages", &messages_anchors).await?;
    ensure_partitions_for_table(db, "websocket_events", &websocket_anchors).await?;

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

async fn ensure_partitions_for_table<C: ConnectionTrait>(
    db: &C,
    table_name: &str,
    anchors: &[DateTime<Utc>],
) -> Result<()> {
    for anchor in anchors {
        db.query_all(Statement::from_sql_and_values(
            db.get_database_backend(),
            r#"
            SELECT child_name
            FROM ensure_time_partitions(
                p_table_name => $1,
                p_anchor => $2,
                p_apply => true
            )
            "#,
            vec![table_name.into(), (*anchor).into()],
        ))
        .await
        .with_context(|| format!("ensure {table_name} partitions"))?;
    }

    Ok(())
}
