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
    let mut payloads = Vec::with_capacity(primitives.len());
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
        payloads.push(payload.clone());
    }
    RuntimePrepareMeshSdfSeed::Ready(Arc::from(payloads))
}
