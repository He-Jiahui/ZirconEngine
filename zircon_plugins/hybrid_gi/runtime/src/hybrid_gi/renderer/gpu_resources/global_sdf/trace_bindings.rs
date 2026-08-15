use std::sync::atomic::Ordering;

use bytemuck::{Pod, Zeroable};

use crate::hybrid_gi::scene_representation::{
    GLOBAL_SDF_CLIPMAP_COUNT, GLOBAL_SDF_MAX_RESIDENT_PAGE_COUNT, GLOBAL_SDF_PAGES_PER_EDGE,
    HybridGiGlobalSdfPageKey, HybridGiGlobalSdfSceneState,
};

use super::GlobalSdfGpuState;
use super::state::{
    GLOBAL_SDF_TRACE_PAGE_TABLE_ENTRY_COUNT, GLOBAL_SDF_TRACE_PAGE_UNAVAILABLE_SLOT,
};

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
        queue: &wgpu::Queue,
        scene: &HybridGiGlobalSdfSceneState,
    ) -> GlobalSdfGpuTraceBindings {
        let page_table = build_trace_page_table(scene);
        let signature = trace_page_signature(&page_table);
        if self.trace_page_signature.load(Ordering::Relaxed) != signature {
            queue.write_buffer(
                &self.trace_page_table_buffer,
                0,
                bytemuck::cast_slice(&page_table.slots),
            );
            self.trace_page_signature
                .store(signature, Ordering::Relaxed);
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
mod tests {
    use zircon_runtime::core::math::Vec3;

    use super::*;

    #[test]
    fn trace_page_table_uses_clipmap_origin_for_sampleable_pages() {
        let mut scene = HybridGiGlobalSdfSceneState::default();
        scene.synchronize(Vec3::ZERO, &[], 4);
        let requests = scene.dirty_page_build_requests();
        scene.commit_pages(&requests);

        let table = build_trace_page_table(&scene);

        assert_eq!(table.page_count, requests.len() as u32);
        for request in requests {
            let index = trace_page_table_index(request.key(), &table.clipmaps)
                .expect("resident page must map into its clipmap table");
            assert_eq!(table.slots[index], request.atlas_slot());
        }
    }

    #[test]
    fn trace_page_table_does_not_publish_uninitialized_pages() {
        let mut scene = HybridGiGlobalSdfSceneState::default();
        scene.synchronize(Vec3::ZERO, &[], 1);
        let request = scene.dirty_page_build_requests()[0];

        let table = build_trace_page_table(&scene);
        let index = trace_page_table_index(request.key(), &table.clipmaps)
            .expect("resident page must map into its clipmap table");

        assert_eq!(table.page_count, 0);
        assert_eq!(table.slots[index], GLOBAL_SDF_TRACE_PAGE_UNAVAILABLE_SLOT);
    }

    #[test]
    fn trace_page_table_keeps_typed_fallback_pages_unavailable() {
        let mut scene = HybridGiGlobalSdfSceneState::default();
        scene.synchronize(Vec3::ZERO, &[], 1);
        let requests = scene.dirty_page_build_requests();
        scene.commit_pages(&requests);
        let fallback = requests[0];
        scene.resolve_pages_to_fallback(&[fallback]);

        let table = build_trace_page_table(&scene);
        let index = trace_page_table_index(fallback.key(), &table.clipmaps)
            .expect("resident fallback page remains in the clipmap domain");

        assert_eq!(table.page_count, 0);
        assert_eq!(table.slots[index], GLOBAL_SDF_TRACE_PAGE_UNAVAILABLE_SLOT);
    }

    #[test]
    fn trace_clipmap_abi_matches_wgsl_storage_layout() {
        assert_eq!(std::mem::size_of::<GlobalSdfGpuTraceClipmap>(), 32);
        assert_eq!(
            std::mem::offset_of!(GlobalSdfGpuTraceClipmap, page_coordinate_origin_and_padding),
            0
        );
        assert_eq!(
            std::mem::offset_of!(GlobalSdfGpuTraceClipmap, page_world_size_and_padding),
            16
        );
    }
}
