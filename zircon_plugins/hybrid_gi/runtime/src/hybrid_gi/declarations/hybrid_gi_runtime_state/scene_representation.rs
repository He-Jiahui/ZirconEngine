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
}
