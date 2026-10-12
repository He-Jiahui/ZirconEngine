/// 页驻留、请求与淘汰的提交计划；需和同帧可见 cluster 及历史请求共同解释。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VisibilityVirtualGeometryPageUploadPlan {
    pub resident_pages: Vec<u32>,
    pub requested_pages: Vec<u32>,
    pub dirty_requested_pages: Vec<u32>,
    pub evictable_pages: Vec<u32>,
}
