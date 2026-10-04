use serde::{Deserialize, Serialize};

use super::{RenderMeshBounds, RenderMeshKind, RenderMeshTopology};

/// 资产侧提供给检查、预览和资源准备的网格摘要；包围盒与计数不代表 GPU 缓冲已就绪。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderMeshDescriptor {
    pub topology: RenderMeshTopology,
    pub bounds: RenderMeshBounds,
    pub primitive_kind: RenderMeshKind,
    pub suitable_for_2d: bool,
    pub suitable_for_3d: bool,
    pub vertex_count: usize,
    pub index_count: usize,
    pub primitive_count: usize,
    pub has_virtual_geometry_payload: bool,
}
