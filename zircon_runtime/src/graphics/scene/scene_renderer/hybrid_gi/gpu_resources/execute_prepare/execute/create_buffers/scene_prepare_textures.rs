use std::collections::{BTreeMap, BTreeSet};

use wgpu::util::DeviceExt;

use super::super::card_capture_shading::scene_card_capture_rgba;
use super::super::hybrid_gi_prepare_execution_inputs::HybridGiPrepareExecutionInputs;
use super::scene_prepare_descriptors::{
    persisted_surface_cache_page_has_present_atlas_sample,
    persisted_surface_cache_page_has_present_capture_sample,
};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;

const CARD_CAPTURE_TILE_EXTENT: u32 = 64;
const CARD_CAPTURE_ATLAS_COLUMNS: u32 = 8;
const CARD_CAPTURE_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const CARD_CAPTURE_BYTES_PER_PIXEL: u32 = 4;
const CARD_CAPTURE_SAMPLE_READBACK_BYTES_PER_ROW: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

pub(super) struct ScenePrepareTextureLayout {
    pub(super) occupied_atlas_slots: Vec<u32>,
    pub(super) occupied_capture_slots: Vec<u32>,
    pub(super) atlas_slot_count: u32,
    pub(super) capture_slot_count: u32,
    pub(super) atlas_texture_extent: (u32, u32),
    pub(super) capture_texture_extent: (u32, u32),
}

pub(super) struct ScenePrepareTextureResources {
    pub(super) atlas_texture: Option<wgpu::Texture>,
    pub(super) atlas_view: Option<wgpu::TextureView>,
    pub(super) atlas_upload_buffer: Option<wgpu::Buffer>,
    pub(super) atlas_slot_sample_buffers: Vec<(u32, wgpu::Buffer)>,
    pub(super) capture_texture: Option<wgpu::Texture>,
    pub(super) capture_views: Vec<wgpu::TextureView>,
    pub(super) capture_upload_buffer: Option<wgpu::Buffer>,
    pub(super) capture_slot_sample_buffers: Vec<(u32, wgpu::Buffer)>,
}

pub(super) fn scene_prepare_texture_layout(
    inputs: &HybridGiPrepareExecutionInputs,
) -> ScenePrepareTextureLayout {
    let occupied_atlas_slots = occupied_slots(&inputs.scene_card_capture_requests, |request| {
        request.atlas_slot_id
    })
    .into_iter()
    .chain(occupied_surface_cache_page_slots(
        &inputs.scene_surface_cache_page_contents,
        persisted_surface_cache_page_has_present_atlas_sample,
        |page_content| page_content.atlas_slot_id,
    ))
    .collect::<BTreeSet<_>>()
    .into_iter()
    .collect::<Vec<_>>();
    let occupied_capture_slots = occupied_slots(&inputs.scene_card_capture_requests, |request| {
        request.capture_slot_id
    })
    .into_iter()
    .chain(occupied_surface_cache_page_slots(
        &inputs.scene_surface_cache_page_contents,
        persisted_surface_cache_page_has_present_capture_sample,
        |page_content| page_content.capture_slot_id,
    ))
    .collect::<BTreeSet<_>>()
    .into_iter()
    .collect::<Vec<_>>();
    let atlas_slot_count = slot_count(&occupied_atlas_slots);
    let capture_slot_count = slot_count(&occupied_capture_slots);

    ScenePrepareTextureLayout {
        occupied_atlas_slots,
        occupied_capture_slots,
        atlas_slot_count,
        capture_slot_count,
        atlas_texture_extent: atlas_extent(atlas_slot_count),
        capture_texture_extent: capture_extent(capture_slot_count),
    }
}

pub(super) fn store_scene_prepare_texture_samples(
    snapshot: &mut HybridGiScenePrepareResourcesSnapshot,
    streamer: &ResourceStreamer,
    inputs: &HybridGiPrepareExecutionInputs,
) {
    let atlas_slot_rgba_samples = atlas_slot_rgba_samples(snapshot, streamer, inputs);
    let capture_slot_rgba_samples = capture_slot_rgba_samples(snapshot, streamer, inputs);
    snapshot.store_texture_slot_rgba_samples(atlas_slot_rgba_samples, capture_slot_rgba_samples);
}

