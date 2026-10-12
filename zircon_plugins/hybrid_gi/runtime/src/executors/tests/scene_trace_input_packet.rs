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
