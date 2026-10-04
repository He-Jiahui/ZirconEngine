use crate::asset::MeshVertex;

use super::{MeshSdfCookSettings, MESH_SDF_SCHEMA_VERSION};

const POSITION_COMPONENT_BYTES: usize = size_of::<u32>();
const POSITION_COMPONENTS: usize = 3;
const POSITION_BYTES: usize = POSITION_COMPONENT_BYTES * POSITION_COMPONENTS;
const POSITION_BATCH_VERTICES: usize = 256;
const POSITION_BATCH_BYTES: usize = POSITION_BYTES * POSITION_BATCH_VERTICES;
const INDEX_BATCH_VALUES: usize = 256;
const INDEX_BATCH_BYTES: usize = size_of::<u32>() * INDEX_BATCH_VALUES;

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
    update_vertex_position_hash(&mut hasher, vertices);
    hasher.update(
        &u64::try_from(indices.len())
            .unwrap_or(u64::MAX)
            .to_le_bytes(),
    );
    update_index_hash(&mut hasher, indices);
    hasher.update(&settings.max_dimension.to_le_bytes());
    hasher.update(&settings.max_voxel_count.to_le_bytes());
    hasher.update(&settings.max_payload_bytes.to_le_bytes());
    hasher.update(&settings.surface_band_voxels.to_le_bytes());
    hasher.update(&[u8::from(settings.two_sided)]);
    *hasher.finalize().as_bytes()
}

fn update_vertex_position_hash(hasher: &mut blake3::Hasher, vertices: &[MeshVertex]) {
    let mut bytes = [0_u8; POSITION_BATCH_BYTES];
    for batch in vertices.chunks(POSITION_BATCH_VERTICES) {
        for (vertex_index, vertex) in batch.iter().enumerate() {
            let vertex_offset = vertex_index * POSITION_BYTES;
            for (component_index, component) in vertex.position.iter().enumerate() {
                let component_offset = vertex_offset + component_index * POSITION_COMPONENT_BYTES;
                bytes[component_offset..component_offset + POSITION_COMPONENT_BYTES]
                    .copy_from_slice(&component.to_bits().to_le_bytes());
            }
        }
        hasher.update(&bytes[..batch.len() * POSITION_BYTES]);
    }
}

fn update_index_hash(hasher: &mut blake3::Hasher, indices: &[u32]) {
    let mut bytes = [0_u8; INDEX_BATCH_BYTES];
    for batch in indices.chunks(INDEX_BATCH_VALUES) {
        for (index, value) in batch.iter().enumerate() {
            let offset = index * size_of::<u32>();
            bytes[offset..offset + size_of::<u32>()].copy_from_slice(&value.to_le_bytes());
        }
        hasher.update(&bytes[..batch.len() * size_of::<u32>()]);
    }
}

#[cfg(test)]
#[path = "tests/source_hash_optimization_tests.rs"]
mod optimization_tests;
