use std::collections::BTreeMap;

use zircon_runtime::core::framework::render::RenderHybridGiScenePrepareReadbackOutputs;

pub(super) const SCENE_TRACE_INPUT_WORD_OFFSET: usize = 294;
pub(super) const SCENE_TRACE_INPUT_PACKET_WORD_COUNT: usize = 416;
pub(super) const SCENE_TRACE_INPUT_TOTAL_WORD_COUNT: usize =
    SCENE_TRACE_INPUT_WORD_OFFSET + SCENE_TRACE_INPUT_PACKET_WORD_COUNT;

const SCENE_TRACE_INPUT_MAGIC: u32 = 0x4847_4949;
const SCENE_TRACE_INPUT_HEADER_WORD_COUNT: usize = 8;
const SURFACE_CACHE_PAGE_CAPACITY: usize = 16;
const SURFACE_CACHE_PAGE_WORD_COUNT: usize = 8;
const VOXEL_CLIPMAP_CAPACITY: usize = 4;
const VOXEL_CLIPMAP_WORD_COUNT: usize = 6;
const VOXEL_CELL_CAPACITY: usize = 64;
const VOXEL_CELL_WORD_COUNT: usize = 4;

const SURFACE_CACHE_PAGE_WORD_OFFSET: usize =
    SCENE_TRACE_INPUT_WORD_OFFSET + SCENE_TRACE_INPUT_HEADER_WORD_COUNT;
const VOXEL_CLIPMAP_WORD_OFFSET: usize =
    SURFACE_CACHE_PAGE_WORD_OFFSET + SURFACE_CACHE_PAGE_CAPACITY * SURFACE_CACHE_PAGE_WORD_COUNT;
const VOXEL_CELL_WORD_OFFSET: usize =
    VOXEL_CLIPMAP_WORD_OFFSET + VOXEL_CLIPMAP_CAPACITY * VOXEL_CLIPMAP_WORD_COUNT;

pub(super) fn scene_trace_input_packet(
    outputs: &RenderHybridGiScenePrepareReadbackOutputs,
) -> [u32; SCENE_TRACE_INPUT_PACKET_WORD_COUNT] {
    let mut packet = [0_u32; SCENE_TRACE_INPUT_PACKET_WORD_COUNT];
    let mut pages = BTreeMap::new();
    for page in &outputs.surface_cache_pages {
        pages
            .entry((page.page_id, page.atlas_slot_id))
            .or_insert(page);
    }
    let pages = pages
        .into_values()
        .take(SURFACE_CACHE_PAGE_CAPACITY)
        .collect::<Vec<_>>();

    let mut clipmaps = BTreeMap::new();
    for clipmap in &outputs.voxel_clipmaps {
        clipmaps.entry(clipmap.clipmap_id).or_insert(clipmap);
    }
    let clipmaps = clipmaps
        .into_values()
        .take(VOXEL_CLIPMAP_CAPACITY)
        .collect::<Vec<_>>();

    let occupancy_by_cell = outputs
        .voxel_cells
        .iter()
        .map(|cell| ((cell.clipmap_id, cell.cell_id), cell.occupancy))
        .collect::<BTreeMap<_, _>>();
    let mut radiance_by_cell = BTreeMap::new();
    for sample in &outputs.voxel_cell_samples {
        radiance_by_cell.insert((sample.clipmap_id, sample.cell_id), sample.rgba8);
    }
    for sample in &outputs.voxel_cell_dominant_samples {
        radiance_by_cell.insert((sample.clipmap_id, sample.cell_id), sample.rgba8);
    }
    let cells = radiance_by_cell
        .into_iter()
        .take(VOXEL_CELL_CAPACITY)
        .collect::<Vec<_>>();

    packet[0] = SCENE_TRACE_INPUT_MAGIC;
    packet[1] = pages.len() as u32;
    packet[2] = clipmaps.len() as u32;
    packet[3] = cells.len() as u32;
    packet[4] = SURFACE_CACHE_PAGE_WORD_OFFSET as u32;
    packet[5] = VOXEL_CLIPMAP_WORD_OFFSET as u32;
    packet[6] = VOXEL_CELL_WORD_OFFSET as u32;
    packet[7] = 0;

    for (record_index, page) in pages.into_iter().enumerate() {
        let offset = relative_packet_offset(
            SURFACE_CACHE_PAGE_WORD_OFFSET + record_index * SURFACE_CACHE_PAGE_WORD_COUNT,
        );
        packet[offset] = page.page_id;
        packet[offset + 1] = page.owner_card_id;
        packet[offset + 2] = page.atlas_slot_id;
        packet[offset + 3] = pack_rgba8(page.radiance_rgba8);
        packet[offset + 4] = page.bounds_center_x_bits;
        packet[offset + 5] = page.bounds_center_y_bits;
        packet[offset + 6] = page.bounds_center_z_bits;
        packet[offset + 7] = page.bounds_radius_bits;
    }

    for (record_index, clipmap) in clipmaps.into_iter().enumerate() {
        let offset = relative_packet_offset(
            VOXEL_CLIPMAP_WORD_OFFSET + record_index * VOXEL_CLIPMAP_WORD_COUNT,
        );
        packet[offset] = clipmap.clipmap_id;
        packet[offset + 1] = clipmap.center_x_bits;
        packet[offset + 2] = clipmap.center_y_bits;
        packet[offset + 3] = clipmap.center_z_bits;
        packet[offset + 4] = clipmap.half_extent_bits;
        packet[offset + 5] = 4;
    }

    for (record_index, ((clipmap_id, cell_id), rgba8)) in cells.into_iter().enumerate() {
        let offset =
            relative_packet_offset(VOXEL_CELL_WORD_OFFSET + record_index * VOXEL_CELL_WORD_COUNT);
        packet[offset] = clipmap_id;
        packet[offset + 1] = cell_id;
        packet[offset + 2] = pack_rgba8(rgba8);
        packet[offset + 3] = occupancy_by_cell
            .get(&(clipmap_id, cell_id))
            .copied()
            .unwrap_or(1)
            .max(1);
    }

    packet[7] = scene_trace_input_signature(&packet);

    packet
}

