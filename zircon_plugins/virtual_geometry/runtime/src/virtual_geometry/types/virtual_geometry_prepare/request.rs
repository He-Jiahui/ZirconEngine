#[derive(Clone, Debug, PartialEq, Eq)]
/// 将待上传页与计划槽位、被替换页一起交给 GPU 上传准备阶段。
pub struct VirtualGeometryPrepareRequest {
    pub page_id: u32,
    pub size_bytes: u64,
    pub generation: u64,
    pub frontier_rank: u32,
    pub assigned_slot: Option<u32>,
    pub recycled_page_id: Option<u32>,
}
