use reqwest::Client;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub http_client: Client,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            http_client: Client::new(),
        }
    }
}
