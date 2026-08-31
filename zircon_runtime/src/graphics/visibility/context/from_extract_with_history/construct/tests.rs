use crate::core::framework::render::{
    CameraRenderDescriptor, CorePipelineKind, DebugOverlayExtract, EnvironmentExtract,
    GeometryExtract, GeometryPhaseInput, LightShadowSettings, LightingExtract, ParticleExtract,
    PostProcessExtract, RenderCameraOrderInput, RenderCameraTarget, RenderDirectionalLightSnapshot,
    RenderFrameExtract, RenderFrameScenePayload, RenderLayerSet, RenderMaterialAlphaMode,
    RenderMeshSnapshot, RenderMeshStaticState, RenderOverlayExtract, RenderPointLightSnapshot,
    RenderSpotLightSnapshot, RenderViewExtract, RenderWorldSnapshotHandle, ShadowPcfQuality,
    ShadowResolutionTier, SpriteExtract, ViewportCameraSnapshot, sort_render_cameras,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Real, Transform, Vec3, Vec4};
use crate::core::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, TextureMarker,
};
use crate::graphics::visibility::{VisibilityContext, VisibilityViewKey};

#[test]
fn visibility_context_records_relevance_and_filters_main_view_layers() {
    let camera = ViewportCameraSnapshot::default();
    let frame = frame_extract(
        RenderViewExtract::from_camera(camera),
        GeometryExtract::from_meshes_and_phase_inputs(
            CorePipelineKind::Core3d,
            vec![
                mesh_at(1, Vec3::new(0.0, 0.0, -5.0), 1),
                mesh_at(2, Vec3::new(0.0, 0.0, -5.0), 1 << 4),
            ],
            vec![
                GeometryPhaseInput::new(1, 0, RenderMaterialAlphaMode::Opaque, -5.0),
                GeometryPhaseInput::new(2, 1, RenderMaterialAlphaMode::Mask { cutoff: 0.5 }, -5.0),
            ],
        ),
        LightingExtract::default(),
    );

    let context = VisibilityContext::from_extract(&frame);

    assert_eq!(context.main_view_visible_entities(), vec![1]);
    assert_eq!(context.main_view_culled_entities(), vec![2]);
    assert_eq!(
        context
            .main_view_visible_batches()
            .iter()
            .flat_map(|batch| batch.entities.iter().copied())
            .collect::<Vec<_>>(),
        vec![1]
    );
    assert_eq!(context.primitive_relevance.len(), 2);
    assert_eq!(context.frame_visibility.entities, vec![1, 2]);
    assert_eq!(context.frame_visibility.bounds.len(), 2);
    assert_eq!(context.frame_visibility.relevance.len(), 2);
    assert_eq!(
        context.frame_visibility.main_view_visible_entities(),
        vec![1]
    );

    let main_view = context.frame_visibility.main_view().unwrap();
    assert_eq!(main_view.view, VisibilityViewKey::MainCamera);
    assert_eq!(main_view.visible, vec![0]);
    assert_eq!(main_view.stats.input_count, 2);
    assert_eq!(main_view.stats.layer_filtered_count, 1);
    assert_eq!(main_view.stats.frustum_culled_count, 0);
    assert_eq!(main_view.stats.occlusion_culled_count, 0);
    assert_eq!(main_view.stats.visible_count, 1);
    assert!(
        context
            .frame_visibility
            .view(&VisibilityViewKey::ShadowCascade {
                light: 10,
                cascade: 0
            })
            .is_none()
    );

    let visible = context
        .primitive_relevance
        .iter()
        .find(|entry| entry.entity == 1)
        .unwrap()
        .relevance;
    assert!(visible.main_view());
    assert!(visible.depth_prepass());

    let hidden = context
        .primitive_relevance
        .iter()
        .find(|entry| entry.entity == 2)
        .unwrap()
        .relevance;
    assert!(!hidden.main_view());
    assert!(!hidden.depth_prepass());
    assert!(hidden.shadow_caster());
}

