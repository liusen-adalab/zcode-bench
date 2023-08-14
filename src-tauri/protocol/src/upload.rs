use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum UploadRequest {
    Register(PathBuf),
    Upload(Vec<u8>),
}

#[derive(Serialize, Deserialize)]
pub enum UploadResponse {
    RegisterResult(bool),
}

crate::impl_codec!(UploadRequest, UploadResponse);
