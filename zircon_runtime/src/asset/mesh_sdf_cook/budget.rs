use crate::asset::MeshSdfCookError;

pub const MAX_MESH_SDF_SOURCE_TRIANGLE_COUNT: u64 = 65_536;
pub const MAX_MESH_SDF_PRIMITIVE_WORK_UNITS: u64 = 256 * 1024 * 1024;
pub const MAX_MESH_SDF_IMPORT_VOXEL_COUNT: u64 = 512 * 1024;
pub const MAX_MESH_SDF_IMPORT_PAYLOAD_BYTES: u64 = 2 * 1024 * 1024;
pub const MAX_MESH_SDF_IMPORT_WORK_UNITS: u64 = 512 * 1024 * 1024;

/// Cumulative guard shared by every Mesh SDF primitive cooked by one import.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MeshSdfCookBudget {
    consumed_voxels: u64,
    consumed_payload_bytes: u64,
    consumed_work_units: u64,
}

impl Default for MeshSdfCookBudget {
    fn default() -> Self {
        Self {
            consumed_voxels: 0,
            consumed_payload_bytes: 0,
            consumed_work_units: 0,
        }
    }
}

impl MeshSdfCookBudget {
    pub fn reserve(
        &mut self,
        voxel_count: u64,
        payload_bytes: u64,
        work_units: u64,
    ) -> Result<(), MeshSdfCookError> {
        let next_voxels = self.consumed_voxels.saturating_add(voxel_count);
        if next_voxels > MAX_MESH_SDF_IMPORT_VOXEL_COUNT {
            return Err(MeshSdfCookError::ImportVoxelBudgetExceeded {
                actual: next_voxels,
                budget: MAX_MESH_SDF_IMPORT_VOXEL_COUNT,
            });
        }
        let next_payload_bytes = self.consumed_payload_bytes.saturating_add(payload_bytes);
        if next_payload_bytes > MAX_MESH_SDF_IMPORT_PAYLOAD_BYTES {
            return Err(MeshSdfCookError::ImportPayloadBudgetExceeded {
                actual: next_payload_bytes,
                budget: MAX_MESH_SDF_IMPORT_PAYLOAD_BYTES,
            });
        }
        let next_work_units = self.consumed_work_units.saturating_add(work_units);
        if next_work_units > MAX_MESH_SDF_IMPORT_WORK_UNITS {
            return Err(MeshSdfCookError::ImportWorkBudgetExceeded {
                actual: next_work_units,
                budget: MAX_MESH_SDF_IMPORT_WORK_UNITS,
            });
        }
        self.consumed_voxels = next_voxels;
        self.consumed_payload_bytes = next_payload_bytes;
        self.consumed_work_units = next_work_units;
        Ok(())
    }

    pub fn consumed_voxels(&self) -> u64 {
        self.consumed_voxels
    }

    pub fn consumed_payload_bytes(&self) -> u64 {
        self.consumed_payload_bytes
    }

    pub fn consumed_work_units(&self) -> u64 {
        self.consumed_work_units
    }
}
