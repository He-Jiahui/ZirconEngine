use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct IndexedIndirectArgs {
    pub(super) index_count: u32,
    pub(super) instance_count: u32,
    pub(super) first_index: u32,
    pub(super) base_vertex: i32,
    pub(super) first_instance: u32,
}
