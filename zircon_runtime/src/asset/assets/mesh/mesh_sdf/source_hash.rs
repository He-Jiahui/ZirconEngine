use crate::asset::MeshVertex;

use super::{MeshSdfCookSettings, MESH_SDF_SCHEMA_VERSION};

pub(crate) fn mesh_sdf_source_hash(
    vertices: &[MeshVertex],
    indices: &[u32],
    settings: MeshSdfCookSettings,
) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"zircon.mesh-sdf.source");
    hasher.update(&MESH_SDF_SCHEMA_VERSION.to_le_bytes());
    hasher.update(
        &u64::try_from(vertices.len())
            .unwrap_or(u64::MAX)
            .to_le_bytes(),
    );
    for vertex in vertices {
        for value in vertex.position {
            hasher.update(&value.to_bits().to_le_bytes());
        }
    }
    hasher.update(
        &u64::try_from(indices.len())
            .unwrap_or(u64::MAX)
            .to_le_bytes(),
    );
    for index in indices {
        hasher.update(&index.to_le_bytes());
    }
    hasher.update(&settings.max_dimension.to_le_bytes());
    hasher.update(&settings.max_voxel_count.to_le_bytes());
    hasher.update(&settings.max_payload_bytes.to_le_bytes());
    hasher.update(&settings.surface_band_voxels.to_le_bytes());
    hasher.update(&[u8::from(settings.two_sided)]);
    *hasher.finalize().as_bytes()
}
