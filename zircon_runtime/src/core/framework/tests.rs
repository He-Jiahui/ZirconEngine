use std::time::Duration;

use crate::core::math::{UVec2, Vec2, Vec3, Vec4};
use crate::core::resource::{
    MaterialMarker, MeshMarker, ModelMarker, ResourceHandle, ResourceId, TextureMarker,
};

#[cfg(feature = "net-contracts")]
use super::net::{NetEndpoint, NetError, NetPacket, NetSocketId};
#[cfg(feature = "physics-contracts")]
use super::physics::PhysicsSettings;
use super::scene::physics::{PhysicsCombineRule, PhysicsMaterialMetadata};
use super::{
    animation::{AnimationParameterValue, AnimationPlaybackSettings, AnimationTrackPath},
    input::{InputButton, InputEvent, InputEventRecord, InputSnapshot},
    render::{
        CameraRenderDescriptor, CapturedFrame, CorePipelineKind, FallbackSkyboxKind,
        FrameHistoryHandle, GeometryExtract, GeometryPhaseInput, PostProcessEffectKind,
        PostProcessEffectSettings, PostProcessGraphResourceNames, PostProcessGraphValidationError,
        PostProcessPassGraph, PostProcessStackDescriptor, PreviewEnvironmentExtract,
        RenderAmbientLightSnapshot, RenderBloomSettings, RenderCameraOrderAmbiguity,
        RenderCameraOrderInput, RenderCameraTarget, RenderCameraTargetOrderKey,
        RenderCapabilityKind, RenderCapabilityMismatchDetail, RenderDirectionalLightSnapshot,
        RenderDynamicResolutionSettings, RenderFeatureQualitySettings, RenderFrameExtract,
        RenderFrameworkError, RenderHybridGiDebugView, RenderHybridGiExtract,
        RenderHybridGiQuality, RenderLayerSet, RenderMaterialAlphaMode,
        RenderMaterialLightingModel, RenderMeshSnapshot, RenderOverlayExtract, RenderPhase,
        RenderPhaseItem, RenderPhaseMeshSource, RenderPhaseQueue, RenderPhaseSortComponents,
        RenderPhaseSortDecisionField, RenderPhaseSortKey, RenderPhaseSortKeyBreakdown,
        RenderPipelineHandle, RenderPointLightSnapshot, RenderPostProcessEffectStackSettings,
        RenderProductFeature, RenderProductProfile, RenderProfileBundle,
        RenderProfileValidationError, RenderQualityProfile, RenderQueueValue,
        RenderRectLightSnapshot, RenderSceneGeometryExtract, RenderSceneSnapshot,
        RenderSpotLightSnapshot, RenderStats, RenderViewportDescriptor, RenderViewportHandle,
        RenderViewportRect, RenderingBackendInfo, SpriteExtract, SpritePhaseExtractInput,
        ViewportCameraSnapshot, DEFAULT_RENDER_LAYER_MASK,
    },
    scene::{ComponentPropertyPath, EntityPath, LevelSummary, Mobility, WorldHandle},
    tasks::{
        AsyncTaskDescriptor, AsyncTaskHandle, AsyncTaskState, AsyncTaskStatus,
        TaskCancellationPolicy, TaskPollBudget, TaskPoolDescriptor, TaskPoolKind,
        DEFAULT_MAIN_THREAD_POLLS_PER_FRAME,
    },
    time::{Fixed, Real, Time, Virtual},
};

mod framework_surfaces;
mod phase_queue_summary;
mod render_product_surface;

