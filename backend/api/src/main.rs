use api::config::Config;
use api::state::AppState;

use tracing_subscriber::EnvFilter;
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let config = Config::from_env().expect("invalid configuration");

    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let db = sea_orm::Database::connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    let state = AppState { db };

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port))
        .await
        .expect("Failed to bind TCP listener");

    tracing::info!(
        "Server listening on {}",
        listener.local_addr().expect("Failed to get local address")
    );

    let app = api::build_app(state);

    axum::serve(listener, app).await.expect("Server failed");
}
