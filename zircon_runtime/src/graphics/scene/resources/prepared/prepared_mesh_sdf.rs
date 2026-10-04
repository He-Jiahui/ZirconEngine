use std::sync::Arc;

use crate::asset::ModelPrimitiveAsset;
use crate::graphics::RuntimePrepareMeshSdfSeed;

/// 全部源 payload 校验通过后才克隆 SDF 数据；晚出现的无效 primitive 直接返回其位置，避免无效批次的复制成本。
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
#[path = "tests/prepared_mesh_sdf_optimization_batch_gu_runtime576_tests.rs"]
mod optimization_batch_gu_runtime576_tests;