#[test]
fn framework_contract_types_are_constructible() {
    let viewport = RenderViewportHandle::new(7);
    let pipeline = RenderPipelineHandle::new(11);
    let descriptor = RenderViewportDescriptor::new(UVec2::new(320, 240));
    let profile =
        RenderQualityProfile::new("editor-high").with_pipeline_asset(RenderPipelineHandle::new(11));
    let frame = CapturedFrame::new(320, 240, vec![0; 320 * 240 * 4], 3);
    let stats = RenderStats::default();
    let backend = RenderingBackendInfo {
        backend_name: "wgpu".into(),
        supports_runtime_preview: true,
        supports_shared_texture_viewports: true,
    };
    let input = InputSnapshot {
        cursor_position: [12.0, 24.0],
        pressed_buttons: vec![InputButton::MouseLeft],
        wheel_accumulator: 1.0,
    };
    let event = InputEventRecord {
        sequence: 1,
        timestamp_millis: 2,
        event: InputEvent::ButtonPressed(InputButton::MouseRight),
    };
    let entity_path = EntityPath::parse("Root/Hero").unwrap();
    let property_path = ComponentPropertyPath::parse("Transform.translation").unwrap();
    let track_path = AnimationTrackPath::new(entity_path.clone(), property_path.clone());
    let playback = AnimationPlaybackSettings::default();
    #[cfg(feature = "net-contracts")]
    let socket = NetSocketId::new(5);
    #[cfg(feature = "net-contracts")]
    let endpoint = NetEndpoint::new("127.0.0.1", 9000);
    #[cfg(feature = "net-contracts")]
    let packet = NetPacket {
        source: endpoint.clone(),
        payload: vec![1, 2, 3],
    };
    let material = PhysicsMaterialMetadata::default();
    let lighting_model = RenderMaterialLightingModel::Unlit;
    #[cfg(feature = "physics-contracts")]
    let physics = PhysicsSettings::default();
    let level = LevelSummary {
        handle: WorldHandle::new(42),
        entity_count: 5,
        active_camera: Some(4),
    };

    assert_eq!(viewport.raw(), 7);
    assert_eq!(pipeline.raw(), 11);
    assert_eq!(descriptor.size, UVec2::new(320, 240));
    assert_eq!(
        profile.pipeline_override,
        Some(RenderPipelineHandle::new(11))
    );
    assert_eq!(frame.generation, 3);
    assert_eq!(stats.active_viewports, 0);
    assert_eq!(backend.backend_name, "wgpu");
    assert_eq!(input.cursor_position, [12.0, 24.0]);
    assert_eq!(event.sequence, 1);
    assert_eq!(entity_path.to_string(), "Root/Hero");
    assert_eq!(property_path.to_string(), "Transform.translation");
    assert_eq!(track_path.to_string(), "Root/Hero:Transform.translation");
    #[cfg(feature = "net-contracts")]
    {
        assert_eq!(socket.raw(), 5);
        assert_eq!(endpoint.host, "127.0.0.1");
        assert_eq!(endpoint.port, 9000);
        assert_eq!(packet.payload, vec![1, 2, 3]);
        assert_eq!(
            NetError::UnknownSocket { socket },
            NetError::UnknownSocket { socket }
        );
    }
    assert_eq!(
        AnimationParameterValue::Trigger,
        AnimationParameterValue::Trigger
    );
    assert!(playback.enabled && playback.property_tracks);
    assert_eq!(material.friction_combine, PhysicsCombineRule::Average);
    assert!(lighting_model.is_unlit());
    assert_eq!(lighting_model.to_string(), "unlit");
    #[cfg(feature = "physics-contracts")]
    assert_eq!(physics.fixed_hz, 60);
    assert_eq!(level.handle.get(), 42);
    assert_eq!(Mobility::default(), Mobility::Dynamic);
    assert_eq!(Time::<Real>::default().elapsed(), Duration::ZERO);
    assert_eq!(Time::<Virtual>::default().delta(), Duration::ZERO);
    assert_eq!(Time::<Fixed>::default().frame_index(), 0);
    assert_eq!(
        RenderFrameworkError::UnknownPipeline { pipeline: 9 },
        RenderFrameworkError::UnknownPipeline { pipeline: 9 }
    );

    let history = FrameHistoryHandle::new(19);
    assert_eq!(history.raw(), 19);
}

