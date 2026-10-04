use std::collections::HashSet;

/// 一次视口构建期间共享的高亮选择、虚拟几何范围和绕序策略。
/// pending draw 收集沿用此快照，避免不同网格读到不一致的视图状态。
pub(super) struct MeshDrawBuildContext {
    pub(super) selection: HashSet<u64>,
    pub(super) allowed_virtual_geometry_entities: Option<HashSet<u64>>,
    pub(super) reverse_view_raster_winding: bool,
}
