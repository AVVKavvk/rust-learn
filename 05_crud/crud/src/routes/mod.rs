pub mod user_route;

pub use user_route::get_user_route;

use axum::{Router, middleware};
use tower_http::{
    cors::{Any, CorsLayer},
    request_id::{MakeRequestUuid, SetRequestIdLayer},
    trace::TraceLayer,
};

use crate::middleware::{AppState, request_id::propagate_request_id};

/// Build the complete application router.
pub fn create_router(state: AppState) -> Router {
    let users_router = get_user_route();

    Router::new()
        .nest("/api/v1/users", users_router)
        // Inject shared state into all handlers
        .with_state(state)
        // ── Middleware stack (outermost = first to run) ──────────────────────
        //
        // 1. Assign a UUID to every request as `x-request-id`
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        // 2. Echo the request-id back in the response headers
        .layer(middleware::from_fn(propagate_request_id))
        // 3. Automatic HTTP tracing (logs method, path, status, latency)
        .layer(TraceLayer::new_for_http())
        // 4. CORS — open for development; tighten for production
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
}
