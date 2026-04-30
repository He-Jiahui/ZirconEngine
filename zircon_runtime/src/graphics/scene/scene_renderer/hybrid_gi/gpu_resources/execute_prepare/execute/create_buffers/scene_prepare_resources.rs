use super::super::hybrid_gi_prepare_execution_buffers::HybridGiPrepareScenePrepareResources;
use super::super::hybrid_gi_prepare_execution_inputs::HybridGiPrepareExecutionInputs;
use super::scene_prepare_descriptors::persisted_surface_cache_page_has_present_sample;
use super::scene_prepare_textures::{
    scene_prepare_texture_layout, scene_prepare_texture_resources,
    store_scene_prepare_texture_samples, ScenePrepareTextureLayout, ScenePrepareTextureResources,
};
use super::scene_prepare_voxel_samples::store_scene_prepare_voxel_resource_samples;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;

pub(super) fn scene_prepare_resources(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    streamer: &ResourceStreamer,
    inputs: &HybridGiPrepareExecutionInputs,
) -> Option<HybridGiPrepareScenePrepareResources> {
    let has_present_surface_cache_page_content = inputs
        .scene_surface_cache_page_contents
        .iter()
        .any(persisted_surface_cache_page_has_present_sample);
    if inputs.scene_card_capture_requests.is_empty()
        && !has_present_surface_cache_page_content
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
    })
}
