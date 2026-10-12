use std::sync::Arc;

use zircon_runtime::asset::{
    MeshSdfAsset, MeshSdfCookSettings, MeshSdfEncoding, MeshSdfValidationError,
    MESH_SDF_SCHEMA_VERSION,
};
use zircon_runtime::core::framework::render::{
    render_mesh_stable_instance_key, render_mesh_transform_revision, RenderLayerSet,
    RenderMeshBounds, RenderMeshSnapshot, RenderMeshStaticState, RendererCommon,
};
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::core::math::{Transform, Vec3, Vec4};
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use zircon_runtime::graphics::{RuntimePrepareMeshSdfDeformationReason, RuntimePrepareMeshSdfSeed};

use crate::hybrid_gi::scene_representation::{
    HybridGiGlobalSdfClipmapBounds, HybridGiGlobalSdfPageBuildRequest, HybridGiGlobalSdfSceneState,
    HybridGiMeshSdfAssetState, HybridGiMeshSdfMaterialFlags, HybridGiMeshSdfObject,
    GLOBAL_SDF_MAX_RESIDENT_PAGE_COUNT,
};

use super::*;

#[test]
fn gpu_abi_is_four_byte_aligned_and_page_capacity_is_bounded() {
    assert_eq!(std::mem::size_of::<GlobalSdfGpuDispatchParams>(), 16);
    assert_eq!(std::mem::size_of::<GlobalSdfGpuPage>(), 32);
    assert_eq!(std::mem::size_of::<GlobalSdfGpuObject>(), 48);
    assert_eq!(std::mem::size_of::<GlobalSdfGpuMeshPayload>(), 128);
    assert_eq!(GLOBAL_SDF_PAGE_VOXEL_COUNT, 512);
    assert_eq!(GLOBAL_SDF_MAX_RESIDENT_PAGE_COUNT, 128);
}

#[test]
fn typed_mesh_sdf_fallback_objects_do_not_publish_global_sdf_pages() {
    let states = [
        HybridGiMeshSdfAssetState::default(),
        HybridGiMeshSdfAssetState::from_runtime(RuntimePrepareMeshSdfSeed::Invalid {
            primitive_index: 0,
            error: MeshSdfValidationError::InvalidDimensions,
        }),
        HybridGiMeshSdfAssetState::from_runtime(RuntimePrepareMeshSdfSeed::Deforming(
            RuntimePrepareMeshSdfDeformationReason::ActiveMorphTargets,
        )),
        HybridGiMeshSdfAssetState::from_runtime(RuntimePrepareMeshSdfSeed::Deforming(
            RuntimePrepareMeshSdfDeformationReason::Skinning,
        )),
    ];

    for (index, state) in states.into_iter().enumerate() {
        let (mut scene, requests, clipmaps) = one_page_scene();
        let object = mesh_sdf_object(
            index as u64 + 1,
            Vec3::new(-2.0, 2.0, 2.0),
            RenderMeshBounds::from_min_max([-0.5; 3], [0.5; 3]),
            state,
            &clipmaps,
        );
        scene.synchronize_influence(std::slice::from_ref(&object));
        let packed = pack_global_sdf_build_inputs(&scene, &[object], &requests, 1);
        assert!(packed.pages.is_empty());
        assert!(packed.requests.is_empty());
        assert!(packed.objects.is_empty());
        assert_eq!(
            packed.dispositions[0].kind,
            GlobalSdfPageBuildDispositionKind::TerminalFallback
        );
    }
}

#[test]
fn payload_count_and_upload_budget_overflow_keep_page_on_voxel_fallback() {
    let local_bounds = RenderMeshBounds::from_min_max([-0.5; 3], [0.5; 3]);
    let too_many_payloads = HybridGiMeshSdfAssetState::Ready(Arc::from(
        (0..GLOBAL_SDF_MAX_OBJECT_PAYLOADS + 1)
            .map(|_| mesh_sdf_asset(local_bounds, 64))
            .collect::<Vec<_>>(),
    ));
    let upload_overflow = HybridGiMeshSdfAssetState::Ready(Arc::from(vec![mesh_sdf_asset(
        local_bounds,
        GLOBAL_SDF_MAX_UPLOAD_VOXEL_WORDS + 1,
    )]));

    for (index, state) in [too_many_payloads, upload_overflow].into_iter().enumerate() {
        let (mut scene, requests, clipmaps) = one_page_scene();
        let object = mesh_sdf_object(
            index as u64 + 10,
            Vec3::new(-2.0, 2.0, 2.0),
            local_bounds,
            state,
            &clipmaps,
        );
        scene.synchronize_influence(std::slice::from_ref(&object));
        assert!(
            pack_global_sdf_build_inputs(&scene, &[object], &requests, 1)
                .pages
                .is_empty()
        );
    }
}