#[test]
fn render_product_pipeline_phase_queue_orders_opaque_mask_and_transparent_for_2d_and_3d() {
    assert_mesh_phase_order(
        CorePipelineKind::Core2d,
        &[
            RenderPhase::Opaque2d,
            RenderPhase::AlphaMask2d,
            RenderPhase::Transparent2d,
        ],
    );
    assert_mesh_phase_order(
        CorePipelineKind::Core3d,
        &[
            RenderPhase::Opaque3d,
            RenderPhase::AlphaMask3d,
            RenderPhase::Transparent3d,
        ],
    );
}

#[test]
fn render_queue_values_select_phase_before_sort_key_order() {
    let geometry = GeometryExtract::from_meshes_and_phase_inputs(
        CorePipelineKind::Core3d,
        Vec::new(),
        vec![
            GeometryPhaseInput::new(10, 0, RenderMaterialAlphaMode::Opaque, 0.0)
                .with_render_queue(2_900),
            GeometryPhaseInput::new(20, 1, RenderMaterialAlphaMode::Blend, 0.0)
                .with_render_queue(2_000),
            GeometryPhaseInput::new(30, 2, RenderMaterialAlphaMode::Blend, 0.0)
                .with_render_queue(-10),
        ],
    );
    let phase_for_mesh = |mesh_index| {
        geometry
            .phase_queue
            .items
            .iter()
            .find(|item| item.mesh_source == RenderPhaseMeshSource::MeshIndex(mesh_index))
            .map(|item| item.phase)
            .unwrap()
    };

    assert_eq!(phase_for_mesh(0), RenderPhase::Transparent3d);
    assert_eq!(phase_for_mesh(1), RenderPhase::Opaque3d);
    assert_eq!(phase_for_mesh(2), RenderPhase::Transparent3d);

    let sprites = SpriteExtract::from_sprites_and_phase_inputs(
        CorePipelineKind::Core2d,
        Vec::new(),
        vec![
            SpritePhaseExtractInput::new(40, 0, RenderMaterialAlphaMode::Opaque, 0, 0.0)
                .with_render_queue(2_900),
            SpritePhaseExtractInput::new(50, 1, RenderMaterialAlphaMode::Blend, 0, 0.0)
                .with_render_queue(2_000),
        ],
    );
    let phase_for_sprite = |sprite_index| {
        sprites
            .phase_queue
            .items
            .iter()
            .find(|item| item.mesh_source == RenderPhaseMeshSource::SpriteIndex(sprite_index))
            .map(|item| item.phase)
            .unwrap()
    };

    assert_eq!(phase_for_sprite(0), RenderPhase::Transparent2d);
    assert_eq!(phase_for_sprite(1), RenderPhase::Opaque2d);
}

#[test]
fn render_phase_queue_order_exposes_submission_phase_precedence() {
    assert_eq!(RenderPhase::Prepass.queue_order(), 0);
    assert_eq!(RenderPhase::Shadow.queue_order(), 1);
    assert_eq!(RenderPhase::Opaque2d.queue_order(), 2);
    assert_eq!(RenderPhase::Opaque3d.queue_order(), 2);
    assert_eq!(RenderPhase::AlphaMask2d.queue_order(), 3);
    assert_eq!(RenderPhase::AlphaMask3d.queue_order(), 3);
    assert_eq!(RenderPhase::Deferred.queue_order(), 4);
    assert_eq!(RenderPhase::Transparent2d.queue_order(), 5);
    assert_eq!(RenderPhase::Transparent3d.queue_order(), 5);
    assert_eq!(RenderPhase::PostProcess.queue_order(), 6);
    assert_eq!(RenderPhase::Ui.queue_order(), 7);
    assert_eq!(RenderPhase::Overlay.queue_order(), 8);
    assert_eq!(RenderPhase::Debug.queue_order(), 9);

    let item = |entity, phase| RenderPhaseItem {
        entity,
        phase,
        sort_key: RenderPhaseSortKey::new(0),
        mesh_source: RenderPhaseMeshSource::MeshIndex(entity as usize),
    };
    let queue = RenderPhaseQueue::new(vec![
        item(40, RenderPhase::Debug),
        item(20, RenderPhase::Shadow),
        item(10, RenderPhase::Prepass),
        item(30, RenderPhase::Ui),
    ]);

    assert_eq!(
        queue
            .items
            .iter()
            .map(|item| item.phase)
            .collect::<Vec<_>>(),
        vec![
            RenderPhase::Prepass,
            RenderPhase::Shadow,
            RenderPhase::Ui,
            RenderPhase::Debug,
        ]
    );

    let ui_key = queue.items[2].ordering_key();
    assert_eq!(ui_key.phase_order, RenderPhase::Ui.queue_order());
    assert_eq!(ui_key.sort_key.raw(), 0);
    assert_eq!(ui_key.raw_sort_key(), 0);
    assert_eq!(ui_key.entity, 30);
}

