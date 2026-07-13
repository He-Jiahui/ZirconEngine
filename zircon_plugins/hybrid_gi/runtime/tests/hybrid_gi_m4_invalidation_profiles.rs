use zircon_plugin_hybrid_gi_runtime::PluginHybridGiRuntimeProvider;
use zircon_runtime::core::framework::render::{
    render_mesh_stable_instance_key, render_mesh_transform_revision, LightmapAtlasDescriptor,
    LightmapAtlasFormat, LightmapConsumeContract, LightmapInstanceSlot,
    RenderDirectionalLightSnapshot, RenderHybridGiExtract, RenderHybridGiFallbackReason,
    RenderHybridGiMode, RenderHybridGiProfile, RenderLayerSet, RenderMeshSnapshot,
    RenderMeshStaticState, HYBRID_GI_SOURCE_BAKED_BASELINE, HYBRID_GI_SOURCE_DYNAMIC_DELTA,
};
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::core::math::{Transform, Vec3, Vec4};
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use zircon_runtime::graphics::{
    HybridGiRuntimeFeedback, HybridGiRuntimePrepareInput, HybridGiRuntimeProvider,
};

#[test]
fn provider_invalidates_epoch_for_scene_light_generation_and_mobility_round_trip() {
    let provider = PluginHybridGiRuntimeProvider;
    let mut state = provider.create_state();
    let extract = RenderHybridGiExtract {
        enabled: true,
        mode: RenderHybridGiMode::BakedStaticDynamic,
        profile: RenderHybridGiProfile::Custom,
        trace_budget: 8,
        card_budget: 8,
        voxel_budget: 8,
        ..RenderHybridGiExtract::default()
    };
    let mut mesh = test_mesh(41, Mobility::Static);
    let mut meshes = vec![mesh.clone()];
    let mut lights = vec![test_directional_light(7, Mobility::Dynamic, 2.0)];
    let baked_v1 = baked_contract(10, mesh.stable_instance_key);

    let (initial_epoch, initial_mask) =
        prepare_epoch_and_mask(state.as_mut(), &extract, &meshes, &lights, Some(&baked_v1));
    assert_eq!(
        initial_mask,
        HYBRID_GI_SOURCE_BAKED_BASELINE | HYBRID_GI_SOURCE_DYNAMIC_DELTA
    );
    assert_eq!(
        prepare_epoch_and_mask(state.as_mut(), &extract, &meshes, &lights, Some(&baked_v1),).0,
        initial_epoch,
        "unchanged scene input must preserve the participation epoch"
    );

    mesh.transform = Transform::from_translation(Vec3::new(1.0, 0.0, 0.0));
    mesh.transform_revision = render_mesh_transform_revision(&mesh.transform);
    meshes[0] = mesh.clone();
    let transform_epoch =
        prepare_epoch_and_mask(state.as_mut(), &extract, &meshes, &lights, Some(&baked_v1)).0;
    assert!(transform_epoch > initial_epoch);

    mesh.static_state.material_revision = 9;
    meshes[0] = mesh.clone();
    let material_epoch =
        prepare_epoch_and_mask(state.as_mut(), &extract, &meshes, &lights, Some(&baked_v1)).0;
    assert!(material_epoch > transform_epoch);

    mesh.mobility = Mobility::Dynamic;
    mesh.static_state.transform_static = false;
    meshes[0] = mesh.clone();
    let (dynamic_epoch, dynamic_mask) =
        prepare_epoch_and_mask(state.as_mut(), &extract, &meshes, &lights, Some(&baked_v1));
    assert!(dynamic_epoch > material_epoch);
    assert_eq!(dynamic_mask, HYBRID_GI_SOURCE_DYNAMIC_DELTA);

    mesh.mobility = Mobility::Static;
    mesh.static_state.transform_static = true;
    meshes[0] = mesh.clone();
    let (restored_epoch, restored_mask) =
        prepare_epoch_and_mask(state.as_mut(), &extract, &meshes, &lights, Some(&baked_v1));
    assert!(restored_epoch > dynamic_epoch);
    assert_eq!(restored_mask, initial_mask);

    lights[0].intensity = 4.0;
    let light_epoch =
        prepare_epoch_and_mask(state.as_mut(), &extract, &meshes, &lights, Some(&baked_v1)).0;
    assert!(light_epoch > restored_epoch);

    let baked_v2 = baked_contract(11, mesh.stable_instance_key);
    let generation_epoch =
        prepare_epoch_and_mask(state.as_mut(), &extract, &meshes, &lights, Some(&baked_v2)).0;
    assert!(generation_epoch > light_epoch);

    let removed = state.prepare_frame(HybridGiRuntimePrepareInput::new(
        Some(&extract),
        &[],
        &lights,
        &[],
        &[],
        Some(&baked_v2),
        false,
        None,
        8,
    ));
    assert!(
        removed
            .prepared_frame()
            .expect("resolved settings keep the neutral frame observable")
            .composite_policy
            .participation_epoch()
            > generation_epoch,
        "streaming removal must invalidate participation"
    );
}

