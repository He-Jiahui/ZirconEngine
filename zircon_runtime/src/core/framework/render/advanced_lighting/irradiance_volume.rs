use serde::{Deserialize, Serialize};

use crate::core::framework::render::RenderLayerSet;
use crate::core::math::{Mat4, Real};
use crate::core::resource::ResourceId as AssetId;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IrradianceVolumeData {
    pub volume_id: u64,
    pub transform: Mat4,
    pub voxels: AssetId,
    pub intensity: Real,
    pub affects_lightmapped_meshes: bool,
    pub priority: i32,
    #[serde(default)]
    pub layer_mask: RenderLayerSet,
}
