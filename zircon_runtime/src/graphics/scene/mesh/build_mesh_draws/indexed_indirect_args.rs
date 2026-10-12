use bytemuck::{Pod, Zeroable};

/// 与 wgpu indexed indirect draw 及 GPU 压缩 pass 共用的五字参数 ABI。
/// 字段次序由 GPU 缓冲区读取约束，不能随意重排。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub(crate) struct IndexedIndirectArgs {
    pub(crate) index_count: u32,
    pub(crate) instance_count: u32,
    pub(crate) first_index: u32,
    pub(crate) base_vertex: i32,
    pub(crate) first_instance: u32,
}