fn scene_trace_input_signature(packet: &[u32; SCENE_TRACE_INPUT_PACKET_WORD_COUNT]) -> u32 {
    let mut hash = 0x811c_9dc5_u32;
    for (index, word) in packet.iter().copied().enumerate() {
        if index == 7 {
            continue;
        }
        hash ^= index as u32;
        hash = hash.wrapping_mul(0x0100_0193);
        hash ^= word;
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash.max(1)
}

const fn relative_packet_offset(absolute_word_offset: usize) -> usize {
    absolute_word_offset - SCENE_TRACE_INPUT_WORD_OFFSET
}

const fn pack_rgba8(rgba8: [u8; 4]) -> u32 {
    rgba8[0] as u32
        | ((rgba8[1] as u32) << 8)
        | ((rgba8[2] as u32) << 16)
        | ((rgba8[3] as u32) << 24)
}

#[cfg(test)]
mod tests {
    use zircon_runtime::core::framework::render::{
        RenderHybridGiScenePrepareReadbackOutputs, RenderHybridGiSurfaceCachePageRecord,
        RenderHybridGiVoxelCellRecord, RenderHybridGiVoxelCellSampleRecord,
        RenderHybridGiVoxelClipmapRecord,
    };

    use super::*;

    #[test]
    fn scene_trace_packet_carries_world_space_surface_cache_and_voxel_records() {
        let outputs = RenderHybridGiScenePrepareReadbackOutputs {
            surface_cache_pages: vec![RenderHybridGiSurfaceCachePageRecord {
                page_id: 7,
                owner_card_id: 11,
                atlas_slot_id: 13,
                bounds_center_x_bits: 1.0_f32.to_bits(),
                bounds_center_y_bits: 2.0_f32.to_bits(),
                bounds_center_z_bits: 3.0_f32.to_bits(),
                bounds_radius_bits: 4.0_f32.to_bits(),
                radiance_rgba8: [24, 48, 96, 255],
            }],
            voxel_clipmaps: vec![RenderHybridGiVoxelClipmapRecord {
                clipmap_id: 5,
                center_x_bits: 0.0_f32.to_bits(),
                center_y_bits: 1.0_f32.to_bits(),
                center_z_bits: 2.0_f32.to_bits(),
                half_extent_bits: 8.0_f32.to_bits(),
            }],
            voxel_cells: vec![RenderHybridGiVoxelCellRecord {
                clipmap_id: 5,
                cell_id: 9,
                occupancy: 3,
            }],
            voxel_cell_samples: vec![RenderHybridGiVoxelCellSampleRecord {
                clipmap_id: 5,
                cell_id: 9,
                rgba8: [12, 36, 72, 255],
            }],
            ..RenderHybridGiScenePrepareReadbackOutputs::default()
        };

        let packet = scene_trace_input_packet(&outputs);

        assert_eq!(packet[0], SCENE_TRACE_INPUT_MAGIC);
        assert_eq!(packet[1..4], [1, 1, 1]);
        assert_eq!(packet[4], SURFACE_CACHE_PAGE_WORD_OFFSET as u32);
        let page_offset = relative_packet_offset(SURFACE_CACHE_PAGE_WORD_OFFSET);
        assert_eq!(
            packet[page_offset..page_offset + 4],
            [7, 11, 13, 0xff60_3018]
        );
        let clipmap_offset = relative_packet_offset(VOXEL_CLIPMAP_WORD_OFFSET);
        assert_eq!(packet[clipmap_offset], 5);
        assert_eq!(packet[clipmap_offset + 5], 4);
        let cell_offset = relative_packet_offset(VOXEL_CELL_WORD_OFFSET);
        assert_eq!(packet[cell_offset..cell_offset + 4], [5, 9, 0xff48_240c, 3]);
        assert_ne!(packet[7], 0);
        assert_eq!(SCENE_TRACE_INPUT_TOTAL_WORD_COUNT, 710);

        let mut changed = outputs;
        changed.surface_cache_pages[0].radiance_rgba8 = [96, 48, 24, 255];
        let changed_packet = scene_trace_input_packet(&changed);
        assert_ne!(changed_packet[7], packet[7]);
        assert_eq!(scene_trace_input_packet(&changed)[7], changed_packet[7]);
    }
}
