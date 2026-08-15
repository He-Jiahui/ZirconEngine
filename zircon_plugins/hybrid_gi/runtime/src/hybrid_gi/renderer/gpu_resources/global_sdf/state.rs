use std::sync::atomic::AtomicU64;

use crate::hybrid_gi::scene_representation::{
    GLOBAL_SDF_CLIPMAP_COUNT, GLOBAL_SDF_MAX_RESIDENT_PAGE_COUNT, GLOBAL_SDF_PAGES_PER_EDGE,
};

use super::super::buffer_helpers::create_u32_storage_buffer;
use super::packing::GLOBAL_SDF_PAGE_VOXEL_COUNT;

pub(super) const GLOBAL_SDF_TRACE_PAGE_TABLE_ENTRY_COUNT: usize = GLOBAL_SDF_CLIPMAP_COUNT
    * GLOBAL_SDF_PAGES_PER_EDGE as usize
    * GLOBAL_SDF_PAGES_PER_EDGE as usize
    * GLOBAL_SDF_PAGES_PER_EDGE as usize;
pub(super) const GLOBAL_SDF_TRACE_PAGE_UNAVAILABLE_SLOT: u32 = u32::MAX;

pub(in crate::hybrid_gi::renderer) struct GlobalSdfGpuState {
    pub(super) atlas_buffer: wgpu::Buffer,
    pub(super) trace_page_table_buffer: wgpu::Buffer,
    pub(super) trace_page_signature: AtomicU64,
}

impl GlobalSdfGpuState {
    pub(in crate::hybrid_gi::renderer) fn new(device: &wgpu::Device) -> Self {
        Self {
            atlas_buffer: create_u32_storage_buffer(
                device,
                "zircon-hybrid-gi-global-sdf-page-atlas",
                &[0; GLOBAL_SDF_MAX_RESIDENT_PAGE_COUNT * GLOBAL_SDF_PAGE_VOXEL_COUNT],
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            ),
            trace_page_table_buffer: create_u32_storage_buffer(
                device,
                "zircon-hybrid-gi-global-sdf-trace-page-table",
                &[GLOBAL_SDF_TRACE_PAGE_UNAVAILABLE_SLOT; GLOBAL_SDF_TRACE_PAGE_TABLE_ENTRY_COUNT],
                wgpu::BufferUsages::STORAGE,
            ),
            trace_page_signature: AtomicU64::new(0),
        }
    }

    pub(in crate::hybrid_gi::renderer) fn persistent_resource_byte_count(&self) -> u64 {
        let word_count = GLOBAL_SDF_MAX_RESIDENT_PAGE_COUNT * GLOBAL_SDF_PAGE_VOXEL_COUNT
            + GLOBAL_SDF_TRACE_PAGE_TABLE_ENTRY_COUNT;
        (word_count * std::mem::size_of::<u32>()) as u64
    }
}
