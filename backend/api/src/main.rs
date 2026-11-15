use tracing_subscriber::EnvFilter;
#[tokio::main]

async fn main() {
    // 1. Initialise tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // 2. Bind TCP listener
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind TCP listener");

    // 3. Log listening message
    tracing::info!("Listening on http://127.0.0.1:3000");

    // 4. Serve application
    let app = api::build_app();

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}