#[test]
fn render_phase_item_ordering_key_matches_queue_sort_tuple() {
    let item = |entity, phase, sort_key| RenderPhaseItem {
        entity,
        phase,
        sort_key: RenderPhaseSortKey::new(sort_key),
        mesh_source: RenderPhaseMeshSource::MeshIndex(entity as usize),
    };
    let opaque_late_entity = item(20, RenderPhase::Opaque3d, 0);
    let opaque_early_entity = item(10, RenderPhase::Opaque3d, 0);
    let opaque_late_sort_key = item(5, RenderPhase::Opaque3d, 5);
    let shadow_late_sort_key = item(99, RenderPhase::Shadow, 5);

    assert!(opaque_early_entity.ordering_key() < opaque_late_entity.ordering_key());
    assert!(opaque_late_entity.ordering_key() < opaque_late_sort_key.ordering_key());
    assert!(shadow_late_sort_key.ordering_key() < opaque_early_entity.ordering_key());

    let queue = RenderPhaseQueue::new(vec![
        opaque_late_sort_key,
        opaque_late_entity,
        shadow_late_sort_key,
        opaque_early_entity,
    ]);
    assert_eq!(
        queue
            .items
            .iter()
            .map(RenderPhaseItem::ordering_key)
            .collect::<Vec<_>>(),
        vec![
            shadow_late_sort_key.ordering_key(),
            opaque_early_entity.ordering_key(),
            opaque_late_entity.ordering_key(),
            opaque_late_sort_key.ordering_key(),
        ]
    );
}

#[test]
fn render_phase_sort_key_uses_unified_queue_layer_depth_order() {
    let base = RenderPhaseSortComponents::new(10.0, 1).with_queue(RenderQueueValue::GEOMETRY);
    let later_render_queue =
        RenderPhaseSortComponents::new(-100.0, 2).with_queue(RenderQueueValue::GEOMETRY_LAST);
    let later_material_queue = RenderPhaseSortComponents::new(-100.0, 3)
        .with_queue(RenderQueueValue::GEOMETRY.with_material_offset_i32(10));
    let later_layer = RenderPhaseSortComponents::new(-100.0, 4).with_order_in_layer(5);
    let later_ui_z = RenderPhaseSortComponents::new(-100.0, 5)
        .with_queue(RenderQueueValue::OVERLAY)
        .with_ui_z_index(6);

    assert!(
        RenderPhaseSortKey::for_components(RenderPhase::Opaque3d, base)
            < RenderPhaseSortKey::for_components(RenderPhase::Opaque3d, later_render_queue)
    );
    assert!(
        RenderPhaseSortKey::for_components(RenderPhase::Opaque3d, base)
            < RenderPhaseSortKey::for_components(RenderPhase::Opaque3d, later_material_queue)
    );
    assert!(
        RenderPhaseSortKey::for_components(RenderPhase::Opaque2d, base)
            < RenderPhaseSortKey::for_components(RenderPhase::Opaque2d, later_layer)
    );
    assert!(
        RenderPhaseSortKey::for_components(
            RenderPhase::Ui,
            base.with_queue(RenderQueueValue::OVERLAY)
        ) < RenderPhaseSortKey::for_components(RenderPhase::Ui, later_ui_z)
    );

    let transparent_far = RenderPhaseSortComponents::new(100.0, 6)
        .with_queue(RenderQueueValue::TRANSPARENT)
        .with_depth_bias(0.5);
    let transparent_near =
        RenderPhaseSortComponents::new(1.0, 7).with_queue(RenderQueueValue::TRANSPARENT);
    assert!(
        RenderPhaseSortKey::for_components(RenderPhase::Transparent3d, transparent_far)
            < RenderPhaseSortKey::for_components(RenderPhase::Transparent3d, transparent_near)
    );
}

