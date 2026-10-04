use std::sync::{atomic::Ordering, Arc};

use bytemuck::{Pod, Zeroable};
use zircon_runtime::graphics::{
    RenderPassBufferUploadSink, RuntimePrepareFrameTransaction,
    RuntimePrepareFrameTransactionRecorder,
};

use crate::hybrid_gi::scene_representation::{
    HybridGiGlobalSdfPageKey, HybridGiGlobalSdfSceneState, GLOBAL_SDF_CLIPMAP_COUNT,
    GLOBAL_SDF_MAX_RESIDENT_PAGE_COUNT, GLOBAL_SDF_PAGES_PER_EDGE,
};

use super::state::{
    GLOBAL_SDF_TRACE_PAGE_TABLE_ENTRY_COUNT, GLOBAL_SDF_TRACE_PAGE_UNAVAILABLE_SLOT,
};
use super::GlobalSdfGpuState;

const FNV64_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV64_PRIME: u64 = 0x0000_0100_0000_01b3;
const GLOBAL_SDF_PAGES_PER_CLIPMAP: usize = GLOBAL_SDF_PAGES_PER_EDGE as usize
    * GLOBAL_SDF_PAGES_PER_EDGE as usize
    * GLOBAL_SDF_PAGES_PER_EDGE as usize;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(in crate::hybrid_gi::renderer::gpu_resources) struct GlobalSdfGpuTraceClipmap {
    pub(in crate::hybrid_gi::renderer::gpu_resources) page_coordinate_origin_and_padding: [i32; 4],
    pub(in crate::hybrid_gi::renderer::gpu_resources) page_world_size_and_padding: [f32; 4],
}

pub(in crate::hybrid_gi::renderer::gpu_resources) struct GlobalSdfGpuTraceBindings {
    pub(in crate::hybrid_gi::renderer::gpu_resources) page_table_buffer: wgpu::Buffer,
    pub(in crate::hybrid_gi::renderer::gpu_resources) atlas_buffer: wgpu::Buffer,
    pub(in crate::hybrid_gi::renderer::gpu_resources) page_count: u32,
    pub(in crate::hybrid_gi::renderer::gpu_resources) clipmaps:
        [GlobalSdfGpuTraceClipmap; GLOBAL_SDF_CLIPMAP_COUNT],
}

struct GlobalSdfTracePageTable {
    slots: [u32; GLOBAL_SDF_TRACE_PAGE_TABLE_ENTRY_COUNT],
    clipmaps: [GlobalSdfGpuTraceClipmap; GLOBAL_SDF_CLIPMAP_COUNT],
    page_count: u32,
}

impl GlobalSdfGpuState {
    pub(in crate::hybrid_gi::renderer::gpu_resources) fn create_trace_bindings(
        &self,
        buffer_uploads: &mut dyn RenderPassBufferUploadSink,
        frame_transactions: &mut RuntimePrepareFrameTransactionRecorder<'_>,
        scene: &HybridGiGlobalSdfSceneState,
    ) -> GlobalSdfGpuTraceBindings {
        let page_table = build_trace_page_table(scene);
        // 页表上传属于当前准备帧；只在帧事务提交后记住签名，失败帧需重新上传。
        let signature = trace_page_signature(&page_table);
        if self.trace_page_signature.load(Ordering::Relaxed) != signature {
            buffer_uploads.write_buffer(
                &self.trace_page_table_buffer,
                0,
                bytemuck::cast_slice(&page_table.slots),
            );
            let committed_signature = Arc::clone(&self.trace_page_signature);
            frame_transactions.register(RuntimePrepareFrameTransaction::new(
                "hybrid-gi.global-sdf.trace-page-table",
                move || committed_signature.store(signature, Ordering::Relaxed),
                || {},
            ));
        }
        GlobalSdfGpuTraceBindings {
            page_table_buffer: self.trace_page_table_buffer.clone(),
            atlas_buffer: self.atlas_buffer.clone(),
            page_count: page_table.page_count,
            clipmaps: page_table.clipmaps,
        }
    }
}

fn build_trace_page_table(scene: &HybridGiGlobalSdfSceneState) -> GlobalSdfTracePageTable {
    let mut table = GlobalSdfTracePageTable {
        slots: [GLOBAL_SDF_TRACE_PAGE_UNAVAILABLE_SLOT; GLOBAL_SDF_TRACE_PAGE_TABLE_ENTRY_COUNT],
        clipmaps: [GlobalSdfGpuTraceClipmap::zeroed(); GLOBAL_SDF_CLIPMAP_COUNT],
        page_count: 0,
    };
    for clipmap in scene.clipmap_bounds().iter().copied() {
        let Ok(clipmap_index) = usize::try_from(clipmap.clipmap_id()) else {
            continue;
        };
        if clipmap_index >= table.clipmaps.len() {
            continue;
        }
        let origin = clipmap.page_coordinate_origin();
        table.clipmaps[clipmap_index] = GlobalSdfGpuTraceClipmap {
            page_coordinate_origin_and_padding: [origin[0], origin[1], origin[2], 0],
            page_world_size_and_padding: [clipmap.page_world_size(), 0.0, 0.0, 0.0],
        };
    }
    for page in scene.sampleable_pages() {
        if page.atlas_slot() >= GLOBAL_SDF_MAX_RESIDENT_PAGE_COUNT as u32 {
            continue;
        }
        let Some(index) = trace_page_table_index(page.key(), &table.clipmaps) else {
            continue;
        };
        if table.slots[index] == GLOBAL_SDF_TRACE_PAGE_UNAVAILABLE_SLOT {
            table.page_count = table.page_count.saturating_add(1);
        }
        table.slots[index] = page.atlas_slot();
    }
    table
}

fn trace_page_table_index(
    key: HybridGiGlobalSdfPageKey,
    clipmaps: &[GlobalSdfGpuTraceClipmap; GLOBAL_SDF_CLIPMAP_COUNT],
) -> Option<usize> {
    let clipmap_index = usize::try_from(key.clipmap_id()).ok()?;
    let clipmap = *clipmaps.get(clipmap_index)?;
    let coordinate = key.page_coordinate();
    let origin = clipmap.page_coordinate_origin_and_padding;
    let local_coordinate = [
        coordinate[0].checked_sub(origin[0])?,
        coordinate[1].checked_sub(origin[1])?,
        coordinate[2].checked_sub(origin[2])?,
    ];
    let edge = GLOBAL_SDF_PAGES_PER_EDGE;
    if local_coordinate
        .iter()
        .any(|coordinate| *coordinate < 0 || *coordinate >= edge)
    {
        return None;
    }
    let x = usize::try_from(local_coordinate[0]).ok()?;
    let y = usize::try_from(local_coordinate[1]).ok()?;
    let z = usize::try_from(local_coordinate[2]).ok()?;
    Some(clipmap_index * GLOBAL_SDF_PAGES_PER_CLIPMAP + (z * edge as usize + y) * edge as usize + x)
}

fn trace_page_signature(table: &GlobalSdfTracePageTable) -> u64 {
    let mut signature = FNV64_OFFSET_BASIS ^ u64::from(table.page_count);
    for byte in bytemuck::cast_slice::<u32, u8>(&table.slots)
        .iter()
        .chain(bytemuck::cast_slice::<GlobalSdfGpuTraceClipmap, u8>(
            &table.clipmaps,
        ))
    {
        signature ^= u64::from(*byte);
        signature = signature.wrapping_mul(FNV64_PRIME);
    }
    signature
}

#[cfg(test)]
#[path = "tests/trace_bindings.rs"]
mod tests;
