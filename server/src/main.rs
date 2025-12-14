use server::{Settings, app, services::data::service::DataService};
use std::{env, sync::Arc};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    dotenv::dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| "8081".to_string());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(
        listener,
        app(
            Arc::new(DataService),
            Arc::new(Settings {
                replay_dir: "./replays".into(),
            }),
        ),
    )
    .await
    .unwrap();
}
