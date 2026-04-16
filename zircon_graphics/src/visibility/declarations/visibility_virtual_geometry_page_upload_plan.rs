#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VisibilityVirtualGeometryPageUploadPlan {
    pub resident_pages: Vec<u32>,
    pub requested_pages: Vec<u32>,
    pub dirty_requested_pages: Vec<u32>,
    pub evictable_pages: Vec<u32>,
}
