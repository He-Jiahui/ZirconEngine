use super::RuntimeAssetManager;
use crate::asset::artifact::RenderArtifactStore;
#[cfg(feature = "native-host")]
pub use crate::asset::pipeline::manager::{
    ProjectAssetGenerationToken as RuntimeAssetGenerationToken,
    ProjectGenerationMatch as RuntimeGenerationMatch,
};

#[cfg(feature = "native-host")]
impl RuntimeAssetManager {
    pub fn render_artifact_generation_snapshot(
        &self,
    ) -> Option<(RenderArtifactStore, RuntimeAssetGenerationToken)> {
        let (project, generation) = self.current_project_generation_snapshot()?.into_parts();
        Some((
            RenderArtifactStore::new(project.paths().asset_artifact_root().to_path_buf()),
            generation,
        ))
    }
    pub fn check_runtime_generation(
        &self,
        expected: &RuntimeAssetGenerationToken,
    ) -> RuntimeGenerationMatch {
        self.check_project_generation(expected)
    }
}

#[cfg(not(feature = "native-host"))]
#[derive(Clone, Debug)]
pub struct RuntimeAssetGenerationToken(pub(super) std::sync::Arc<std::sync::Mutex<bool>>);
#[cfg(not(feature = "native-host"))]
impl PartialEq for RuntimeAssetGenerationToken {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.0, &other.0)
    }
}
#[cfg(not(feature = "native-host"))]
impl Eq for RuntimeAssetGenerationToken {}
#[cfg(not(feature = "native-host"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeGenerationMatch {
    Current,
    Superseded { newer_same_project_generation: bool },
}
#[cfg(not(feature = "native-host"))]
impl RuntimeAssetManager {
    pub fn render_artifact_generation_snapshot(
        &self,
    ) -> Option<(RenderArtifactStore, RuntimeAssetGenerationToken)> {
        if !*self.generation.0.lock().unwrap_or_else(|e| e.into_inner()) {
            return None;
        }
        Some((
            self.source.render_artifact_store()?,
            self.generation.clone(),
        ))
    }
    pub fn check_runtime_generation(
        &self,
        expected: &RuntimeAssetGenerationToken,
    ) -> RuntimeGenerationMatch {
        if self.generation == *expected
            && *self.generation.0.lock().unwrap_or_else(|e| e.into_inner())
        {
            RuntimeGenerationMatch::Current
        } else {
            RuntimeGenerationMatch::Superseded {
                newer_same_project_generation: false,
            }
        }
    }
}
