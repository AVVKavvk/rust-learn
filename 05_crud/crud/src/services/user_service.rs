// The Service layer sits between Handler and Repository.
// It owns the business logic — validation, uniqueness checks,
// mapping DB rows to API responses, etc.
//
// Key Rust concept: `Arc<dyn UserRepository>`
//   - `Arc`       = Atomically-Reference-Counted smart pointer (thread-safe shared ownership)
//   - `dyn Trait` = a trait object — runtime polymorphism, like a Python base class reference
//   - Together they let you inject any concrete repository (Postgres, mock, etc.)

use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    errors::{AppError, AppResult},
    models::user::{
        CreateUserRequest, PaginatedResponse, PaginationQuery, UpdateUserRequest, UserResponse,
    },
    repositories::UserRepository,
};

// ─── Service Trait ────────────────────────────────────────────────────────────
#[async_trait]
pub trait UserService: Send + Sync {
    async fn list_users(
        &self,
        query: PaginationQuery,
    ) -> AppResult<PaginatedResponse<UserResponse>>;

    async fn get_user(&self, id: i32) -> AppResult<UserResponse>;
    async fn create_user(&self, user: CreateUserRequest) -> AppResult<UserResponse>;
    async fn update_user(&self, id: i32, user: UpdateUserRequest) -> AppResult<UserResponse>;
    async fn delete_user(&self, id: i32) -> AppResult<()>;
}

// ─── Concrete Implementation ──────────────────────────────────────────────────
#[derive(Clone)]

pub struct UserServiceImpl {
    // `Arc<dyn UserRepository>` — we hold a shared reference to *any* type
    // that implements UserRepository. This enables dependency injection and
    // easy mocking in tests.
    repo: Arc<dyn UserRepository>,
}

impl UserServiceImpl {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl UserService for UserServiceImpl {
    // ── List with pagination
    async fn list_users(
        &self,
        query: PaginationQuery,
    ) -> AppResult<PaginatedResponse<UserResponse>> {
        let per_page = query.per_page.min(100) as i64;
        let offset = ((query.page).saturating_sub(1) as i64) * per_page;

        let (rows, total) = self.repo.find_all(per_page, offset).await?;
        let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;

        Ok(PaginatedResponse {
            data: rows.into_iter().map(UserResponse::from).collect(),
            page: query.page,
            per_page: query.per_page,
            total,
            total_pages,
        })
    }
    // ── Get single user
    async fn get_user(&self, id: i32) -> AppResult<UserResponse> {
        let row = self.repo.find_by_id(id).await?;

        row.map(UserResponse::from)
            .ok_or_else(|| AppError::NotFound(format!("User {} not found", id)))
    }
    // ── Create

    async fn create_user(&self, req: CreateUserRequest) -> AppResult<UserResponse> {
        // Rule: email must be unique

        if self.repo.find_by_email(&req.email).await?.is_some() {
            return Err(AppError::Conflict(format!(
                "A user with email '{}' already exists",
                req.email
            )));
        }

        let row = self.repo.create(&req).await?;

        Ok(UserResponse::from(row))
    }
    // ── Update
    async fn update_user(&self, id: i32, req: UpdateUserRequest) -> AppResult<UserResponse> {
        if let Some(ref new_email) = req.email {
            if let Some(existing_user) = self.repo.find_by_email(new_email).await? {
                if existing_user.id != id {
                    return Err(AppError::Conflict(format!(
                        "Email '{}' is already in use",
                        new_email
                    )));
                }
            }
        }

        self.repo
            .update(id, &req)
            .await?
            .map(UserResponse::from)
            .ok_or_else(|| AppError::NotFound(format!("User {} not found", id)))
    }
    // ── Delete
    async fn delete_user(&self, id: i32) -> AppResult<()> {
        let deleted = self.repo.delete(id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("User {} not found", id)));
        }
        Ok(())
    }
}