#[test]
fn provider_exposes_all_profile_budgets_and_structured_baked_fallback() {
    let mesh = test_mesh(73, Mobility::Static);
    let meshes = vec![mesh.clone()];
    let baked = baked_contract(20, mesh.stable_instance_key);
    let cases = [
        (
            RenderHybridGiProfile::FullyDynamic,
            RenderHybridGiMode::DynamicOnly,
            (96, 192, 96),
            None,
        ),
        (
            RenderHybridGiProfile::IndoorStatic,
            RenderHybridGiMode::BakedStaticDynamic,
            (64, 256, 64),
            Some(&baked),
        ),
        (
            RenderHybridGiProfile::OpenWorld,
            RenderHybridGiMode::BakedStaticDynamic,
            (64, 192, 128),
            Some(&baked),
        ),
        (
            RenderHybridGiProfile::Cinematic,
            RenderHybridGiMode::BakedStaticDynamic,
            (192, 512, 192),
            Some(&baked),
        ),
    ];

    for (profile, expected_mode, expected_budgets, baked_input) in cases {
        let provider = PluginHybridGiRuntimeProvider;
        let mut state = provider.create_state();
        let extract = RenderHybridGiExtract {
            enabled: true,
            profile,
            ..RenderHybridGiExtract::default()
        };
        let output = state.prepare_frame(HybridGiRuntimePrepareInput::new(
            Some(&extract),
            &meshes,
            &[],
            &[],
            &[],
            baked_input,
            false,
            None,
            1,
        ));
        let resolved = output
            .prepared_frame()
            .and_then(|frame| frame.resolved_settings)
            .expect("enabled profile must expose effective settings");
        assert_eq!(resolved.profile, profile);
        assert_eq!(resolved.mode, expected_mode);
        assert_eq!(
            (
                resolved.trace_budget,
                resolved.card_budget,
                resolved.voxel_budget,
            ),
            expected_budgets
        );
        assert_eq!(resolved.fallback_reason, None);
        assert_eq!(
            state
                .update_after_render(HybridGiRuntimeFeedback::default())
                .stats()
                .resolved_settings(),
            Some(resolved),
            "provider runtime stats must expose the exact effective settings retained by the prepared frame"
        );
        let voxel_clipmaps = &output
            .renderer_outputs()
            .hybrid_gi
            .scene_prepare
            .voxel_clipmaps;
        assert!(voxel_clipmaps.len() <= 8);
        assert!(voxel_clipmaps.iter().all(|clipmap| {
            f32::from_bits(clipmap.center_x_bits).is_finite()
                && f32::from_bits(clipmap.center_y_bits).is_finite()
                && f32::from_bits(clipmap.center_z_bits).is_finite()
                && f32::from_bits(clipmap.half_extent_bits).is_finite()
        }));
    }

    let provider = PluginHybridGiRuntimeProvider;
    let mut state = provider.create_state();
    let extract = RenderHybridGiExtract {
        enabled: true,
        profile: RenderHybridGiProfile::IndoorStatic,
        ..RenderHybridGiExtract::default()
    };
    let output = state.prepare_frame(HybridGiRuntimePrepareInput::new(
        Some(&extract),
        &meshes,
        &[],
        &[],
        &[],
        None,
        false,
        None,
        2,
    ));
    let resolved = output
        .prepared_frame()
        .and_then(|frame| frame.resolved_settings)
        .expect("fallback settings must remain observable");
    assert_eq!(resolved.mode, RenderHybridGiMode::DynamicOnly);
    assert_eq!(
        resolved.fallback_reason,
        Some(RenderHybridGiFallbackReason::BakedLightingUnavailable)
    );
    assert_eq!(
        state
            .update_after_render(HybridGiRuntimeFeedback::default())
            .stats()
            .resolved_settings(),
        Some(resolved),
        "structured fallback must survive the provider update boundary"
    );
}

fn prepare_epoch_and_mask(
    state: &mut dyn zircon_runtime::graphics::HybridGiRuntimeState,
    extract: &RenderHybridGiExtract,
    meshes: &[RenderMeshSnapshot],
    directional_lights: &[RenderDirectionalLightSnapshot],
    baked: Option<&LightmapConsumeContract>,
) -> (u64, u32) {
    let output = state.prepare_frame(HybridGiRuntimePrepareInput::new(
        Some(extract),
        meshes,
        directional_lights,
        &[],
        &[],
        baked,
        false,
        None,
        1,
    ));
    let frame = output
        .prepared_frame()
        .expect("enabled HybridGI scene must produce a neutral prepared frame");
    let probe = frame
        .resident_probes
        .first()
        .expect("test mesh must derive one screen probe");
    (
        frame.composite_policy.participation_epoch(),
        probe.source_mask,
    )
}

fn test_mesh(node_id: u64, mobility: Mobility) -> RenderMeshSnapshot {
    let transform = Transform::default();
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: render_mesh_stable_instance_key(node_id, 0),
        transform_revision: render_mesh_transform_revision(&transform),
        transform,
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
            "res://hybrid-gi/m4/invalidation-model.obj",
        )),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
            "res://hybrid-gi/m4/invalidation-material.mat",
        )),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility,
        static_state: RenderMeshStaticState::new(mobility == Mobility::Static, 1, 1),
        render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
    }
}

fn baked_contract(generation: u64, stable_instance_key: u64) -> LightmapConsumeContract {
    LightmapConsumeContract::new(
        generation,
        ResourceId::from_stable_label(&format!(
            "res://hybrid-gi/m4/lightmap-generation-{generation}"
        )),
        LightmapAtlasDescriptor {
            page_size: 4,
            page_count: 1,
            format: LightmapAtlasFormat::Rgba16Float,
        },
        vec![(
            stable_instance_key,
            LightmapInstanceSlot {
                atlas_page: 0,
                uv_rect: Vec4::new(1.0, 1.0, 0.0, 0.0),
            },
        )],
    )
}

fn test_directional_light(
    light_id: u64,
    mobility: Mobility,
    intensity: f32,
) -> RenderDirectionalLightSnapshot {
    RenderDirectionalLightSnapshot {
        node_id: light_id,
        light_id,
        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
        direction: Vec3::NEG_Y,
        color: Vec3::ONE,
        intensity,
        mobility,
        shadow: None,
    }
}
