use serde::{Deserialize, Serialize};

/// 资产摘要中的平面性分类；与 `suitable_for_2d` 一起描述可选的渲染用途。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderMeshKind {
    Planar2d,
    #[default]
    Spatial3d,
}
