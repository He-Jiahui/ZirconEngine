use std::collections::{BTreeMap, BTreeSet};

use bytemuck::{Pod, Zeroable};

use super::super::super::super::seed_quantization::{quantized_positive, quantized_signed};
use super::super::card_capture_shading::scene_card_capture_rgba;
use super::super::hybrid_gi_prepare_execution_inputs::HybridGiPrepareExecutionInputs;
use crate::hybrid_gi::types::{
    hybrid_gi_voxel_clipmap_cell_center, HybridGiPrepareSurfaceCachePageContent,
    HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT, HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION,
};

use super::super::material_capture_source::HybridGiMaterialCaptureSource;

const SCENE_CARD_CAPTURE_RADIUS_SCALE: f32 = 64.0;
const SCENE_VOXEL_CLIPMAP_HALF_EXTENT_SCALE: f32 = 64.0;
const SCENE_VOXEL_CELL_HALF_EXTENT_SCALE: f32 = 64.0;
const SCENE_PREPARE_DESCRIPTOR_KIND_CARD_CAPTURE: u32 = 1;
const SCENE_PREPARE_DESCRIPTOR_KIND_VOXEL_CLIPMAP: u32 = 2;
const SCENE_PREPARE_DESCRIPTOR_KIND_VOXEL_CELL: u32 = 3;

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
pub(super) struct GpuScenePrepareDescriptor {
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

impl GpuScenePrepareDescriptor {
    pub(super) fn is_voxel_cell(self) -> bool {
        self.descriptor_kind == SCENE_PREPARE_DESCRIPTOR_KIND_VOXEL_CELL
    }

    pub(super) fn clipmap_id(self) -> u32 {
        self.primary_id
    }

