use serde::{Deserialize, Serialize};

pub const DEFAULT_MESH_SDF_MAX_DIMENSION: u32 = 32;
pub const DEFAULT_MESH_SDF_MAX_VOXEL_COUNT: u64 = 32 * 32 * 32;
pub const DEFAULT_MESH_SDF_MAX_PAYLOAD_BYTES: u64 = 128 * 1024;
pub const DEFAULT_MESH_SDF_SURFACE_BAND_VOXELS: u32 = 4;

/// Deterministic import settings embedded into the payload source identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeshSdfCookSettings {
    pub max_dimension: u32,
    pub max_voxel_count: u64,
    pub max_payload_bytes: u64,
    pub surface_band_voxels: u32,
    pub two_sided: bool,
}

impl Default for MeshSdfCookSettings {
    fn default() -> Self {
        Self {
            max_dimension: DEFAULT_MESH_SDF_MAX_DIMENSION,
            max_voxel_count: DEFAULT_MESH_SDF_MAX_VOXEL_COUNT,
            max_payload_bytes: DEFAULT_MESH_SDF_MAX_PAYLOAD_BYTES,
            surface_band_voxels: DEFAULT_MESH_SDF_SURFACE_BAND_VOXELS,
            two_sided: false,
        }
    }
}
