use std::{path::PathBuf, sync::atomic::AtomicU32, time::Duration};

use anyhow::{bail, ensure, Context, Result};
use bytes::BytesMut;
use futures::{SinkExt, StreamExt};
use protocol::{
    register_client::ClientType,
    upload::{ClientCodec, UploadRequest},
};
use serde::Serialize;
use tauri::{Runtime, Window};
use tokio::{fs::File, io::AsyncReadExt, net::TcpStream};
use tokio_util::codec::Framed;
use tracing::{debug, error, info, warn};

use crate::{client, log_if_err};

use super::UnixPath;

pub struct UploadClient<R: Runtime> {
    pub task_event_key: String,
    local_path: PathBuf,
    dst_path: UnixPath,
    framed: Framed<TcpStream, ClientCodec>,
    window: Window<R>,
}

impl<R: Runtime> UploadClient<R> {
    pub async fn new(
        src: PathBuf,
        dst: UnixPath,
        window: tauri::Window<R>,
    ) -> anyhow::Result<Self> {
        let framed = client::build_client_frame(ClientCodec::new(), ClientType::Upload)
            .await
            .context("connect server")?;
        let mut this = Self {
            local_path: src,
            framed,
            window,
            task_event_key: next_event_key(),
            dst_path: dst,
        };
        this.handshake().await?;
        Ok(this)
    }

    async fn handshake(&mut self) -> Result<()> {
        self.framed
            .send(UploadRequest::Register(
                self.dst_path.to_string_lossy().to_string(),
            ))
            .await?;
        match self.framed.next().await {
            Some(Ok(msg)) => match msg {
                protocol::upload::UploadResponse::RegisterResult(ok) => {
                    ensure!(ok, "server rejected upload client handshake")
                }
            },
            Some(Err(err)) => {
                error!(?err);
                bail!("failed to decode server handshake msg: {}", err)
            }
            None => {
                bail!("server didn't send handshake result")
            }
        }
        debug!("handshake ok");
        Ok(())
    }

    pub fn run(self) {
        tokio::spawn(async move { log_if_err!(self.runn_inner().await) });
    }

    async fn runn_inner(mut self) -> Result<()> {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let win_name = self.window.label();

        debug!(?self.local_path, %win_name, "sending file");
        let mut file = File::open(&self.local_path).await?;
        let size = file.metadata().await?.len();

        let mut bytes = BytesMut::with_capacity(500);
        let mut read_size = 0;

        loop {
            let len = file.read_buf(&mut bytes).await?;
            if len == 0 {
                break;
            }
            read_size += len;

            // send to server
            self.framed
                .send(UploadRequest::Upload(bytes.to_vec()))
                .await?;

            let percent = format!("{:.02}", read_size as f64 / size as f64 * 100.0);
            // nofity frontend
            self.window
                .emit(
                    &self.task_event_key,
                    UploadEvent {
                        percent,
                        is_done: false,
                    },
                )
                .unwrap();

            bytes.clear();
        }

        self.window
            .emit(
                &self.task_event_key,
                UploadEvent {
                    percent: "100".to_string(),
                    is_done: true,
                },
            )
            .unwrap();

        if read_size as u64 != size {
            warn!("file size mismatch!");
        }

        info!("send file done");

        Ok(())
    }
}

#[derive(Serialize, Clone)]
pub struct UploadEvent {
    percent: String,
    is_done: bool,
}

fn next_load_task_id() -> u32 {
    static ID: AtomicU32 = AtomicU32::new(0);
    ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

fn next_event_key() -> String {
    format!("upload-progress-{}", next_load_task_id())
}
