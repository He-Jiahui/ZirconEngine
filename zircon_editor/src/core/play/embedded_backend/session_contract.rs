use std::path::{Path, PathBuf};
use std::sync::Arc;

use zircon_runtime_interface::project::RelPath;

use crate::core::gateway::SharedEditorRuntimeGateway;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaySessionLaunchRequest {
    project_root: PathBuf,
    scene: RelPath,
}

impl PlaySessionLaunchRequest {
    pub fn new(project_root: impl Into<PathBuf>, scene: RelPath) -> Self {
        Self {
            project_root: project_root.into(),
            scene,
        }
    }

    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub fn scene(&self) -> &RelPath {
        &self.scene
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlaySessionRetireReport {
    pub diagnostics: Vec<String>,
}

/// Opaque App-owned runtime session exposed to Editor only as a gateway and retirement lease.
pub trait PlaySessionLease: Send {
    fn gateway(&self) -> SharedEditorRuntimeGateway;

    fn retire(&mut self) -> Result<PlaySessionRetireReport, String>;
}

/// App composition authority for creating one isolated runtime-profile Play session.
pub trait PlaySessionFactory: Send + Sync {
    fn create(
        &self,
        request: &PlaySessionLaunchRequest,
    ) -> Result<Box<dyn PlaySessionLease>, String>;
}

pub type SharedPlaySessionFactory = Arc<dyn PlaySessionFactory>;