#[test]
fn render_phase_sort_key_breakdown_explains_depth_and_queue_order() {
    let components = RenderPhaseSortComponents::new(10.25, 42)
        .with_queue(RenderQueueValue::GEOMETRY_LAST)
        .with_queue_offset(-25)
        .with_depth_bias(0.5)
        .with_order_in_layer(7)
        .with_ui_z_index(11);
    let opaque = RenderPhaseSortKey::breakdown(RenderPhase::Opaque3d, components);
    let transparent = RenderPhaseSortKey::breakdown(RenderPhase::Transparent3d, components);
    let non_finite = RenderPhaseSortKey::breakdown(
        RenderPhase::Opaque3d,
        RenderPhaseSortComponents::new(f32::NAN, 77),
    );

    assert_eq!(opaque.phase, RenderPhase::Opaque3d);
    assert_eq!(opaque.camera_order, 0);
    assert_eq!(opaque.camera_order_key, 128);
    assert_eq!(opaque.queue, RenderQueueValue::new(2_475));
    assert_eq!(opaque.queue_key, 2_475);
    assert_eq!(opaque.sorting_layer, 0);
    assert_eq!(opaque.sorting_layer_key, 128);
    assert_eq!(opaque.order_in_layer, 7);
    assert_eq!(opaque.order_in_layer_key, 16_391);
    assert_eq!(opaque.y_sort, None);
    assert_eq!(opaque.y_sort_key, 512);
    assert_eq!(opaque.ui_z_index, 11);
    assert_eq!(opaque.ui_z_index_key, 4_194_315);
    assert_eq!(opaque.entity_tie_breaker, 42);
    assert_eq!(opaque.phase_order, 2);
    assert_eq!(opaque.tie_breaker_key, 42);
    assert_eq!(opaque.effective_depth, 10.75);
    assert_eq!(opaque.depth_key, 10_750);
    assert_eq!(opaque.ordered_depth_key, 10_750);
    assert_eq!(opaque.opaque_depth_key, 86);
    assert_eq!(opaque.transparent_depth_key, 8_377_857);
    assert_eq!(opaque.pipeline_cluster_key, 0);
    assert_eq!(opaque.material_cluster_key, 0);
    assert_eq!(opaque.domain_key, 86);
    assert_eq!(
        opaque.raw_sort_key,
        RenderPhaseSortKey::for_components(RenderPhase::Opaque3d, components).raw()
    );

    assert_eq!(transparent.depth_key, 10_750);
    assert_eq!(transparent.ordered_depth_key, -10_750);
    assert_eq!(
        transparent.domain_key,
        u64::from(transparent.transparent_depth_key) << 10
    );
    assert!(
        transparent.raw_sort_key
            < RenderPhaseSortKey::for_components(
                RenderPhase::Transparent3d,
                RenderPhaseSortComponents::new(1.0, 99)
                    .with_queue(RenderQueueValue::GEOMETRY_LAST)
                    .with_queue_offset(-25)
                    .with_order_in_layer(7)
                    .with_ui_z_index(11),
            )
            .raw()
    );

    assert!(!non_finite.effective_depth.is_finite());
    assert_eq!(non_finite.depth_key, 0);
    assert_eq!(non_finite.ordered_depth_key, 0);
    assert_eq!(non_finite.opaque_depth_key, 0);
}

