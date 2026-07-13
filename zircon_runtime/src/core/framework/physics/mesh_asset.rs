use serde::{Deserialize, Serialize};

use crate::core::math::Real;

/// Backend-neutral payload resolved by a physics plugin for asset-backed collider shapes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PhysicsMeshAsset {
    TriangleMesh {
        vertices: Vec<[Real; 3]>,
        indices: Vec<[u32; 3]>,
    },
    HeightField {
        resolution: [u32; 2],
        heights: Vec<Real>,
    },
}
