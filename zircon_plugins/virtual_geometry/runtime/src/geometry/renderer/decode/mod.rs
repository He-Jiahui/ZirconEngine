#[path = "../../../completed_page_assignments.rs"]
mod completed_page_assignments;
#[path = "../../page_table_entries.rs"]
mod page_table_entries;
#[path = "../../decode/read_buffer_u32s.rs"]
mod read_buffer_u32s;

pub(in crate::virtual_geometry::renderer::gpu_readback) use completed_page_assignments::completed_page_assignments;
pub(in crate::virtual_geometry::renderer::gpu_readback) use page_table_entries::page_table_entries;