#[test]
fn render_phase_sort_key_breakdown_reports_first_ordering_difference() {
    let base_components =
        RenderPhaseSortComponents::new(5.0, 10).with_queue(RenderQueueValue::GEOMETRY);
    let phase_decision = RenderPhaseSortKey::breakdown(RenderPhase::Prepass, base_components)
        .first_difference(RenderPhaseSortKey::breakdown(
            RenderPhase::Shadow,
            base_components.with_queue(RenderQueueValue::BACKGROUND),
        ))
        .unwrap();
    assert_eq!(
        phase_decision.field,
        RenderPhaseSortDecisionField::PhaseOrder
    );
    assert!(phase_decision.left_before_right);
    assert_eq!(phase_decision.left_value, 0);
    assert_eq!(phase_decision.right_value, 1);

    let queue_decision = RenderPhaseSortKey::breakdown(RenderPhase::Opaque3d, base_components)
        .first_difference(RenderPhaseSortKey::breakdown(
            RenderPhase::Opaque3d,
            base_components.with_queue_offset(10).with_depth_bias(-10.0),
        ))
        .unwrap();
    assert_eq!(queue_decision.field, RenderPhaseSortDecisionField::Queue);
    assert!(queue_decision.left_before_right);
    assert_eq!(queue_decision.left_value, 2_000);
    assert_eq!(queue_decision.right_value, 2_010);

    let domain_left = RenderPhaseSortKeyBreakdown::from_components_with_clusters(
        RenderPhase::Opaque3d,
        base_components,
        1,
        0,
    );
    let domain_right = RenderPhaseSortKeyBreakdown::from_components_with_clusters(
        RenderPhase::Opaque3d,
        base_components,
        2,
        0,
    );
    let domain_decision = domain_left.first_difference(domain_right).unwrap();
    assert_eq!(domain_decision.field, RenderPhaseSortDecisionField::Domain);
    assert!(domain_decision.left_before_right);
    assert_eq!(domain_decision.left_value, domain_left.domain_key as i64);
    assert_eq!(domain_decision.right_value, domain_right.domain_key as i64);

    let transparent_far = RenderPhaseSortKey::breakdown(
        RenderPhase::Transparent3d,
        RenderPhaseSortComponents::new(100.0, 1).with_queue(RenderQueueValue::TRANSPARENT),
    );
    let transparent_near = RenderPhaseSortKey::breakdown(
        RenderPhase::Transparent3d,
        RenderPhaseSortComponents::new(1.0, 2).with_queue(RenderQueueValue::TRANSPARENT),
    );
    let depth_decision = transparent_far.first_difference(transparent_near).unwrap();
    assert_eq!(depth_decision.field, RenderPhaseSortDecisionField::Domain);
    assert!(depth_decision.left_before_right);
    assert_eq!(depth_decision.left_value, transparent_far.domain_key as i64);
    assert_eq!(
        depth_decision.right_value,
        transparent_near.domain_key as i64
    );

    let entity_key_decision = RenderPhaseSortKey::breakdown(
        RenderPhase::Opaque3d,
        RenderPhaseSortComponents::new(0.0, 2),
    )
    .first_difference(RenderPhaseSortKey::breakdown(
        RenderPhase::Opaque3d,
        RenderPhaseSortComponents::new(0.0, 1),
    ))
    .unwrap();
    assert_eq!(
        entity_key_decision.field,
        RenderPhaseSortDecisionField::TieBreakerKey
    );
    assert!(!entity_key_decision.left_before_right);
    assert_eq!(entity_key_decision.left_value, 2);
    assert_eq!(entity_key_decision.right_value, 1);

    let entity_decision = RenderPhaseSortKey::breakdown(
        RenderPhase::Opaque3d,
        RenderPhaseSortComponents::new(0.0, 65_537),
    )
    .first_difference(RenderPhaseSortKey::breakdown(
        RenderPhase::Opaque3d,
        RenderPhaseSortComponents::new(0.0, 1),
    ))
    .unwrap();
    assert_eq!(
        entity_decision.field,
        RenderPhaseSortDecisionField::EntityTieBreaker
    );
    assert!(!entity_decision.left_before_right);
    assert_eq!(entity_decision.left_value, 65_537);
    assert_eq!(entity_decision.right_value, 1);

    let identical = RenderPhaseSortKey::breakdown(RenderPhase::Opaque3d, base_components);
    assert_eq!(identical.first_difference(identical), None);
}

