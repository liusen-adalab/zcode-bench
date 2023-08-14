use anyhow::Result;
use server::{listener, restful_api, settings::load_setttings};
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<()> {
    load_setttings()?;

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    info!("start tcp server");

    let rx = listener::start_listener();
    rx.await??;

    info!("start http server");
    let server = restful_api::build_server().await?;
    server.await?;

    std::future::pending().await
}
