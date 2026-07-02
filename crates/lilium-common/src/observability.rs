// Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 utils/sentry.py
// NOTE: Python uses profile_session_sample_rate/profile_lifecycle/logging integration; Rust omits those.
// NOTE: Python event_text checks logentry "formatted" key; Rust checks "message" only.

use sentry::integrations::tracing::EventFilter;
use sentry::protocol::{Context, Event, Value, map::Map};
use sentry::{ClientInitGuard, ClientOptions};
use std::borrow::Cow;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

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

fn backend_sentry_release() -> Option<Cow<'static, str>> {
    env_string("SENTRY_RELEASE")
        .map(Cow::Owned)
        .or_else(|| option_env!("LILIUM_GIT_COMMIT_HASH").map(Cow::Borrowed))
}

fn init_backend_tracing_subscriber() {
    let sentry_layer =
        sentry::integrations::tracing::layer().event_filter(|md| match *md.level() {
            // Capture error and warn level events as both logs and events in Sentry
            tracing::Level::ERROR => EventFilter::Event | EventFilter::Log,
            // Capture everything else as both a breadcrumb and a log
            _ => EventFilter::Breadcrumb | EventFilter::Log,
        });

    // Global filter gates every layer (including sentry) and respects RUST_LOG.
    let global_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info".into());

    // Console-only filter: by default the same as `global_filter`, but silences the
    // high-cardinality db / event-processor transaction spans. Their span names and
    // span fields (`sentry.name`, `sentry.op`, `db.system`, ...) would otherwise prefix
    // every event line emitted inside a transaction, which is the bulk of console noise.
    // These spans still reach the sentry layer through `global_filter`.
    //
    // When RUST_LOG is set explicitly we defer to the user's value (debugging should see
    // everything); suppression only applies to the default `info` case.
    let console_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info,lilium.db.transaction=warn,lilium.processor.batch=warn".into());

    let _ = tracing_subscriber::registry()
        .with(sentry_layer)
        .with(global_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_level(true) // don't include levels in formatted output
                .with_target(false)
                .compact()
                .with_filter(console_filter),
        )
        .try_init();
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
        init_backend_tracing_subscriber();
        tracing::info!("Sentry disabled for pytest context");
        return None;
    }

    let dsn = match get_backend_sentry_dsn() {
        Some(dsn) => dsn,
        None => {
            init_backend_tracing_subscriber();
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

    options.release = backend_sentry_release();

    let guard = sentry::init(options);
    init_backend_tracing_subscriber();
    apply_backend_service_scope(service_name);
    tracing::info!("Sentry initialized for {}", service_name);
    Some(guard)
}