pub(super) fn scene_prepare_texture_resources(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    snapshot: &HybridGiScenePrepareResourcesSnapshot,
    streamer: &ResourceStreamer,
    inputs: &HybridGiPrepareExecutionInputs,
) -> ScenePrepareTextureResources {
    let (atlas_texture, atlas_view, atlas_upload_buffer, atlas_slot_sample_buffers) =
        if snapshot.atlas_slot_count() > 0 {
            let atlas_rgba = atlas_texture_rgba(snapshot, streamer, inputs);
            let atlas_texture_extent = snapshot.atlas_texture_extent();
            let (atlas_texture, atlas_view) =
                create_card_capture_atlas_texture(device, atlas_texture_extent);
            let atlas_upload_buffer = create_texture_upload_buffer(
                device,
                "zircon-hybrid-gi-scene-prepare-card-capture-atlas-upload",
                &atlas_rgba,
            );
            upload_texture_rgba(
                encoder,
                &atlas_upload_buffer,
                atlas_texture_extent,
                1,
                &atlas_texture,
            );
            let atlas_slot_sample_buffers = snapshot
                .occupied_atlas_slots()
                .iter()
                .map(|slot_id| {
                    let buffer = create_texture_sample_readback_buffer(
                        device,
                        "zircon-hybrid-gi-scene-prepare-card-capture-atlas-sample",
                    );
                    let (origin_x, origin_y) = atlas_slot_origin(*slot_id);
                    enqueue_texture_sample_readback(
                        encoder,
                        &atlas_texture,
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
            (
                Some(atlas_texture),
                Some(atlas_view),
                Some(atlas_upload_buffer),
                atlas_slot_sample_buffers,
            )
        } else {
            (None, None, None, Vec::new())
        };
    let (capture_texture, capture_views, capture_upload_buffer, capture_slot_sample_buffers) =
        if snapshot.capture_slot_count() > 0 {
            let capture_rgba = capture_texture_rgba(snapshot, streamer, inputs);
            let capture_texture_extent = snapshot.capture_texture_extent();
            let capture_layer_count = snapshot.capture_layer_count();
            let (capture_texture, capture_views) =
                create_card_capture_texture(device, capture_texture_extent, capture_layer_count);
            let capture_upload_buffer = create_texture_upload_buffer(
                device,
                "zircon-hybrid-gi-scene-prepare-card-capture-texture-upload",
                &capture_rgba,
            );
            upload_texture_rgba(
                encoder,
                &capture_upload_buffer,
                capture_texture_extent,
                capture_layer_count,
                &capture_texture,
            );
            let capture_slot_sample_buffers = snapshot
                .occupied_capture_slots()
                .iter()
                .map(|slot_id| {
                    let buffer = create_texture_sample_readback_buffer(
                        device,
                        "zircon-hybrid-gi-scene-prepare-card-capture-layer-sample",
                    );
                    enqueue_texture_sample_readback(
                        encoder,
                        &capture_texture,
                        wgpu::Origin3d {
                            x: 0,
                            y: 0,
                            z: *slot_id,
                        },
                        &buffer,
                    );
                    (*slot_id, buffer)
                })
                .collect();
            (
                Some(capture_texture),
                capture_views,
                Some(capture_upload_buffer),
                capture_slot_sample_buffers,
            )
        } else {
            (None, Vec::new(), None, Vec::new())
        };

    ScenePrepareTextureResources {
        atlas_texture,
        atlas_view,
        atlas_upload_buffer,
        atlas_slot_sample_buffers,
        capture_texture,
        capture_views,
        capture_upload_buffer,
        capture_slot_sample_buffers,
    }
}

fn occupied_slots(
    requests: &[crate::graphics::types::HybridGiPrepareCardCaptureRequest],
    projection: impl Fn(&crate::graphics::types::HybridGiPrepareCardCaptureRequest) -> u32,
) -> Vec<u32> {
    let mut slots = requests.iter().map(projection).collect::<Vec<_>>();
    slots.sort_unstable();
    slots.dedup();
    slots
}

fn occupied_surface_cache_page_slots(
    page_contents: &[crate::graphics::types::HybridGiPrepareSurfaceCachePageContent],
    presence: impl Fn(&crate::graphics::types::HybridGiPrepareSurfaceCachePageContent) -> bool,
    projection: impl Fn(&crate::graphics::types::HybridGiPrepareSurfaceCachePageContent) -> u32,
) -> Vec<u32> {
    let mut slots = page_contents
        .iter()
        .filter(|page_content| presence(page_content))
        .map(projection)
        .collect::<Vec<_>>();
    slots.sort_unstable();
    slots.dedup();
    slots
}

fn slot_count(slots: &[u32]) -> u32 {
    slots.last().copied().map(|slot| slot + 1).unwrap_or(0)
}

fn atlas_extent(atlas_slot_count: u32) -> (u32, u32) {
    if atlas_slot_count == 0 {
        return (0, 0);
    }

    let atlas_row_count = atlas_slot_count.div_ceil(CARD_CAPTURE_ATLAS_COLUMNS);
    (
        CARD_CAPTURE_TILE_EXTENT * CARD_CAPTURE_ATLAS_COLUMNS,
        CARD_CAPTURE_TILE_EXTENT * atlas_row_count,
    )
}

fn capture_extent(capture_slot_count: u32) -> (u32, u32) {
    if capture_slot_count == 0 {
        return (0, 0);
    }

    (CARD_CAPTURE_TILE_EXTENT, CARD_CAPTURE_TILE_EXTENT)
}

fn create_card_capture_atlas_texture(
    device: &wgpu::Device,
    extent: (u32, u32),
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-hybrid-gi-scene-prepare-card-capture-atlas"),
        size: wgpu::Extent3d {
            width: extent.0,
            height: extent.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: CARD_CAPTURE_TEXTURE_FORMAT,
        usage: wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("zircon-hybrid-gi-scene-prepare-card-capture-atlas-view"),
        ..Default::default()
    });
    (texture, view)
}

fn create_card_capture_texture(
    device: &wgpu::Device,
    extent: (u32, u32),
    layer_count: u32,
) -> (wgpu::Texture, Vec<wgpu::TextureView>) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-hybrid-gi-scene-prepare-card-capture-texture"),
        size: wgpu::Extent3d {
            width: extent.0,
            height: extent.1,
            depth_or_array_layers: layer_count,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: CARD_CAPTURE_TEXTURE_FORMAT,
        usage: wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let capture_views = (0..layer_count)
        .map(|layer_index| {
            texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("zircon-hybrid-gi-scene-prepare-card-capture-layer-view"),
                format: Some(CARD_CAPTURE_TEXTURE_FORMAT),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: Some(1),
                base_array_layer: layer_index,
                array_layer_count: Some(1),
                usage: Some(
                    wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
                ),
            })
        })
        .collect();
    (texture, capture_views)
}

