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
        gpu_scene_prepare_voxel_cell_descriptor_range(&descriptors),
        (1, 1)
    );
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
fn descriptor_range_excludes_cards_clipmap_headers_and_empty_voxel_cells() {
    let requests = vec![HybridGiPrepareCardCaptureRequest {
        card_id: 7,
        page_id: 8,
        atlas_slot_id: 9,
        capture_slot_id: 10,
        bounds_center: Vec3::ZERO,
        bounds_radius: 1.0,
    }];
    let clipmaps = vec![HybridGiPrepareVoxelClipmap {
        clipmap_id: 7,
        center: Vec3::ZERO,
        half_extent: 4.0,
    }];
    let descriptors = gpu_scene_prepare_descriptors(
        &requests,
        &[],
        &[Some(pack_rgb8([1, 2, 3]))],
        &[],
        &clipmaps,
        &[
            HybridGiPrepareVoxelCell {
                clipmap_id: 7,
                cell_index: 1,
                occupancy_count: 0,
                dominant_card_id: 0,
                radiance_present: false,
                radiance_rgb: [0; 3],
            },
            HybridGiPrepareVoxelCell {
                clipmap_id: 7,
                cell_index: 2,
                occupancy_count: 1,
                dominant_card_id: 7,
                radiance_present: true,
                radiance_rgb: [4, 5, 6],
            },
        ],
    );

    assert_eq!(descriptors.len(), 3);
    assert_eq!(
        gpu_scene_prepare_voxel_cell_descriptor_range(&descriptors),
        (2, 1)
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
