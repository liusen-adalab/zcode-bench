use anyhow::Result;
use server::listener;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    info!("start server");

    listener::start_listener("0.0.0.0:5987".to_string()).await?;

    std::future::pending().await
}
