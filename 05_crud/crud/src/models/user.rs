// Rust concepts demonstrated:
//  - Separate structs for DB row (UserRow), API response (UserResponse),
//    and request bodies (CreateUserRequest, UpdateUserRequest).
//  - `#[derive(...)]` for auto-generated trait impls.
//  - `Option<T>` for nullable / optional fields.
//  - `validator` crate for field-level validation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

// ─── Database Row ────────────────────────────────────────────────────────────
// This struct maps 1-to-1 with the `users` table columns.
// `sqlx::FromRow` lets sqlx deserialise query results directly into this type.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRow {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub bio: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── API Response ─────────────────────────────────────────────────────────────
// What the client receives. We derive `Serialize` so Axum can JSON-encode it.
// We intentionally do NOT expose internal fields (e.g. password hashes) here.

#[derive(Debug, Clone, Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub bio: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Convert a DB row into an API response.
/// Implementing `From<UserRow>` gives us `.into()` for free everywhere.
impl From<UserRow> for UserResponse {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            email: row.email,
            bio: row.bio,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 1, max = 100, message = "Name must be 1-100 characters"))]
    pub name: String,

    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    #[validate(length(max = 500, message = "Bio must be at most 500 characters"))]
    pub bio: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,

    #[validate(email)]
    pub email: Option<String>,

    #[validate(length(max = 500))]
    pub bio: Option<String>,
}

// ─── Pagination ───────────────────────────────────────────────────────────────
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}
fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    20
}
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
    pub total_pages: u32,
}
