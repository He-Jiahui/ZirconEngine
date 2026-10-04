use std::sync::Arc;

use crate::graphics::scene::resources::GpuMeshResource;

use super::gpu_scene::GpuScene;

#[derive(Clone)]
pub(crate) struct GpuSceneSkinnedGpuSourceState {
    pub(crate) morph_shape_signature: u64,
    pub(crate) mesh: Arc<GpuMeshResource>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuScenePrevSkinnedSourceRollReport {
    pub(crate) current_source_count: usize,
    pub(crate) previous_source_count: usize,
    pub(crate) removed_previous_source_count: usize,
}

impl GpuScene {
    pub(crate) fn previous_skinned_gpu_source_state(
        &self,
        stable_instance_key: u64,
    ) -> Option<GpuSceneSkinnedGpuSourceState> {
        self.previous_skinned_gpu_sources
            .get(&stable_instance_key)
            .cloned()
    }

    pub(crate) fn stage_current_skinned_gpu_source(
        &mut self,
        stable_instance_key: u64,
        source: Option<GpuSceneSkinnedGpuSourceState>,
    ) {
        if let Some(source) = source {
            self.current_skinned_gpu_sources
                .insert(stable_instance_key, source);
        } else {
            self.current_skinned_gpu_sources
                .remove(&stable_instance_key);
        }
    }

    pub(crate) fn roll_prev_skinned_gpu_sources_after_success(
        &mut self,
    ) -> GpuScenePrevSkinnedSourceRollReport {
        let removed_previous_source_count = self
            .previous_skinned_gpu_sources
            .keys()
            .filter(|key| !self.current_skinned_gpu_sources.contains_key(*key))
            .count();
        self.previous_skinned_gpu_sources.clear();
        self.previous_skinned_gpu_sources.extend(
            self.current_skinned_gpu_sources
                .iter()
                .map(|(key, source)| (*key, source.clone())),
        );

        let previous_source_count = self.previous_skinned_gpu_sources.len();
        GpuScenePrevSkinnedSourceRollReport {
            current_source_count: self.current_skinned_gpu_sources.len(),
            previous_source_count,
            removed_previous_source_count,
        }
    }
}

#[cfg(test)]
#[path = "tests/prev_skinned_source.rs"]
mod tests;
