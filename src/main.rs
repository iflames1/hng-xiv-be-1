mod app;
mod db;
mod error;
mod external;
mod handlers;
mod models;
mod state;
mod utils;

#[tokio::main]
async fn main() -> Result<(), error::AppError> {
    app::run().await
}