#[test]
fn visibility_batch_key_preserves_layers_above_legacy_mask_width() {
    let high_layer = RenderLayerSet::layer(40);
    let frame = frame_from_meshes(vec![mesh_at_layers(
        40,
        Vec3::new(0.0, 0.0, -5.0),
        high_layer.clone(),
    )]);

    let context = VisibilityContext::from_extract(&frame);
    let batch_layers = &context.batches[0].key.render_layer_mask;

    assert!(batch_layers.contains(40));
    assert_eq!(batch_layers.to_scene_schema_v1_mask_lossy(), 0);
    assert!(context.frame_visibility.render_layer_masks[0].contains(40));
    assert_eq!(
        context.frame_visibility.render_layer_masks[0].to_scene_schema_v1_mask_lossy(),
        0
    );
}

#[test]
fn visibility_context_builds_shadow_view_independent_from_main_layers() {
    let camera = ViewportCameraSnapshot::default();
    let frame = frame_extract(
        RenderViewExtract::from_camera(camera),
        GeometryExtract::from_meshes_and_phase_inputs(
            CorePipelineKind::Core3d,
            vec![
                mesh_at(1, Vec3::new(0.0, 0.0, -5.0), 1),
                mesh_at(2, Vec3::new(0.0, 0.0, -5.0), 1 << 4),
            ],
            vec![
                GeometryPhaseInput::new(1, 0, RenderMaterialAlphaMode::Opaque, -5.0),
                GeometryPhaseInput::new(2, 1, RenderMaterialAlphaMode::Mask { cutoff: 0.5 }, -5.0),
            ],
        ),
        LightingExtract {
            directional_lights: vec![RenderDirectionalLightSnapshot {
                node_id: 10,
                light_id: 10,
                layer_mask: RenderLayerSet::from_scene_schema_v1_mask(1),
                direction: Vec3::new(0.0, -1.0, -1.0),
                color: Vec3::ONE,
                intensity: 1.0,
                mobility: crate::core::framework::scene::Mobility::Dynamic,
                shadow: None,
            }],
            ..LightingExtract::default()
        },
    );

    let context = VisibilityContext::from_extract(&frame);

    assert_eq!(context.main_view_visible_entities(), vec![1]);
    let shadow_view = context
        .frame_visibility
        .view(&VisibilityViewKey::ShadowCascade {
            light: 10,
            cascade: 0,
        })
        .unwrap();
    assert_eq!(shadow_view.visible, vec![0, 1]);
    assert_eq!(shadow_view.stats.input_count, 2);
    assert_eq!(shadow_view.stats.layer_filtered_count, 0);
    assert_eq!(shadow_view.stats.frustum_culled_count, 0);
    assert_eq!(shadow_view.stats.occlusion_culled_count, 0);
    assert_eq!(shadow_view.stats.visible_count, 2);
    assert_eq!(
        context
            .frame_visibility
            .shadow_visible_entity_set()
            .into_iter()
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
}

#[test]
fn visibility_context_builds_shadow_views_for_atlas_light_slots() {
    let camera = ViewportCameraSnapshot::default();
    let frame = frame_extract(
        RenderViewExtract::from_camera(camera),
        GeometryExtract::from_meshes_and_phase_inputs(
            CorePipelineKind::Core3d,
            vec![mesh_at(1, Vec3::new(0.0, 0.0, -5.0), 1)],
            vec![GeometryPhaseInput::new(
                1,
                0,
                RenderMaterialAlphaMode::Opaque,
                -5.0,
            )],
        ),
        LightingExtract {
            directional_lights: vec![RenderDirectionalLightSnapshot {
                node_id: 10,
                light_id: 10,
                layer_mask: RenderLayerSet::from_scene_schema_v1_mask(1),
                direction: Vec3::new(0.0, -1.0, -1.0),
                color: Vec3::ONE,
                intensity: 1.0,
                mobility: crate::core::framework::scene::Mobility::Dynamic,
                shadow: Some(shadow_settings()),
            }],
            point_lights: vec![RenderPointLightSnapshot {
                node_id: 20,
                light_id: 20,
                layer_mask: RenderLayerSet::from_scene_schema_v1_mask(1),
                position: Vec3::ZERO,
                color: Vec3::ONE,
                intensity: 1.0,
                range: 8.0,
                mobility: crate::core::framework::scene::Mobility::Dynamic,
                shadow: Some(shadow_settings()),
            }],
            spot_lights: vec![RenderSpotLightSnapshot {
                node_id: 30,
                light_id: 30,
                layer_mask: RenderLayerSet::from_scene_schema_v1_mask(1),
                position: Vec3::ZERO,
                direction: Vec3::new(0.0, 0.0, -1.0),
                color: Vec3::ONE,
                intensity: 1.0,
                range: 8.0,
                inner_angle_radians: 0.25,
                outer_angle_radians: 0.5,
                mobility: crate::core::framework::scene::Mobility::Dynamic,
                shadow: Some(shadow_settings()),
            }],
            ..LightingExtract::default()
        },
    );

    let context = VisibilityContext::from_extract(&frame);

    assert_eq!(context.frame_visibility.shadow_views().count(), 11);
    for cascade in 0..4 {
        assert!(
            context
                .frame_visibility
                .view(&VisibilityViewKey::ShadowCascade { light: 10, cascade })
                .is_some()
        );
    }
    let first_cascade = context
        .frame_visibility
        .view(&VisibilityViewKey::ShadowCascade {
            light: 10,
            cascade: 0,
        })
        .unwrap();
    let last_cascade = context
        .frame_visibility
        .view(&VisibilityViewKey::ShadowCascade {
            light: 10,
            cascade: 3,
        })
        .unwrap();
    assert!(last_cascade.camera.ortho_size > first_cascade.camera.ortho_size);
    assert_ne!(
        last_cascade.camera.transform.translation,
        first_cascade.camera.transform.translation
    );
    for face in 0..6 {
        assert!(
            context
                .frame_visibility
                .view(&VisibilityViewKey::ShadowPointFace { light: 20, face })
                .is_some()
        );
    }
    assert!(
        context
            .frame_visibility
            .view(&VisibilityViewKey::ShadowSpot { light: 30 })
            .is_some()
    );
}

#[test]
fn visibility_context_builds_custom_target_view_from_camera_descriptors() {
    let main_camera = camera_descriptor_with_layers(20, 0, RenderCameraTarget::PrimarySurface);
    let mut custom_camera = camera_descriptor_with_layers(
        10,
        1,
        RenderCameraTarget::Texture(ResourceHandle::<TextureMarker>::new(
            ResourceId::from_stable_label("res://textures/custom-target-camera.png"),
        )),
    );
    custom_camera.camera.transform.translation = Vec3::new(0.0, 0.0, 0.0);

    let scene_camera_order_report = sort_render_cameras([
        RenderCameraOrderInput::from_descriptor(10, custom_camera.clone()),
        RenderCameraOrderInput::from_descriptor(20, main_camera.clone()),
    ]);
    let frame = frame_extract(
        RenderViewExtract::from_camera(main_camera.camera.clone())
            .with_cameras(vec![custom_camera.clone(), main_camera.clone()])
            .with_scene_camera_order_report(20, scene_camera_order_report),
        GeometryExtract::from_meshes_and_phase_inputs(
            CorePipelineKind::Core3d,
            vec![
                mesh_at(1, Vec3::new(0.0, 0.0, -5.0), 1),
                mesh_at(2, Vec3::new(0.0, 0.0, -5.0), 1 << 1),
            ],
            vec![
                GeometryPhaseInput::new(1, 0, RenderMaterialAlphaMode::Opaque, -5.0),
                GeometryPhaseInput::new(2, 1, RenderMaterialAlphaMode::Opaque, -5.0),
            ],
        ),
        LightingExtract::default(),
    );

    let context = VisibilityContext::from_extract(&frame);

    assert_eq!(context.main_view_visible_entities(), vec![1]);
    let custom_view = context
        .frame_visibility
        .view(&VisibilityViewKey::CustomTarget { camera: 10 })
        .expect("texture target scene camera should produce a custom visibility view");
    assert_eq!(custom_view.visible, vec![1]);
    assert_eq!(custom_view.stats.input_count, 2);
    assert_eq!(custom_view.stats.layer_filtered_count, 1);
    assert_eq!(custom_view.stats.frustum_culled_count, 0);
    assert_eq!(custom_view.stats.visible_count, 1);
}

fn camera_descriptor_with_layers(
    entity: crate::core::framework::scene::EntityId,
    layer: crate::core::framework::render::RenderLayer,
    target: RenderCameraTarget,
) -> CameraRenderDescriptor {
    let mut camera = CameraRenderDescriptor::from_camera_payload(
        Some(entity),
        ViewportCameraSnapshot::default(),
    );
    camera.target = target;
    camera.culling_mask = crate::core::framework::render::RenderLayerSet::layer(layer);
    camera.volume_mask = camera.culling_mask.clone();
    camera
}

#[test]
fn visibility_context_reuses_static_index_without_frame_rebuild() {
    let frame = frame_from_meshes(vec![
        mesh_at(1, Vec3::new(0.0, 0.0, -5.0), 1),
        mesh_at(2, Vec3::new(32.0, 0.0, -5.0), 1),
    ]);
    let first = VisibilityContext::from_extract_with_history_and_static_index(&frame, None, None);
    let previous_history = first.history_snapshot.clone();
    let previous_static_index = first.static_index().clone();

    let second = VisibilityContext::from_extract_with_history_and_static_index(
        &frame,
        Some(&previous_history),
        Some(&previous_static_index),
    );

    assert_eq!(first.static_index_report.frame_full_rebuild_count, 1);
    assert_eq!(first.static_index_report.indexed_entity_count, 2);
    assert_eq!(second.static_index_report.frame_full_rebuild_count, 0);
    assert_eq!(second.static_index_report.frame_incremental_update_count, 1);
    assert_eq!(second.static_index_report.indexed_entity_count, 2);
}

#[test]
fn visibility_context_rebuilds_static_index_when_previous_index_is_missing() {
    let frame = frame_from_meshes(vec![
        mesh_at(1, Vec3::new(0.0, 0.0, -5.0), 1),
        mesh_at(2, Vec3::new(32.0, 0.0, -5.0), 1),
    ]);
    let first = VisibilityContext::from_extract_with_history_and_static_index(&frame, None, None);

    let second = VisibilityContext::from_extract_with_history_and_static_index(
        &frame,
        Some(&first.history_snapshot),
        None,
    );

    assert_eq!(second.static_index_report.frame_full_rebuild_count, 1);
    assert_eq!(second.static_index_report.frame_incremental_update_count, 0);
    assert_eq!(second.static_index_report.indexed_entity_count, 2);
}

#[test]
fn visibility_context_uses_static_index_prefilter_above_threshold() {
    let mut meshes = Vec::with_capacity(super::STATIC_INDEX_PREFILTER_MIN_STATIC_INSTANCES + 1);
    meshes.push(mesh_at(1, Vec3::new(0.0, 0.0, -5.0), 1));
    for index in 0..super::STATIC_INDEX_PREFILTER_MIN_STATIC_INSTANCES {
        meshes.push(mesh_at(
            1_000 + index as u64,
            Vec3::new(10_000.0 + index as Real * 32.0, 0.0, -5.0),
            1,
        ));
    }
    let frame = frame_from_meshes(meshes);

    let context = VisibilityContext::from_extract_with_history_and_static_index(&frame, None, None);

    assert!(context.static_index_report.main_view_prefilter_used);
    assert_eq!(
        context.static_index_report.main_view_static_input_count,
        super::STATIC_INDEX_PREFILTER_MIN_STATIC_INSTANCES + 1
    );
    assert!(
        context.static_index_report.main_view_static_candidate_count
            < context.static_index_report.main_view_static_input_count
    );
    assert_eq!(context.main_view_visible_entities(), vec![1]);
}

#[test]
fn visibility_context_skips_static_prefilter_when_camera_bounds_exceed_cell_budget() {
    let mut meshes = Vec::with_capacity(super::STATIC_INDEX_PREFILTER_MIN_STATIC_INSTANCES + 1);
    meshes.push(mesh_at(1, Vec3::new(0.0, 0.0, -5.0), 1));
    for index in 0..super::STATIC_INDEX_PREFILTER_MIN_STATIC_INSTANCES {
        meshes.push(mesh_at(
            1_000 + index as u64,
            Vec3::new(10_000.0 + index as Real * 32.0, 0.0, -5.0),
            1,
        ));
    }
    let mut frame = frame_from_meshes(meshes);
    frame.view.camera.z_far = 1_000.0;

    let context = VisibilityContext::from_extract_with_history_and_static_index(&frame, None, None);

    assert!(!context.static_index_report.main_view_prefilter_used);
    assert_eq!(
        context.static_index_report.main_view_static_candidate_count,
        super::STATIC_INDEX_PREFILTER_MIN_STATIC_INSTANCES + 1
    );
    assert_eq!(context.main_view_visible_entities(), vec![1]);
}

fn frame_from_meshes(meshes: Vec<RenderMeshSnapshot>) -> RenderFrameExtract {
    let phase_inputs = meshes
        .iter()
        .enumerate()
        .map(|(index, mesh)| {
            GeometryPhaseInput::new(mesh.node_id, index, RenderMaterialAlphaMode::Opaque, -5.0)
        })
        .collect::<Vec<_>>();

    frame_extract(
        RenderViewExtract::from_camera(ViewportCameraSnapshot::default()),
        GeometryExtract::from_meshes_and_phase_inputs(
            CorePipelineKind::Core3d,
            meshes,
            phase_inputs,
        ),
        LightingExtract::default(),
    )
}

fn frame_extract(
    view: RenderViewExtract,
    geometry: GeometryExtract,
    lighting: LightingExtract,
) -> RenderFrameExtract {
    let scene = RenderFrameScenePayload::new(
        RenderWorldSnapshotHandle::new(1),
        geometry,
        Vec::new(),
        lighting,
        EnvironmentExtract::default(),
        PostProcessExtract::default(),
        DebugOverlayExtract {
            overlays: RenderOverlayExtract::default(),
        },
        SpriteExtract::default(),
        ParticleExtract::default(),
        Default::default(),
    );
    RenderFrameExtract::new(scene, view, Default::default())
}

fn mesh_at(node_id: u64, translation: Vec3, legacy_layer_bits: u32) -> RenderMeshSnapshot {
    mesh_at_layers(
        node_id,
        translation,
        RenderLayerSet::from_scene_schema_v1_mask(legacy_layer_bits),
    )
}

fn mesh_at_layers(
    node_id: u64,
    translation: Vec3,
    render_layer_mask: RenderLayerSet,
) -> RenderMeshSnapshot {
    let mut transform = Transform::default();
    transform.translation = translation;

    RenderMeshSnapshot {
        node_id,
        stable_instance_key: node_id << 16,
        transform_revision: 0,
        transform,
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("tests/model")),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
            "tests/material",
        )),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Static,
        static_state: RenderMeshStaticState::default(),
        common: crate::core::framework::render::RendererCommon {
            layer_mask: render_layer_mask,
            is_static: true,
            ..Default::default()
        },
    }
}

fn shadow_settings() -> LightShadowSettings {
    LightShadowSettings {
        casts_shadow: true,
        depth_bias: 0.0,
        normal_bias: 0.0,
        strength: 1.0,
        resolution_preference: ShadowResolutionTier::T512,
        pcf_quality: ShadowPcfQuality::High,
    }
}

#[test]
fn visibility_context_borrows_history_entries_for_bvh_plan() {
    let source = include_str!("../construct.rs");

    assert!(!source.contains(concat!("history_entries", ".clone()")));
}
