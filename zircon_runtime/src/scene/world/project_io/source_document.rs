//! Authoring worlds and revisions are derived from the same source bytes, not an artifact cache.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::asset::assets::SceneAsset;
use crate::asset::project::{
    EditorDocumentCommitOutcome, ProjectManager, ProjectReferenceDiagnosticPhase,
};
use crate::asset::{AssetImportContext, AssetUri};
use zr_resource::io::source_revision::SourceSnapshot;

use super::super::World;
use super::SceneProjectError;

pub use zr_resource::io::source_revision::SourceRevision as SceneSourceRevision;

#[derive(Clone, Debug)]
pub struct LoadedSceneDocument {
    world: World,
    source_revision: SceneSourceRevision,
}

impl LoadedSceneDocument {
    pub fn into_parts(self) -> (World, SceneSourceRevision) {
        (self.world, self.source_revision)
    }
}

#[must_use]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenePublicationReceipt {
    source_path: PathBuf,
    source_revision: SceneSourceRevision,
    outcome: EditorDocumentCommitOutcome,
}

impl ScenePublicationReceipt {
    pub(crate) fn new(
        source_path: PathBuf,
        source_revision: SceneSourceRevision,
        outcome: EditorDocumentCommitOutcome,
    ) -> Self {
        Self {
            source_path,
            source_revision,
            outcome,
        }
    }

    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    pub fn source_revision(&self) -> &SceneSourceRevision {
        &self.source_revision
    }

    pub fn outcome(&self) -> &EditorDocumentCommitOutcome {
        &self.outcome
    }

    pub fn is_durable(&self) -> bool {
        matches!(self.outcome, EditorDocumentCommitOutcome::Durable)
    }

    pub fn ensure_durable(&self) -> Result<(), SceneProjectError> {
        if self.is_durable() {
            Ok(())
        } else {
            Err(SceneProjectError::PublicationRecoveryDeferred {
                receipt: Box::new(self.clone()),
            })
        }
    }
}

impl World {
    pub fn load_scene_document(
        project: &ProjectManager,
        uri: &AssetUri,
    ) -> Result<LoadedSceneDocument, SceneProjectError> {
        Self::load_scene_document_with_source(project, uri, || {
            let path = project.source_path_for_uri(uri)?;
            let snapshot = SourceSnapshot::read(&path)?;
            Ok((path, snapshot))
        })
    }

    /// Parses the exact source object opened by the caller's no-follow path lease.
    /// `source_path` must be the same path leased by that caller; it supplies parser context
    /// and diagnostics without reselecting or canonicalizing the source after it was opened.
    #[cfg(unix)]
    pub fn load_scene_document_from_snapshot(
        project: &ProjectManager,
        uri: &AssetUri,
        source_path: PathBuf,
        snapshot: SourceSnapshot,
    ) -> Result<LoadedSceneDocument, SceneProjectError> {
        Self::load_scene_document_with_source(project, uri, || Ok((source_path, snapshot)))
    }

    fn load_scene_document_with_source(
        project: &ProjectManager,
        uri: &AssetUri,
        read_source: impl FnOnce() -> Result<(PathBuf, SourceSnapshot), SceneProjectError>,
    ) -> Result<LoadedSceneDocument, SceneProjectError> {
        let result = (|| {
            let (path, snapshot) = read_source()?;
            let (bytes, source_revision) = snapshot.into_parts();
            let roots = Arc::new(
                project
                    .manifest()
                    .asset_roots
                    .iter()
                    .cloned()
                    .zip(project.project_asset_roots().iter().cloned())
                    .collect::<Vec<_>>(),
            );
            let context = AssetImportContext::new(path, uri.clone(), bytes, toml::Table::new())
                .with_project_resolver(project.asset_registry_shared(), roots);
            let scene = SceneAsset::from_project_toml_str(context.source_str()?, |reference| {
                context.resolve_project_asset_ref(reference)
            })?;
            let world = World::from_scene_asset(project, &scene)?;
            Ok(LoadedSceneDocument {
                world,
                source_revision,
            })
        })();
        super::scene_asset::publish_scene_reference_diagnostics(
            project,
            uri,
            ProjectReferenceDiagnosticPhase::Load,
            result.as_ref().err(),
        );
        result
    }
}

#[cfg(test)]
#[path = "tests/source_document.rs"]
mod tests;
