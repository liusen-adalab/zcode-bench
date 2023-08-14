use anyhow::Result;
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::oneshot,
};
use tokio_util::codec::Framed;
use tracing::info;

use crate::{file_system::upload, log_if_err, settings::get_settings};

#[derive(Deserialize)]
pub struct TcpListenerConfig {
    pub bind: String,
}

pub fn start_listener() -> oneshot::Receiver<Result<()>> {
    let settings = &get_settings().tcp_server;
    let (tx, rx) = oneshot::channel();
    tokio::spawn(async move {
        let listener = match TcpListener::bind(&settings.bind).await {
            Ok(l) => {
                let _ = tx.send(Ok(()));
                l
            }
            Err(err) => {
                let _ = tx.send(Err(err.into()));
                return;
            }
        };

        loop {
            log_if_err!(listen_inner(&listener).await);
        }
    });

    rx
}

async fn listen_inner(listener: &TcpListener) -> Result<()> {
    let (stream, peer_addr) = listener.accept().await?;
    info!(?peer_addr, "new tcp connection");

    switch_client(stream).await?;

    Ok(())
}

pub async fn switch_client(stream: TcpStream) -> Result<()> {
    use protocol::register_client::ServerCodec;

    let mut framed = Framed::new(stream, ServerCodec::new());
    let msg = framed
        .next()
        .await
        .ok_or_else(|| anyhow::anyhow!("no handshake msg"))??;

    framed
        .send(protocol::register_client::RegisterResult::Ok)
        .await?;

    match msg {
        protocol::register_client::RegisterClientReq::SwitchProtocol(client_type) => {
            match client_type {
                protocol::register_client::ClientType::Upload => {
                    upload::client_manager().spawn_client(framed.into_inner());
                }
            }
        }
    }
    Ok(())
}
