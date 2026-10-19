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

#[test]
fn provider_probe_history_requires_complete_scene_and_world_provenance() {
    use zircon_runtime::core::framework::render::{
        RenderHybridGiDebugView, RenderWorldSnapshotHandle,
    };
    fn prepare(
        state: &mut PluginHybridGiRuntimeState,
        extract: Option<&RenderHybridGiExtract>,
        meshes: &[RenderMeshSnapshot],
        world: Option<RenderWorldSnapshotHandle>,
        invalidated: bool,
    ) -> RenderHybridGiPreparedFrame {
        let mut input = HybridGiRuntimePrepareInput::new(
            extract,
            meshes,
            &[],
            &[],
            &[],
            None,
            false,
            None,
            100,
        )
        .with_view_state(None, invalidated);
        if let Some(world) = world {
            input = input.with_source_world(world);
        }
        state
            .prepare_frame(input)
            .prepared_frame()
            .cloned()
            .unwrap_or_default()
    }
    fn seed(state: &mut PluginHybridGiRuntimeState) {
        state.update_after_render(HybridGiRuntimeFeedback::new(
            Some(RuntimeHybridGiGpuCompletion::new(
                vec![],
                vec![0],
                vec![],
                vec![(0, [13, 29, 47])],
                vec![(0, [97, 53, 19])],
                None,
            )),
            None,
        ));
    }
    let extract = scene_prepare_extract();
    let base = scene_prepare_mesh(11, Vec3::ZERO, Vec4::ONE);
    let world = RenderWorldSnapshotHandle::new(71).with_generation(5);
    let mut state = PluginHybridGiRuntimeState::default();
    let first = prepare(
        &mut state,
        Some(&extract),
        &[base.clone()],
        Some(world),
        false,
    );
    assert_ne!(first.radiance_cache_instance_id, 0);
    seed(&mut state);
    let stable = prepare(
        &mut state,
        Some(&extract),
        &[base.clone()],
        Some(world.with_generation(6)),
        false,
    );
    assert_eq!(
        stable.radiance_cache_instance_id,
        first.radiance_cache_instance_id
    );
    assert_eq!(
        stable.probe_rt_lighting_rgb[0].rt_lighting_rgb,
        [97, 53, 19]
    );
    assert_eq!(
        state.state.build_resolve_runtime().probe_rt_lighting_rgb(0),
        Some([97, 53, 19])
    );
    for (label, changed_world, invalidate) in [
        (
            "different-world",
            Some(RenderWorldSnapshotHandle::new(72).with_generation(6)),
            false,
        ),
        ("generation-rollback", Some(world.with_generation(4)), false),
        ("unknown-world", None, false),
        (
            "offline-zero",
            Some(RenderWorldSnapshotHandle::new(0)),
            false,
        ),
        (
            "explicit-invalidation",
            Some(world.with_generation(7)),
            true,
        ),
    ] {
        let mut state = PluginHybridGiRuntimeState::default();
        let before = prepare(
            &mut state,
            Some(&extract),
            &[base.clone()],
            Some(world),
            false,
        );
        seed(&mut state);
        let after = prepare(
            &mut state,
            Some(&extract),
            &[base.clone()],
            changed_world,
            invalidate,
        );
        assert_ne!(
            after.radiance_cache_instance_id, before.radiance_cache_instance_id,
            "{label}"
        );
        assert_ne!(after.radiance_cache_instance_id, 0, "{label}");
        assert!(after.probe_rt_lighting_rgb.is_empty(), "{label}");
    }
    let mut variants = Vec::new();
    let mut v = base.clone();
    v.model = ResourceHandle::new(ResourceId::from_stable_label("different-model"));
    variants.push(v);
    let mut v = base.clone();
    v.mesh = Some(ResourceHandle::new(ResourceId::from_stable_label(
        "different-mesh",
    )));
    variants.push(v);
    let mut v = base.clone();
    v.material = ResourceHandle::new(ResourceId::from_stable_label("different-material"));
    variants.push(v);
    let mut v = base.clone();
    v.static_state.geometry_revision += 1;
    variants.push(v);
    let mut v = base.clone();
    v.static_state.material_revision += 1;
    variants.push(v);
    let mut v = base.clone();
    v.transform.translation.x += 0.25;
    variants.push(v);
    let mut v = base.clone();
    v.transform_revision = v.transform_revision.wrapping_add(1);
    variants.push(v);
    let mut v = base.clone();
    v.morph_weights.push(0.5);
    variants.push(v);
    let mut v = base.clone();
    v.mesh_lod = Some(zircon_runtime::core::framework::render::RenderMeshLodSelection::new(1, 4.0));
    variants.push(v);
    let mut v = base.clone();
    v.tint.x = 0.25;
    variants.push(v);
    let mut v = base.clone();
    v.common.is_static = false;
    variants.push(v);
    let mut v = base.clone();
    v.mobility = Mobility::Dynamic;
    variants.push(v);
    let mut v = base.clone();
    v.stable_instance_key += 1;
    variants.push(v);
    let mut v = base.clone();
    v.node_id += 1;
    variants.push(v);
    let mut v = base.clone();
    v.transform.translation.x = f32::NAN;
    variants.push(v);
    for (index, changed) in variants.into_iter().enumerate() {
        let mut state = PluginHybridGiRuntimeState::default();
        let before = prepare(
            &mut state,
            Some(&extract),
            &[base.clone()],
            Some(world),
            false,
        );
        seed(&mut state);
        let after = prepare(
            &mut state,
            Some(&extract),
            &[changed],
            Some(world.with_generation(6)),
            false,
        );
        assert_ne!(
            before.radiance_cache_instance_id, after.radiance_cache_instance_id,
            "mesh variant {index}"
        );
        assert!(
            after.probe_rt_lighting_rgb.is_empty(),
            "mesh variant {index}"
        );
    }
    let mut changed = extract.clone();
    changed.debug_view = RenderHybridGiDebugView::SurfaceCache;
    seed(&mut state);
    let changed_settings = prepare(
        &mut state,
        Some(&changed),
        &[base.clone()],
        Some(world.with_generation(7)),
        false,
    );
    assert_ne!(
        stable.radiance_cache_instance_id,
        changed_settings.radiance_cache_instance_id
    );
    assert!(changed_settings.probe_rt_lighting_rgb.is_empty());
    seed(&mut state);
    let disabled = prepare(
        &mut state,
        None,
        &[base.clone()],
        Some(world.with_generation(8)),
        false,
    );
    assert!(disabled.is_empty());
    let restarted = prepare(
        &mut state,
        Some(&extract),
        &[base.clone()],
        Some(world.with_generation(9)),
        false,
    );
    assert_ne!(
        restarted.radiance_cache_instance_id,
        changed_settings.radiance_cache_instance_id
    );
    assert!(restarted.probe_rt_lighting_rgb.is_empty());
    seed(&mut state);
    state.state.register_extract(Some(&extract));
    assert_eq!(
        state.state.build_resolve_runtime().probe_rt_lighting_rgb(0),
        None
    );
}

