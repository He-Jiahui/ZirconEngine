use std::collections::BTreeMap;

use wgpu::util::DeviceExt;

use super::super::hybrid_gi_prepare_execution_inputs::HybridGiPrepareExecutionInputs;
use super::scene_prepare_descriptors::persisted_surface_cache_page_has_present_sample;
use super::surface_cache_depth_hierarchy::{
    build_surface_cache_depth_hierarchy, surface_cache_depth_hierarchy_mip_level_count,
    SURFACE_CACHE_DEPTH_HIERARCHY_FORMAT,
};
use crate::hybrid_gi::renderer::HybridGiScenePrepareResourcesSnapshot;
use zircon_runtime::core::math::Vec3;

const SURFACE_CACHE_DEPTH_TILE_EXTENT: u32 = 64;
const SURFACE_CACHE_DEPTH_ATLAS_COLUMNS: u32 = 8;
const SURFACE_CACHE_DEPTH_BYTES_PER_PIXEL: u32 = 4;
const SURFACE_CACHE_DEPTH_SAMPLE_READBACK_BYTES_PER_ROW: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

pub(super) struct ScenePrepareSurfaceCacheDepthResources {
    pub(super) depth_texture: Option<wgpu::Texture>,
    pub(super) depth_view: Option<wgpu::TextureView>,
    pub(super) depth_upload_buffer: Option<wgpu::Buffer>,
    pub(super) depth_slot_sample_buffers: Vec<(u32, wgpu::Buffer)>,
}

pub(super) fn store_scene_prepare_surface_cache_depth_samples(
    snapshot: &mut HybridGiScenePrepareResourcesSnapshot,
    inputs: &HybridGiPrepareExecutionInputs,
) {
    let samples = surface_cache_depth_samples(snapshot, inputs);
    snapshot.store_surface_cache_depth_samples(samples);
}

pub(super) fn scene_prepare_surface_cache_depth_resources(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    snapshot: &HybridGiScenePrepareResourcesSnapshot,
) -> ScenePrepareSurfaceCacheDepthResources {
    if snapshot.atlas_slot_count() == 0 || snapshot.surface_cache_depth_rgba_samples().is_empty() {
        return ScenePrepareSurfaceCacheDepthResources {
            depth_texture: None,
            depth_view: None,
            depth_upload_buffer: None,
            depth_slot_sample_buffers: Vec::new(),
        };
    }

    let atlas_extent = snapshot.atlas_texture_extent();
    let depth_rgba = surface_cache_depth_texture_rgba(snapshot);
    let mip_level_count = surface_cache_depth_hierarchy_mip_level_count(atlas_extent);
    let (depth_texture, depth_view) =
        create_surface_cache_depth_texture(device, atlas_extent, mip_level_count);
    let depth_upload_buffer = create_depth_texture_upload_buffer(
        device,
        "zircon-hybrid-gi-scene-prepare-surface-cache-depth-upload",
        &depth_rgba,
    );
    upload_depth_texture_rgba(encoder, &depth_upload_buffer, atlas_extent, &depth_texture);
    build_surface_cache_depth_hierarchy(
        device,
        encoder,
        &depth_texture,
        atlas_extent,
        mip_level_count,
    );
    let depth_slot_sample_buffers = snapshot
        .surface_cache_depth_rgba_samples()
        .iter()
        .map(|(slot_id, _)| {
            let buffer = create_depth_texture_sample_readback_buffer(
                device,
                "zircon-hybrid-gi-scene-prepare-surface-cache-depth-sample",
            );
            let (origin_x, origin_y) = atlas_slot_origin(*slot_id);
            enqueue_depth_texture_sample_readback(
                encoder,
                &depth_texture,
                wgpu::Origin3d {
                    x: origin_x,
                    y: origin_y,
                    z: 0,
                },
                &buffer,
            );
            (*slot_id, buffer)
        })
        .collect();

    ScenePrepareSurfaceCacheDepthResources {
        depth_texture: Some(depth_texture),
        depth_view: Some(depth_view),
        depth_upload_buffer: Some(depth_upload_buffer),
        depth_slot_sample_buffers,
    }
}

