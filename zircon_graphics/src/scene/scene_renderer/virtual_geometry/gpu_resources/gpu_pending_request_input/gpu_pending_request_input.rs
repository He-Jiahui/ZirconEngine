use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct GpuPendingRequestInput {
    pub(super) page_id: u32,
    pub(super) size_bytes: u32,
}
