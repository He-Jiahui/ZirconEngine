use super::*;
use crate::hybrid_gi::{
    HybridGiPrepareProbe, HybridGiResolveProbeSceneData, HybridGiScenePrepareResourceSamples,
};
use zircon_runtime::core::framework::render::{
    render_mesh_stable_instance_key, render_mesh_transform_revision, RenderHybridGiCompositePolicy,
    RenderHybridGiExtract, RenderHybridGiScenePrepareSample,
    RenderHybridGiVoxelCellDominantNodeRecord, RenderHybridGiVoxelCellRecord,
    RenderHybridGiVoxelCellSampleRecord, RenderHybridGiVoxelOccupancyMaskRecord, RenderLayerSet,
    RenderMeshSnapshot, RenderMeshStaticState, RendererCommon, HYBRID_GI_SOURCE_FULL_DYNAMIC,
};
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::core::math::{Transform, Vec3, Vec4};
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

#[test]
fn provider_projects_scene_screen_probes_into_neutral_prepared_frame_sideband() {
    let provider = PluginHybridGiRuntimeProvider;
    let mut state = provider.create_state();
    let mut extract = scene_prepare_extract();
    extract.trace_budget = 2;
    let meshes = vec![
        scene_prepare_mesh(11, Vec3::new(-1.0, 0.0, 0.0), Vec4::ONE),
        scene_prepare_mesh(22, Vec3::new(3.0, 0.0, 0.0), Vec4::ONE),
    ];

    let prepare = state.prepare_frame(HybridGiRuntimePrepareInput::new(
        Some(&extract),
        &meshes,
        &[],
        &[],
        &[],
        None,
        false,
        None,
        7,
    ));

    let prepared_frame = prepare
        .prepared_frame()
        .expect("scene screen probes should be projected into neutral prepared frame");
    assert_eq!(prepared_frame.resident_probes.len(), 2);
    assert_eq!(prepared_frame.resident_probes[0].probe_id, 0);
    assert_eq!(prepared_frame.resident_probes[0].slot, 0);
    assert_eq!(prepared_frame.resident_probes[0].ray_budget, 1);
    assert_eq!(prepared_frame.resident_probes[1].probe_id, 1);
    assert!(
        prepared_frame
            .resident_probes
            .iter()
            .all(|probe| probe.irradiance_rgb != [0, 0, 0]),
        "screen-probe prepared sideband should carry radiance cache seeds"
    );
    assert_eq!(prepared_frame.probe_scene_data.len(), 2);
    assert_eq!(prepared_frame.probe_scene_data[0].probe_id, 0);
    assert_eq!(prepared_frame.probe_scene_data[0].position_x_q, 1984);
    assert_eq!(prepared_frame.probe_scene_data[0].radius_q, 96);
    assert_eq!(prepared_frame.probe_scene_data[1].probe_id, 1);
    assert_eq!(prepared_frame.probe_scene_data[1].position_x_q, 2240);
    assert_eq!(prepared_frame.probe_scene_data[1].radius_q, 96);
    let scene_prepare = prepared_frame
        .scene_prepare
        .as_ref()
        .expect("scene descriptors must cross the neutral prepared-frame boundary");
    assert_eq!(scene_prepare.card_owners.len(), 2);
    assert_eq!(scene_prepare.voxel_clipmaps.len(), 1);
    assert!(scene_prepare
        .card_owners
        .iter()
        .any(|owner| owner.stable_instance_key == meshes[0].stable_instance_key));
}

#[test]
fn provider_projects_probe_rt_lighting_history_into_neutral_prepared_frame_sideband() {
    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 77,
            slot: 0,
            stable_instance_key: 0,
            source_mask: HYBRID_GI_SOURCE_FULL_DYNAMIC,
            dynamic_weight_q8: u8::MAX,
            ray_budget: 24,
            irradiance_rgb: [4, 8, 12],
        }],
        ..HybridGiPrepareFrame::default()
    };
    let resolve_runtime = HybridGiResolveRuntime::new(
        BTreeMap::from([(77, HybridGiResolveProbeSceneData::new(2000, 2010, 2020, 96))]),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(77, [96, 48, 24])]),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeSet::new(),
        BTreeSet::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
    );

    let prepared_frame = neutral_prepared_frame_from_prepare(
        &prepare,
        &resolve_runtime,
        &HybridGiScenePrepareFrame::default(),
        1,
        RenderHybridGiCompositePolicy::default(),
        Some(RenderHybridGiExtract::default().resolved_settings(false)),
    );

    assert_eq!(prepared_frame.probe_scene_data[0].probe_id, 77);
    assert_eq!(prepared_frame.probe_rt_lighting_rgb.len(), 1);
    assert_eq!(prepared_frame.probe_rt_lighting_rgb[0].probe_id, 77);
    assert_eq!(
        prepared_frame.probe_rt_lighting_rgb[0].rt_lighting_rgb,
        [96, 48, 24]
    );
}