fn surface_cache_depth_samples(
    snapshot: &HybridGiScenePrepareResourcesSnapshot,
    inputs: &HybridGiPrepareExecutionInputs,
) -> Vec<(u32, [u8; 4])> {
    let mut depth_by_atlas_slot = inputs
        .scene_surface_cache_depth_source_samples
        .iter()
        .filter(|sample| sample.atlas_slot_id != u32::MAX && sample.depth_rgba[3] > 0)
        .map(|sample| (sample.atlas_slot_id, sample.depth_rgba))
        .collect::<BTreeMap<_, _>>();

    for page_content in inputs
        .scene_surface_cache_page_contents
        .iter()
        .filter(|page_content| persisted_surface_cache_page_has_present_sample(page_content))
        .filter(|page_content| page_content.atlas_slot_id != u32::MAX)
    {
        depth_by_atlas_slot
            .entry(page_content.atlas_slot_id)
            .or_insert_with(|| {
                depth_rgba_from_bounds(page_content.bounds_center, page_content.bounds_radius)
            });
    }

    for request in inputs
        .scene_card_capture_requests
        .iter()
        .filter(|request| request.atlas_slot_id != u32::MAX)
    {
        depth_by_atlas_slot
            .entry(request.atlas_slot_id)
            .or_insert_with(|| {
                depth_rgba_from_bounds(request.bounds_center, request.bounds_radius)
            });
    }

    snapshot
        .occupied_atlas_slots()
        .iter()
        .filter_map(|slot_id| {
            depth_by_atlas_slot
                .get(slot_id)
                .copied()
                .map(|rgba| (*slot_id, rgba))
        })
        .collect()
}

fn depth_rgba_from_bounds(bounds_center: Vec3, bounds_radius: f32) -> [u8; 4] {
    let radius = bounds_radius.max(0.0);
    let depth = (bounds_center.z.abs() + radius)
        / (bounds_center.length() + radius + 1.0).max(f32::EPSILON);
    let encoded = (depth.clamp(0.0, 1.0) * 255.0).round() as u8;
    [encoded, encoded, encoded, u8::MAX]
}

fn surface_cache_depth_texture_rgba(snapshot: &HybridGiScenePrepareResourcesSnapshot) -> Vec<u8> {
    let atlas_extent = snapshot.atlas_texture_extent();
    let mut pixels = vec![
        0_u8;
        (atlas_extent.0 * atlas_extent.1 * SURFACE_CACHE_DEPTH_BYTES_PER_PIXEL)
            as usize
    ];
    for &(slot_id, rgba) in snapshot.surface_cache_depth_rgba_samples() {
        fill_rgba_rect(
            &mut pixels,
            atlas_extent.0,
            atlas_slot_origin(slot_id),
            (
                SURFACE_CACHE_DEPTH_TILE_EXTENT,
                SURFACE_CACHE_DEPTH_TILE_EXTENT,
            ),
            rgba,
        );
    }
    pixels
}

fn create_surface_cache_depth_texture(
    device: &wgpu::Device,
    extent: (u32, u32),
    mip_level_count: u32,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-hybrid-gi-scene-prepare-surface-cache-depth"),
        size: wgpu::Extent3d {
            width: extent.0,
            height: extent.1,
            depth_or_array_layers: 1,
        },
        mip_level_count,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: SURFACE_CACHE_DEPTH_HIERARCHY_FORMAT,
        usage: wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("zircon-hybrid-gi-scene-prepare-surface-cache-depth-view"),
        ..Default::default()
    });
    (texture, view)
}

fn create_depth_texture_upload_buffer(
    device: &wgpu::Device,
    label: &'static str,
    contents: &[u8],
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents,
        usage: wgpu::BufferUsages::COPY_SRC,
    })
}

fn create_depth_texture_sample_readback_buffer(
    device: &wgpu::Device,
    label: &'static str,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: SURFACE_CACHE_DEPTH_SAMPLE_READBACK_BYTES_PER_ROW as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}

fn upload_depth_texture_rgba(
    encoder: &mut wgpu::CommandEncoder,
    upload_buffer: &wgpu::Buffer,
    extent: (u32, u32),
    texture: &wgpu::Texture,
) {
    encoder.copy_buffer_to_texture(
        wgpu::TexelCopyBufferInfo {
            buffer: upload_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(extent.0 * SURFACE_CACHE_DEPTH_BYTES_PER_PIXEL),
                rows_per_image: Some(extent.1),
            },
        },
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::Extent3d {
            width: extent.0,
            height: extent.1,
            depth_or_array_layers: 1,
        },
    );
}

