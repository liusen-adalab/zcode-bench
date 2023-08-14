use anyhow::Result;
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
    path: String,
    children: Option<Vec<FileNode>>,
}

pub async fn load_fs_tree(root: PathBuf) -> Result<FileNode> {
    info!("loading {:?}", root);
    let tree = {
        let root = root.to_owned();
        tokio::task::spawn_blocking(move || load_fs_tree_inner(&root))
            .await
            .unwrap()?
    };
    Ok(tree)
}

pub fn load_fs_tree_inner(root: &Path) -> Result<FileNode> {
    let mut children = Vec::new();

    for entry in fs::read_dir(root)? {
        let entry = entry?;

        let f_type = entry.file_type()?;
        if f_type.is_file() {
            children.push(FileNode {
                label: entry.file_name().to_lossy_string(),
                children: None,
                path: entry.path().to_lossy_string(),
            })
        } else if f_type.is_dir() {
            let node = load_fs_tree_inner(&entry.path())?;
            children.push(node);
        }
    }

    Ok(FileNode {
        label: root.file_name().unwrap().to_lossy_string(),
        children: Some(children),
        path: root.to_lossy_string(),
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn t_load_fs_tree() -> Result<()> {
        let root = "./icons";
        let tree = load_fs_tree(Path::new(root).to_owned()).await?;
        dbg!(tree);
        Ok(())
    }
}
