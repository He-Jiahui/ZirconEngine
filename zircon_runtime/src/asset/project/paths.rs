use std::fs;
use std::path::{Path, PathBuf};

use zircon_runtime_interface::project::RelPath;

/// Canonical project paths. All regenerable state lives below `.zircon`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectPaths {
    root: PathBuf,
    manifest: PathBuf,
    derived_root: PathBuf,
    cache_root: PathBuf,
    asset_artifact_root: PathBuf,
    registry_root: PathBuf,
    autosave_root: PathBuf,
    play_root: PathBuf,
    thumbnails_root: PathBuf,
}

impl ProjectPaths {
    pub fn from_root(root: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let root = root.as_ref();
        let root = if root.is_absolute() {
            root.to_path_buf()
        } else {
            std::env::current_dir()?.join(root)
        };
        let derived_root = root.join(".zircon");
        let cache_root = derived_root.join("cache");
        Ok(Self {
            manifest: root.join("zircon-project.toml"),
            asset_artifact_root: cache_root.join("assets"),
            registry_root: derived_root.join("registry"),
            autosave_root: derived_root.join("autosave"),
            play_root: derived_root.join("play"),
            thumbnails_root: derived_root.join("thumbnails"),
            cache_root,
            derived_root,
            root,
        })
    }

    pub fn ensure_layout(&self, asset_roots: &[RelPath]) -> Result<(), std::io::Error> {
        self.ensure_derived_layout()?;
        self.ensure_asset_roots(asset_roots)
    }

    pub fn ensure_derived_layout(&self) -> Result<(), std::io::Error> {
        for root in [
            &self.cache_root,
            &self.asset_artifact_root,
            &self.registry_root,
            &self.autosave_root,
            &self.play_root,
            &self.thumbnails_root,
        ] {
            fs::create_dir_all(root)?;
        }
        Ok(())
    }

    pub fn ensure_asset_roots(&self, roots: &[RelPath]) -> Result<(), std::io::Error> {
        for root in roots {
            fs::create_dir_all(root.join_to(&self.root))?;
        }
        Ok(())
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn manifest_path(&self) -> &Path {
        &self.manifest
    }

    pub fn asset_root(&self, relative: &RelPath) -> PathBuf {
        relative.join_to(&self.root)
    }

    pub fn derived_root(&self) -> &Path {
        &self.derived_root
    }

    pub fn cache_root(&self) -> &Path {
        &self.cache_root
    }

    pub fn asset_artifact_root(&self) -> &Path {
        &self.asset_artifact_root
    }

    pub fn registry_root(&self) -> &Path {
        &self.registry_root
    }

    pub fn autosave_root(&self) -> &Path {
        &self.autosave_root
    }

    pub fn play_root(&self) -> &Path {
        &self.play_root
    }

    pub fn thumbnails_root(&self) -> &Path {
        &self.thumbnails_root
    }
}
