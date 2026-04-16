#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VisibilityVirtualGeometryFeedback {
    pub visible_cluster_ids: Vec<u32>,
    pub requested_pages: Vec<u32>,
    pub evictable_pages: Vec<u32>,
}