#[test]
fn page_influence_band_includes_ready_geometry_in_the_adjacent_page() {
    let (mut scene, requests, clipmaps) = one_page_scene();
    let local_bounds = RenderMeshBounds::from_min_max([-0.5; 3], [0.5; 3]);
    let object = mesh_sdf_object(
        20,
        Vec3::new(0.75, 2.0, 2.0),
        local_bounds,
        ready_state(local_bounds),
        &clipmaps,
    );
    scene.synchronize_influence(std::slice::from_ref(&object));
    let page_bounds = scene.page_bounds(requests[0].key()).unwrap();
    assert!(object.bounds().min[0] > page_bounds.max[0]);

    let packed = pack_global_sdf_build_inputs(&scene, &[object], &requests, 1);

    assert_eq!(packed.pages.len(), 1);
    assert_eq!(packed.candidates, vec![0]);
}

#[test]
fn thirty_third_page_candidate_keeps_dense_page_uninitialized() {
    let (mut scene, requests, clipmaps) = one_page_scene();
    let local_bounds = RenderMeshBounds::from_min_max([-0.25; 3], [0.25; 3]);
    let objects = (0..GLOBAL_SDF_MAX_PAGE_CANDIDATES + 1)
        .map(|index| {
            mesh_sdf_object(
                index as u64 + 100,
                Vec3::new(-2.0, 2.0, 2.0),
                local_bounds,
                ready_state(local_bounds),
                &clipmaps,
            )
        })
        .collect::<Vec<_>>();
    scene.synchronize_influence(&objects);

    let packed = pack_global_sdf_build_inputs(&scene, &objects, &requests, 1);

    assert!(packed.pages.is_empty());
    assert!(packed.requests.is_empty());
    assert_eq!(
        packed.dispositions[0].kind,
        GlobalSdfPageBuildDispositionKind::TerminalFallback
    );
    assert_eq!(packed.stats.candidate_overflow_page_count, 1);
    assert_eq!(packed.stats.dispatched_page_count, 0);
    assert_eq!(packed.stats.transient_upload_byte_count, 0);
}

#[test]
fn packed_global_sdf_build_reports_unique_upload_bytes() {
    let (mut scene, requests, clipmaps) = one_page_scene();
    let local_bounds = RenderMeshBounds::from_min_max([-0.5; 3], [0.5; 3]);
    let object = mesh_sdf_object(
        21,
        Vec3::new(-2.0, 2.0, 2.0),
        local_bounds,
        ready_state(local_bounds),
        &clipmaps,
    );
    scene.synchronize_influence(std::slice::from_ref(&object));

    let packed = pack_global_sdf_build_inputs(&scene, &[object], &requests, 1);

    assert_eq!(packed.stats.dispatched_page_count, 1);
    assert_eq!(packed.stats.candidate_overflow_page_count, 0);
    assert_eq!(packed.stats.transient_buffer_creation_count, 7);
    assert_eq!(packed.stats.transient_bind_group_creation_count, 1);
    assert_eq!(packed.stats.transient_parameter_upload_byte_count, 16);
    assert_eq!(packed.stats.transient_page_upload_byte_count, 36);
    assert_eq!(packed.stats.transient_mesh_upload_byte_count, 432);
    assert_eq!(packed.stats.transient_completion_upload_byte_count, 4);
    assert_eq!(packed.stats.transient_upload_byte_count, 488);
    assert_eq!(
        packed.stats.transient_upload_byte_count,
        packed
            .stats
            .transient_parameter_upload_byte_count
            .saturating_add(packed.stats.transient_page_upload_byte_count)
            .saturating_add(packed.stats.transient_mesh_upload_byte_count)
            .saturating_add(packed.stats.transient_completion_upload_byte_count),
    );
}

#[test]
fn terminal_fallback_prefix_does_not_starve_a_later_ready_page() {
    let mut scene = HybridGiGlobalSdfSceneState::default();
    scene.synchronize(Vec3::ZERO, &[], 2);
    let requests = scene.dirty_page_build_requests();
    assert_eq!(requests.len(), 2);
    let clipmaps = scene.clipmap_bounds().to_vec();
    let local_bounds = RenderMeshBounds::from_min_max([-0.25; 3], [0.25; 3]);
    let object = mesh_sdf_object(
        500,
        Vec3::new(15.5, 4.0, 4.0),
        local_bounds,
        ready_state(local_bounds),
        &clipmaps,
    );
    assert_eq!(requests[0].key().clipmap_id(), 0);
    assert_eq!(requests[1].key().clipmap_id(), 1);
    let fine_influence = scene.page_influence_bounds(requests[0].key()).unwrap();
    let coarse_influence = scene.page_influence_bounds(requests[1].key()).unwrap();
    assert!(object.bounds().min[0] > fine_influence.max[0]);
    assert!(object.bounds().min[0] <= coarse_influence.max[0]);
    scene.synchronize_influence(std::slice::from_ref(&object));

    let packed = pack_global_sdf_build_inputs(&scene, &[object], &requests, 1);

    assert_eq!(packed.pages.len(), 1);
    assert_eq!(packed.requests, vec![requests[1]]);
    assert_eq!(packed.dispositions.len(), requests.len());
    assert_eq!(
        packed.dispositions,
        vec![
            GlobalSdfPageBuildDisposition {
                request: requests[0],
                kind: GlobalSdfPageBuildDispositionKind::TerminalFallback,
            },
            GlobalSdfPageBuildDisposition {
                request: requests[1],
                kind: GlobalSdfPageBuildDispositionKind::Build,
            },
        ]
    );
}

