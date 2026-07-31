use std::path::{Path, PathBuf};

use crate::core::editor_message::DocumentId;

use super::{PlayKind, PlaySceneSource};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlayStartRequest {
    pub kind: PlayKind,
    pub project_root: Option<PathBuf>,
    pub requires_build: bool,
    pub scene_source: Option<PlaySceneSource>,
    pub running_document: Option<DocumentId>,
}

impl PlayStartRequest {
    pub fn immediate(kind: PlayKind, project_root: Option<&Path>) -> Self {
        Self {
            kind,
            project_root: project_root.map(Path::to_path_buf),
            requires_build: false,
            scene_source: None,
            running_document: None,
        }
    }

    pub fn after_build(kind: PlayKind, project_root: Option<&Path>) -> Self {
        Self {
            kind,
            project_root: project_root.map(Path::to_path_buf),
            requires_build: true,
            scene_source: None,
            running_document: None,
        }
    }

    pub fn with_scene_source(mut self, source: PlaySceneSource) -> Self {
        self.scene_source = Some(source);
        self
    }

    pub fn with_running_document(mut self, document: DocumentId) -> Self {
        self.running_document = Some(document);
        self
    }
}
