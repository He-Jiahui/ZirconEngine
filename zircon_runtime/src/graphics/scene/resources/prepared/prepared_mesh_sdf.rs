use std::sync::Arc;

use crate::asset::ModelPrimitiveAsset;
use crate::graphics::RuntimePrepareMeshSdfSeed;

pub(in crate::graphics::scene::resources) fn mesh_sdf_seed_from_primitives(
    primitives: &[ModelPrimitiveAsset],
) -> RuntimePrepareMeshSdfSeed {
    let payload_count = primitives
        .iter()
        .filter(|primitive| primitive.mesh_sdf.is_some())
        .count();
    if payload_count != primitives.len() || primitives.is_empty() {
        return RuntimePrepareMeshSdfSeed::Missing {
            primitive_count: primitives.len(),
            payload_count,
        };
    }
    for (primitive_index, primitive) in primitives.iter().enumerate() {
        let Some(payload) = primitive.mesh_sdf.as_ref() else {
            continue;
        };
        if let Err(error) = payload.validate_for_source(&primitive.vertices, &primitive.indices) {
            return RuntimePrepareMeshSdfSeed::Invalid {
                primitive_index,
                error,
            };
        }
    }
    let mut payloads = Vec::with_capacity(primitives.len());
    payloads.extend(primitives.iter().map(|primitive| {
        primitive
            .mesh_sdf
            .as_ref()
            .expect("payload count preflight")
            .clone()
    }));
    RuntimePrepareMeshSdfSeed::Ready(Arc::from(payloads))
}

#[cfg(test)]
mod optimization_batch_gu_runtime576_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use crate::asset::{
        cook_mesh_sdf_from_mesh, MeshSdfCookSettings, MeshVertex, ModelPrimitiveAsset,
    };
    use crate::core::math::{Vec2, Vec3};

    use super::*;

    const SAMPLE_PAIRS: usize = 31;
    const ITERATIONS: usize = 100;
    const PRIMITIVES: usize = 16;

    #[test]
    fn optimization_batch_gu_runtime576_late_invalid_seed_preserves_error() {
        let primitives = invalid_last_primitives();

        let seed = mesh_sdf_seed_from_primitives(&primitives);

        assert!(matches!(
            seed,
            RuntimePrepareMeshSdfSeed::Invalid {
                primitive_index,
                ..
            } if primitive_index == PRIMITIVES - 1
        ));
        let production = include_str!("prepared_mesh_sdf.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        let validation = production
            .find("payload.validate_for_source")
            .expect("validation preflight");
        let cloning = production
            .find("payload count preflight")
            .expect("payload clone pass");
        assert!(validation < cloning);
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_gu_runtime576_mesh_sdf_seed_validation_preflight_p95() {
        let primitives = invalid_last_primitives();
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure(&primitives, false));
                optimized_samples.push(measure(&primitives, true));
            } else {
                optimized_samples.push(measure(&primitives, true));
                legacy_samples.push(measure(&primitives, false));
            }
        }

        let legacy_p95_ns = p95(&mut legacy_samples);
        let optimized_p95_ns = p95(&mut optimized_samples);
        println!(
            "RUNTIME576_MESH_SDF_SEED_PREFLIGHT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} iterations={ITERATIONS} primitives={PRIMITIVES} late_invalid_index={} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            PRIMITIVES - 1,
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(90),
            "expected validation preflight to lower p95 by at least 10%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn measure(primitives: &[ModelPrimitiveAsset], optimized: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..ITERATIONS {
            let seed = if optimized {
                mesh_sdf_seed_from_primitives(primitives)
            } else {
                legacy_mesh_sdf_seed_from_primitives(primitives)
            };
            if let RuntimePrepareMeshSdfSeed::Invalid {
                primitive_index, ..
            } = seed
            {
                checksum ^= primitive_index;
            }
        }
        black_box(checksum);
        started.elapsed().as_nanos()
    }

    fn legacy_mesh_sdf_seed_from_primitives(
        primitives: &[ModelPrimitiveAsset],
    ) -> RuntimePrepareMeshSdfSeed {
        let payload_count = primitives
            .iter()
            .filter(|primitive| primitive.mesh_sdf.is_some())
            .count();
        if payload_count != primitives.len() || primitives.is_empty() {
            return RuntimePrepareMeshSdfSeed::Missing {
                primitive_count: primitives.len(),
                payload_count,
            };
        }
        let mut payloads = Vec::with_capacity(primitives.len());
        for (primitive_index, primitive) in primitives.iter().enumerate() {
            let Some(payload) = primitive.mesh_sdf.as_ref() else {
                continue;
            };
            if let Err(error) = payload.validate_for_source(&primitive.vertices, &primitive.indices)
            {
                return RuntimePrepareMeshSdfSeed::Invalid {
                    primitive_index,
                    error,
                };
            }
            payloads.push(payload.clone());
        }
        RuntimePrepareMeshSdfSeed::Ready(Arc::from(payloads))
    }

    fn invalid_last_primitives() -> Vec<ModelPrimitiveAsset> {
        let (vertices, indices) = cube_geometry();
        let settings = MeshSdfCookSettings {
            max_dimension: 16,
            max_voxel_count: 4_096,
            max_payload_bytes: 16 * 1_024,
            surface_band_voxels: 4,
            two_sided: false,
        };
        let payload =
            cook_mesh_sdf_from_mesh(&vertices, &indices, settings).expect("fixture Mesh SDF cook");
        let mut primitives = (0..PRIMITIVES)
            .map(|_| ModelPrimitiveAsset {
                vertices: vertices.clone(),
                indices: indices.clone(),
                mesh: None,
                mesh_sdf: Some(payload.clone()),
                virtual_geometry: None,
            })
            .collect::<Vec<_>>();
        primitives
            .last_mut()
            .and_then(|primitive| primitive.mesh_sdf.as_mut())
            .expect("last fixture payload")
            .source_hash[0] ^= 1;
        primitives
    }

    fn cube_geometry() -> (Vec<MeshVertex>, Vec<u32>) {
        let positions = [
            [-1.0, -1.0, -1.0],
            [1.0, -1.0, -1.0],
            [1.0, 1.0, -1.0],
            [-1.0, 1.0, -1.0],
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, 1.0, 1.0],
            [-1.0, 1.0, 1.0],
        ];
        let vertices = positions
            .into_iter()
            .map(|position| MeshVertex::new(Vec3::from_array(position), Vec3::Z, Vec2::ZERO))
            .collect();
        let indices = vec![
            0, 2, 1, 0, 3, 2, 4, 5, 6, 4, 6, 7, 0, 1, 5, 0, 5, 4, 1, 2, 6, 1, 6, 5, 2, 3, 7, 2, 7,
            6, 3, 0, 4, 3, 4, 7,
        ];
        (vertices, indices)
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[samples.len() * 95 / 100]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