#[test]
fn provider_projects_neutral_voxel_readback_into_scene_prepare_resources() {
    let readback = RenderHybridGiScenePrepareReadbackOutputs {
        voxel_cells: vec![RenderHybridGiVoxelCellRecord {
            clipmap_id: 4,
            cell_id: 9,
            occupancy: 3,
        }],
        voxel_cell_dominant_nodes: vec![RenderHybridGiVoxelCellDominantNodeRecord {
            clipmap_id: 4,
            cell_id: 9,
            dominant_node_id: 77,
        }],
        voxel_cell_samples: vec![RenderHybridGiVoxelCellSampleRecord {
            clipmap_id: 4,
            cell_id: 9,
            rgba8: [16, 24, 32, 255],
        }],
        voxel_cell_dominant_samples: vec![RenderHybridGiVoxelCellSampleRecord {
            clipmap_id: 4,
            cell_id: 9,
            rgba8: [48, 56, 64, 255],
        }],
        ..RenderHybridGiScenePrepareReadbackOutputs::default()
    };

    let resources = scene_prepare_resources_from_readback(Some(&readback))
        .expect("voxel cell readback should be runtime-consumable");

    assert_eq!(
        resources.voxel_cells(),
        &[HybridGiPrepareVoxelCell {
            clipmap_id: 4,
            cell_index: 9,
            occupancy_count: 3,
            dominant_card_id: 77,
            radiance_present: true,
            radiance_rgb: [48, 56, 64],
        }]
    );
}

#[test]
fn provider_prepared_scene_frame_waits_for_collector_readback() {
    let provider = PluginHybridGiRuntimeProvider;
    let mut state = provider.create_state();
    let extract = scene_prepare_extract();
    let mesh = scene_prepare_mesh(77, Vec3::ZERO, Vec4::new(1.0, 0.45, 0.2, 1.0));
    let meshes = vec![mesh];

    let (atlas_slot_id, capture_slot_id) = {
        let prepare = state.prepare_frame(HybridGiRuntimePrepareInput::new(
            Some(&extract),
            &meshes,
            &[],
            &[],
            &[],
            None,
            false,
            None,
            11,
        ));
        assert!(prepare.renderer_outputs().is_empty());
        let scene_prepare = prepare
            .prepared_frame()
            .and_then(|frame| frame.scene_prepare.as_ref())
            .expect("provider must publish scene preparation through the neutral sideband");
        (
            scene_prepare.card_capture_requests[0].atlas_slot_id,
            scene_prepare.card_capture_requests[0].capture_slot_id,
        )
    };
    state.update_after_render(HybridGiRuntimeFeedback::new(
        Some(RuntimeHybridGiGpuCompletion::new(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Some(RenderHybridGiScenePrepareReadbackOutputs {
                atlas_samples: vec![RenderHybridGiScenePrepareSample {
                    index: atlas_slot_id,
                    rgba8: [255, 115, 51, 255],
                }],
                capture_samples: vec![RenderHybridGiScenePrepareSample {
                    index: capture_slot_id,
                    rgba8: [255, 115, 51, 255],
                }],
                ..RenderHybridGiScenePrepareReadbackOutputs::default()
            }),
        )),
        None,
    ));
    let prepare = state.prepare_frame(HybridGiRuntimePrepareInput::new(
        Some(&extract),
        &meshes,
        &[],
        &[],
        &[],
        None,
        false,
        None,
        12,
    ));
    assert!(prepare.renderer_outputs().is_empty());
    let scene_prepare = prepare
        .prepared_frame()
        .and_then(|frame| frame.scene_prepare.as_ref())
        .expect("completion must update the next neutral scene-prepare sideband");

    assert!(
        !scene_prepare.surface_cache_page_contents.is_empty(),
        "completion-backed surface cache state must cross the neutral prepare sideband"
    );
    assert!(scene_prepare
        .surface_cache_page_contents
        .iter()
        .any(|page| {
            page.owner_card_id == 77
                && page.bounds_radius > 0.0
                && page.atlas_sample_rgba[3] == u8::MAX
        }));
    assert!(scene_prepare
        .voxel_clipmaps
        .iter()
        .any(|clipmap| clipmap.clipmap_id == 0 && clipmap.half_extent > 0.0));
    assert!(scene_prepare
        .voxel_cells
        .iter()
        .any(|cell| cell.clipmap_id == 0 && cell.occupancy_count > 0));
}

