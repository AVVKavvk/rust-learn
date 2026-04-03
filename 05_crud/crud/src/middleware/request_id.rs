//
// This middleware attaches a unique `x-request-id` to every request
// and response
//
// Tower middleware in Axum works in layers:
//   Request → [RequestIdLayer] → [TraceLayer] → [Handler]
//                                                    ↓
//   Response ← [RequestIdLayer] ← [TraceLayer] ← [Handler]

use axum::{extract::Request, middleware::Next, response::Response};
use tower_http::request_id::RequestId;
use uuid::Uuid;

/// Extract the `x-request-id` header value as a String.
/// If the header is missing (shouldn't happen after RequestIdLayer), generate one.
pub fn extract_request_id(req: &Request) -> String {
    req.extensions()
        .get::<RequestId>()
        .and_then(|id| id.header_value().to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}

/// Middleware: ensure `x-request-id` is echoed back in the response headers.
pub async fn propagate_request_id(req: Request, next: Next) -> Response {
    let request_id = extract_request_id(&req);
    let mut response = next.run(req).await;

    // Add x-request-id to the response so clients can correlate logs
    if let Ok(val) = request_id.parse() {
        response.headers_mut().insert("x-request-id", val);
    }

    response
}
