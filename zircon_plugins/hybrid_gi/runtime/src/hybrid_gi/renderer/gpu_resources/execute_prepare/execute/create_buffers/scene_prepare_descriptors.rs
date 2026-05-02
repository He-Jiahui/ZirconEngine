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

#[cfg(test)]
mod tests {
    use crate::hybrid_gi::types::{
        HybridGiPrepareCardCaptureRequest, HybridGiPrepareSurfaceCachePageContent,
        HybridGiPrepareVoxelCell, HybridGiPrepareVoxelClipmap,
    };
    use zircon_runtime::core::math::Vec3;

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

        let descriptors = gpu_scene_prepare_descriptors(
            &requests,
            &[],
            &[Some(pack_rgb8([32, 64, 96]))],
            &[],
            &[],
            &[],
        );

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

        let descriptors = gpu_scene_prepare_descriptors(&requests, &[], &[Some(0)], &[], &[], &[]);

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

    #[test]
    fn gpu_scene_prepare_descriptors_include_clean_frame_persisted_surface_cache_pages() {
        let descriptors = gpu_scene_prepare_descriptors(
            &[],
            &[HybridGiPrepareSurfaceCachePageContent {
                page_id: 22,
                owner_card_id: 7,
                atlas_slot_id: 3,
                capture_slot_id: 4,
                bounds_center: Vec3::new(1.25, -2.5, 3.75),
                bounds_radius: 1.5,
                atlas_sample_rgba: [10, 20, 30, 255],
                capture_sample_rgba: [40, 50, 60, 255],
            }],
            &[],
            &[Some(pack_rgb8([40, 50, 60]))],
            &[],
            &[],
        );

        assert_eq!(descriptors.len(), 1);
        assert_eq!(
            descriptors[0],
            GpuScenePrepareDescriptor {
                descriptor_kind: SCENE_PREPARE_DESCRIPTOR_KIND_CARD_CAPTURE,
                primary_id: 7,
                secondary_id: 22,
                tertiary_id: 3,
                quaternary_id: 4,
                scalar0: 2128,
                scalar1: 1888,
                scalar2: 2288,
                scalar3: 96,
                _padding0: pack_rgb8([40, 50, 60]),
                _padding1: 1,
                _padding2: 0,
            }
        );
    }

    #[test]
    fn gpu_scene_prepare_descriptors_skip_absent_clean_frame_persisted_surface_cache_pages() {
        let descriptors = gpu_scene_prepare_descriptors(
            &[],
            &[HybridGiPrepareSurfaceCachePageContent {
                page_id: 22,
                owner_card_id: 22,
                atlas_slot_id: 3,
                capture_slot_id: 4,
                bounds_center: Vec3::new(1.25, -2.5, 3.75),
                bounds_radius: 1.5,
                atlas_sample_rgba: [0, 0, 0, 0],
                capture_sample_rgba: [0, 0, 0, 0],
            }],
            &[],
            &[None],
            &[],
            &[],
        );

        assert!(
            descriptors.is_empty(),
            "expected absent persisted surface-cache page samples to skip synthetic card-descriptor staging instead of creating false black GPU authority"
        );
    }

    #[test]
    fn gpu_scene_persisted_page_card_capture_seed_rgb_uses_atlas_when_capture_sample_is_absent() {
        let packed = gpu_scene_persisted_page_card_capture_seed_rgb(
            &[],
            &[HybridGiPrepareSurfaceCachePageContent {
                page_id: 22,
                owner_card_id: 22,
                atlas_slot_id: 3,
                capture_slot_id: 4,
                bounds_center: Vec3::new(1.25, -2.5, 3.75),
                bounds_radius: 1.5,
                atlas_sample_rgba: [10, 20, 30, 255],
                capture_sample_rgba: [0, 0, 0, 0],
            }],
        );

        assert_eq!(
            packed,
            vec![Some(pack_rgb8([10, 20, 30]))],
            "expected atlas sample RGB to seed the persisted page descriptor when capture sample is absent"
        );
    }
}
