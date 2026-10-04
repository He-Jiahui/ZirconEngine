/// 视口调试指令会影响冻结裁剪和可视化，不能当作场景几何的持久状态。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryDebugState {
    pub forced_mip: Option<u8>,
    pub freeze_cull: bool,
    pub visualize_bvh: bool,
    pub visualize_visbuffer: bool,
    pub print_leaf_clusters: bool,
}
