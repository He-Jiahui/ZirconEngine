use crate::core::framework::render::{
    LightmapConsumeContract, RenderDirectionalLightSnapshot, RenderHybridGiExtract,
    RenderMeshSnapshot, RenderPointLightSnapshot, RenderSpotLightSnapshot,
};
use crate::core::math::Vec3;
use crate::graphics::{
    runtime_provider::RuntimeProviderPrepareInput, VisibilityHybridGiUpdatePlan,
};

pub struct HybridGiRuntimePrepareInput<'a> {
    input: RuntimeProviderPrepareInput<'a, RenderHybridGiExtract>,
    meshes: &'a [RenderMeshSnapshot],
    directional_lights: &'a [RenderDirectionalLightSnapshot],
    point_lights: &'a [RenderPointLightSnapshot],
    spot_lights: &'a [RenderSpotLightSnapshot],
    baked_lighting: Option<&'a LightmapConsumeContract>,
    has_baked_probe_grid: bool,
    update_plan: Option<&'a VisibilityHybridGiUpdatePlan>,
    camera_position: Option<Vec3>,
    history_invalidated: bool,
}

impl<'a> HybridGiRuntimePrepareInput<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        extract: Option<&'a RenderHybridGiExtract>,
        meshes: &'a [RenderMeshSnapshot],
        directional_lights: &'a [RenderDirectionalLightSnapshot],
        point_lights: &'a [RenderPointLightSnapshot],
        spot_lights: &'a [RenderSpotLightSnapshot],
        baked_lighting: Option<&'a LightmapConsumeContract>,
        has_baked_probe_grid: bool,
        update_plan: Option<&'a VisibilityHybridGiUpdatePlan>,
        generation: u64,
    ) -> Self {
        Self {
            input: RuntimeProviderPrepareInput::new(extract, generation),
            meshes,
            directional_lights,
            point_lights,
            spot_lights,
            baked_lighting,
            has_baked_probe_grid,
            update_plan,
            camera_position: None,
            history_invalidated: false,
        }
    }

    pub fn with_view_state(
        mut self,
        camera_position: Option<Vec3>,
        history_invalidated: bool,
    ) -> Self {
        self.camera_position = camera_position.filter(|position| position.is_finite());
        self.history_invalidated = history_invalidated;
        self
    }

    pub fn extract(&self) -> Option<&'a RenderHybridGiExtract> {
        self.input.extract()
    }

    pub fn meshes(&self) -> &'a [RenderMeshSnapshot] {
        self.meshes
    }

    pub fn directional_lights(&self) -> &'a [RenderDirectionalLightSnapshot] {
        self.directional_lights
    }

    pub fn point_lights(&self) -> &'a [RenderPointLightSnapshot] {
        self.point_lights
    }

    pub fn spot_lights(&self) -> &'a [RenderSpotLightSnapshot] {
        self.spot_lights
    }

    pub fn baked_lighting(&self) -> Option<&'a LightmapConsumeContract> {
        self.baked_lighting
    }

    pub fn has_baked_probe_grid(&self) -> bool {
        self.has_baked_probe_grid
    }

    pub fn update_plan(&self) -> Option<&'a VisibilityHybridGiUpdatePlan> {
        self.update_plan
    }

    pub fn camera_position(&self) -> Option<Vec3> {
        self.camera_position
    }

    pub fn history_invalidated(&self) -> bool {
        self.history_invalidated
    }

    pub fn generation(&self) -> u64 {
        self.input.generation()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_state_preserves_finite_camera_and_history_invalidation() {
        let input =
            HybridGiRuntimePrepareInput::new(None, &[], &[], &[], &[], None, false, None, 7)
                .with_view_state(Some(Vec3::new(1.0, 2.0, 3.0)), true);

        assert_eq!(input.camera_position(), Some(Vec3::new(1.0, 2.0, 3.0)));
        assert!(input.history_invalidated());
        assert_eq!(input.generation(), 7);
    }

    #[test]
    fn view_state_discards_nonfinite_camera_without_changing_history_semantics() {
        let input =
            HybridGiRuntimePrepareInput::new(None, &[], &[], &[], &[], None, false, None, 3)
                .with_view_state(Some(Vec3::new(f32::NAN, 0.0, 0.0)), false);

        assert_eq!(input.camera_position(), None);
        assert!(!input.history_invalidated());
        assert_eq!(input.generation(), 3);
    }
}
