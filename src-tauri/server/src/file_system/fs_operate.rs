use anyhow::Result;
use chrono::{DateTime, Local};
use path_slash::PathExt;
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tracing::info;

pub trait PathToString {
    fn to_lossy_string(&self) -> String;
}

impl<P: AsRef<Path>> PathToString for P {
    fn to_lossy_string(&self) -> String {
        let path = self.as_ref();
        let path = path.to_slash_lossy();
        path.to_string()
    }
}

#[derive(Debug, Serialize)]
pub struct FileNode {
    label: String,
    path: PathBuf,
    last_modified: String,
    children: Option<Vec<FileNode>>,
}

impl FileNode {
    async fn from_tokio_entry(entry: tokio::fs::DirEntry) -> Result<Self> {
        let meta = entry.metadata().await?;

        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path().to_owned();
        let modified = meta.modified()?;
        let modified = DateTime::<Local>::from(modified);
        let children = if meta.is_dir() { Some(vec![]) } else { None };
        Ok(Self {
            label: name,
            path,
            last_modified: modified.timestamp().to_string(),
            children,
        })
    }

    fn from_fs_entry(entry: std::fs::DirEntry) -> Result<Self> {
        let meta = entry.metadata()?;

        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path().to_owned();
        let modified = meta.modified()?;
        let modified = DateTime::<Local>::from(modified);
        let children = if meta.is_dir() { Some(vec![]) } else { None };
        Ok(Self {
            label: name,
            path,
            last_modified: modified.timestamp().to_string(),
            children,
        })
    }
}

pub async fn load_tree(root: PathBuf) -> Result<FileNode> {
    info!(?root, "loading tree");
    let tree = {
        let root = root.to_owned();
        tokio::task::spawn_blocking(move || load_fs_tree_inner(&root, false))
            .await
            .unwrap()?
    };
    Ok(tree)
}

pub async fn load_dir_tree(root: &Path) -> Result<FileNode> {
    info!(?root, "loading dir content");
    let tree = {
        let root = root.to_owned();
        tokio::task::spawn_blocking(move || load_fs_tree_inner(&root, true))
            .await
            .unwrap()?
    };
    Ok(tree)
}

pub async fn load_in_dir(path: &Path) -> Result<Vec<FileNode>> {
    use tokio::fs;

    let mut dir = fs::read_dir(path).await?;
    let mut nodes = vec![];
    while let Some(entry) = dir.next_entry().await? {
        nodes.push(FileNode::from_tokio_entry(entry).await?)
    }
    Ok(nodes)
}

pub fn load_fs_tree_inner(root: &Path, only_dir: bool) -> Result<FileNode> {
    let mut children = Vec::new();

    for entry in fs::read_dir(root)? {
        let entry = entry?;

        let f_type = entry.file_type()?;

        if f_type.is_file() && !only_dir {
            children.push(FileNode::from_fs_entry(entry)?)
        } else if f_type.is_dir() {
            let node = load_fs_tree_inner(&entry.path(), only_dir)?;
            children.push(node);
        }
    }
    let last_modified = {
        let t = root.metadata()?.modified()?;
        DateTime::<Local>::from(t)
    };

    Ok(FileNode {
        label: root.file_name().unwrap().to_lossy_string(),
        children: Some(children),
        path: root.to_owned(),
        last_modified: last_modified.timestamp().to_string(),
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn t_load_fs_tree() -> Result<()> {
        let root = "./icons";
        let tree = load_tree(Path::new(root).to_owned()).await?;
        dbg!(tree);
        Ok(())
    }
}