fn atlas_slot_origin(slot_id: u32) -> (u32, u32) {
    (
        (slot_id % CARD_CAPTURE_ATLAS_COLUMNS) * CARD_CAPTURE_TILE_EXTENT,
        (slot_id / CARD_CAPTURE_ATLAS_COLUMNS) * CARD_CAPTURE_TILE_EXTENT,
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
            let pixel_index = ((y * texture_width + x) * CARD_CAPTURE_BYTES_PER_PIXEL) as usize;
            pixels[pixel_index..pixel_index + CARD_CAPTURE_BYTES_PER_PIXEL as usize]
                .copy_from_slice(&rgba);
        }
    }
}

fn fill_rgba_layer(pixels: &mut [u8], texture_extent: (u32, u32), layer_index: u32, rgba: [u8; 4]) {
    let layer_stride =
        (texture_extent.0 * texture_extent.1 * CARD_CAPTURE_BYTES_PER_PIXEL) as usize;
    let layer_offset = layer_index as usize * layer_stride;
    for pixel in pixels[layer_offset..layer_offset + layer_stride].chunks_exact_mut(4) {
        pixel.copy_from_slice(&rgba);
    }
}

fn atlas_texture_rgba(
    snapshot: &HybridGiScenePrepareResourcesSnapshot,
    streamer: &ResourceStreamer,
    inputs: &HybridGiPrepareExecutionInputs,
) -> Vec<u8> {
    let atlas_texture_extent = snapshot.atlas_texture_extent();
    let mut pixels = vec![
        0_u8;
        (atlas_texture_extent.0 * atlas_texture_extent.1 * CARD_CAPTURE_BYTES_PER_PIXEL)
            as usize
    ];
    for page_content in inputs
        .scene_surface_cache_page_contents
        .iter()
        .filter(|page_content| persisted_surface_cache_page_has_present_atlas_sample(page_content))
    {
        fill_rgba_rect(
            &mut pixels,
            atlas_texture_extent.0,
            atlas_slot_origin(page_content.atlas_slot_id),
            (CARD_CAPTURE_TILE_EXTENT, CARD_CAPTURE_TILE_EXTENT),
            page_content.atlas_sample_rgba,
        );
    }
    for request in &inputs.scene_card_capture_requests {
        fill_rgba_rect(
            &mut pixels,
            atlas_texture_extent.0,
            atlas_slot_origin(request.atlas_slot_id),
            (CARD_CAPTURE_TILE_EXTENT, CARD_CAPTURE_TILE_EXTENT),
            scene_card_capture_rgba(request, streamer, inputs),
        );
    }
    pixels
}

