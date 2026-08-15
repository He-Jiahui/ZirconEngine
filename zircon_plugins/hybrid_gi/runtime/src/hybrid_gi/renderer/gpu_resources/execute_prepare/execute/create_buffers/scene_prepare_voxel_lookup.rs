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
mod tests {
    use super::*;
    use crate::hybrid_gi::types::{HybridGiPrepareVoxelCell, HybridGiPrepareVoxelClipmap};
    use zircon_runtime::core::math::Vec3;

    #[test]
    fn voxel_lookup_uses_fixed_slots_and_invalid_sentinels() {
        let source = include_str!("scene_prepare_voxel_lookup.rs");

        assert!(source.contains("SCENE_PREPARE_VOXEL_LOOKUP_MAX_CLIPMAPS: usize = 8"));
        assert!(source.contains("1 + HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT"));
        assert!(
            source.contains("SCENE_PREPARE_VOXEL_LOOKUP_INVALID_DESCRIPTOR_INDEX: u32 = u32::MAX")
        );
        assert!(source
            .contains("cells[cell_index] != SCENE_PREPARE_VOXEL_LOOKUP_INVALID_DESCRIPTOR_INDEX"));
        assert!(source.contains("BTreeMap::<u32, [u32; HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT]>::new"));
        assert!(source.contains("SCENE_PREPARE_VOXEL_LOOKUP_MAX_CLIPMAPS"));
        assert!(source.contains("SCENE_PREPARE_VOXEL_LOOKUP_WORDS_PER_CLIPMAP"));
        assert!(source.contains(
            "SCENE_PREPARE_VOXEL_LOOKUP_MAX_CLIPMAPS * HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT"
        ));
    }

    #[test]
    fn voxel_lookup_sorts_clipmaps_and_uses_cell_slots_without_a_descriptor_scan() {
        let clipmaps = [
            HybridGiPrepareVoxelClipmap {
                clipmap_id: 9,
                center: Vec3::ZERO,
                half_extent: 8.0,
            },
            HybridGiPrepareVoxelClipmap {
                clipmap_id: 3,
                center: Vec3::ZERO,
                half_extent: 8.0,
            },
        ];
        let descriptors = super::super::scene_prepare_descriptors::gpu_scene_prepare_descriptors(
            &[],
            &[],
            &[],
            &[],
            &clipmaps,
            &[
                HybridGiPrepareVoxelCell {
                    clipmap_id: 9,
                    cell_index: 42,
                    occupancy_count: 1,
                    dominant_card_id: 1,
                    radiance_present: true,
                    radiance_rgb: [10, 20, 30],
                },
                HybridGiPrepareVoxelCell {
                    clipmap_id: 3,
                    cell_index: 58,
                    occupancy_count: 1,
                    dominant_card_id: 2,
                    radiance_present: true,
                    radiance_rgb: [30, 20, 10],
                },
            ],
        );
        let (offset, count) =
            super::super::scene_prepare_descriptors::gpu_scene_prepare_voxel_cell_descriptor_range(
                &descriptors,
            );

        let lookup = gpu_scene_prepare_voxel_lookup_words(&descriptors, offset, count, &clipmaps)
            .expect("valid voxel descriptors must build the fixed lookup");
        let words_per_clipmap = SCENE_PREPARE_VOXEL_LOOKUP_WORDS_PER_CLIPMAP;

        assert_eq!(
            lookup.words.len(),
            SCENE_PREPARE_VOXEL_LOOKUP_MAX_CLIPMAPS * words_per_clipmap
        );
        assert_eq!(lookup.clipmap_count, 2);
        assert_eq!(lookup.words[0], 3);
        assert_eq!(lookup.words[1 + 58], 3);
        assert_eq!(lookup.words[words_per_clipmap], 9);
        assert_eq!(lookup.words[words_per_clipmap + 1 + 42], 2);
        assert_eq!(
            lookup.words[1],
            SCENE_PREPARE_VOXEL_LOOKUP_INVALID_DESCRIPTOR_INDEX
        );
    }

    #[test]
    fn voxel_lookup_fails_closed_for_ambiguous_or_out_of_contract_clipmap_input() {
        assert!(voxel_lookup_for(
            &[voxel_clipmap(
                SCENE_PREPARE_VOXEL_LOOKUP_INVALID_DESCRIPTOR_INDEX
            )],
            &[voxel_cell(
                SCENE_PREPARE_VOXEL_LOOKUP_INVALID_DESCRIPTOR_INDEX,
                0
            )],
        )
        .is_none());
        assert!(
            voxel_lookup_for(&[voxel_clipmap(4)], &[voxel_cell(4, 7), voxel_cell(4, 7)],).is_none()
        );
        assert!(
            voxel_lookup_for(&[voxel_clipmap(4), voxel_clipmap(4)], &[voxel_cell(4, 7)],).is_none()
        );
        assert!(voxel_lookup_for(&[voxel_clipmap(4)], &[voxel_cell(5, 7)]).is_none());

        let too_many_clipmaps = (0..=SCENE_PREPARE_VOXEL_LOOKUP_MAX_CLIPMAPS as u32)
            .map(voxel_clipmap)
            .collect::<Vec<_>>();
        assert!(voxel_lookup_for(&too_many_clipmaps, &[voxel_cell(0, 0)]).is_none());
    }

    fn voxel_lookup_for(
        clipmaps: &[HybridGiPrepareVoxelClipmap],
        voxel_cells: &[HybridGiPrepareVoxelCell],
    ) -> Option<GpuScenePrepareVoxelLookup> {
        let descriptors = super::super::scene_prepare_descriptors::gpu_scene_prepare_descriptors(
            &[],
            &[],
            &[],
            &[],
            clipmaps,
            voxel_cells,
        );
        let (offset, count) =
            super::super::scene_prepare_descriptors::gpu_scene_prepare_voxel_cell_descriptor_range(
                &descriptors,
            );
        gpu_scene_prepare_voxel_lookup_words(&descriptors, offset, count, clipmaps)
    }

    fn voxel_clipmap(clipmap_id: u32) -> HybridGiPrepareVoxelClipmap {
        HybridGiPrepareVoxelClipmap {
            clipmap_id,
            center: Vec3::ZERO,
            half_extent: 8.0,
        }
    }

    fn voxel_cell(clipmap_id: u32, cell_index: u32) -> HybridGiPrepareVoxelCell {
        HybridGiPrepareVoxelCell {
            clipmap_id,
            cell_index,
            occupancy_count: 1,
            dominant_card_id: 1,
            radiance_present: true,
            radiance_rgb: [10, 20, 30],
        }
    }
}
