use super::super::super::{
    RenderAmbientLightSnapshot, RenderDirectionalLightSnapshot, RenderPointLightSnapshot,
    RenderRectLightSnapshot, RenderSpotLightSnapshot, ViewportCameraSnapshot,
};
use super::super::mesh::RenderMeshSnapshot;

#[derive(Clone, Debug, PartialEq)]
pub struct RenderSceneGeometryExtract {
    pub camera: ViewportCameraSnapshot,
    pub meshes: Vec<RenderMeshSnapshot>,
    pub directional_lights: Vec<RenderDirectionalLightSnapshot>,
    pub point_lights: Vec<RenderPointLightSnapshot>,
    pub spot_lights: Vec<RenderSpotLightSnapshot>,
    pub ambient_lights: Vec<RenderAmbientLightSnapshot>,
    pub rect_lights: Vec<RenderRectLightSnapshot>,
}