#[test]
fn scene_probe_history_compares_all_lights_and_the_complete_baked_contract() {
    use zircon_runtime::core::framework::render::{
        LightmapAtlasDescriptor, LightmapAtlasFormat, LightmapConsumeContract,
        LightmapInstanceSlot, RenderDirectionalLightSnapshot, RenderHybridGiMode,
        RenderPointLightSnapshot, RenderSpotLightSnapshot,
    };
    let mesh = scene_prepare_mesh(11, Vec3::ZERO, Vec4::ONE);
    let mut extract = scene_prepare_extract();
    extract.mode = RenderHybridGiMode::BakedStaticDynamic;
    let directional = RenderDirectionalLightSnapshot {
        node_id: 1,
        light_id: 1,
        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
        direction: Vec3::NEG_Y,
        color: Vec3::ONE,
        intensity: 1.0,
        mobility: Mobility::Static,
        shadow: None,
    };
    let point = RenderPointLightSnapshot {
        node_id: 2,
        light_id: 2,
        layer_mask: directional.layer_mask.clone(),
        position: Vec3::ONE,
        color: Vec3::ONE,
        intensity: 1.0,
        range: 4.0,
        mobility: Mobility::Static,
        shadow: None,
    };
    let spot = RenderSpotLightSnapshot {
        node_id: 3,
        light_id: 3,
        layer_mask: directional.layer_mask.clone(),
        position: Vec3::ONE,
        direction: Vec3::NEG_Y,
        color: Vec3::ONE,
        intensity: 1.0,
        range: 4.0,
        inner_angle_radians: 0.25,
        outer_angle_radians: 0.5,
        mobility: Mobility::Static,
        shadow: None,
    };
    let contract = |generation, atlas, page_size, uv| {
        LightmapConsumeContract::new(
            generation,
            ResourceId::from_stable_label(atlas),
            LightmapAtlasDescriptor {
                page_size,
                page_count: 1,
                format: LightmapAtlasFormat::Rgba16Float,
            },
            vec![(
                mesh.stable_instance_key,
                LightmapInstanceSlot {
                    atlas_page: 0,
                    uv_rect: uv,
                },
            )],
        )
    };
    let baked = contract(7, "baked-history-atlas", 4, Vec4::new(1.0, 1.0, 0.0, 0.0));
    for variant in 0..9 {
        let mut state = HybridGiRuntimeState::default();
        state.register_scene_extract(
            Some(&extract),
            &[mesh.clone()],
            &[directional.clone()],
            &[point.clone()],
            &[spot.clone()],
            Some(&baked),
            true,
        );
        state.complete_gpu_updates([0], [], &[(0, [13, 29, 47])], &[(0, [97, 53, 19])], &[]);
        assert!(state.register_scene_extract_with_view_state(
            Some(&extract),
            &[mesh.clone()],
            &[directional.clone()],
            &[point.clone()],
            &[spot.clone()],
            Some(&baked),
            true,
            None,
            false
        ));
        assert_eq!(
            state.build_resolve_runtime().probe_rt_lighting_rgb(0),
            Some([97, 53, 19])
        );
        let mut d = directional.clone();
        let mut p = point.clone();
        let mut s = spot.clone();
        let mut b = baked.clone();
        let mut grid = true;
        let mut settings = extract.clone();
        match variant {
            0 => d.color.x = 0.25,
            1 => p.range = 8.0,
            2 => s.outer_angle_radians = 0.75,
            3 => b = contract(8, "baked-history-atlas", 4, Vec4::new(1.0, 1.0, 0.0, 0.0)),
            4 => b = contract(7, "different-baked-atlas", 4, Vec4::new(1.0, 1.0, 0.0, 0.0)),
            5 => b = contract(7, "baked-history-atlas", 8, Vec4::new(1.0, 1.0, 0.0, 0.0)),
            6 => b = contract(7, "baked-history-atlas", 4, Vec4::new(0.5, 0.5, 0.0, 0.0)),
            7 => grid = false,
            8 => settings.trace_budget += 1,
            _ => unreachable!(),
        }
        assert!(
            !state.register_scene_extract_with_view_state(
                Some(&settings),
                &[mesh.clone()],
                &[d],
                &[p],
                &[s],
                Some(&b),
                grid,
                None,
                false
            ),
            "source variant {variant}"
        );
        assert_eq!(
            state.build_resolve_runtime().probe_rt_lighting_rgb(0),
            None,
            "source variant {variant}"
        );
    }
}
