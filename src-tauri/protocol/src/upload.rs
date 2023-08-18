use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum UploadRequest {
    Register(String),
    Upload(Vec<u8>),
}

#[derive(Serialize, Deserialize)]
pub enum UploadResponse {
    RegisterResult(bool),
}

crate::impl_codec!(UploadRequest, UploadResponse);
