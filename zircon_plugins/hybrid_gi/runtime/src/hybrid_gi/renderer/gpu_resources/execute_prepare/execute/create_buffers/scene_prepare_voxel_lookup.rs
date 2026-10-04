use std::collections::{BTreeMap, BTreeSet};

use super::scene_prepare_descriptors::GpuScenePrepareDescriptor;
use crate::hybrid_gi::types::{HybridGiPrepareVoxelClipmap, HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT};

pub(super) const SCENE_PREPARE_VOXEL_LOOKUP_MAX_CLIPMAPS: usize = 8;
const SCENE_PREPARE_VOXEL_LOOKUP_INVALID_DESCRIPTOR_INDEX: u32 = u32::MAX;
const SCENE_PREPARE_VOXEL_LOOKUP_WORDS_PER_CLIPMAP: usize = 1 + HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT;

pub(super) struct GpuScenePrepareVoxelLookup {
    pub(super) words: Vec<u32>,
    pub(super) clipmap_count: usize,
}

pub(super) fn gpu_scene_prepare_voxel_lookup_words(
    descriptors: &[GpuScenePrepareDescriptor],
    voxel_cell_descriptor_offset: usize,
    voxel_cell_descriptor_count: usize,
    voxel_clipmaps: &[HybridGiPrepareVoxelClipmap],
) -> Option<GpuScenePrepareVoxelLookup> {
    if voxel_cell_descriptor_count == 0
        || voxel_clipmaps.is_empty()
        || voxel_clipmaps.len() > SCENE_PREPARE_VOXEL_LOOKUP_MAX_CLIPMAPS
        || voxel_cell_descriptor_count
            > SCENE_PREPARE_VOXEL_LOOKUP_MAX_CLIPMAPS * HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT
    {
        return None;
    }

    // 任何重复、越界或未声明的映射都会使整张表失效，trace 路由随之禁用体素后端。
    let mut declared_clipmap_ids = BTreeSet::new();
    for clipmap in voxel_clipmaps {
        if clipmap.clipmap_id == SCENE_PREPARE_VOXEL_LOOKUP_INVALID_DESCRIPTOR_INDEX
            || !declared_clipmap_ids.insert(clipmap.clipmap_id)
        {
            return None;
        }
    }

    let Some(voxel_descriptors) = descriptors.get(
        voxel_cell_descriptor_offset
            ..voxel_cell_descriptor_offset.saturating_add(voxel_cell_descriptor_count),
    ) else {
        return None;
    };
    let mut cells_by_clipmap = BTreeMap::<u32, [u32; HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT]>::new();

    for (local_descriptor_index, descriptor) in voxel_descriptors.iter().enumerate() {
        if !descriptor.is_voxel_cell() {
            return None;
        }
        let cell_index = descriptor.voxel_cell_index() as usize;
        if cell_index >= HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT {
            return None;
        }
        let clipmap_id = descriptor.clipmap_id();
        if clipmap_id == SCENE_PREPARE_VOXEL_LOOKUP_INVALID_DESCRIPTOR_INDEX
            || !declared_clipmap_ids.contains(&clipmap_id)
        {
            return None;
        }
        let descriptor_index = voxel_cell_descriptor_offset
            .saturating_add(local_descriptor_index)
            .try_into()
            .ok()?;
        if descriptor_index == SCENE_PREPARE_VOXEL_LOOKUP_INVALID_DESCRIPTOR_INDEX {
            return None;
        }
        let cells = cells_by_clipmap.entry(clipmap_id).or_insert(
            [SCENE_PREPARE_VOXEL_LOOKUP_INVALID_DESCRIPTOR_INDEX;
                HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT],
        );
        if cells[cell_index] != SCENE_PREPARE_VOXEL_LOOKUP_INVALID_DESCRIPTOR_INDEX {
            return None;
        }
        cells[cell_index] = descriptor_index;
    }

    if cells_by_clipmap.is_empty()
        || cells_by_clipmap.len() > SCENE_PREPARE_VOXEL_LOOKUP_MAX_CLIPMAPS
    {
        return None;
    }
    // The frame-owned table prevents trace work from scanning every packed descriptor.
    let mut words = vec![
        SCENE_PREPARE_VOXEL_LOOKUP_INVALID_DESCRIPTOR_INDEX;
        SCENE_PREPARE_VOXEL_LOOKUP_MAX_CLIPMAPS
            * SCENE_PREPARE_VOXEL_LOOKUP_WORDS_PER_CLIPMAP
    ];
    for (lookup_index, (clipmap_id, cells)) in cells_by_clipmap.into_iter().enumerate() {
        let base = lookup_index * SCENE_PREPARE_VOXEL_LOOKUP_WORDS_PER_CLIPMAP;
        words[base] = clipmap_id;
        words[base + 1..base + SCENE_PREPARE_VOXEL_LOOKUP_WORDS_PER_CLIPMAP]
            .copy_from_slice(&cells);
    }
    let clipmap_count = words
        .chunks_exact(SCENE_PREPARE_VOXEL_LOOKUP_WORDS_PER_CLIPMAP)
        .take_while(|entry| entry[0] != SCENE_PREPARE_VOXEL_LOOKUP_INVALID_DESCRIPTOR_INDEX)
        .count();
    Some(GpuScenePrepareVoxelLookup {
        words,
        clipmap_count,
    })
}

#[cfg(test)]
#[path = "tests/scene_prepare_voxel_lookup.rs"]
mod tests;
