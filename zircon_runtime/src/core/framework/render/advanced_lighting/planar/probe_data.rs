use serde::{Deserialize, Serialize};

use crate::core::framework::render::RenderLayerSet;
use crate::core::math::{Mat4, Vec3};
use crate::core::resource::{ResourceHandle, TextureMarker};

use super::PlanarUpdateMode;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlanarReflectionProbeData {
    pub probe_id: u64,
    pub plane_transform: Mat4,
    pub local_reference_position: Vec3,
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
    pub resolution: u32,
    pub update: PlanarUpdateMode,
    /// Runtime capture target used by the ordinary camera loop. `None` keeps
    /// the authored probe inert until a render texture has been assigned.
    #[serde(default)]
    pub capture_target: Option<ResourceHandle<TextureMarker>>,
    #[serde(default)]
    pub layer_mask: RenderLayerSet,
}

impl PlanarReflectionProbeData {
    pub fn capture_target(&self) -> Option<ResourceHandle<TextureMarker>> {
        self.capture_target
    }
}
