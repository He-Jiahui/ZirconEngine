mod completed_page_assignments;
mod page_table_entries;
mod read_buffer_u32s;

pub(in crate::virtual_geometry::renderer::gpu_readback) use completed_page_assignments::completed_page_assignments;
pub(in crate::virtual_geometry::renderer::gpu_readback) use page_table_entries::page_table_entries;
