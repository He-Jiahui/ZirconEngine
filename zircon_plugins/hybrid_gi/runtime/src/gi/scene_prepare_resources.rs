use std::collections::BTreeMap;

use zircon_runtime::core::framework::render::{
    RenderHybridGiSurfaceCachePageRecord, RenderHybridGiVoxelClipmapRecord,
};

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
use crate::hybrid_gi::renderer::{HybridGiGpuResources, HybridGiScenePrepareResourcesSnapshot};

use super::super::material_capture_source::HybridGiMaterialCaptureSource;

pub(super) fn scene_prepare_resources(
    resources: &HybridGiGpuResources,
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    streamer: &impl HybridGiMaterialCaptureSource,
    inputs: &HybridGiPrepareExecutionInputs,
    tracing_budget: Option<u32>,
) -> Option<HybridGiPrepareScenePrepareResources> {
    let has_present_surface_cache_page_content = inputs
        .scene_surface_cache_page_contents
        .iter()
        .any(persisted_surface_cache_page_has_present_sample);
    // 干净帧可能仍有持久页或独立深度源；四类输入均缺席时才跳过资源创建。
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

    let mut pages = BTreeMap::new();
    for page in &inputs.scene_surface_cache_page_contents {
        if !persisted_surface_cache_page_has_present_sample(page) {
            continue;
        }
        pages.insert(
            (page.page_id, page.atlas_slot_id),
            RenderHybridGiSurfaceCachePageRecord {
                page_id: page.page_id,
                owner_card_id: page.owner_card_id,
                atlas_slot_id: page.atlas_slot_id,
                bounds_center_x_bits: page.bounds_center.x.to_bits(),
                bounds_center_y_bits: page.bounds_center.y.to_bits(),
                bounds_center_z_bits: page.bounds_center.z.to_bits(),
                bounds_radius_bits: page.bounds_radius.to_bits(),
                radiance_rgba8: [0; 4],
            },
        );
    }
    // Dirty capture requests replace the same physical page's previous descriptor.
    for request in &inputs.scene_card_capture_requests {
        pages.insert(
            (request.page_id, request.atlas_slot_id),
            RenderHybridGiSurfaceCachePageRecord {
                page_id: request.page_id,
                owner_card_id: request.card_id,
                atlas_slot_id: request.atlas_slot_id,
                bounds_center_x_bits: request.bounds_center.x.to_bits(),
                bounds_center_y_bits: request.bounds_center.y.to_bits(),
                bounds_center_z_bits: request.bounds_center.z.to_bits(),
                bounds_radius_bits: request.bounds_radius.to_bits(),
                radiance_rgba8: [0; 4],
            },
        );
    }
    snapshot.store_source_descriptors(
        pages
            .into_values()
            .filter(|page| {
                snapshot
                    .occupied_atlas_slots()
                    .contains(&page.atlas_slot_id)
            })
            .collect(),
        inputs
            .scene_voxel_clipmaps
            .iter()
            .map(|clipmap| RenderHybridGiVoxelClipmapRecord {
                clipmap_id: clipmap.clipmap_id,
                center_x_bits: clipmap.center.x.to_bits(),
                center_y_bits: clipmap.center.y.to_bits(),
                center_z_bits: clipmap.center.z.to_bits(),
                half_extent_bits: clipmap.half_extent.to_bits(),
            })
            .collect(),
    );

    store_scene_prepare_voxel_resource_samples(&mut snapshot, streamer, inputs);
    store_scene_prepare_texture_samples(&mut snapshot, streamer, inputs);
    store_scene_prepare_surface_cache_depth_samples(&mut snapshot, inputs);
    store_scene_prepare_probe_trace_tiles(&mut snapshot, inputs, tracing_budget);
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
        probe_trace_indirect_args_buffer,
        probe_trace_tile_word_count,
        probe_trace_tile_record_count,
        probe_trace_indirect_arg_word_count,
    } = scene_prepare_probe_trace_tile_resources(resources, device, encoder, &snapshot);

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
        probe_trace_indirect_args_buffer,
        probe_trace_tile_word_count,
        probe_trace_tile_record_count,
        probe_trace_indirect_arg_word_count,
    })
}
