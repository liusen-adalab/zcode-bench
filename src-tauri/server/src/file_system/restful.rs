use std::path::PathBuf;

use crate::{api_ok, file_system::fs_operate, restful_api::ApiResponse, settings::get_settings};

use super::fs_operate::FileNode;
use actix_web::web::Query;
use serde::Deserialize;

pub async fn load_fs_dir_tree() -> ApiResponse<FileNode> {
    let root = &get_settings().file_system.root;
    let tree = fs_operate::load_dir_tree(root).await?;
    api_ok!(tree)
}

#[derive(Deserialize)]
pub struct LoadInDirParams {
    path: PathBuf,
}

pub async fn load_in_dir(params: Query<LoadInDirParams>) -> ApiResponse<Vec<FileNode>> {
    let nodes = fs_operate::load_in_dir(&params.path).await?;
    api_ok!(nodes)
}
