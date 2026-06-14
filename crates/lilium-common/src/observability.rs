use sentry::protocol::{map::Map, Context, Event, Value};
use sentry::{ClientInitGuard, ClientOptions};
use std::borrow::Cow;
use std::env;
use std::sync::Arc;
use std::time::Duration;

fn env_float(name: &str, default: f32) -> f32 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(default)
}

fn env_string(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}

fn get_backend_sentry_dsn() -> Option<String> {
    env_string("SENTRY_BACKEND_DSN").or_else(|| env_string("SENTRY_DSN"))
}

fn event_text(event: &Event<'_>) -> String {
    let mut parts = Vec::new();

    if let Some(message) = event.message.as_ref() {
        parts.push(message.as_str());
    }

    if let Some(logger) = event.logger.as_ref() {
        parts.push(logger.as_str());
    }

    if let Some(logentry) = event.logentry.as_ref() {
        parts.push(logentry.message.as_str());
    }

    for exception in event.exception.as_ref() {
        if !exception.ty.is_empty() {
            parts.push(exception.ty.as_str());
        }
        if let Some(value) = exception.value.as_ref() {
            parts.push(value.as_str());
        }
        if let Some(mechanism) = exception.mechanism.as_ref() {
            parts.push(mechanism.ty.as_str());
        }
    }

    parts.join(" ").to_lowercase()
}

fn is_llm_rate_limit_text(text: &str) -> bool {
    [
        "rate limit exceeded",
        "resource_exhausted",
        "quota exceeded",
        "too many requests",
    ]
    .iter()
    .any(|token| text.contains(token))
}

fn is_google_genai_rate_limit_exception_event(event: &Event<'_>) -> bool {
    for exception in event.exception.as_ref() {
        let Some(mechanism) = exception.mechanism.as_ref() else {
            continue;
        };

        if mechanism.ty != "google_genai" {
            continue;
        }

        let mut text_parts = Vec::new();
        if !exception.ty.is_empty() {
            text_parts.push(exception.ty.as_str());
        }
        if let Some(value) = exception.value.as_ref() {
            text_parts.push(value.as_str());
        }
        if let Some(description) = mechanism.description.as_ref() {
            text_parts.push(description.as_str());
        }

        if is_llm_rate_limit_text(&text_parts.join(" ").to_lowercase()) {
            return true;
        }
    }

    false
}

fn is_handled_llm_rate_limit_log_event(event: &Event<'_>) -> bool {
    if event
        .tags
        .get("llm_rate_limit_terminal")
        .is_some_and(|value| value == "true")
    {
        return false;
    }

    let text = event_text(event);
    if !is_llm_rate_limit_text(&text) {
        return false;
    }

    [
        "llm api error",
        "gemini api error",
        "chat generation error",
        "text generation error",
        "structured generation error",
    ]
    .iter()
    .any(|token| text.contains(token))
}

fn before_send_backend_event(event: Event<'static>) -> Option<Event<'static>> {
    if is_google_genai_rate_limit_exception_event(&event) {
        return None;
    }

    if is_handled_llm_rate_limit_log_event(&event) {
        return None;
    }

    Some(event)
}

fn apply_backend_service_scope(service_name: &str) {
    sentry::configure_scope(|scope| {
        scope.set_tag("service", service_name);

        let mut service_context = Map::new();
        service_context.insert("name".to_string(), Value::String(service_name.to_string()));
        scope.set_context("service", Context::Other(service_context));
    });
}

pub fn init_backend_sentry(service_name: &str) -> Option<ClientInitGuard> {
    if env::var_os("PYTEST_CURRENT_TEST").is_some() {
        tracing::info!("Sentry disabled for pytest context");
        return None;
    }

    let dsn = match get_backend_sentry_dsn() {
        Some(dsn) => dsn,
        None => {
            tracing::info!("Sentry disabled: no SENTRY_BACKEND_DSN or SENTRY_DSN configured");
            return None;
        }
    };

    let mut options = ClientOptions {
        dsn: dsn.parse().ok(),
        environment: Some(Cow::Owned(
            env_string("SENTRY_ENVIRONMENT").unwrap_or_else(|| "production".to_string()),
        )),
        traces_sample_rate: env_float("SENTRY_TRACES_SAMPLE_RATE", 0.1),
        send_default_pii: true,
        enable_logs: true,
        shutdown_timeout: Duration::from_secs(2),
        before_send: Some(Arc::new(before_send_backend_event)),
        ..Default::default()
    };

    if let Some(release) = env_string("SENTRY_RELEASE") {
        options.release = Some(release.into());
    }

    let guard = sentry::init(options);
    apply_backend_service_scope(service_name);
    tracing::info!("Sentry initialized for {}", service_name);
    Some(guard)
}
