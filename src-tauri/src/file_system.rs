use std::{path::PathBuf, sync::atomic::AtomicU32, time::Duration};

use anyhow::{bail, ensure, Context, Result};
use bytes::BytesMut;
use futures::{SinkExt, StreamExt};
use protocol::{
    register_client::ClientType,
    upload::{ClientCodec, UploadRequest},
};
use serde::{Deserialize, Serialize};
use tauri::{Runtime, Window};
use tokio::{fs::File, io::AsyncReadExt, net::TcpStream};
use tokio_util::codec::Framed;
use tracing::{debug, error, info, warn};

use crate::{client, get, log_if_err, my_err::MyResult, settings::RemoteServerConfig};

#[derive(Debug, Deserialize, Serialize)]
pub struct FileNode {
    label: String,
    path: String,
    children: Option<Vec<FileNode>>,
}

#[tauri::command]
pub async fn load_dir_tree() -> MyResult<FileNode> {
    let tree = get!(RemoteServerConfig::api_load_dir_tree());
    Ok(tree)
}

#[tauri::command]
pub async fn load_dir_content(path: PathBuf) -> MyResult<Vec<FileNode>> {
    let tree = get!(RemoteServerConfig::api_load_dir_tree(), query: {"path": path});
    Ok(tree)
}

struct UploadClient<R: Runtime> {
    task_id: u32,
    local_path: PathBuf,
    remote_path: PathBuf,
    framed: Framed<TcpStream, ClientCodec>,
    window: Window<R>,
}

impl<R: Runtime> UploadClient<R> {
    async fn new(src: PathBuf, dst: PathBuf, window: tauri::Window<R>) -> anyhow::Result<Self> {
        let framed = client::build_client_frame(ClientCodec::new(), ClientType::Upload)
            .await
            .context("connect server")?;
        let mut this = Self {
            local_path: src,
            framed,
            window,
            task_id: next_load_task_id(),
            remote_path: dst,
        };
        this.handshake().await?;
        Ok(this)
    }

    async fn handshake(&mut self) -> Result<()> {
        self.framed
            .send(UploadRequest::Register(self.remote_path.clone()))
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

    fn run(self) {
        tokio::spawn(async move { log_if_err!(self.runn_inner().await) });
    }

    async fn runn_inner(mut self) -> Result<()> {
        tokio::time::sleep(Duration::from_secs(1)).await;

        debug!(?self.local_path, "sending file");
        let mut file = File::open(&self.local_path).await?;
        let size = file.metadata().await?.len();
        let event_key = format!("slice-uploaded-{}", self.task_id);

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
                .emit(&event_key, UploadEvent { percent })
                .unwrap();

            bytes.clear();
        }

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
}

pub fn next_load_task_id() -> u32 {
    static ID: AtomicU32 = AtomicU32::new(0);
    ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

#[tauri::command]
pub async fn upload_file<R: Runtime>(
    window: tauri::Window<R>,
    local_path: PathBuf,
    remote_path: PathBuf,
) -> MyResult<u32> {
    let client = UploadClient::new(local_path, remote_path, window).await?;
    let task_id = client.task_id;
    client.run();

    Ok(task_id)
}
