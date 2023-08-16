use anyhow::{bail, Context, Result};
use futures::{SinkExt, StreamExt};
use protocol::register_client::{ClientCodec, ClientType, RegisterClientReq};
use tokio::net::TcpStream;
use tokio_util::codec::{Decoder, Framed};

use crate::settings::get_settings;

pub async fn build_client_frame<C>(
    codec: C,
    client_type: ClientType,
) -> Result<Framed<TcpStream, C>> {
    let server = &get_settings().remote_server.tcp;

    let stream = TcpStream::connect(server).await?;
    let mut framed = ClientCodec::new().framed(stream);

    framed
        .send(RegisterClientReq::SwitchProtocol(client_type))
        .await
        .context("send switch msg")?;

    match framed.next().await {
        Some(Ok(msg)) => match msg {
            protocol::register_client::RegisterResult::Ok => {
                Ok(Framed::new(framed.into_inner(), codec))
            }
            protocol::register_client::RegisterResult::Err(reason) => {
                bail!("server rejected register request. reason = {:?}", reason)
            }
        },
        Some(Err(err)) => {
            bail!("failed to decode server msg: {err}");
        }
        None => {
            bail!("server closed connection early");
        }
    }
}
