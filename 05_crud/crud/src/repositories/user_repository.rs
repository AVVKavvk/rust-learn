// This file shows the Repository Pattern in Rust with:
//   1. A `trait`   — the interface (like a Python ABC / Protocol)
//   2. A `struct`  — the concrete Postgres implementation
//   3. `async-trait` — because Rust traits don't support async natively yet
//   4. Lifetimes   — `'_` on queries tells the borrow checker the query
//                       borrows from `self` only for the duration of the call.

use anyhow::Context;
use async_trait::async_trait;
use sqlx::PgPool;

use crate::{
    errors::AppResult,
    models::user::{CreateUserRequest, UpdateUserRequest, UserRow},
};

// ─── Trait (the interface) ────────────────────────────────────────────────────
//
// `async_trait` rewrites each `async fn` so it returns a `Pin<Box<dyn Future>>`.
// This is needed because Rust trait objects (`dyn Trait`) don't yet support
// `async fn` natively (stabilisation is in progress).

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_all(&self, limit: i64, offset: i64) -> AppResult<(Vec<UserRow>, i64)>;
    async fn find_by_id(&self, id: i32) -> AppResult<Option<UserRow>>;
    async fn find_by_email(&self, email: &str) -> AppResult<Option<UserRow>>;
    async fn create(&self, user: &CreateUserRequest) -> AppResult<UserRow>;
    async fn update(&self, id: i32, req: &UpdateUserRequest) -> AppResult<Option<UserRow>>;
    async fn delete(&self, id: i32) -> AppResult<bool>;
}

// ─── Concrete Postgres Implementation ─────────────────────────────────────────
//
// `PgUserRepository` holds a *clone* of the connection pool.
// Cloning a pool is cheap — it just increments an Arc reference count.
#[derive(Clone)]
pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
// ─── Implementing the Trait ────────────────────────────────────────────────────
//
// Lifetimes in sqlx queries:
//   `query_as!` borrows string parameters for the duration of the `.await`.
//   sqlx infers the anonymous lifetime `'_` automatically in most cases.

#[async_trait]
impl UserRepository for PgUserRepository {
    // ── List with pagination ─────────────────────────────────────────────────
    async fn find_all(&self, limit: i64, offset: i64) -> AppResult<(Vec<UserRow>, i64)> {
        // Run both queries concurrently with tokio::try_join!
        let (rows, count): (Vec<UserRow>, Option<i64>) = tokio::try_join!(
            sqlx::query_as!(
                UserRow,
                r#"
                SELECT id, name, email, bio, created_at, updated_at
                FROM users
                ORDER BY created_at DESC
                LIMIT $1 OFFSET $2
                "#,
                limit,
                offset
            )
            .fetch_all(&self.pool),
            sqlx::query_scalar!("SELECT COUNT(*) FROM users").fetch_one(&self.pool),
        )
        .context("Failed to fetch users")?;

        Ok((rows, count.unwrap_or(0)))
    }

    async fn find_by_id(&self, id: i32) -> AppResult<Option<UserRow>> {
        let row = sqlx::query_as!(
            UserRow,
            r#"
            SELECT id, name, email, bio, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch user by id")?;

        Ok(row)
    }
    // ── Find by email (for uniqueness checks) ───────────────────────────────
    async fn find_by_email(&self, email: &str) -> AppResult<Option<UserRow>> {
        let row = sqlx::query_as!(
            UserRow,
            "SELECT id, name, email, bio, created_at, updated_at FROM users WHERE email = $1",
            email
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch user by email")?;

        Ok(row)
    }

    // ── Create ───────────────────────────────────────────────────────────────
    async fn create(&self, req: &CreateUserRequest) -> AppResult<UserRow> {
        let row = sqlx::query_as!(
            UserRow,
            r#"
            INSERT INTO users (name, email, bio, created_at, updated_at)
            VALUES ($1, $2, $3, NOW(), NOW())
            RETURNING id, name, email, bio, created_at, updated_at
            "#,
            req.name,
            req.email,
            req.bio,
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to insert user")?;

        Ok(row)
    }

    // ── Update (partial — only set provided fields) ──────────────────────────
    async fn update(&self, id: i32, req: &UpdateUserRequest) -> AppResult<Option<UserRow>> {
        // COALESCE: if the caller passes NULL, keep the existing value.
        // This is the standard SQL way to do a partial update without
        // building a dynamic query string.
        let row = sqlx::query_as!(
            UserRow,
            r#"
            UPDATE users
            SET
                name       = COALESCE($2, name),
                email      = COALESCE($3, email),
                bio        = COALESCE($4, bio),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, email, bio, created_at, updated_at
            "#,
            id,
            req.name,
            req.email,
            req.bio,
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to update user")?;

        Ok(row)
    }

    // ── Delete ───────────────────────────────────────────────────────────────
    async fn delete(&self, id: i32) -> AppResult<bool> {
        let result = sqlx::query!("DELETE FROM users WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .context("Failed to delete user")?;

        // `rows_affected() == 1` means we actually deleted something.
        Ok(result.rows_affected() == 1)
    }
}
