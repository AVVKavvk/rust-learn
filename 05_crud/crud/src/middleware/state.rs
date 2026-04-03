//
// AppState is the shared state injected into every Axum handler via
// `State<AppState>` extractor.
//
// It holds `Arc<dyn UserService>` — a thread-safe reference-counted pointer
// to any concrete UserService. This is dependency injection, Rust style.

use crate::services::UserService;
use std::sync::Arc;

/// Shared application state.
/// `Clone` is required because Axum clones State for each request.
/// Since Arc is cheap to clone (just increments a counter), this is fine.
#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<dyn UserService>,
}

impl AppState {
    pub fn new(user_service: Arc<dyn UserService>) -> Self {
        Self { user_service }
    }
}
