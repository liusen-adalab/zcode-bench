use std::{
    borrow::Cow,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use anyhow::Result;

use path_slash::PathBufExt;
use protocol::http::Response;
use serde::{Deserialize, Serialize};
use tauri::Runtime;

use tracing::{debug, info, instrument};

use crate::{
    file_system::upload::UploadClient, get, my_err::MyResult, post, settings::RemoteServerConfig,
};

mod upload;

#[derive(Deserialize, Debug, Clone)]
pub struct UnixPath(PathBuf);

impl Serialize for UnixPath {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&*self.to_string_lossy())
    }
}

impl UnixPath {
    fn to_string_lossy(&self) -> Cow<str> {
        self.0.to_slash_lossy()
    }

    fn join(&self, path: impl AsRef<Path>) -> Self {
        Self(self.0.join(path))
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct FileNode {
    #[serde(alias = "label")]
    name: String,
    path: UnixPath,
    last_modified: String,
    children: Option<Vec<FileNode>>,
}

#[tauri::command]
pub async fn load_dir_tree() -> MyResult<FileNode> {
    debug!("loading");
    let tree: Response<FileNode> = get!(RemoteServerConfig::api_load_structure());
    let tree = tree.to_result()?.expect("expect root file node");
    Ok(tree)
}

#[instrument()]
#[tauri::command]
pub async fn load_dir_content(path: PathBuf) -> MyResult<Vec<FileNode>> {
    debug!("loading");
    let nodes: Response<Vec<FileNode>> =
        get!(RemoteServerConfig::api_load_dir_content(), query: {"path": path});
    let nodes = nodes.to_result()?.unwrap();
    Ok(nodes)
}

#[instrument]
#[tauri::command]
pub async fn create_dir(path: &Path) -> MyResult<()> {
    debug!("creating dir");
    let url = RemoteServerConfig::url_create_dir();
    let res: Response<()> = post!(url, body: {"path": path});
    res.to_result()?;

    Ok(())
}

#[instrument]
#[tauri::command]
pub async fn move_to(from: &Path, to_dir: &Path) -> MyResult<()> {
    debug!("moving");

    let to = to_dir.join(get_file_name(from)?);
    let to = to.to_slash_lossy();

    let url = RemoteServerConfig::url_move();
    let res: Response<()> = post!(url, body: {"from": from, "to": to});
    res.to_result()?;

    Ok(())
}

#[tauri::command]
pub async fn delete_file(path: &Path) -> MyResult<()> {
    debug!("deleting");
    let url = RemoteServerConfig::url_delete_file();
    let res: Response<()> = post!(url, body: {"path": path});
    res.to_result()?;

    Ok(())
}

#[tauri::command]
pub async fn upload_file<R: Runtime>(
    window: tauri::Window<R>,
    local_path: PathBuf,
    to_dir: UnixPath,
) -> MyResult<String> {
    info!(?local_path, ?to_dir, "uploading");
    let dst = to_dir.join(get_file_name(&local_path)?);
    let client = UploadClient::new(local_path, dst, window).await?;
    let event_key = client.task_event_key.clone();
    client.run();

    debug!(%event_key);
    Ok(event_key)
}

fn get_file_name(path: &Path) -> Result<&OsStr> {
    path.file_name()
        .ok_or_else(|| ::anyhow::anyhow!("no file name"))
}
