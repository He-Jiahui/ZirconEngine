use crate::core::framework::render::{
    LightmapConsumeContract, RenderDirectionalLightSnapshot, RenderHybridGiExtract,
    RenderMeshSnapshot, RenderPointLightSnapshot, RenderSpotLightSnapshot,
};
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
        }
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

    pub fn generation(&self) -> u64 {
        self.input.generation()
    }
}
