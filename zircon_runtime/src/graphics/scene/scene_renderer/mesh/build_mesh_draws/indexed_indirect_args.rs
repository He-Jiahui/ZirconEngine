use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub(crate) struct IndexedIndirectArgs {
    pub(crate) index_count: u32,
    pub(crate) instance_count: u32,
    pub(crate) first_index: u32,
    pub(crate) base_vertex: i32,
    pub(crate) first_instance: u32,
}