fn enqueue_depth_texture_sample_readback(
    encoder: &mut wgpu::CommandEncoder,
    texture: &wgpu::Texture,
    origin: wgpu::Origin3d,
    readback_buffer: &wgpu::Buffer,
) {
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: readback_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(SURFACE_CACHE_DEPTH_SAMPLE_READBACK_BYTES_PER_ROW),
                rows_per_image: Some(1),
            },
        },
        wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
}

fn atlas_slot_origin(slot_id: u32) -> (u32, u32) {
    (
        (slot_id % SURFACE_CACHE_DEPTH_ATLAS_COLUMNS) * SURFACE_CACHE_DEPTH_TILE_EXTENT,
        (slot_id / SURFACE_CACHE_DEPTH_ATLAS_COLUMNS) * SURFACE_CACHE_DEPTH_TILE_EXTENT,
    )
}

fn fill_rgba_rect(
    pixels: &mut [u8],
    texture_width: u32,
    origin: (u32, u32),
    extent: (u32, u32),
    rgba: [u8; 4],
) {
    for y in origin.1..origin.1.saturating_add(extent.1) {
        for x in origin.0..origin.0.saturating_add(extent.0) {
            let pixel_index =
                ((y * texture_width + x) * SURFACE_CACHE_DEPTH_BYTES_PER_PIXEL) as usize;
            pixels[pixel_index..pixel_index + SURFACE_CACHE_DEPTH_BYTES_PER_PIXEL as usize]
                .copy_from_slice(&rgba);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::hybrid_gi::types::{
        HybridGiPrepareCardCaptureRequest, HybridGiPrepareSurfaceCacheDepthSourceSample,
        HybridGiPrepareSurfaceCachePageContent,
    };

    use super::*;

    #[test]
    fn surface_cache_depth_samples_prefer_scene_depth_source_samples_over_bounds_fallback() {
        let snapshot = single_atlas_slot_snapshot(0);
        let mut inputs = HybridGiPrepareExecutionInputs::default();
        inputs.scene_surface_cache_depth_source_samples =
            vec![HybridGiPrepareSurfaceCacheDepthSourceSample {
                page_id: 7,
                atlas_slot_id: 0,
                depth_rgba: [17, 19, 23, 255],
            }];
        inputs.scene_surface_cache_page_contents = vec![HybridGiPrepareSurfaceCachePageContent {
            page_id: 7,
            owner_card_id: 7,
            atlas_slot_id: 0,
            capture_slot_id: 0,
            bounds_center: Vec3::new(16.0, 0.0, 0.0),
            bounds_radius: 0.5,
            atlas_sample_rgba: [64, 96, 128, 255],
            capture_sample_rgba: [0, 0, 0, 0],
        }];

        assert_eq!(
            surface_cache_depth_samples(&snapshot, &inputs),
            vec![(0, [17, 19, 23, 255])]
        );
    }

    #[test]
    fn surface_cache_depth_samples_keep_bounds_fallback_when_scene_depth_source_is_absent() {
        let snapshot = single_atlas_slot_snapshot(1);
        let mut inputs = HybridGiPrepareExecutionInputs::default();
        inputs.scene_card_capture_requests = vec![HybridGiPrepareCardCaptureRequest {
            card_id: 7,
            page_id: 7,
            atlas_slot_id: 1,
            capture_slot_id: 0,
            bounds_center: Vec3::new(0.0, 0.0, 3.0),
            bounds_radius: 1.0,
        }];

        assert_eq!(
            surface_cache_depth_samples(&snapshot, &inputs),
            vec![(1, depth_rgba_from_bounds(Vec3::new(0.0, 0.0, 3.0), 1.0))]
        );
    }

    fn single_atlas_slot_snapshot(slot_id: u32) -> HybridGiScenePrepareResourcesSnapshot {
        let slot_count = slot_id + 1;
        HybridGiScenePrepareResourcesSnapshot::new(
            0,
            Vec::new(),
            vec![slot_id],
            Vec::new(),
            slot_count,
            0,
            (
                SURFACE_CACHE_DEPTH_TILE_EXTENT * SURFACE_CACHE_DEPTH_ATLAS_COLUMNS,
                SURFACE_CACHE_DEPTH_TILE_EXTENT
                    * slot_count.div_ceil(SURFACE_CACHE_DEPTH_ATLAS_COLUMNS),
            ),
            (0, 0),
            0,
        )
    }
}
