use axum::{
    Router,
    routing::{delete, get, patch, post},
};

use crate::{handlers::user_handler, middleware::AppState};

pub fn get_user_route() -> Router<AppState> {
    let user_route = Router::new()
        .route("/", get(user_handler::list_users))
        .route("/", post(user_handler::create_user))
        .route("/:id", get(user_handler::get_user))
        .route("/:id", patch(user_handler::update_user))
        .route("/:id", delete(user_handler::delete_user));

    user_route
}
