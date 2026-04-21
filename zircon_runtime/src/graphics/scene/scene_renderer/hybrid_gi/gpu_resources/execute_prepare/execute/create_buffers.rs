use std::collections::BTreeMap;

use super::super::super::buffer_helpers::{
    buffer_size_for_words, create_pod_storage_buffer, create_readback_buffer,
    create_u32_storage_buffer,
};
use super::super::super::seed_quantization::{quantized_positive, quantized_signed};
use super::card_capture_shading::{scene_card_capture_rgba, scene_voxel_clipmap_rgba};
use super::hybrid_gi_prepare_execution_buffers::{
    HybridGiPrepareExecutionBuffers, HybridGiPrepareScenePrepareResources,
};
use super::hybrid_gi_prepare_execution_inputs::HybridGiPrepareExecutionInputs;
use super::voxel_clipmap_debug::{
    scene_voxel_clipmap_cell_dominant_node_ids, scene_voxel_clipmap_cell_dominant_rgba_samples,
    scene_voxel_clipmap_cell_rgba_samples,
};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;
use crate::graphics::types::{
    hybrid_gi_voxel_clipmap_cell_center, HybridGiPrepareVoxelCell, HybridGiPrepareVoxelClipmap,
    HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT, HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION,
};
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

const SCENE_CARD_CAPTURE_RADIUS_SCALE: f32 = 64.0;
const SCENE_VOXEL_CLIPMAP_HALF_EXTENT_SCALE: f32 = 64.0;
const SCENE_VOXEL_CELL_HALF_EXTENT_SCALE: f32 = 64.0;
const SCENE_PREPARE_DESCRIPTOR_KIND_CARD_CAPTURE: u32 = 1;
const SCENE_PREPARE_DESCRIPTOR_KIND_VOXEL_CLIPMAP: u32 = 2;
const SCENE_PREPARE_DESCRIPTOR_KIND_VOXEL_CELL: u32 = 3;
const CARD_CAPTURE_TILE_EXTENT: u32 = 64;
const CARD_CAPTURE_ATLAS_COLUMNS: u32 = 8;
const CARD_CAPTURE_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const CARD_CAPTURE_BYTES_PER_PIXEL: u32 = 4;
const CARD_CAPTURE_SAMPLE_READBACK_BYTES_PER_ROW: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
struct GpuSceneCardCaptureRequest {
    card_id: u32,
    page_id: u32,
    atlas_slot_id: u32,
    capture_slot_id: u32,
    bounds_center_x_q: u32,
    bounds_center_y_q: u32,
    bounds_center_z_q: u32,
    bounds_radius_q: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
struct GpuSceneVoxelClipmap {
    clipmap_id: u32,
    center_x_q: u32,
    center_y_q: u32,
    center_z_q: u32,
    half_extent_q: u32,
    _padding0: u32,
    _padding1: u32,
    _padding2: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
struct GpuScenePrepareDescriptor {
    descriptor_kind: u32,
    primary_id: u32,
    secondary_id: u32,
    tertiary_id: u32,
    quaternary_id: u32,
    scalar0: u32,
    scalar1: u32,
    scalar2: u32,
    scalar3: u32,
    _padding0: u32,
    _padding1: u32,
    _padding2: u32,
}

fn gpu_scene_card_capture_requests(
    requests: &[crate::graphics::types::HybridGiPrepareCardCaptureRequest],
) -> Vec<GpuSceneCardCaptureRequest> {
    requests
        .iter()
        .map(|request| GpuSceneCardCaptureRequest {
            card_id: request.card_id,
            page_id: request.page_id,
            atlas_slot_id: request.atlas_slot_id,
            capture_slot_id: request.capture_slot_id,
            bounds_center_x_q: quantized_signed(request.bounds_center.x),
            bounds_center_y_q: quantized_signed(request.bounds_center.y),
            bounds_center_z_q: quantized_signed(request.bounds_center.z),
            bounds_radius_q: quantized_positive(
                request.bounds_radius,
                SCENE_CARD_CAPTURE_RADIUS_SCALE,
            ),
        })
        .collect()
}

fn gpu_scene_voxel_clipmaps(
    clipmaps: &[crate::graphics::types::HybridGiPrepareVoxelClipmap],
) -> Vec<GpuSceneVoxelClipmap> {
    clipmaps
        .iter()
        .map(|clipmap| GpuSceneVoxelClipmap {
            clipmap_id: clipmap.clipmap_id,
            center_x_q: quantized_signed(clipmap.center.x),
            center_y_q: quantized_signed(clipmap.center.y),
            center_z_q: quantized_signed(clipmap.center.z),
            half_extent_q: quantized_positive(
                clipmap.half_extent,
                SCENE_VOXEL_CLIPMAP_HALF_EXTENT_SCALE,
            ),
            _padding0: 0,
            _padding1: 0,
            _padding2: 0,
        })
        .collect()
}

fn pack_rgb8(rgb: [u8; 3]) -> u32 {
    rgb[0] as u32 | ((rgb[1] as u32) << 8) | ((rgb[2] as u32) << 16)
}

fn gpu_scene_card_capture_seed_rgb(
    requests: &[crate::graphics::types::HybridGiPrepareCardCaptureRequest],
    streamer: &ResourceStreamer,
    inputs: &HybridGiPrepareExecutionInputs,
) -> Vec<Option<u32>> {
    requests
        .iter()
        .map(|request| {
            let rgba = scene_card_capture_rgba(request, streamer, inputs);
            Some(pack_rgb8([rgba[0], rgba[1], rgba[2]]))
        })
        .collect()
}

fn gpu_scene_prepare_descriptors(
    card_capture_requests: &[crate::graphics::types::HybridGiPrepareCardCaptureRequest],
    card_capture_seed_rgb: &[Option<u32>],
    voxel_clipmaps: &[crate::graphics::types::HybridGiPrepareVoxelClipmap],
    voxel_cells: &[crate::graphics::types::HybridGiPrepareVoxelCell],
) -> Vec<GpuScenePrepareDescriptor> {
    let staged_card_capture_requests = gpu_scene_card_capture_requests(card_capture_requests);
    let staged_voxel_clipmaps = gpu_scene_voxel_clipmaps(voxel_clipmaps);
    let clipmaps_by_id = voxel_clipmaps
        .iter()
        .map(|clipmap| (clipmap.clipmap_id, clipmap))
        .collect::<BTreeMap<_, _>>();
    let mut descriptors = Vec::with_capacity(
        staged_card_capture_requests.len() + staged_voxel_clipmaps.len() + voxel_cells.len(),
    );

    descriptors.extend(staged_card_capture_requests.into_iter().enumerate().map(
        |(index, request)| {
            let packed_seed_rgb = card_capture_seed_rgb.get(index).copied().flatten();
            GpuScenePrepareDescriptor {
                descriptor_kind: SCENE_PREPARE_DESCRIPTOR_KIND_CARD_CAPTURE,
                primary_id: request.card_id,
                secondary_id: request.page_id,
                tertiary_id: request.atlas_slot_id,
                quaternary_id: request.capture_slot_id,
                scalar0: request.bounds_center_x_q,
                scalar1: request.bounds_center_y_q,
                scalar2: request.bounds_center_z_q,
                scalar3: request.bounds_radius_q,
                _padding0: packed_seed_rgb.unwrap_or(0),
                _padding1: u32::from(packed_seed_rgb.is_some()),
                _padding2: 0,
            }
        },
    ));
    descriptors.extend(staged_voxel_clipmaps.into_iter().map(|clipmap| {
        GpuScenePrepareDescriptor {
            descriptor_kind: SCENE_PREPARE_DESCRIPTOR_KIND_VOXEL_CLIPMAP,
            primary_id: clipmap.clipmap_id,
            secondary_id: 0,
            tertiary_id: 0,
            quaternary_id: 0,
            scalar0: clipmap.center_x_q,
            scalar1: clipmap.center_y_q,
            scalar2: clipmap.center_z_q,
            scalar3: clipmap.half_extent_q,
            _padding0: 0,
            _padding1: 0,
            _padding2: 0,
        }
    }));
    descriptors.extend(voxel_cells.iter().filter_map(|cell| {
        if cell.occupancy_count == 0 {
            return None;
        }

        let clipmap = clipmaps_by_id.get(&cell.clipmap_id).copied()?;
        let cell_index = cell.cell_index as usize;
        if cell_index >= HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT {
            return None;
        }

        let cell_x = cell_index % HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION;
        let cell_y = (cell_index / HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION)
            % HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION;
        let cell_z = cell_index
            / (HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION * HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION);
        let cell_center = hybrid_gi_voxel_clipmap_cell_center(clipmap, cell_x, cell_y, cell_z);
        let cell_half_extent = clipmap.half_extent / HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION as f32;

        Some(GpuScenePrepareDescriptor {
            descriptor_kind: SCENE_PREPARE_DESCRIPTOR_KIND_VOXEL_CELL,
            primary_id: cell.clipmap_id,
            secondary_id: cell.cell_index,
            tertiary_id: cell.occupancy_count,
            quaternary_id: pack_rgb8(cell.radiance_rgb),
            scalar0: quantized_signed(cell_center.x),
            scalar1: quantized_signed(cell_center.y),
            scalar2: quantized_signed(cell_center.z),
            scalar3: quantized_positive(cell_half_extent, SCENE_VOXEL_CELL_HALF_EXTENT_SCALE),
            _padding0: cell.dominant_card_id,
            _padding1: u32::from(cell.radiance_present),
            _padding2: 0,
        })
    }));

    descriptors
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
    let mut pixels = vec![
        0_u8;
        (snapshot.atlas_texture_extent.0
            * snapshot.atlas_texture_extent.1
            * CARD_CAPTURE_BYTES_PER_PIXEL) as usize
    ];
    for request in &inputs.scene_card_capture_requests {
        fill_rgba_rect(
            &mut pixels,
            snapshot.atlas_texture_extent.0,
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
    let mut pixels = vec![
        0_u8;
        (snapshot.capture_texture_extent.0
            * snapshot.capture_texture_extent.1
            * snapshot.capture_layer_count
            * CARD_CAPTURE_BYTES_PER_PIXEL) as usize
    ];
    for request in &inputs.scene_card_capture_requests {
        fill_rgba_layer(
            &mut pixels,
            snapshot.capture_texture_extent,
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
    let rgba_by_slot = inputs
        .scene_card_capture_requests
        .iter()
        .map(|request| {
            (
                request.atlas_slot_id,
                scene_card_capture_rgba(request, streamer, inputs),
            )
        })
        .collect::<BTreeMap<_, _>>();

    snapshot
        .occupied_atlas_slots
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
    let rgba_by_slot = inputs
        .scene_card_capture_requests
        .iter()
        .map(|request| {
            (
                request.capture_slot_id,
                scene_card_capture_rgba(request, streamer, inputs),
            )
        })
        .collect::<BTreeMap<_, _>>();

    snapshot
        .occupied_capture_slots
        .iter()
        .filter_map(|slot_id| {
            rgba_by_slot
                .get(slot_id)
                .copied()
                .map(|rgba| (*slot_id, rgba))
        })
        .collect()
}

fn voxel_cell_occupancy_counts_from_scene_prepare(
    clipmaps: &[HybridGiPrepareVoxelClipmap],
    voxel_cells: &[HybridGiPrepareVoxelCell],
) -> Vec<(u32, u32, u32)> {
    let occupancy_by_cell = voxel_cells
        .iter()
        .map(|cell| ((cell.clipmap_id, cell.cell_index), cell.occupancy_count))
        .collect::<BTreeMap<_, _>>();
    let occupancy_by_cell = &occupancy_by_cell;

    clipmaps
        .iter()
        .flat_map(|clipmap| {
            (0..HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT).map(|cell_index| {
                (
                    clipmap.clipmap_id,
                    cell_index as u32,
                    occupancy_by_cell
                        .get(&(clipmap.clipmap_id, cell_index as u32))
                        .copied()
                        .unwrap_or(0),
                )
            })
        })
        .collect()
}

fn runtime_voxel_cell_radiance_rgba_overrides(
    voxel_cells: &[HybridGiPrepareVoxelCell],
) -> BTreeMap<(u32, u32), [u8; 4]> {
    voxel_cells
        .iter()
        .filter(|cell| cell.occupancy_count > 0)
        .filter_map(|cell| {
            cell.radiance_present.then_some((
                (cell.clipmap_id, cell.cell_index),
                [
                    cell.radiance_rgb[0],
                    cell.radiance_rgb[1],
                    cell.radiance_rgb[2],
                    255,
                ],
            ))
        })
        .collect()
}

fn runtime_voxel_clipmap_radiance_rgba_overrides(
    voxel_cells: &[HybridGiPrepareVoxelCell],
) -> BTreeMap<u32, [u8; 4]> {
    let mut weighted_rgb_sums = BTreeMap::<u32, [u64; 3]>::new();
    let mut total_weights = BTreeMap::<u32, u64>::new();

    for cell in voxel_cells.iter().filter(|cell| cell.occupancy_count > 0) {
        if !cell.radiance_present {
            continue;
        }

        let weight = cell.occupancy_count.max(1) as u64;
        let rgb_sum = weighted_rgb_sums
            .entry(cell.clipmap_id)
            .or_insert([0, 0, 0]);
        rgb_sum[0] += cell.radiance_rgb[0] as u64 * weight;
        rgb_sum[1] += cell.radiance_rgb[1] as u64 * weight;
        rgb_sum[2] += cell.radiance_rgb[2] as u64 * weight;
        *total_weights.entry(cell.clipmap_id).or_insert(0) += weight;
    }

    weighted_rgb_sums
        .into_iter()
        .filter_map(|(clipmap_id, rgb_sum)| {
            let total_weight = total_weights.get(&clipmap_id).copied().unwrap_or(0);
            (total_weight > 0).then_some((
                clipmap_id,
                [
                    ((rgb_sum[0] + total_weight / 2) / total_weight) as u8,
                    ((rgb_sum[1] + total_weight / 2) / total_weight) as u8,
                    ((rgb_sum[2] + total_weight / 2) / total_weight) as u8,
                    255,
                ],
            ))
        })
        .collect()
}

fn runtime_voxel_cell_dominant_node_overrides(
    voxel_cells: &[HybridGiPrepareVoxelCell],
) -> BTreeMap<(u32, u32), u64> {
    voxel_cells
        .iter()
        .filter(|cell| cell.occupancy_count > 0)
        .filter_map(|cell| {
            (cell.dominant_card_id != 0).then_some((
                (cell.clipmap_id, cell.cell_index),
                cell.dominant_card_id as u64,
            ))
        })
        .collect()
}

fn voxel_occupancy_masks_from_counts(
    clipmaps: &[HybridGiPrepareVoxelClipmap],
    counts: &[(u32, u32, u32)],
) -> Vec<(u32, u64)> {
    let mut masks_by_clipmap = clipmaps
        .iter()
        .map(|clipmap| (clipmap.clipmap_id, 0_u64))
        .collect::<BTreeMap<_, _>>();
    for &(clipmap_id, cell_index, occupancy_count) in counts {
        if occupancy_count == 0 || cell_index >= u64::BITS {
            continue;
        }
        *masks_by_clipmap.entry(clipmap_id).or_default() |= 1_u64 << cell_index;
    }
    masks_by_clipmap.into_iter().collect()
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

fn scene_prepare_resources(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    streamer: &ResourceStreamer,
    inputs: &HybridGiPrepareExecutionInputs,
) -> Option<HybridGiPrepareScenePrepareResources> {
    if inputs.scene_card_capture_requests.is_empty() && inputs.scene_voxel_clipmaps.is_empty() {
        return None;
    }

    let occupied_atlas_slots = occupied_slots(&inputs.scene_card_capture_requests, |request| {
        request.atlas_slot_id
    });
    let occupied_capture_slots = occupied_slots(&inputs.scene_card_capture_requests, |request| {
        request.capture_slot_id
    });
    let atlas_slot_count = slot_count(&occupied_atlas_slots);
    let capture_slot_count = slot_count(&occupied_capture_slots);
    let mut snapshot = HybridGiScenePrepareResourcesSnapshot {
        card_capture_request_count: inputs.scene_card_capture_requests.len() as u32,
        voxel_clipmap_ids: inputs
            .scene_voxel_clipmaps
            .iter()
            .map(|clipmap| clipmap.clipmap_id)
            .collect(),
        occupied_atlas_slots,
        occupied_capture_slots,
        atlas_slot_rgba_samples: Vec::new(),
        capture_slot_rgba_samples: Vec::new(),
        voxel_clipmap_rgba_samples: Vec::new(),
        voxel_clipmap_occupancy_masks: Vec::new(),
        voxel_clipmap_cell_rgba_samples: Vec::new(),
        voxel_clipmap_cell_occupancy_counts: Vec::new(),
        voxel_clipmap_cell_dominant_node_ids: Vec::new(),
        voxel_clipmap_cell_dominant_rgba_samples: Vec::new(),
        atlas_slot_count,
        capture_slot_count,
        atlas_texture_extent: atlas_extent(atlas_slot_count),
        capture_texture_extent: capture_extent(capture_slot_count),
        capture_layer_count: capture_slot_count,
    };
    let voxel_cell_occupancy_counts = voxel_cell_occupancy_counts_from_scene_prepare(
        &inputs.scene_voxel_clipmaps,
        &inputs.scene_voxel_cells,
    );
    let runtime_voxel_clipmap_radiance_overrides =
        runtime_voxel_clipmap_radiance_rgba_overrides(&inputs.scene_voxel_cells);
    let runtime_voxel_cell_radiance_overrides =
        runtime_voxel_cell_radiance_rgba_overrides(&inputs.scene_voxel_cells);
    let runtime_voxel_cell_dominant_node_overrides =
        runtime_voxel_cell_dominant_node_overrides(&inputs.scene_voxel_cells);
    snapshot.voxel_clipmap_rgba_samples = inputs
        .scene_voxel_clipmaps
        .iter()
        .map(|clipmap| {
            (
                clipmap.clipmap_id,
                runtime_voxel_clipmap_radiance_overrides
                    .get(&clipmap.clipmap_id)
                    .copied()
                    .unwrap_or_else(|| scene_voxel_clipmap_rgba(clipmap, streamer, inputs)),
            )
        })
        .collect();
    snapshot.voxel_clipmap_occupancy_masks = voxel_occupancy_masks_from_counts(
        &inputs.scene_voxel_clipmaps,
        &voxel_cell_occupancy_counts,
    );
    snapshot.voxel_clipmap_cell_rgba_samples = inputs
        .scene_voxel_clipmaps
        .iter()
        .flat_map(|clipmap| {
            scene_voxel_clipmap_cell_rgba_samples(clipmap, streamer, inputs)
                .into_iter()
                .map(|(cell_index, rgba)| {
                    (
                        clipmap.clipmap_id,
                        cell_index,
                        runtime_voxel_cell_radiance_overrides
                            .get(&(clipmap.clipmap_id, cell_index))
                            .copied()
                            .unwrap_or(rgba),
                    )
                })
        })
        .collect();
    snapshot.voxel_clipmap_cell_occupancy_counts = voxel_cell_occupancy_counts;
    snapshot.voxel_clipmap_cell_dominant_node_ids = inputs
        .scene_voxel_clipmaps
        .iter()
        .flat_map(|clipmap| {
            scene_voxel_clipmap_cell_dominant_node_ids(clipmap, streamer, inputs)
                .into_iter()
                .map(|(cell_index, node_id)| {
                    (
                        clipmap.clipmap_id,
                        cell_index,
                        runtime_voxel_cell_dominant_node_overrides
                            .get(&(clipmap.clipmap_id, cell_index))
                            .copied()
                            .unwrap_or(node_id),
                    )
                })
        })
        .collect();
    snapshot.voxel_clipmap_cell_dominant_rgba_samples = inputs
        .scene_voxel_clipmaps
        .iter()
        .flat_map(|clipmap| {
            scene_voxel_clipmap_cell_dominant_rgba_samples(clipmap, streamer, inputs)
                .into_iter()
                .map(|(cell_index, rgba)| {
                    (
                        clipmap.clipmap_id,
                        cell_index,
                        runtime_voxel_cell_radiance_overrides
                            .get(&(clipmap.clipmap_id, cell_index))
                            .copied()
                            .unwrap_or(rgba),
                    )
                })
        })
        .collect();
    snapshot.atlas_slot_rgba_samples = atlas_slot_rgba_samples(&snapshot, streamer, inputs);
    snapshot.capture_slot_rgba_samples = capture_slot_rgba_samples(&snapshot, streamer, inputs);
    let (
        atlas_texture,
        atlas_view,
        atlas_upload_buffer,
        atlas_slot_sample_buffers,
        capture_texture,
        capture_views,
        capture_upload_buffer,
        capture_slot_sample_buffers,
    ) = if snapshot.card_capture_request_count > 0 {
        let atlas_rgba = atlas_texture_rgba(&snapshot, streamer, inputs);
        let capture_rgba = capture_texture_rgba(&snapshot, streamer, inputs);
        let (atlas_texture, atlas_view) =
            create_card_capture_atlas_texture(device, snapshot.atlas_texture_extent);
        let atlas_upload_buffer = create_texture_upload_buffer(
            device,
            "zircon-hybrid-gi-scene-prepare-card-capture-atlas-upload",
            &atlas_rgba,
        );
        upload_texture_rgba(
            encoder,
            &atlas_upload_buffer,
            snapshot.atlas_texture_extent,
            1,
            &atlas_texture,
        );
        let atlas_slot_sample_buffers = snapshot
            .occupied_atlas_slots
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
        let (capture_texture, capture_views) = create_card_capture_texture(
            device,
            snapshot.capture_texture_extent,
            snapshot.capture_layer_count,
        );
        let capture_upload_buffer = create_texture_upload_buffer(
            device,
            "zircon-hybrid-gi-scene-prepare-card-capture-texture-upload",
            &capture_rgba,
        );
        upload_texture_rgba(
            encoder,
            &capture_upload_buffer,
            snapshot.capture_texture_extent,
            snapshot.capture_layer_count,
            &capture_texture,
        );
        let capture_slot_sample_buffers = snapshot
            .occupied_capture_slots
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
            Some(atlas_texture),
            Some(atlas_view),
            Some(atlas_upload_buffer),
            atlas_slot_sample_buffers,
            Some(capture_texture),
            capture_views,
            Some(capture_upload_buffer),
            capture_slot_sample_buffers,
        )
    } else {
        (
            None,
            None,
            None,
            Vec::new(),
            None,
            Vec::new(),
            None,
            Vec::new(),
        )
    };

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

pub(super) fn create_buffers(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    streamer: &ResourceStreamer,
    inputs: &HybridGiPrepareExecutionInputs,
) -> HybridGiPrepareExecutionBuffers {
    let scene_card_capture_seed_rgb =
        gpu_scene_card_capture_seed_rgb(&inputs.scene_card_capture_requests, streamer, inputs);
    let scene_prepare_descriptors = gpu_scene_prepare_descriptors(
        &inputs.scene_card_capture_requests,
        &scene_card_capture_seed_rgb,
        &inputs.scene_voxel_clipmaps,
        &inputs.scene_voxel_cells,
    );
    let scene_prepare_resources = scene_prepare_resources(device, encoder, streamer, inputs);
    let cache_buffer = create_u32_storage_buffer(
        device,
        "zircon-hybrid-gi-cache-buffer",
        bytemuck::cast_slice(&inputs.cache_entries),
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let cache_readback = create_readback_buffer(
        device,
        "zircon-hybrid-gi-cache-readback",
        inputs.cache_word_count,
    );
    encoder.copy_buffer_to_buffer(
        &cache_buffer,
        0,
        &cache_readback,
        0,
        buffer_size_for_words(inputs.cache_word_count),
    );

    let resident_probe_buffer = create_pod_storage_buffer(
        device,
        "zircon-hybrid-gi-resident-probes",
        &inputs.resident_probe_inputs,
        wgpu::BufferUsages::STORAGE,
    );
    let pending_probe_buffer = create_pod_storage_buffer(
        device,
        "zircon-hybrid-gi-pending-probes",
        &inputs.pending_probe_inputs,
        wgpu::BufferUsages::STORAGE,
    );
    let trace_region_buffer = create_pod_storage_buffer(
        device,
        "zircon-hybrid-gi-trace-regions",
        &inputs.trace_region_inputs,
        wgpu::BufferUsages::STORAGE,
    );
    let scene_prepare_descriptor_buffer = create_pod_storage_buffer(
        device,
        "zircon-hybrid-gi-scene-prepare-descriptors",
        &scene_prepare_descriptors,
        wgpu::BufferUsages::STORAGE,
    );
    let completed_probe_buffer = create_u32_storage_buffer(
        device,
        "zircon-hybrid-gi-completed-probes",
        &vec![0_u32; inputs.completed_probe_word_count.max(1)],
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let completed_trace_buffer = create_u32_storage_buffer(
        device,
        "zircon-hybrid-gi-completed-traces",
        &vec![0_u32; inputs.completed_trace_word_count.max(1)],
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let completed_probe_readback = create_readback_buffer(
        device,
        "zircon-hybrid-gi-completed-probe-readback",
        inputs.completed_probe_word_count.max(1),
    );
    let completed_trace_readback = create_readback_buffer(
        device,
        "zircon-hybrid-gi-completed-trace-readback",
        inputs.completed_trace_word_count.max(1),
    );
    let irradiance_buffer = create_u32_storage_buffer(
        device,
        "zircon-hybrid-gi-irradiance-buffer",
        &vec![0_u32; inputs.irradiance_word_count.max(1)],
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let irradiance_readback = create_readback_buffer(
        device,
        "zircon-hybrid-gi-irradiance-readback",
        inputs.irradiance_word_count.max(1),
    );
    let trace_lighting_buffer = create_u32_storage_buffer(
        device,
        "zircon-hybrid-gi-trace-lighting-buffer",
        &vec![0_u32; inputs.trace_lighting_word_count.max(1)],
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let trace_lighting_readback = create_readback_buffer(
        device,
        "zircon-hybrid-gi-trace-lighting-readback",
        inputs.trace_lighting_word_count.max(1),
    );

    HybridGiPrepareExecutionBuffers {
        cache_readback,
        resident_probe_buffer,
        pending_probe_buffer,
        trace_region_buffer,
        scene_prepare_descriptor_buffer,
        completed_probe_buffer,
        completed_trace_buffer,
        completed_probe_readback,
        completed_trace_readback,
        irradiance_buffer,
        irradiance_readback,
        trace_lighting_buffer,
        trace_lighting_readback,
        scene_prepare_resources,
    }
}

#[cfg(test)]
mod tests {
    use crate::core::math::Vec3;
    use crate::graphics::types::{
        HybridGiPrepareCardCaptureRequest, HybridGiPrepareVoxelCell, HybridGiPrepareVoxelClipmap,
    };

    use super::*;

    #[test]
    fn gpu_scene_card_capture_requests_quantize_scene_prepare_requests() {
        let requests = vec![HybridGiPrepareCardCaptureRequest {
            card_id: 7,
            page_id: 8,
            atlas_slot_id: 9,
            capture_slot_id: 10,
            bounds_center: Vec3::new(1.25, -2.5, 3.75),
            bounds_radius: 1.5,
        }];

        let staged = gpu_scene_card_capture_requests(&requests);

        assert_eq!(staged.len(), 1);
        assert_eq!(staged[0].card_id, 7);
        assert_eq!(staged[0].page_id, 8);
        assert_eq!(staged[0].atlas_slot_id, 9);
        assert_eq!(staged[0].capture_slot_id, 10);
        assert_eq!(staged[0].bounds_center_x_q, 2128);
        assert_eq!(staged[0].bounds_center_y_q, 1888);
        assert_eq!(staged[0].bounds_center_z_q, 2288);
        assert_eq!(staged[0].bounds_radius_q, 96);
    }

    #[test]
    fn gpu_scene_voxel_clipmaps_quantize_scene_prepare_clipmaps() {
        let clipmaps = vec![HybridGiPrepareVoxelClipmap {
            clipmap_id: 5,
            center: Vec3::new(-4.0, 0.5, 2.0),
            half_extent: 12.25,
        }];

        let staged = gpu_scene_voxel_clipmaps(&clipmaps);

        assert_eq!(staged.len(), 1);
        assert_eq!(staged[0].clipmap_id, 5);
        assert_eq!(staged[0].center_x_q, 1792);
        assert_eq!(staged[0].center_y_q, 2080);
        assert_eq!(staged[0].center_z_q, 2176);
        assert_eq!(staged[0].half_extent_q, 784);
    }

    #[test]
    fn gpu_scene_prepare_descriptors_include_runtime_voxel_cells() {
        let clipmaps = vec![HybridGiPrepareVoxelClipmap {
            clipmap_id: 7,
            center: Vec3::ZERO,
            half_extent: 4.0,
        }];
        let descriptors = gpu_scene_prepare_descriptors(
            &[],
            &[],
            &clipmaps,
            &[HybridGiPrepareVoxelCell {
                clipmap_id: 7,
                cell_index: 42,
                occupancy_count: 4,
                dominant_card_id: 99,
                radiance_present: true,
                radiance_rgb: [32, 64, 96],
            }],
        );

        assert_eq!(descriptors.len(), 2);
        assert_eq!(
            descriptors[1],
            GpuScenePrepareDescriptor {
                descriptor_kind: SCENE_PREPARE_DESCRIPTOR_KIND_VOXEL_CELL,
                primary_id: 7,
                secondary_id: 42,
                tertiary_id: 4,
                quaternary_id: 6_307_872,
                scalar0: 2112,
                scalar1: 2112,
                scalar2: 2112,
                scalar3: 64,
                _padding0: 99,
                _padding1: 1,
                _padding2: 0,
            }
        );
    }

    #[test]
    fn gpu_scene_prepare_descriptors_pack_explicit_card_capture_seed_rgb() {
        let requests = vec![HybridGiPrepareCardCaptureRequest {
            card_id: 7,
            page_id: 8,
            atlas_slot_id: 9,
            capture_slot_id: 10,
            bounds_center: Vec3::new(1.25, -2.5, 3.75),
            bounds_radius: 1.5,
        }];

        let descriptors =
            gpu_scene_prepare_descriptors(&requests, &[Some(pack_rgb8([32, 64, 96]))], &[], &[]);

        assert_eq!(descriptors.len(), 1);
        assert_eq!(
            descriptors[0],
            GpuScenePrepareDescriptor {
                descriptor_kind: SCENE_PREPARE_DESCRIPTOR_KIND_CARD_CAPTURE,
                primary_id: 7,
                secondary_id: 8,
                tertiary_id: 9,
                quaternary_id: 10,
                scalar0: 2128,
                scalar1: 1888,
                scalar2: 2288,
                scalar3: 96,
                _padding0: pack_rgb8([32, 64, 96]),
                _padding1: 1,
                _padding2: 0,
            }
        );
    }

    #[test]
    fn gpu_scene_prepare_descriptors_preserve_explicit_black_card_capture_seed() {
        let requests = vec![HybridGiPrepareCardCaptureRequest {
            card_id: 7,
            page_id: 8,
            atlas_slot_id: 9,
            capture_slot_id: 10,
            bounds_center: Vec3::new(1.25, -2.5, 3.75),
            bounds_radius: 1.5,
        }];

        let descriptors = gpu_scene_prepare_descriptors(&requests, &[Some(0)], &[], &[]);

        assert_eq!(descriptors.len(), 1);
        assert_eq!(descriptors[0]._padding0, 0);
        assert_eq!(
            descriptors[0]._padding1, 1,
            "explicit black card-capture seeds must stay distinguishable from missing packed seed payload"
        );
    }

    #[test]
    fn gpu_scene_prepare_descriptors_preserve_explicit_black_runtime_voxel_radiance() {
        let clipmaps = vec![HybridGiPrepareVoxelClipmap {
            clipmap_id: 7,
            center: Vec3::ZERO,
            half_extent: 4.0,
        }];
        let descriptors = gpu_scene_prepare_descriptors(
            &[],
            &[],
            &clipmaps,
            &[HybridGiPrepareVoxelCell {
                clipmap_id: 7,
                cell_index: 42,
                occupancy_count: 4,
                dominant_card_id: 99,
                radiance_present: true,
                radiance_rgb: [0, 0, 0],
            }],
        );

        assert_eq!(descriptors.len(), 2);
        assert_eq!(descriptors[1].quaternary_id, 0);
        assert_eq!(
            descriptors[1]._padding1, 1,
            "explicit black runtime voxel radiance must stay distinguishable from missing voxel radiance authority"
        );
    }
}
