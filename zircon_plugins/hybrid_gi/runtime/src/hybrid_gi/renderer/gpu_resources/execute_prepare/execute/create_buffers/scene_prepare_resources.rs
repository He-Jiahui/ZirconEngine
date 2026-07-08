use super::super::hybrid_gi_prepare_execution_buffers::HybridGiPrepareScenePrepareResources;
use super::super::hybrid_gi_prepare_execution_inputs::HybridGiPrepareExecutionInputs;
use super::scene_prepare_depth_samples::{
    scene_prepare_surface_cache_depth_resources, store_scene_prepare_surface_cache_depth_samples,
    ScenePrepareSurfaceCacheDepthResources,
};
use super::scene_prepare_descriptors::persisted_surface_cache_page_has_present_sample;
use super::scene_prepare_textures::{
    scene_prepare_texture_layout, scene_prepare_texture_resources,
    store_scene_prepare_texture_samples, ScenePrepareTextureLayout, ScenePrepareTextureResources,
};
use super::scene_prepare_trace_tiles::{
    scene_prepare_probe_trace_tile_resources, store_scene_prepare_probe_trace_tiles,
    ScenePrepareProbeTraceTileResources,
};
use super::scene_prepare_voxel_samples::store_scene_prepare_voxel_resource_samples;
use crate::hybrid_gi::renderer::HybridGiScenePrepareResourcesSnapshot;

use super::super::material_capture_source::HybridGiMaterialCaptureSource;

pub(super) fn scene_prepare_resources(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    streamer: &impl HybridGiMaterialCaptureSource,
    inputs: &HybridGiPrepareExecutionInputs,
) -> Option<HybridGiPrepareScenePrepareResources> {
    let has_present_surface_cache_page_content = inputs
        .scene_surface_cache_page_contents
        .iter()
        .any(persisted_surface_cache_page_has_present_sample);
    if inputs.scene_card_capture_requests.is_empty()
        && !has_present_surface_cache_page_content
        && inputs
            .scene_surface_cache_depth_source_samples
            .iter()
            .all(|sample| sample.depth_rgba[3] == 0 || sample.atlas_slot_id == u32::MAX)
        && inputs.scene_voxel_clipmaps.is_empty()
    {
        return None;
    }

    let ScenePrepareTextureLayout {
        occupied_atlas_slots,
        occupied_capture_slots,
        atlas_slot_count,
        capture_slot_count,
        atlas_texture_extent,
        capture_texture_extent,
    } = scene_prepare_texture_layout(inputs);
    let mut snapshot = HybridGiScenePrepareResourcesSnapshot::new(
        inputs.scene_card_capture_requests.len() as u32,
        inputs
            .scene_voxel_clipmaps
            .iter()
            .map(|clipmap| clipmap.clipmap_id)
            .collect(),
        occupied_atlas_slots,
        occupied_capture_slots,
        atlas_slot_count,
        capture_slot_count,
        atlas_texture_extent,
        capture_texture_extent,
        capture_slot_count,
    );

    store_scene_prepare_voxel_resource_samples(&mut snapshot, streamer, inputs);
    store_scene_prepare_texture_samples(&mut snapshot, streamer, inputs);
    store_scene_prepare_surface_cache_depth_samples(&mut snapshot, inputs);
    store_scene_prepare_probe_trace_tiles(&mut snapshot, inputs);
    let ScenePrepareTextureResources {
        atlas_texture,
        atlas_view,
        atlas_upload_buffer,
        atlas_slot_sample_buffers,
        capture_texture,
        capture_views,
        capture_upload_buffer,
        capture_slot_sample_buffers,
    } = scene_prepare_texture_resources(device, encoder, &snapshot, streamer, inputs);
    let ScenePrepareSurfaceCacheDepthResources {
        depth_texture: surface_cache_depth_texture,
        depth_view: surface_cache_depth_view,
        depth_upload_buffer: surface_cache_depth_upload_buffer,
        depth_slot_sample_buffers: surface_cache_depth_slot_sample_buffers,
    } = scene_prepare_surface_cache_depth_resources(device, encoder, &snapshot);
    let ScenePrepareProbeTraceTileResources {
        probe_trace_tile_seed_buffer,
        probe_trace_tile_params_buffer,
        probe_trace_tile_buffer,
        probe_trace_tile_readback,
        probe_trace_indirect_args_buffer,
        probe_trace_indirect_args_readback,
        probe_trace_tile_word_count,
        probe_trace_tile_record_count,
        probe_trace_indirect_arg_word_count,
    } = scene_prepare_probe_trace_tile_resources(device, encoder, &snapshot);

    Some(HybridGiPrepareScenePrepareResources {
        snapshot,
        atlas_texture,
        atlas_view,
        atlas_upload_buffer,
        atlas_slot_sample_buffers,
        capture_texture,
        capture_views,
        capture_upload_buffer,
        capture_slot_sample_buffers,
        surface_cache_depth_texture,
        surface_cache_depth_view,
        surface_cache_depth_upload_buffer,
        surface_cache_depth_slot_sample_buffers,
        probe_trace_tile_seed_buffer,
        probe_trace_tile_params_buffer,
        probe_trace_tile_buffer,
        probe_trace_tile_readback,
        probe_trace_indirect_args_buffer,
        probe_trace_indirect_args_readback,
        probe_trace_tile_word_count,
        probe_trace_tile_record_count,
        probe_trace_indirect_arg_word_count,
    })
}