#[test]
fn geometry_phase_inputs_feed_unified_sort_components_into_queue() {
    let extract = GeometryExtract::from_meshes_and_phase_inputs(
        CorePipelineKind::Core3d,
        Vec::new(),
        vec![
            GeometryPhaseInput::new(30, 0, RenderMaterialAlphaMode::Opaque, 10.0)
                .with_render_queue(2_000)
                .with_material_queue(0),
            GeometryPhaseInput::new(10, 1, RenderMaterialAlphaMode::Opaque, 1.0)
                .with_render_queue(1_000)
                .with_material_queue(50),
            GeometryPhaseInput::new(20, 2, RenderMaterialAlphaMode::Opaque, 0.0)
                .with_render_queue(2_000)
                .with_material_queue(-10)
                .with_order_in_layer(5),
        ],
    );

    assert_eq!(
        extract
            .phase_queue
            .items
            .iter()
            .map(|item| item.mesh_source)
            .collect::<Vec<_>>(),
        vec![
            RenderPhaseMeshSource::MeshIndex(1),
            RenderPhaseMeshSource::MeshIndex(2),
            RenderPhaseMeshSource::MeshIndex(0),
        ]
    );
}

#[test]
fn geometry_extract_builds_static_mesh_batches_by_resource_key() {
    let geometry = GeometryExtract::from_meshes(
        CorePipelineKind::Core3d,
        vec![
            static_batch_test_mesh(
                10,
                "res://models/tree.obj",
                None,
                "res://materials/bark.mat",
            ),
            static_batch_test_mesh(
                20,
                "res://models/tree.obj",
                None,
                "res://materials/bark.mat",
            ),
            {
                let mut mesh = static_batch_test_mesh(
                    30,
                    "res://models/tree.obj",
                    None,
                    "res://materials/bark.mat",
                );
                mesh.mobility = Mobility::Dynamic;
                mesh
            },
            static_batch_test_mesh(
                40,
                "res://models/tree.obj",
                None,
                "res://materials/leaves.mat",
            ),
        ],
    );

    assert_eq!(geometry.static_batches.len(), 1);
    let batch = &geometry.static_batches[0];
    assert_eq!(batch.entities, vec![10, 20]);
    assert_eq!(batch.mesh_indices, vec![0, 1]);
    assert_eq!(batch.instance_count(), 2);
    assert_eq!(
        batch.model,
        ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("res://models/tree.obj"))
    );
    assert_eq!(
        batch.material,
        ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
            "res://materials/bark.mat"
        ))
    );
}

fn static_batch_test_mesh(
    node_id: u64,
    model: &str,
    mesh: Option<&str>,
    material: &str,
) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: node_id << 16,
        transform_revision: 0,
        transform: crate::core::math::Transform::default(),
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(model)),
        mesh: mesh
            .map(|mesh| ResourceHandle::<MeshMarker>::new(ResourceId::from_stable_label(mesh))),
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(material)),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Static,
        static_state: Default::default(),
        common: crate::core::framework::render::RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            is_static: true,
            ..Default::default()
        },
    }
}

fn assert_mesh_phase_order(pipeline: CorePipelineKind, expected: &[RenderPhase; 3]) {
    let queue = GeometryExtract::from_meshes_and_phase_inputs(
        pipeline,
        Vec::new(),
        vec![
            GeometryPhaseInput::new(30, 0, RenderMaterialAlphaMode::Blend, 2.0),
            GeometryPhaseInput::new(10, 1, RenderMaterialAlphaMode::Opaque, 1.0),
            GeometryPhaseInput::new(20, 2, RenderMaterialAlphaMode::Mask { cutoff: 0.5 }, 1.5),
        ],
    )
    .phase_queue;

    assert_eq!(
        queue
            .items
            .iter()
            .map(|item| item.phase)
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(
        queue
            .items
            .iter()
            .map(|item| item.mesh_source)
            .collect::<Vec<_>>(),
        vec![
            RenderPhaseMeshSource::MeshIndex(1),
            RenderPhaseMeshSource::MeshIndex(2),
            RenderPhaseMeshSource::MeshIndex(0),
        ]
    );
}
