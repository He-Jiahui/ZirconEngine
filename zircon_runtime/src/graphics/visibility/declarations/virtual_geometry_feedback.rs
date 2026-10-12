/// 将当前可见 cluster 与页请求回馈给虚拟几何驻留管理器，用于下帧预算和驱逐。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VisibilityVirtualGeometryFeedback {
    pub visible_cluster_ids: Vec<u32>,
    pub requested_pages: Vec<u32>,
    pub evictable_pages: Vec<u32>,
    pub hot_resident_pages: Vec<u32>,
}
