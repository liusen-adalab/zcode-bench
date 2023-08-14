use anyhow::Result;
use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Framed;
use tracing::info;

use crate::{file_system::upload, log_if_err};

pub async fn start_listener(addr: String) -> Result<()> {
    let listener = TcpListener::bind(&addr).await?;

    tokio::spawn(async move {
        let mut listener = Some(listener);
        loop {
            let listener = match listener.take() {
                Some(l) => l,
                None => TcpListener::bind(&addr).await.unwrap(),
            };

            log_if_err!(listen_inner(listener).await);
        }
    });

    Ok(())
}

async fn listen_inner(listener: TcpListener) -> Result<()> {
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
