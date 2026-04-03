//
// Handlers are thin — they:
//   1. Extract data from the request (path params, query, body)
//   2. Validate input
//   3. Call the service
//   4. Return a JSON response
//
// They should NOT contain business logic (that lives in the service).
//
// Key Axum concepts:
//   - `State<AppState>`    — injects shared state (DI container)
//   - `Path<i32>`         — extracts `:id` from the URL
//   - `Query<T>`           — extracts ?page=1&per_page=20
//   - `Json<T>`            — deserialises the request body
//   - Return type `impl IntoResponse` — anything Axum can turn into HTTP

use axum::{
    Json,
    extract::{FromRequest, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};

use validator::Validate;

use crate::{
    errors::{AppError, AppResult},
    log_response,
    middleware::{AppState, request_id::extract_request_id},
    models::user::{CreateUserRequest, PaginationQuery, UpdateUserRequest},
};

// ─── GET /users
pub async fn list_users(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
    req: axum::extract::Request,
) -> AppResult<impl IntoResponse> {
    let request_id = extract_request_id(&req);
    tracing::info!(
        x_request_id = %request_id,
        page = pagination.page,
        per_page = pagination.per_page,
        "Listing users"
    );

    let result = state.user_service.list_users(pagination).await?;

    log_response!(
        &request_id,
        "Users listed successfully",
        total = result.total
    );

    Ok((StatusCode::OK, Json(result)))
}

// ─── GET /users/:id
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    req: axum::extract::Request,
) -> AppResult<impl IntoResponse> {
    let request_id = extract_request_id(&req);

    tracing::info!(x_request_id = %request_id, user_id = %id, "Fetching user");

    let user = state.user_service.get_user(id).await?;

    log_response!(&request_id, "User fetched", user_id = user.id);

    Ok((StatusCode::OK, Json(user)))
}

// ─── POST /users
pub async fn create_user(
    State(state): State<AppState>,
    req: axum::extract::Request,
) -> AppResult<impl IntoResponse> {
    // We manually extract body + request_id from the raw request
    // so we can log the request_id BEFORE consuming the body.
    let request_id = extract_request_id(&req);

    // Consume the request body as JSON.
    // `axum::Json` extractor validates that Content-Type is application/json.

    let (parts, body) = req.into_parts();
    let inner_req = axum::extract::Request::from_parts(parts, body);

    let Json(payload): Json<CreateUserRequest> = Json::from_request(inner_req, &())
        .await
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Run validator rules from the #[validate(...)] attributes
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    tracing::info!(
        x_request_id = %request_id,
        email = %payload.email,
        "Creating user"
    );

    let user = state.user_service.create_user(payload).await?;

    log_response!(&request_id, "User created", user_id = user.id);

    // 201 Created + the new resource in the body
    Ok((StatusCode::CREATED, Json(user)))
}

// ─── PATCH /users/:id
pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    req: axum::extract::Request,
) -> AppResult<impl IntoResponse> {
    let request_id = extract_request_id(&req);

    let (parts, body) = req.into_parts();
    let inner_req = axum::extract::Request::from_parts(parts, body);
    let Json(payload): Json<UpdateUserRequest> = Json::from_request(inner_req, &())
        .await
        .map_err(|e| AppError::Validation(e.to_string()))?;

    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    tracing::info!(x_request_id = %request_id, user_id = %id, "Updating user");

    let user = state.user_service.update_user(id, payload).await?;

    log_response!(&request_id, "User updated", user_id = user.id);

    Ok((StatusCode::OK, Json(user)))
}

// ─── DELETE /users/:id ────────────────────────────────────────────────────────
pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    req: axum::extract::Request,
) -> AppResult<impl IntoResponse> {
    let request_id = extract_request_id(&req);

    tracing::info!(x_request_id = %request_id, user_id = %id, "Deleting user");

    state.user_service.delete_user(id).await?;

    log_response!(&request_id, "User deleted", user_id = id);

    // 204 No Content — success, nothing to return
    Ok(StatusCode::NO_CONTENT)
}
