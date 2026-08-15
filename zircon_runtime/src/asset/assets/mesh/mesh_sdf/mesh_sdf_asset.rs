use serde::{Deserialize, Serialize};

use crate::asset::MeshVertex;
use crate::core::framework::render::RenderMeshBounds;

use super::{MeshSdfCookSettings, MeshSdfEncoding, MeshSdfValidationError};

pub const MESH_SDF_SCHEMA_VERSION: u32 = 1;
pub(crate) const MESH_SDF_FIXED_METADATA_BYTES: u64 = 128;

/// Import-time cooked local-space signed-distance volume for one mesh primitive.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeshSdfAsset {
    pub schema_version: u32,
    pub source_hash: [u8; 32],
    pub local_bounds: RenderMeshBounds,
    pub dimensions: [u32; 3],
    pub voxel_size: [f32; 3],
    pub distance_range: [f32; 2],
    pub encoding: MeshSdfEncoding,
    pub cook_settings: MeshSdfCookSettings,
    #[serde(default)]
    pub voxels: Vec<i16>,
}

impl MeshSdfAsset {
    pub fn voxel_count(&self) -> Option<u64> {
        self.dimensions
            .into_iter()
            .map(u64::from)
            .try_fold(1_u64, u64::checked_mul)
    }

    pub fn encoded_size_bytes(&self) -> Option<u64> {
        u64::try_from(self.voxels.len())
            .ok()?
            .checked_mul(std::mem::size_of::<i16>() as u64)?
            .checked_add(MESH_SDF_FIXED_METADATA_BYTES)
    }

    pub fn validate(&self) -> Result<(), MeshSdfValidationError> {
        super::validate::validate_mesh_sdf_asset(self)
    }

    pub fn validate_for_source(
        &self,
        vertices: &[MeshVertex],
        indices: &[u32],
    ) -> Result<(), MeshSdfValidationError> {
        super::validate::validate_mesh_sdf_asset_for_source(self, vertices, indices)
    }
}