fn capture_texture_rgba(
    snapshot: &HybridGiScenePrepareResourcesSnapshot,
    streamer: &ResourceStreamer,
    inputs: &HybridGiPrepareExecutionInputs,
) -> Vec<u8> {
    let capture_texture_extent = snapshot.capture_texture_extent();
    let mut pixels = vec![
        0_u8;
        (capture_texture_extent.0
            * capture_texture_extent.1
            * snapshot.capture_layer_count()
            * CARD_CAPTURE_BYTES_PER_PIXEL) as usize
    ];
    for page_content in inputs
        .scene_surface_cache_page_contents
        .iter()
        .filter(|page_content| {
            persisted_surface_cache_page_has_present_capture_sample(page_content)
        })
    {
        fill_rgba_layer(
            &mut pixels,
            capture_texture_extent,
            page_content.capture_slot_id,
            page_content.capture_sample_rgba,
        );
    }
    for request in &inputs.scene_card_capture_requests {
        fill_rgba_layer(
            &mut pixels,
            capture_texture_extent,
            request.capture_slot_id,
            scene_card_capture_rgba(request, streamer, inputs),
        );
    }
    pixels
}

fn atlas_slot_rgba_samples(
    snapshot: &HybridGiScenePrepareResourcesSnapshot,
    streamer: &ResourceStreamer,
    inputs: &HybridGiPrepareExecutionInputs,
) -> Vec<(u32, [u8; 4])> {
    let mut rgba_by_slot = inputs
        .scene_surface_cache_page_contents
        .iter()
        .filter(|page_content| persisted_surface_cache_page_has_present_atlas_sample(page_content))
        .map(|page_content| (page_content.atlas_slot_id, page_content.atlas_sample_rgba))
        .collect::<BTreeMap<_, _>>();
    rgba_by_slot.extend(inputs.scene_card_capture_requests.iter().map(|request| {
        (
            request.atlas_slot_id,
            scene_card_capture_rgba(request, streamer, inputs),
        )
    }));

    snapshot
        .occupied_atlas_slots()
        .iter()
        .filter_map(|slot_id| {
            rgba_by_slot
                .get(slot_id)
                .copied()
                .map(|rgba| (*slot_id, rgba))
        })
        .collect()
}

fn capture_slot_rgba_samples(
    snapshot: &HybridGiScenePrepareResourcesSnapshot,
    streamer: &ResourceStreamer,
    inputs: &HybridGiPrepareExecutionInputs,
) -> Vec<(u32, [u8; 4])> {
    let mut rgba_by_slot = inputs
        .scene_surface_cache_page_contents
        .iter()
        .filter(|page_content| {
            persisted_surface_cache_page_has_present_capture_sample(page_content)
        })
        .map(|page_content| {
            (
                page_content.capture_slot_id,
                page_content.capture_sample_rgba,
            )
        })
        .collect::<BTreeMap<_, _>>();
    rgba_by_slot.extend(inputs.scene_card_capture_requests.iter().map(|request| {
        (
            request.capture_slot_id,
            scene_card_capture_rgba(request, streamer, inputs),
        )
    }));

    snapshot
        .occupied_capture_slots()
        .iter()
        .filter_map(|slot_id| {
            rgba_by_slot
                .get(slot_id)
                .copied()
                .map(|rgba| (*slot_id, rgba))
        })
        .collect()
}

fn create_texture_upload_buffer(
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

fn create_texture_sample_readback_buffer(
    device: &wgpu::Device,
    label: &'static str,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: CARD_CAPTURE_SAMPLE_READBACK_BYTES_PER_ROW as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}

fn upload_texture_rgba(
    encoder: &mut wgpu::CommandEncoder,
    upload_buffer: &wgpu::Buffer,
    extent: (u32, u32),
    layer_count: u32,
    texture: &wgpu::Texture,
) {
    encoder.copy_buffer_to_texture(
        wgpu::TexelCopyBufferInfo {
            buffer: upload_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(extent.0 * CARD_CAPTURE_BYTES_PER_PIXEL),
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
            depth_or_array_layers: layer_count,
        },
    );
}

fn enqueue_texture_sample_readback(
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
                bytes_per_row: Some(CARD_CAPTURE_SAMPLE_READBACK_BYTES_PER_ROW),
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
