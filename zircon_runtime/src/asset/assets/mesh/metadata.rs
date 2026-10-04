use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::MeshAttributeValues;

/// Per-target mesh displacement data keyed by the same attribute names used by the root mesh.
/// 属性数组按根网格顶点序号逐项对齐；MeshAsset::validate 要求它们与根顶点数等长，后续形变按此顺序叠加增量。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeshMorphTargetAsset {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default)]
    pub attributes: BTreeMap<String, MeshAttributeValues>,
}

/// Skin bind-pose metadata carried with a mesh until dedicated skin subassets own richer bindings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeshSkinAsset {
    #[serde(default)]
    pub inverse_bind_matrices: Vec<[[f32; 4]; 4]>,
}