#[test]
fn provider_projects_neutral_voxel_mask_readback_into_fallback_cells() {
    let readback = RenderHybridGiScenePrepareReadbackOutputs {
        voxel_samples: vec![RenderHybridGiScenePrepareSample {
            index: 4,
            rgba8: [20, 40, 60, 255],
        }],
        voxel_occupancy_masks: vec![RenderHybridGiVoxelOccupancyMaskRecord {
            clipmap_id: 4,
            occupancy_mask: 0b1010,
        }],
        ..RenderHybridGiScenePrepareReadbackOutputs::default()
    };

    let resources = scene_prepare_resources_from_readback(Some(&readback))
        .expect("voxel occupancy mask readback should create fallback cells");

    assert_eq!(
        resources.voxel_cells(),
        &[
            HybridGiPrepareVoxelCell {
                clipmap_id: 4,
                cell_index: 1,
                occupancy_count: 1,
                dominant_card_id: 0,
                radiance_present: true,
                radiance_rgb: [20, 40, 60],
            },
            HybridGiPrepareVoxelCell {
                clipmap_id: 4,
                cell_index: 3,
                occupancy_count: 1,
                dominant_card_id: 0,
                radiance_present: true,
                radiance_rgb: [20, 40, 60],
            },
        ]
    );
}

#[test]
fn provider_projects_neutral_voxel_aggregate_count_into_low_detail_fallback_cell() {
    let readback = RenderHybridGiScenePrepareReadbackOutputs {
        voxel_clipmap_ids: vec![8],
        voxel_occupancy: vec![5],
        voxel_samples: vec![RenderHybridGiScenePrepareSample {
            index: 8,
            rgba8: [72, 88, 104, 255],
        }],
        ..RenderHybridGiScenePrepareReadbackOutputs::default()
    };

    let resources = scene_prepare_resources_from_readback(Some(&readback))
        .expect("aggregate voxel occupancy should create a low-detail fallback cell");

    assert_eq!(
        resources.voxel_cells(),
        &[HybridGiPrepareVoxelCell {
            clipmap_id: 8,
            cell_index: LOW_DETAIL_VOXEL_FALLBACK_CELL_INDEX,
            occupancy_count: 5,
            dominant_card_id: 0,
            radiance_present: true,
            radiance_rgb: [72, 88, 104],
        }]
    );
}

fn scene_prepare_extract() -> RenderHybridGiExtract {
    RenderHybridGiExtract {
        enabled: true,
        trace_budget: 1,
        card_budget: 1,
        voxel_budget: 1,
        ..RenderHybridGiExtract::default()
    }
}

fn scene_prepare_mesh(node_id: u64, translation: Vec3, tint: Vec4) -> RenderMeshSnapshot {
    let transform = Transform::from_translation(translation).with_scale(Vec3::splat(2.0));
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: render_mesh_stable_instance_key(node_id, 0),
        transform_revision: render_mesh_transform_revision(&transform),
        transform,
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
            "res://models/provider-scene-prepare-card.obj",
        )),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
            "res://materials/provider-scene-prepare-card.mat",
        )),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint,
        mobility: Mobility::Static,
        static_state: RenderMeshStaticState::from_transform_static(true),
        common: RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            is_static: true,
            ..RendererCommon::default()
        },
    }
}
