use axum::{
    Router,
    routing::{get, post},
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{error::AppError, handlers, state::AppState};

const DEFAULT_PORT: &str = "8080";

pub async fn run() -> Result<(), AppError> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .without_time()
        .init();

    let database_url =
        std::env::var("DATABASE_URL").map_err(|_| AppError::internal("DATABASE_URL is not set"))?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .map_err(|_| AppError::internal("failed to connect to the database"))?;

    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(|_| AppError::internal("failed to run database migrations"))?;

    let state = AppState::new(pool);
    let app = Router::new()
        .nest(
            "/api/profiles",
            Router::new()
                .route(
                    "/",
                    post(handlers::create_profile).get(handlers::list_profiles),
                )
                .route(
                    "/{id}",
                    get(handlers::get_profile).delete(handlers::delete_profile),
                ),
        )
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port = std::env::var("PORT")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_PORT.to_string());
    let address: SocketAddr = format!("0.0.0.0:{port}")
        .parse()
        .map_err(|_| AppError::internal("invalid PORT value"))?;
    let listener = TcpListener::bind(address)
        .await
        .map_err(|_| AppError::internal("failed to bind server socket"))?;

    tracing::info!("server listening on port {port}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|_| AppError::internal("server stopped unexpectedly"))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C signal handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install terminate signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("shutdown signal received from Ctrl+C. cleaning up and stopping server");
        },
        _ = terminate => {
            tracing::info!("shutdown signal received from terminate. cleaning up and stopping server");
        },
    }
}
