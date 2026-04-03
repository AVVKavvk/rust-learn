mod config;
mod errors;
mod handlers;
mod logger;
mod middleware;
mod models;
mod repositories;
mod routes;
mod services;

use anyhow::Context;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

use config::Config;
use middleware::AppState;
use repositories::PgUserRepository;
use services::UserServiceImpl;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ── 1. Load .env file
    dotenv::dotenv().ok();

    // ── 2. Load typed config
    let config = Config::from_env().context("Failed to load configuration")?;

    // ── 3. Initialise JSON logger
    //
    // After this line, `tracing::info!()` etc. emit structured JSON to stdout.

    logger::init(&config.log_level);

    tracing::info!(
        env  = %config.app_env,
        port = config.port,
        "Starting rust-crud-api"
    );

    // ── 4. Connect to PostgreSQL
    //
    // PgPool is a connection pool — it manages multiple DB connections
    // and hands them out to concurrent requests.

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .context("Failed to connect to database")?;

    tracing::info!("Connected to PostgreSQL");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("Failed to run database migrations")?;

    tracing::info!("Migrations applied");

    // ── 6. Dependency Injection
    //
    // We build the dependency graph bottom-up:
    //   PgPool → Repository → Service → AppState
    //
    // Each layer wraps the previous in Arc<dyn Trait> so:
    //   - It's heap-allocated (needed for trait objects)
    //   - It's reference-counted (cheap to clone across threads)
    //   - It's behind a trait (easy to swap for a mock in tests)
    //
    // Python analogy:
    //   repo    = PgUserRepository(pool)
    //   service = UserServiceImpl(repo)
    //   state   = AppState(service)

    let user_repo = Arc::new(PgUserRepository::new(pool));
    let user_service = Arc::new(UserServiceImpl::new(user_repo));
    let state = AppState::new(user_service);

    // ── 7. Build router
    let app = routes::create_router(state);
    // ── 8. Start server
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context(format!("Failed to bind to {}", addr))?;

    tracing::info!(address = %addr, "Server listening");

    axum::serve(listener, app).await.context("Server error")?;

    Ok(())
}
