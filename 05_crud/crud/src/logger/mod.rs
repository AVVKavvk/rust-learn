/**
 * In Rust, `tracing` uses "subscribers" (like handlers) and "layers" (like formatters).
 * We configure a JSON layer that emits structured logs
 */
use tracing_subscriber::{
    EnvFilter, Layer,
    fmt::{self, time::UtcTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

/**
 *  Initialise the global JSON logger.
 *  Call this once at the very start of `main()`.
 *  After this, any `tracing::info!()`, `tracing::error!()`, etc.
 *  will emit a JSON line like:
 *
 * ```json
 *   {
 *   "timestamp": "2024-01-15 10:30:45",
 *   "level": "INFO",
 *   "target": "rust_crud::handlers::users",
 *   "filename": "src/handlers/users.rs",
 *   "line": 42,
 *   "message": "Fetching user",
 *   "fields": { "user_id": "abc-123" }
 *   }
 *```
 */

pub fn init(log_level: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    let json_layer = fmt::layer()
        .json() // emit JSON
        .with_timer(UtcTime::rfc_3339()) // ISO-8601 timestamps
        .with_target(true) // include module path
        .with_file(true) // include filename  (record.filename)
        .with_line_number(true) // include line no   (record.lineno)
        .with_current_span(true) // include span fields (request_id, etc.)
        .with_filter(filter);

    // Register the subscriber globally.
    // Python equivalent: logging.getLogger("trunk-logger")  +  addHandler(stream_handler)
    tracing_subscriber::registry().with(json_layer).init();
}

// pub struct ResponseLog<'a> {
//     pub x_request_id: &'a str,
//     pub message: &'a str,
// }

// impl<'a> ResponseLog<'a> {
//     pub fn new(x_request_id: &'a str, message: &'a str) -> Self {
//         ResponseLog {
//             x_request_id,
//             message,
//         }
//     }

//     pub fn info(&self) {
//         tracing::info!(x_request_id = self.x_request_id, message = self.message)
//     }

//     pub fn error(&self) {
//         tracing::error!(x_request_id = self.x_request_id, message = self.message)
//     }

//     pub fn debug(&self) {
//         tracing::debug!(x_request_id = self.x_request_id, message = self.message)
//     }

//     pub fn warn(&self) {
//         tracing::warn!(x_request_id = self.x_request_id, message = self.message)
//     }
// }

/// Convenience macro — lets you add arbitrary key=value fields just like **kwargs.
///
/// Usage:
/// ```rust
/// log_response!(req_id, "User created", user_id = %user.id, status = 201);
/// ```
#[macro_export]
macro_rules! log_response {
    ($req_id:expr, $msg:expr $(, $key:ident = $val:expr)* $(,)?) => {
        tracing::info!(
            x_request_id = $req_id,
            message       = $msg,
            $($key = $val,)*
        )
    };
}
