use serde::{Deserialize, Serialize};

use super::{RenderMeshBounds, RenderMeshKind, RenderMeshTopology};

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