    pub(super) fn voxel_cell_index(self) -> u32 {
        self.secondary_id
    }
}

fn gpu_scene_card_capture_requests(
    requests: &[crate::hybrid_gi::types::HybridGiPrepareCardCaptureRequest],
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

fn gpu_scene_persisted_page_card_capture_requests(
    requests: &[crate::hybrid_gi::types::HybridGiPrepareCardCaptureRequest],
    page_contents: &[HybridGiPrepareSurfaceCachePageContent],
) -> Vec<GpuSceneCardCaptureRequest> {
    let requested_page_ids = requests
        .iter()
        .map(|request| request.page_id)
        .collect::<BTreeSet<_>>();
    page_contents
        .iter()
        .filter(|page_content| {
            !requested_page_ids.contains(&page_content.page_id)
                && persisted_surface_cache_page_has_present_sample(page_content)
        })
        .map(|page_content| GpuSceneCardCaptureRequest {
            card_id: page_content.owner_card_id,
            page_id: page_content.page_id,
            atlas_slot_id: page_content.atlas_slot_id,
            capture_slot_id: page_content.capture_slot_id,
            bounds_center_x_q: quantized_signed(page_content.bounds_center.x),
            bounds_center_y_q: quantized_signed(page_content.bounds_center.y),
            bounds_center_z_q: quantized_signed(page_content.bounds_center.z),
            bounds_radius_q: quantized_positive(
                page_content.bounds_radius,
                SCENE_CARD_CAPTURE_RADIUS_SCALE,
            ),
        })
        .collect()
}

fn gpu_scene_voxel_clipmaps(
    clipmaps: &[crate::hybrid_gi::types::HybridGiPrepareVoxelClipmap],
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

pub(super) fn gpu_scene_card_capture_seed_rgb(
    requests: &[crate::hybrid_gi::types::HybridGiPrepareCardCaptureRequest],
    streamer: &impl HybridGiMaterialCaptureSource,
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

pub(super) fn gpu_scene_persisted_page_card_capture_seed_rgb(
    requests: &[crate::hybrid_gi::types::HybridGiPrepareCardCaptureRequest],
    page_contents: &[HybridGiPrepareSurfaceCachePageContent],
) -> Vec<Option<u32>> {
    let requested_page_ids = requests
        .iter()
        .map(|request| request.page_id)
        .collect::<BTreeSet<_>>();
    page_contents
        .iter()
        .filter(|page_content| {
            !requested_page_ids.contains(&page_content.page_id)
                && persisted_surface_cache_page_has_present_sample(page_content)
        })
        .map(persisted_surface_cache_page_seed_rgb)
        .collect()
}

pub(super) fn persisted_surface_cache_page_has_present_sample(
    page_content: &HybridGiPrepareSurfaceCachePageContent,
) -> bool {
    persisted_surface_cache_page_has_present_capture_sample(page_content)
        || persisted_surface_cache_page_has_present_atlas_sample(page_content)
}

pub(super) fn persisted_surface_cache_page_has_present_atlas_sample(
    page_content: &HybridGiPrepareSurfaceCachePageContent,
) -> bool {
    page_content.atlas_sample_rgba[3] > 0
}

pub(super) fn persisted_surface_cache_page_has_present_capture_sample(
    page_content: &HybridGiPrepareSurfaceCachePageContent,
) -> bool {
    page_content.capture_sample_rgba[3] > 0
}

fn persisted_surface_cache_page_seed_rgb(
    page_content: &HybridGiPrepareSurfaceCachePageContent,
) -> Option<u32> {
    if persisted_surface_cache_page_has_present_capture_sample(page_content) {
        return Some(pack_rgb8([
            page_content.capture_sample_rgba[0],
            page_content.capture_sample_rgba[1],
            page_content.capture_sample_rgba[2],
        ]));
    }

    if persisted_surface_cache_page_has_present_atlas_sample(page_content) {
        return Some(pack_rgb8([
            page_content.atlas_sample_rgba[0],
            page_content.atlas_sample_rgba[1],
            page_content.atlas_sample_rgba[2],
        ]));
    }

    None
}

pub(super) fn gpu_scene_prepare_descriptors(
    card_capture_requests: &[crate::hybrid_gi::types::HybridGiPrepareCardCaptureRequest],
    surface_cache_page_contents: &[HybridGiPrepareSurfaceCachePageContent],
    card_capture_seed_rgb: &[Option<u32>],
    persisted_page_seed_rgb: &[Option<u32>],
    voxel_clipmaps: &[crate::hybrid_gi::types::HybridGiPrepareVoxelClipmap],
    voxel_cells: &[crate::hybrid_gi::types::HybridGiPrepareVoxelCell],
) -> Vec<GpuScenePrepareDescriptor> {
    let staged_card_capture_requests = gpu_scene_card_capture_requests(card_capture_requests);
    let staged_persisted_page_requests = gpu_scene_persisted_page_card_capture_requests(
        card_capture_requests,
        surface_cache_page_contents,
    );
    let staged_voxel_clipmaps = gpu_scene_voxel_clipmaps(voxel_clipmaps);
    let clipmaps_by_id = voxel_clipmaps
        .iter()
        .map(|clipmap| (clipmap.clipmap_id, clipmap))
        .collect::<BTreeMap<_, _>>();
    let mut descriptors = Vec::with_capacity(
        staged_card_capture_requests.len()
            + staged_persisted_page_requests.len()
            + staged_voxel_clipmaps.len()
            + voxel_cells.len(),
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
    // 干净帧仍需把已驻留页写入 GPU 描述符；present 位使显式黑色与缺席样本可区分。
    descriptors.extend(staged_persisted_page_requests.into_iter().enumerate().map(
        |(index, request)| {
            let packed_seed_rgb = persisted_page_seed_rgb.get(index).copied().flatten();
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

pub(super) fn gpu_scene_prepare_voxel_cell_descriptor_range(
    descriptors: &[GpuScenePrepareDescriptor],
) -> (usize, usize) {
    let Some(offset) = descriptors
        .iter()
        .position(|descriptor| descriptor.is_voxel_cell())
    else {
        return (descriptors.len(), 0);
    };
    let count = descriptors[offset..]
        .iter()
        .take_while(|descriptor| descriptor.is_voxel_cell())
        .count();
    (offset, count)
}

#[cfg(test)]
#[path = "tests/scene_prepare_descriptors.rs"]
mod tests;
