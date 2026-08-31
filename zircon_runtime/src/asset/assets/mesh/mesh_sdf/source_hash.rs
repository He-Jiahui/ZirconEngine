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
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 13;
    const VERTEX_COUNT: usize = 16_384;
    const ITERATIONS: usize = 64;

    #[test]
    fn optimization_batch_20260831fd_runtime569_batched_mesh_sdf_hash_preserves_bytes() {
        let vertices = vertices(VERTEX_COUNT + 1);
        let indices = indices(VERTEX_COUNT * 3 + 1);
        for vertex_count in [0, 1, 255, 256, 257, VERTEX_COUNT + 1] {
            let index_count = (vertex_count * 3).min(indices.len());
            assert_eq!(
                mesh_sdf_source_hash(
                    &vertices[..vertex_count],
                    &indices[..index_count],
                    MeshSdfCookSettings::default(),
                ),
                legacy_mesh_sdf_source_hash(
                    &vertices[..vertex_count],
                    &indices[..index_count],
                    MeshSdfCookSettings::default(),
                )
            );
        }
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260831fd_runtime569_batched_mesh_sdf_hash_p95() {
        let vertices = vertices(VERTEX_COUNT);
        let indices = indices(VERTEX_COUNT * 3);
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false, &vertices, &indices));
                optimized.push(measure(true, &vertices, &indices));
            } else {
                optimized.push(measure(true, &vertices, &indices));
                legacy.push(measure(false, &vertices, &indices));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME569_MESH_SDF_HASH_BATCH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
vertices={VERTEX_COUNT} iterations={ITERATIONS} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(75));
    }

    fn measure(optimized: bool, vertices: &[MeshVertex], indices: &[u32]) -> u128 {
        let started = Instant::now();
        let mut checksum = 0_u8;
        for _ in 0..ITERATIONS {
            let hash = if optimized {
                mesh_sdf_source_hash(vertices, indices, MeshSdfCookSettings::default())
            } else {
                legacy_mesh_sdf_source_hash(vertices, indices, MeshSdfCookSettings::default())
            };
            checksum ^= hash[0];
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn legacy_mesh_sdf_source_hash(
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

    fn vertices(count: usize) -> Vec<MeshVertex> {
        (0..count)
            .map(|index| MeshVertex {
                position: [
                    f32::from_bits((index as u32).wrapping_mul(17)),
                    f32::from_bits((index as u32).wrapping_mul(31) | 0x8000_0000),
                    f32::from_bits((index as u32).wrapping_mul(47) ^ 0x7fc0_0000),
                ],
                normal: [0.0; 3],
                uv: [0.0; 2],
                uv1: [0.0; 2],
                tangent: [0.0; 4],
                color: [0.0; 4],
                joint_indices: [0; 4],
                joint_weights: [0.0; 4],
            })
            .collect()
    }

    fn indices(count: usize) -> Vec<u32> {
        (0..count)
            .map(|index| (index as u32).wrapping_mul(2_654_435_761))
            .collect()
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
