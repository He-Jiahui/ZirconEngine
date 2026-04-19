use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectPaths {
    root: PathBuf,
    manifest: PathBuf,
    assets_root: PathBuf,
    library_root: PathBuf,
}

impl ProjectPaths {
    pub fn from_root(root: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let root = root.as_ref();
        let root = if root.is_absolute() {
            root.to_path_buf()
        } else {
            std::env::current_dir()?.join(root)
        };
        Ok(Self {
            manifest: root.join("zircon-project.toml"),
            assets_root: root.join("assets"),
            library_root: root.join("library"),
            root,
        })
    }

    pub fn ensure_layout(&self) -> Result<(), std::io::Error> {
        fs::create_dir_all(&self.assets_root)?;
        fs::create_dir_all(&self.library_root)?;
        Ok(())
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn manifest_path(&self) -> &Path {
        &self.manifest
    }

    pub fn assets_root(&self) -> &Path {
        &self.assets_root
    }

    pub fn library_root(&self) -> &Path {
        &self.library_root
    }
}
