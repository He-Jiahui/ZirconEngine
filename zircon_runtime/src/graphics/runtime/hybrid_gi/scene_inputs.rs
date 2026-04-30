use crate::core::framework::render::{
    RenderDirectionalLightSnapshot, RenderMeshSnapshot, RenderPointLightSnapshot,
    RenderSpotLightSnapshot,
};

/// Frame-local scene truth consumed by the Hybrid GI runtime host.
#[derive(Clone, Debug, Default)]
pub(in crate::graphics::runtime) struct HybridGiSceneInputs {
    meshes: Vec<RenderMeshSnapshot>,
    directional_lights: Vec<RenderDirectionalLightSnapshot>,
    point_lights: Vec<RenderPointLightSnapshot>,
    spot_lights: Vec<RenderSpotLightSnapshot>,
}

impl HybridGiSceneInputs {
    pub(in crate::graphics::runtime) fn new(
        meshes: Vec<RenderMeshSnapshot>,
        directional_lights: Vec<RenderDirectionalLightSnapshot>,
        point_lights: Vec<RenderPointLightSnapshot>,
        spot_lights: Vec<RenderSpotLightSnapshot>,
    ) -> Self {
        Self {
            meshes,
            directional_lights,
            point_lights,
            spot_lights,
        }
    }

    pub(in crate::graphics::runtime) fn meshes(&self) -> &[RenderMeshSnapshot] {
        &self.meshes
    }

    pub(in crate::graphics::runtime) fn directional_lights(
        &self,
    ) -> &[RenderDirectionalLightSnapshot] {
        &self.directional_lights
    }

    pub(in crate::graphics::runtime) fn point_lights(&self) -> &[RenderPointLightSnapshot] {
        &self.point_lights
    }

    pub(in crate::graphics::runtime) fn spot_lights(&self) -> &[RenderSpotLightSnapshot] {
        &self.spot_lights
    }
}
