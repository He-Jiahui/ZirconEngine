use serde::{Deserialize, Serialize};

use crate::core::math::Real;

/// Backend-neutral payload resolved by a physics plugin for asset-backed collider shapes.
/// 资源型碰撞几何解析后的后端中立载荷；Jolt 等后端据此构建真实碰撞形状。
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
