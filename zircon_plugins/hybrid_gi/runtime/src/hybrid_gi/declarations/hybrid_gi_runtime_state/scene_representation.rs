use super::runtime_state::HybridGiRuntimeState;
use crate::hybrid_gi::scene_representation::HybridGiSceneRepresentation;

impl HybridGiRuntimeState {
    pub(in crate::hybrid_gi) fn scene_representation(&self) -> &HybridGiSceneRepresentation {
        &self.scene_representation
    }

    pub(in crate::hybrid_gi) fn scene_representation_mut(
        &mut self,
    ) -> &mut HybridGiSceneRepresentation {
        &mut self.scene_representation
    }

    pub(in crate::hybrid_gi) fn scene_representation_owns_runtime(&self) -> bool {
        let settings = self.scene_representation.settings();
        settings.trace_budget() > 0 || settings.card_budget() > 0 || settings.voxel_budget() > 0
    }

    pub(in crate::hybrid_gi) fn has_live_gpu_feedback_probe(&self, probe_id: u32) -> bool {
        if self.probe_scene_data().contains_key(&probe_id) {
            return true;
        }

        self.scene_representation_owns_runtime()
            && self
                .scene_representation()
                .screen_probe_runtime_descriptors()
                .into_iter()
                .any(|probe| probe.probe_id() == probe_id)
    }
}
