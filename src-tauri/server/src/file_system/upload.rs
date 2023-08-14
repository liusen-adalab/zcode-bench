use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use anyhow::{bail, Context, Result};
use futures::{SinkExt, StreamExt};
use protocol::upload::{ServerCodec, UploadRequest, UploadResponse};
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
    net::TcpStream,
    sync::{oneshot, Mutex},
};
use tokio_util::codec::{Decoder, Framed};
use tracing::{debug, warn};
use tracing::{error, info};

#[macro_export]
macro_rules! log_if_err {
    ($run:expr) => {
        crate::log_if_err!($run, stringify!($run))
    };

    ($run:expr, $msg:expr $(,)?) => {
        if let Err(err) = $run {
            ::tracing::error!(?err, concat!("FAILED: ", $msg))
        }
    };
}

pub struct UploadClient {
    dst: PathBuf,
    stream: Framed<TcpStream, ServerCodec>,
    kill_handler: oneshot::Receiver<oneshot::Sender<()>>,
}

type UploadClientHandler = oneshot::Sender<oneshot::Sender<()>>;

// upload / download
// transcode progress

impl UploadClient {
    async fn handshake(stream: TcpStream) -> Result<(Self, UploadClientHandler)> {
        let mut framed = ServerCodec::new().framed(stream);
        let msg = framed
            .next()
            .await
            .ok_or_else(|| anyhow::anyhow!("upload client didn't send handshake msg"))??;
        match msg {
            protocol::upload::UploadRequest::Register(path) => {
                // send result
                framed.send(UploadResponse::RegisterResult(true)).await?;

                let (tx, rx) = oneshot::channel();
                let this = Self {
                    dst: path.clone(),
                    stream: framed,
                    kill_handler: rx,
                };
                this.remove_old_file().await?;

                Ok((this, tx))
            }
            _ => bail!("upload client didn't follow protocol"),
        }
    }

    async fn remove_old_file(&self) -> Result<()> {
        info!(?self.dst, "removing old file");
        if fs::try_exists(&self.dst).await? {
            fs::remove_file(&self.dst)
                .await
                .context("remove old uploed file")?;
        }
        Ok(())
    }

    pub fn run(mut self) {
        tokio::spawn(async move {
            log_if_err!(self.run_inner().await);
            client_manager().client_shutdown(&self.dst).await;
        });
    }

    async fn run_inner(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                content = self.stream.next() => {
                    match content {
                        Some(Ok(req)) => {
                            self.handle_req(req).await?;
                        }
                        Some(Err(err)) => {
                            error!(?err, "failed to read client tcp stream");
                            break;
                        }
                        None => {
                            info!(?self.dst, "all content uploaded!");
                            break
                        }
                    }
                }
                _ = &mut self.kill_handler => {
                    warn!(?self.dst, "client killed by other client");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_req(&self, req: UploadRequest) -> Result<()> {
        match req {
            UploadRequest::Register(_) => {
                warn!("unexpect msg")
            }
            UploadRequest::Upload(bytes) => self.write_file(&bytes).await?,
        }

        Ok(())
    }

    async fn write_file(&self, bytes: &[u8]) -> Result<()> {
        info!(?self.dst, "writing byte chunk");
        let mut options = OpenOptions::new();
        let mut file = options
            .create(true)
            .append(true)
            .open(&self.dst)
            .await
            .context("open file")?;
        file.write_all(bytes).await.context("write file")?;

        Ok(())
    }
}

pub struct ClienManager {
    running_clients: Mutex<HashMap<PathBuf, UploadClientHandler>>,
}

impl ClienManager {
    pub fn next_client_token(&self) -> u32 {
        todo!()
    }

    pub fn spawn_client(&'static self, stream: TcpStream) {
        tokio::spawn(async move {
            log_if_err!(self.spawn_client_inner(stream).await);
        });
    }

    async fn client_shutdown(&self, path: &Path) {
        info!(?path, "client shutdown");
        let mut lock = self.running_clients.lock().await;
        lock.remove(path);
    }

    async fn kill_running(&self, path: &Path) {
        let mut lock = self.running_clients.lock().await;
        debug!("killing duplicated client");
        if let Some(client) = lock.remove(path) {
            let (tx, rx) = oneshot::channel();
            if let Err(_) = client.send(tx) {
                info!("client is already dead");
                return;
            }
            let _ = rx.await;
        }
    }

    async fn spawn_client_inner(&self, stream: TcpStream) -> Result<()> {
        let (client, handler) = UploadClient::handshake(stream).await?;
        self.kill_running(&client.dst).await;

        let mut lock = self.running_clients.lock().await;
        lock.insert(client.dst.to_path_buf(), handler);

        client.run();

        let paths = lock.keys();
        info!(?paths, "new client spawned");

        Ok(())
    }
}

pub fn client_manager() -> &'static ClienManager {
    static MANAGER: OnceLock<ClienManager> = OnceLock::new();
    MANAGER.get_or_init(|| ClienManager {
        running_clients: Default::default(),
    })
}
