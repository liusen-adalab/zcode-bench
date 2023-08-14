use std::path::PathBuf;

use serde::Deserialize;

pub mod fs_operate;
pub mod restful;
pub mod upload;

pub use restful::*;

#[derive(Deserialize)]
pub struct FileSystemConfig {
    pub root: PathBuf,
}