#[test]
fn page_budget_defers_a_complete_page_without_downgrading_it_to_fallback() {
    let mut scene = HybridGiGlobalSdfSceneState::default();
    scene.synchronize(Vec3::ZERO, &[], 2);
    let requests = scene.dirty_page_build_requests();
    assert_eq!(requests.len(), 2);
    let clipmaps = scene.clipmap_bounds().to_vec();
    let local_bounds = RenderMeshBounds::from_min_max([-0.25; 3], [0.25; 3]);
    let objects = requests
        .iter()
        .enumerate()
        .map(|(index, request)| {
            let bounds = scene.page_bounds(request.key()).unwrap();
            let center = (Vec3::from_array(bounds.min) + Vec3::from_array(bounds.max)) * 0.5;
            mesh_sdf_object(
                index as u64 + 600,
                center,
                local_bounds,
                ready_state(local_bounds),
                &clipmaps,
            )
        })
        .collect::<Vec<_>>();
    scene.synchronize_influence(&objects);

    let packed = pack_global_sdf_build_inputs(&scene, &objects, &requests, 1);

    assert_eq!(packed.pages.len(), 1);
    assert_eq!(
        packed
            .dispositions
            .iter()
            .filter(|disposition| disposition.kind == GlobalSdfPageBuildDispositionKind::Build)
            .count(),
        1
    );
    assert_eq!(
        packed
            .dispositions
            .iter()
            .filter(|disposition| {
                disposition.kind == GlobalSdfPageBuildDispositionKind::DeferredPageBudget
            })
            .count(),
        1
    );
    assert!(!packed.dispositions.iter().any(|disposition| {
        disposition.kind == GlobalSdfPageBuildDispositionKind::TerminalFallback
    }));
}

fn one_page_scene() -> (
    HybridGiGlobalSdfSceneState,
    Vec<HybridGiGlobalSdfPageBuildRequest>,
    Vec<HybridGiGlobalSdfClipmapBounds>,
) {
    let mut scene = HybridGiGlobalSdfSceneState::default();
    scene.synchronize(Vec3::new(-4.0, 0.0, 0.0), &[], 1);
    let requests = scene.dirty_page_build_requests();
    let clipmaps = scene.clipmap_bounds().to_vec();
    (scene, requests, clipmaps)
}

fn ready_state(local_bounds: RenderMeshBounds) -> HybridGiMeshSdfAssetState {
    HybridGiMeshSdfAssetState::Ready(Arc::from(vec![mesh_sdf_asset(local_bounds, 64)]))
}

fn mesh_sdf_asset(local_bounds: RenderMeshBounds, voxel_count: usize) -> MeshSdfAsset {
    MeshSdfAsset {
        schema_version: MESH_SDF_SCHEMA_VERSION,
        source_hash: [1; 32],
        local_bounds,
        dimensions: [4; 3],
        voxel_size: [0.25; 3],
        distance_range: [-1.0, 1.0],
        encoding: MeshSdfEncoding::SignedNormalized16,
        cook_settings: MeshSdfCookSettings::default(),
        voxels: vec![0; voxel_count],
    }
}

fn mesh_sdf_object(
    node_id: u64,
    translation: Vec3,
    local_bounds: RenderMeshBounds,
    state: HybridGiMeshSdfAssetState,
    clipmaps: &[HybridGiGlobalSdfClipmapBounds],
) -> HybridGiMeshSdfObject {
    let transform = Transform::from_translation(translation);
    let mesh = RenderMeshSnapshot {
        node_id,
        stable_instance_key: render_mesh_stable_instance_key(node_id, 0),
        transform_revision: render_mesh_transform_revision(&transform),
        transform,
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
            "res://models/global-sdf-packing.model.toml",
        )),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
            "res://materials/global-sdf-packing.zmaterial",
        )),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Static,
        static_state: RenderMeshStaticState::from_transform_static(true),
        common: RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            is_static: true,
            ..RendererCommon::default()
        },
    };
    HybridGiMeshSdfObject::from_sources(
        &mesh,
        local_bounds,
        1,
        1,
        state,
        HybridGiMeshSdfMaterialFlags::default(),
        clipmaps,
    )
}
