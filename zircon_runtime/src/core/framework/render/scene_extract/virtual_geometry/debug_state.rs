#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryDebugState {
    pub forced_mip: Option<u8>,
    pub freeze_cull: bool,
    pub visualize_bvh: bool,
    pub visualize_visbuffer: bool,
    pub print_leaf_clusters: bool,
}
