use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::asset::AssetImportError;

use super::{ProjectManager, ProjectManifest};

/// Immutable filesystem and manifest identity captured before project activation.
///
/// Only [`PreparedProjectOpen`] constructs this witness. Consumers must revalidate the
/// witness immediately before an activation transaction rather than reopening a project root.
#[derive(Debug)]
pub struct ProjectRootWitness {
    root: PathBuf,
    identity: ProjectRootIdentity,
    manifest: ProjectManifest,
}

impl ProjectRootWitness {
    fn capture(root: PathBuf, manifest: ProjectManifest) -> std::io::Result<Self> {
        Ok(Self {
            identity: ProjectRootIdentity::from_path(&root)?,
            root,
            manifest,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn manifest(&self) -> &ProjectManifest {
        &self.manifest
    }

    pub fn revalidate(&self) -> Result<(), PreparedProjectOpenStale> {
        let canonical_root = fs::canonicalize(&self.root).map_err(|source| {
            PreparedProjectOpenStale::RecanonicalizeRoot {
                root: self.root.clone(),
                source,
            }
        })?;
        let identity = ProjectRootIdentity::from_path(&canonical_root).map_err(|source| {
            PreparedProjectOpenStale::RecanonicalizeRoot {
                root: self.root.clone(),
                source,
            }
        })?;
        if canonical_root != self.root || identity != self.identity {
            return Err(PreparedProjectOpenStale::RootReplaced {
                expected_root: self.root.clone(),
                actual_root: canonical_root,
            });
        }

        let manifest_path = self.root.join("zircon-project.toml");
        let manifest = ProjectManifest::load(&manifest_path).map_err(|source| {
            PreparedProjectOpenStale::ManifestLoad {
                manifest_path: manifest_path.clone(),
                source,
            }
        })?;
        if manifest != self.manifest {
            return Err(PreparedProjectOpenStale::ManifestChanged { manifest_path });
        }
        Ok(())
    }
}

#[cfg(windows)]
#[derive(Debug, PartialEq, Eq)]
struct ProjectRootIdentity {
    volume_serial_number: Option<u32>,
    file_index: Option<u64>,
}

#[cfg(windows)]
impl ProjectRootIdentity {
    fn from_path(path: &Path) -> std::io::Result<Self> {
        use std::os::windows::fs::MetadataExt;

        let metadata = fs::metadata(path)?;
        Ok(Self {
            volume_serial_number: metadata.volume_serial_number(),
            file_index: metadata.file_index(),
        })
    }
}

#[cfg(unix)]
#[derive(Debug, PartialEq, Eq)]
struct ProjectRootIdentity {
    device: u64,
    inode: u64,
}

#[cfg(unix)]
impl ProjectRootIdentity {
    fn from_path(path: &Path) -> std::io::Result<Self> {
        use std::os::unix::fs::MetadataExt;

        let metadata = fs::metadata(path)?;
        Ok(Self {
            device: metadata.dev(),
            inode: metadata.ino(),
        })
    }
}

#[cfg(not(any(unix, windows)))]
#[derive(Debug, PartialEq, Eq)]
struct ProjectRootIdentity {
    creation_time: Option<std::time::SystemTime>,
}

#[cfg(not(any(unix, windows)))]
impl ProjectRootIdentity {
    fn from_path(path: &Path) -> std::io::Result<Self> {
        Ok(Self {
            creation_time: fs::metadata(path)?.created().ok(),
        })
    }
}

/// A move-only runtime project capability prepared before activation.
///
/// The capability owns the only pre-activation `ProjectManager` together with the filesystem
/// witness that proves the parsed project is still current. It deliberately offers no public
/// manager accessor, so Editor, dynamic startup, and asset activation cannot fork project truth.
#[derive(Debug)]
pub struct PreparedProjectOpen {
    witness: ProjectRootWitness,
    project: ProjectManager,
}

impl PreparedProjectOpen {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, AssetImportError> {
        let requested_root = root.as_ref().to_path_buf();
        let canonical_root = fs::canonicalize(&requested_root).map_err(|source| {
            AssetImportError::CanonicalProjectRoot {
                path: requested_root,
                source,
            }
        })?;
        let project = ProjectManager::open(&canonical_root)?;
        let witness = ProjectRootWitness::capture(canonical_root, project.manifest().clone())?;
        Ok(Self { witness, project })
    }

    pub fn root(&self) -> &Path {
        self.witness.root()
    }

    pub fn manifest(&self) -> &ProjectManifest {
        self.witness.manifest()
    }

    pub fn revalidate(&self) -> Result<(), PreparedProjectOpenStale> {
        self.witness.revalidate()
    }

    pub(crate) fn into_parts(self) -> (ProjectRootWitness, ProjectManager) {
        (self.witness, self.project)
    }
}

#[derive(Debug, Error)]
pub enum PreparedProjectOpenStale {
    #[error("prepared project root {root} can no longer be canonicalized: {source}")]
    RecanonicalizeRoot {
        root: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(
        "prepared project root changed from {expected_root} to canonical target {actual_root}"
    )]
    RootReplaced {
        expected_root: PathBuf,
        actual_root: PathBuf,
    },
    #[error("prepared project manifest {manifest_path} can no longer be loaded: {source}")]
    ManifestLoad {
        manifest_path: PathBuf,
        #[source]
        source: super::ProjectManifestError,
    },
    #[error("prepared project manifest {manifest_path} changed before activation")]
    ManifestChanged { manifest_path: PathBuf },
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::asset::AssetUri;

    use super::{PreparedProjectOpen, PreparedProjectOpenStale, ProjectManifest};

    #[test]
    fn prepared_project_open_rejects_a_manifest_rewrite_before_activation() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zircon_prepared_project_open_{}_{}",
            std::process::id(),
            unique
        ));
        std::fs::create_dir_all(&root).unwrap();
        let manifest_path = root.join("zircon-project.toml");
        ProjectManifest::new(
            "Prepared Project One",
            AssetUri::parse("res://scenes/one.scene.toml").unwrap(),
            1,
        )
        .save(&manifest_path)
        .unwrap();

        let prepared = PreparedProjectOpen::open(&root).unwrap();
        ProjectManifest::new(
            "Prepared Project Two",
            AssetUri::parse("res://scenes/two.scene.toml").unwrap(),
            2,
        )
        .save(&manifest_path)
        .unwrap();

        assert!(matches!(
            prepared.revalidate(),
            Err(PreparedProjectOpenStale::ManifestChanged { .. })
        ));

        drop(prepared);
        let _ = std::fs::remove_dir_all(root);
    }
}
