use std::time::SystemTime;

use sea_orm::{DatabaseConnection, Statement};
use sentry::protocol::{SpanId, SpanStatus, Value};
use sentry::{TransactionContext, TransactionOrSpan};

const DB_SPAN_OP: &str = "db";
const DB_SYSTEM_NAME: &str = "postgresql";
const DB_DRIVER_NAME: &str = "sqlx";
const DB_ORM_NAME: &str = "sea-orm";

pub(crate) fn install_sea_orm_query_metrics(
    connection: &mut DatabaseConnection,
    database_namespace: Option<String>,
) {
    connection.set_metric_callback(move |info| {
        capture_sea_orm_query(info, database_namespace.as_deref())
    });
}

fn capture_sea_orm_query(info: &sea_orm::metric::Info<'_>, database_namespace: Option<&str>) {
    let data = SqlSpanData::from_statement(info.statement);
    let end_timestamp = SystemTime::now();
    let start_timestamp = end_timestamp
        .checked_sub(info.elapsed)
        .unwrap_or(end_timestamp);

    let span: TransactionOrSpan = match sentry::configure_scope(|scope| scope.get_span()) {
        Some(parent) => parent
            .start_child_with_details(
                DB_SPAN_OP,
                &data.description,
                SpanId::default(),
                start_timestamp,
            )
            .into(),
        None => sentry::start_transaction_with_timestamp(
            TransactionContext::new(&data.description, DB_SPAN_OP),
            start_timestamp,
        )
        .into(),
    };

    span.set_data("db.system", Value::from(DB_SYSTEM_NAME));
    span.set_data("db.system.name", Value::from(DB_SYSTEM_NAME));
    span.set_data("db.driver.name", Value::from(DB_DRIVER_NAME));
    span.set_data("db.orm", Value::from(DB_ORM_NAME));
    span.set_data("db.operation.name", Value::from(data.operation));
    span.set_data("db.query.text", Value::from(data.query_text));
    span.set_data("db.query.summary", Value::from(data.summary));
    span.set_data(
        "db.query.duration_ms",
        Value::from(info.elapsed.as_secs_f64() * 1000.0),
    );
    if let Some(namespace) = database_namespace {
        span.set_data("db.name", Value::from(namespace));
        span.set_data("db.namespace", Value::from(namespace));
    }
    if let Some(collection) = data.collection {
        span.set_data("db.collection.name", Value::from(collection));
    }

    span.set_status(if info.failed {
        SpanStatus::InternalError
    } else {
        SpanStatus::Ok
    });
    span.finish_with_timestamp(end_timestamp);
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SqlSpanData {
    query_text: String,
    operation: String,
    collection: Option<String>,
    summary: String,
    description: String,
}

impl SqlSpanData {
    fn from_statement(statement: &Statement) -> Self {
        let query_text = statement.sql.clone();
        let tokens = sql_tokens(&query_text);
        let operation = operation_from_tokens(&tokens).unwrap_or_else(|| "SQL".to_string());
        let collection = collection_from_tokens(&tokens, &operation);
        let summary = match collection.as_deref() {
            Some(collection) => format!("{operation} {collection}"),
            None => operation.clone(),
        };

        Self {
            query_text,
            operation,
            collection,
            description: statement.sql.clone(),
            summary,
        }
    }
}

fn operation_from_tokens(tokens: &[String]) -> Option<String> {
    tokens
        .iter()
        .map(|token| token.to_ascii_uppercase())
        .find(|token| {
            matches!(
                token.as_str(),
                "SELECT" | "INSERT" | "UPDATE" | "DELETE" | "CREATE" | "ALTER" | "DROP"
            )
        })
}

fn collection_from_tokens(tokens: &[String], operation: &str) -> Option<String> {
    let raw_collection = match operation {
        "SELECT" => token_after(tokens, "FROM"),
        "INSERT" => token_after(tokens, "INTO"),
        "UPDATE" => tokens.get(1),
        "DELETE" => token_after(tokens, "FROM"),
        "CREATE" | "ALTER" | "DROP" => tokens.get(2),
        _ => None,
    }?;

    normalize_collection_name(raw_collection)
}

fn token_after<'a>(tokens: &'a [String], marker: &str) -> Option<&'a String> {
    tokens
        .windows(2)
        .find(|window| window[0].eq_ignore_ascii_case(marker))
        .map(|window| &window[1])
}

fn normalize_collection_name(token: &str) -> Option<String> {
    let normalized = token
        .split('.')
        .next_back()
        .unwrap_or(token)
        .trim_matches(|character| {
            matches!(
                character,
                '"' | '\'' | '`' | '[' | ']' | '(' | ')' | ',' | ';'
            )
        });

    (!normalized.is_empty()).then(|| normalized.to_string())
}

fn sql_tokens(sql: &str) -> Vec<String> {
    sql.split_whitespace()
        .map(|token| {
            token
                .trim_matches(|character| matches!(character, '(' | ')' | ',' | ';'))
                .to_string()
        })
        .filter(|token| !token.is_empty())
        .collect()
}
