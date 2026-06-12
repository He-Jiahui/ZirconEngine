use std::time::Duration;

use crate::core::math::{UVec2, Vec2, Vec3, Vec4};
use crate::core::resource::{
    MaterialMarker, MeshMarker, ModelMarker, ResourceHandle, ResourceId, TextureMarker,
};

use super::{
    animation::{AnimationParameterValue, AnimationPlaybackSettings, AnimationTrackPath},
    input::{InputButton, InputEvent, InputEventRecord, InputSnapshot},
    net::{NetEndpoint, NetError, NetPacket, NetSocketId},
    physics::{PhysicsCombineRule, PhysicsMaterialMetadata, PhysicsSettings},
    render::{
        CapturedFrame, CorePipelineKind, FallbackSkyboxKind, FrameHistoryHandle, GeometryExtract,
        GeometryPhaseInput, PostProcessEffectKind, PostProcessEffectSettings,
        PostProcessGraphResourceNames, PostProcessGraphValidationError, PostProcessPassGraph,
        PostProcessStackDescriptor, PreviewEnvironmentExtract, RenderAmbientLightSnapshot,
        RenderBloomSettings, RenderCameraOrderAmbiguity, RenderCameraOrderInput,
        RenderCameraTarget, RenderCameraTargetOrderKey, RenderCapabilityKind,
        RenderCapabilityMismatchDetail, RenderDirectionalLightSnapshot,
        RenderDynamicResolutionSettings, RenderFeatureQualitySettings, RenderFrameExtract,
        RenderFrameworkError, RenderHybridGiDebugView, RenderHybridGiExtract,
        RenderHybridGiQuality, RenderLayerSet, RenderMaterialAlphaMode,
        RenderMaterialLightingModel, RenderMeshSnapshot, RenderOverlayExtract, RenderPhase,
        RenderPhaseItem, RenderPhaseMeshSource, RenderPhaseQueue, RenderPhaseSortComponents,
        RenderPhaseSortDecisionField, RenderPhaseSortKey, RenderPipelineHandle,
        RenderPointLightSnapshot, RenderPostProcessEffectStackSettings, RenderProductFeature,
        RenderProductProfile, RenderProfileBundle, RenderProfileValidationError,
        RenderQualityProfile, RenderRectLightSnapshot, RenderSceneGeometryExtract,
        RenderSceneSnapshot, RenderSpotLightSnapshot, RenderStats, RenderViewportDescriptor,
        RenderViewportHandle, RenderViewportRect, RenderingBackendInfo, ViewportCameraSnapshot,
    },
    scene::{ComponentPropertyPath, EntityPath, LevelSummary, Mobility, WorldHandle},
    tasks::{
        AsyncTaskDescriptor, AsyncTaskHandle, AsyncTaskState, AsyncTaskStatus,
        TaskCancellationPolicy, TaskPollBudget, TaskPoolDescriptor, TaskPoolKind,
        DEFAULT_MAIN_THREAD_POLLS_PER_FRAME,
    },
    time::{Fixed, Real, Time, Virtual},
};

mod phase_queue_summary;

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
    let socket = NetSocketId::new(5);
    let endpoint = NetEndpoint::new("127.0.0.1", 9000);
    let packet = NetPacket {
        source: endpoint.clone(),
        payload: vec![1, 2, 3],
    };
    let material = PhysicsMaterialMetadata::default();
    let lighting_model = RenderMaterialLightingModel::Unlit;
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
    assert_eq!(socket.raw(), 5);
    assert_eq!(endpoint.host, "127.0.0.1");
    assert_eq!(endpoint.port, 9000);
    assert_eq!(packet.payload, vec![1, 2, 3]);
    assert_eq!(
        NetError::UnknownSocket { socket },
        NetError::UnknownSocket { socket }
    );
    assert_eq!(
        AnimationParameterValue::Trigger,
        AnimationParameterValue::Trigger
    );
    assert!(playback.enabled && playback.property_tracks);
    assert_eq!(material.friction_combine, PhysicsCombineRule::Average);
    assert!(lighting_model.is_unlit());
    assert_eq!(lighting_model.to_string(), "unlit");
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
    let base = RenderPhaseSortComponents::new(10.0, 1)
        .with_render_queue(2_000)
        .with_material_queue(0);
    let later_render_queue = RenderPhaseSortComponents::new(-100.0, 2)
        .with_render_queue(2_500)
        .with_material_queue(0);
    let later_material_queue = RenderPhaseSortComponents::new(-100.0, 3)
        .with_render_queue(2_000)
        .with_material_queue(10);
    let later_layer = RenderPhaseSortComponents::new(-100.0, 4)
        .with_render_queue(2_000)
        .with_material_queue(0)
        .with_order_in_layer(5);
    let later_ui_z = RenderPhaseSortComponents::new(-100.0, 5)
        .with_render_queue(2_000)
        .with_material_queue(0)
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
        RenderPhaseSortKey::for_components(RenderPhase::Opaque3d, base)
            < RenderPhaseSortKey::for_components(RenderPhase::Opaque3d, later_layer)
    );
    assert!(
        RenderPhaseSortKey::for_components(RenderPhase::Ui, base)
            < RenderPhaseSortKey::for_components(RenderPhase::Ui, later_ui_z)
    );

    let transparent_far = RenderPhaseSortComponents::new(100.0, 6)
        .with_render_queue(3_000)
        .with_depth_bias(0.5);
    let transparent_near = RenderPhaseSortComponents::new(1.0, 7).with_render_queue(3_000);
    assert!(
        RenderPhaseSortKey::for_components(RenderPhase::Transparent3d, transparent_far)
            < RenderPhaseSortKey::for_components(RenderPhase::Transparent3d, transparent_near)
    );
}

#[test]
fn render_phase_sort_key_breakdown_explains_depth_and_queue_order() {
    let components = RenderPhaseSortComponents::new(10.25, 42)
        .with_render_queue(2_500)
        .with_material_queue(-25)
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
    assert_eq!(opaque.render_queue, 2_500);
    assert_eq!(opaque.render_queue_sort_key, 18_884);
    assert_eq!(opaque.material_queue, -25);
    assert_eq!(opaque.material_queue_sort_key, 16_359);
    assert_eq!(opaque.order_in_layer, 7);
    assert_eq!(opaque.order_in_layer_sort_key, 4_194_311);
    assert_eq!(opaque.ui_z_index, 11);
    assert_eq!(opaque.ui_z_index_sort_key, 4_194_315);
    assert_eq!(opaque.entity_tie_breaker, 42);
    assert_eq!(opaque.phase_order, 2);
    assert_eq!(opaque.entity_tie_breaker_key, 42);
    assert_eq!(opaque.entity_tie_breaker_sort_key, 42);
    assert_eq!(opaque.effective_depth, 10.75);
    assert_eq!(opaque.depth_key, 10_750);
    assert_eq!(opaque.ordered_depth_key, 10_750);
    assert_eq!(opaque.ordered_depth_sort_key, 17_179_879_934);
    assert!(!opaque.transparent_back_to_front);
    assert_eq!(
        opaque.raw_sort_key,
        RenderPhaseSortKey::for_components(RenderPhase::Opaque3d, components).raw()
    );

    assert!(transparent.transparent_back_to_front);
    assert_eq!(transparent.depth_key, 10_750);
    assert_eq!(transparent.ordered_depth_key, -10_750);
    assert_eq!(transparent.ordered_depth_sort_key, 17_179_858_434);
    assert!(
        transparent.raw_sort_key
            < RenderPhaseSortKey::for_components(
                RenderPhase::Transparent3d,
                RenderPhaseSortComponents::new(1.0, 99)
                    .with_render_queue(2_500)
                    .with_material_queue(-25)
                    .with_order_in_layer(7)
                    .with_ui_z_index(11),
            )
            .raw()
    );

    assert!(!non_finite.effective_depth.is_finite());
    assert_eq!(non_finite.depth_key, 0);
    assert_eq!(non_finite.ordered_depth_key, 0);
    assert_eq!(non_finite.ordered_depth_sort_key, 17_179_869_184);
}

#[test]
fn render_phase_sort_key_breakdown_reports_first_ordering_difference() {
    let base_components = RenderPhaseSortComponents::new(5.0, 10)
        .with_render_queue(2_000)
        .with_material_queue(0);
    let phase_decision = RenderPhaseSortKey::breakdown(RenderPhase::Prepass, base_components)
        .first_difference(RenderPhaseSortKey::breakdown(
            RenderPhase::Shadow,
            base_components.with_render_queue(-2_000),
        ))
        .unwrap();
    assert_eq!(
        phase_decision.field,
        RenderPhaseSortDecisionField::PhaseOrder
    );
    assert!(phase_decision.left_before_right);
    assert_eq!(phase_decision.left_value, 0);
    assert_eq!(phase_decision.right_value, 1);

    let material_decision = RenderPhaseSortKey::breakdown(RenderPhase::Opaque3d, base_components)
        .first_difference(RenderPhaseSortKey::breakdown(
            RenderPhase::Opaque3d,
            base_components
                .with_material_queue(10)
                .with_depth_bias(-10.0),
        ))
        .unwrap();
    assert_eq!(
        material_decision.field,
        RenderPhaseSortDecisionField::MaterialQueue
    );
    assert!(material_decision.left_before_right);
    assert_eq!(material_decision.left_value, 0);
    assert_eq!(material_decision.right_value, 10);

    let saturated_queue_left = RenderPhaseSortKey::breakdown(
        RenderPhase::Opaque3d,
        RenderPhaseSortComponents::new(0.0, 1)
            .with_render_queue(20_000)
            .with_material_queue(0),
    );
    let saturated_queue_right = RenderPhaseSortKey::breakdown(
        RenderPhase::Opaque3d,
        RenderPhaseSortComponents::new(0.0, 1)
            .with_render_queue(16_383)
            .with_material_queue(5),
    );
    assert_eq!(saturated_queue_left.render_queue, 20_000);
    assert_eq!(saturated_queue_right.render_queue, 16_383);
    assert_eq!(
        saturated_queue_left.render_queue_sort_key,
        saturated_queue_right.render_queue_sort_key
    );
    let saturated_queue_decision = saturated_queue_left
        .first_difference(saturated_queue_right)
        .unwrap();
    assert_eq!(
        saturated_queue_decision.field,
        RenderPhaseSortDecisionField::MaterialQueue
    );
    assert!(saturated_queue_decision.left_before_right);
    assert_eq!(saturated_queue_decision.left_value, 0);
    assert_eq!(saturated_queue_decision.right_value, 5);

    let transparent_far = RenderPhaseSortKey::breakdown(
        RenderPhase::Transparent3d,
        RenderPhaseSortComponents::new(100.0, 1).with_render_queue(3_000),
    );
    let transparent_near = RenderPhaseSortKey::breakdown(
        RenderPhase::Transparent3d,
        RenderPhaseSortComponents::new(1.0, 2).with_render_queue(3_000),
    );
    let depth_decision = transparent_far.first_difference(transparent_near).unwrap();
    assert_eq!(
        depth_decision.field,
        RenderPhaseSortDecisionField::OrderedDepthKey
    );
    assert!(depth_decision.left_before_right);
    assert_eq!(depth_decision.left_value, -100_000);
    assert_eq!(depth_decision.right_value, -1_000);

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
        RenderPhaseSortDecisionField::EntityTieBreakerKey
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
        render_layer_mask: u32::MAX,
    }
}

#[test]
fn render_product_pipeline_camera_projection_selects_core_pipeline_kind() {
    let perspective = ViewportCameraSnapshot::default();
    assert_eq!(perspective.core_pipeline_kind(), CorePipelineKind::Core3d);

    let orthographic = ViewportCameraSnapshot {
        projection_mode: super::render::ProjectionMode::Orthographic,
        ..ViewportCameraSnapshot::default()
    };
    assert_eq!(orthographic.core_pipeline_kind(), CorePipelineKind::Core2d);
}

#[test]
fn render_product_post_process_graph_elides_disabled_effects() {
    let stack = PostProcessStackDescriptor::default();

    let graph = PostProcessPassGraph::validate_stack(&stack).unwrap();

    assert_eq!(graph.node_count(), 1);
    assert_eq!(graph.skipped_node_count(), 3);
    assert_eq!(
        graph.final_composite_node.as_deref(),
        Some("final-composite")
    );
}

#[test]
fn render_product_post_process_stack_elides_history_until_history_is_available() {
    let stack = PostProcessStackDescriptor::from_extract_settings(
        &Default::default(),
        &Default::default(),
        true,
        false,
    );

    let graph = PostProcessPassGraph::validate_stack(&stack).unwrap();

    assert_eq!(graph.node_count(), 1);
    assert!(!graph
        .nodes
        .iter()
        .any(|node| node.kind == PostProcessEffectKind::HistoryResolve));
}

#[test]
fn render_product_post_process_stack_can_drop_history_from_validated_graph() {
    let stack = PostProcessStackDescriptor::from_extract_settings(
        &Default::default(),
        &Default::default(),
        true,
        true,
    );

    let stack = stack.without_history_resources();
    let graph = PostProcessPassGraph::validate_stack(&stack).unwrap();

    assert_eq!(graph.node_count(), 1);
    assert!(!graph
        .nodes
        .iter()
        .any(|node| node.kind == PostProcessEffectKind::HistoryResolve));
    assert!(graph
        .skipped_nodes
        .iter()
        .any(|node| node.kind == PostProcessEffectKind::HistoryResolve));
}

#[test]
fn render_product_post_process_stack_splits_history_previous_and_output_slots() {
    let stack = PostProcessStackDescriptor::from_extract_settings(
        &Default::default(),
        &Default::default(),
        true,
        true,
    );

    let graph = PostProcessPassGraph::validate_stack(&stack).unwrap();
    let history = graph
        .nodes
        .iter()
        .find(|node| node.kind == PostProcessEffectKind::HistoryResolve)
        .expect("history resolve should be executable when history is available");

    assert!(stack
        .initial_resources
        .contains(&PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCENE_COLOR.to_string()));
    assert!(history
        .required_inputs
        .contains(&PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCENE_COLOR.to_string()));
    assert!(history
        .produced_outputs
        .contains(&PostProcessGraphResourceNames::HISTORY_OUTPUT_SCENE_COLOR.to_string()));
    assert!(!history
        .required_inputs
        .iter()
        .any(|input| history.produced_outputs.contains(input)));
}

#[test]
fn render_product_post_process_graph_rejects_missing_scene_color() {
    let stack = PostProcessStackDescriptor {
        initial_resources: vec![PostProcessGraphResourceNames::SCENE_DEPTH.to_string()],
        effects: vec![
            PostProcessEffectSettings::new(PostProcessEffectKind::FinalComposite)
                .with_required_inputs([PostProcessGraphResourceNames::SCENE_COLOR])
                .with_produced_outputs([PostProcessGraphResourceNames::FINAL_COLOR]),
        ],
    };

    assert_eq!(
        PostProcessPassGraph::validate_stack(&stack),
        Err(PostProcessGraphValidationError::MissingRequiredInput {
            node: "final-composite".to_string(),
            resource: PostProcessGraphResourceNames::SCENE_COLOR.to_string(),
        })
    );
}

#[test]
fn render_product_post_process_graph_rejects_invalid_history_dependency() {
    let stack = PostProcessStackDescriptor {
        initial_resources: vec![PostProcessGraphResourceNames::SCENE_COLOR.to_string()],
        effects: vec![
            PostProcessEffectSettings::new(PostProcessEffectKind::HistoryResolve)
                .with_required_inputs([
                    PostProcessGraphResourceNames::SCENE_COLOR,
                    PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCENE_COLOR,
                ])
                .with_produced_outputs([PostProcessGraphResourceNames::HISTORY_OUTPUT_SCENE_COLOR]),
        ],
    };

    assert_eq!(
        PostProcessPassGraph::validate_stack(&stack),
        Err(PostProcessGraphValidationError::MissingRequiredInput {
            node: "history-resolve".to_string(),
            resource: PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCENE_COLOR.to_string(),
        })
    );
}

#[test]
fn render_product_post_process_graph_rejects_duplicate_output_resource() {
    let stack = PostProcessStackDescriptor {
        initial_resources: vec![PostProcessGraphResourceNames::SCENE_COLOR.to_string()],
        effects: vec![
            PostProcessEffectSettings::new(PostProcessEffectKind::Bloom)
                .with_required_inputs([PostProcessGraphResourceNames::SCENE_COLOR])
                .with_produced_outputs([PostProcessGraphResourceNames::BLOOM]),
            PostProcessEffectSettings::new(PostProcessEffectKind::ColorGrading)
                .with_required_inputs([PostProcessGraphResourceNames::SCENE_COLOR])
                .with_produced_outputs([PostProcessGraphResourceNames::BLOOM]),
        ],
    };

    assert_eq!(
        PostProcessPassGraph::validate_stack(&stack),
        Err(PostProcessGraphValidationError::DuplicateOutputResource {
            node: "color-grading".to_string(),
            resource: PostProcessGraphResourceNames::BLOOM.to_string(),
        })
    );
}

#[test]
fn render_product_post_process_graph_rejects_cycles() {
    let stack = PostProcessStackDescriptor {
        initial_resources: vec![PostProcessGraphResourceNames::SCENE_COLOR.to_string()],
        effects: vec![
            PostProcessEffectSettings::new(PostProcessEffectKind::Bloom)
                .with_required_inputs([PostProcessGraphResourceNames::SCENE_COLOR])
                .with_produced_outputs([PostProcessGraphResourceNames::BLOOM])
                .with_after([PostProcessEffectKind::ColorGrading]),
            PostProcessEffectSettings::new(PostProcessEffectKind::ColorGrading)
                .with_required_inputs([PostProcessGraphResourceNames::SCENE_COLOR])
                .with_produced_outputs([PostProcessGraphResourceNames::COLOR_GRADED])
                .with_after([PostProcessEffectKind::Bloom]),
        ],
    };

    assert_eq!(
        PostProcessPassGraph::validate_stack(&stack),
        Err(PostProcessGraphValidationError::CycleDetected)
    );
}

#[test]
fn render_product_post_process_graph_rejects_missing_effect_dependency() {
    let stack = PostProcessStackDescriptor {
        initial_resources: vec![PostProcessGraphResourceNames::SCENE_COLOR.to_string()],
        effects: vec![
            PostProcessEffectSettings::new(PostProcessEffectKind::FinalComposite)
                .with_required_inputs([PostProcessGraphResourceNames::SCENE_COLOR])
                .with_produced_outputs([PostProcessGraphResourceNames::FINAL_COLOR])
                .with_after([PostProcessEffectKind::Bloom]),
        ],
    };

    assert_eq!(
        PostProcessPassGraph::validate_stack(&stack),
        Err(PostProcessGraphValidationError::MissingDependency {
            node: "final-composite".to_string(),
            dependency: PostProcessEffectKind::Bloom,
        })
    );
}

#[test]
fn render_product_post_process_graph_allows_color_grading_without_bloom() {
    let stack = PostProcessStackDescriptor::from_extract_settings(
        &Default::default(),
        &super::render::RenderColorGradingSettings {
            exposure: 1.05,
            contrast: 1.0,
            saturation: 1.0,
            gamma: 1.0,
            tint: Vec3::ONE,
        },
        false,
        false,
    );

    let graph = PostProcessPassGraph::validate_stack(&stack).unwrap();

    assert_eq!(
        graph.nodes.iter().map(|node| node.kind).collect::<Vec<_>>(),
        vec![
            PostProcessEffectKind::ColorGrading,
            PostProcessEffectKind::FinalComposite,
        ]
    );
}

#[test]
fn render_product_post_process_effect_stack_runs_before_final_composite_when_authored() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &RenderBloomSettings {
            threshold: 1.0,
            intensity: 0.5,
            radius: 0.25,
        },
        &super::render::RenderColorGradingSettings {
            exposure: 1.05,
            contrast: 1.0,
            saturation: 1.0,
            gamma: 1.0,
            tint: Vec3::ONE,
        },
        &RenderPostProcessEffectStackSettings {
            vignette: super::render::RenderVignetteSettings {
                intensity: 0.35,
                ..Default::default()
            },
            grain: super::render::RenderFilmGrainSettings {
                intensity: 0.2,
                ..Default::default()
            },
            chromatic_aberration: super::render::RenderChromaticAberrationSettings {
                intensity: 0.1,
                ..Default::default()
            },
            fog: super::render::RenderFogSettings {
                density: 0.05,
                color: Vec3::new(0.5, 0.6, 0.7),
                ..Default::default()
            },
            ..Default::default()
        },
        false,
        false,
        &super::render::AntiAliasSettings::off(),
    );

    let graph = PostProcessPassGraph::validate_stack(&stack).unwrap();
    let effect_stack = graph
        .nodes
        .iter()
        .find(|node| node.kind == PostProcessEffectKind::EffectStack)
        .expect("authored effect-stack settings should enable the graph node");
    let final_composite = graph
        .nodes
        .iter()
        .find(|node| node.kind == PostProcessEffectKind::FinalComposite)
        .expect("postprocess graph should still end in final composite");

    assert_eq!(
        graph.nodes.iter().map(|node| node.kind).collect::<Vec<_>>(),
        vec![
            PostProcessEffectKind::Bloom,
            PostProcessEffectKind::ColorGrading,
            PostProcessEffectKind::EffectStack,
            PostProcessEffectKind::FinalComposite,
        ]
    );
    assert!(effect_stack
        .required_inputs
        .contains(&PostProcessGraphResourceNames::BLOOM.to_string()));
    assert!(effect_stack
        .required_inputs
        .contains(&PostProcessGraphResourceNames::COLOR_GRADED.to_string()));
    assert!(effect_stack
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_DEPTH.to_string()));
    assert_eq!(
        effect_stack.produced_outputs,
        vec![PostProcessGraphResourceNames::EFFECT_STACKED.to_string()]
    );
    assert_eq!(
        final_composite.required_inputs,
        vec![PostProcessGraphResourceNames::EFFECT_STACKED.to_string()]
    );
    assert_eq!(
        final_composite.after,
        vec![PostProcessEffectKind::EffectStack]
    );
}

#[test]
fn render_product_post_process_extended_effect_stack_settings_enable_product_node() {
    let stack = PostProcessStackDescriptor::from_extract_settings_with_effect_stack_and_anti_alias(
        &RenderBloomSettings::default(),
        &super::render::RenderColorGradingSettings::default(),
        &RenderPostProcessEffectStackSettings {
            tonemap: super::render::RenderTonemapSettings {
                operator: super::render::RenderTonemapOperator::Filmic,
                ..Default::default()
            },
            dither: super::render::RenderDitherSettings {
                intensity: 0.2,
                ..Default::default()
            },
            screen_space_reflection: super::render::RenderScreenSpaceReflectionSettings {
                intensity: 0.4,
                max_steps: 24,
                ..Default::default()
            },
            ..Default::default()
        },
        false,
        false,
        &super::render::AntiAliasSettings::off(),
    );

    let graph = PostProcessPassGraph::validate_stack(&stack).unwrap();
    let effect_stack = graph
        .nodes
        .iter()
        .find(|node| node.kind == PostProcessEffectKind::EffectStack)
        .expect("SSR settings should enable the effect-stack node");

    assert_eq!(
        graph.nodes.iter().map(|node| node.kind).collect::<Vec<_>>(),
        vec![
            PostProcessEffectKind::EffectStack,
            PostProcessEffectKind::FinalComposite,
        ]
    );
    assert!(effect_stack
        .required_inputs
        .contains(&PostProcessGraphResourceNames::SCENE_DEPTH.to_string()));
}

#[test]
fn render_camera_contracts_cover_viewports_and_bevy_layer_intersection() {
    let viewport = RenderViewportRect::new(UVec2::new(600, 400), UVec2::new(100, 100))
        .clamped_to_size(UVec2::new(640, 480));
    assert_eq!(viewport.physical_position, UVec2::new(600, 400));
    assert_eq!(viewport.physical_size, UVec2::new(40, 80));

    let layers = RenderLayerSet::from_layers([0, 3, 70]);
    assert!(layers.contains(0));
    assert!(layers.contains(3));
    assert!(layers.contains(70));
    assert!(layers.intersects(&RenderLayerSet::layer(70)));
    assert!(!layers.intersects(&RenderLayerSet::layer(4)));
    assert!(!RenderLayerSet::none().intersects(&RenderLayerSet::none()));
    assert_eq!(
        RenderLayerSet::from_legacy_mask(0b1010).to_legacy_mask_lossy(),
        0b1010
    );

    let mut camera = ViewportCameraSnapshot {
        viewport: Some(RenderViewportRect::new(
            UVec2::new(100, 0),
            UVec2::new(320, 160),
        )),
        render_layers: RenderLayerSet::from_layers([3]),
        hdr: true,
        msaa_samples: 4,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(UVec2::new(1920, 1080));

    assert_eq!(camera.aspect_ratio, 2.0);
    assert!(camera.hdr);
    assert_eq!(camera.msaa_samples, 4);
    assert!(camera.render_layers.intersects_legacy_mask(0b1000));
    assert!(!camera.render_layers.intersects_legacy_mask(0b0010));

    camera.dynamic_resolution = RenderDynamicResolutionSettings::fixed_scale(0.5);
    assert_eq!(
        camera.effective_viewport_size(UVec2::new(1920, 1080)),
        UVec2::new(320, 160),
        "dynamic resolution must not change the camera viewport/present size"
    );
    assert_eq!(
        camera.effective_render_size(UVec2::new(1920, 1080)),
        UVec2::new(160, 80),
        "dynamic resolution should scale only the internal render extent"
    );

    camera.dynamic_resolution = RenderDynamicResolutionSettings::fixed_scale(0.0);
    assert_eq!(
        camera.effective_render_size(UVec2::new(1920, 1080)),
        UVec2::new(32, 16),
        "render scale is clamped so graph resources never collapse to zero"
    );

    camera.dynamic_resolution = RenderDynamicResolutionSettings::fixed_scale(f32::NAN);
    assert_eq!(
        camera.effective_render_size(UVec2::new(1920, 1080)),
        UVec2::new(320, 160),
        "non-finite render scale falls back to unscaled viewport size"
    );
}

#[test]
fn render_camera_ordering_sorts_by_order_then_target_and_tracks_target_hdr_index() {
    let texture_a = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "res://textures/camera-a.png",
    ));

    let report = super::render::sort_render_cameras([
        RenderCameraOrderInput::new(
            40,
            camera_order_input(2, RenderCameraTarget::PrimarySurface),
        ),
        RenderCameraOrderInput::new(
            30,
            ViewportCameraSnapshot {
                target: RenderCameraTarget::Texture(texture_a),
                hdr: true,
                ..camera_order_input(2, RenderCameraTarget::PrimarySurface)
            },
        ),
        RenderCameraOrderInput::new(
            10,
            camera_order_input(
                -1,
                RenderCameraTarget::Headless {
                    size: UVec2::new(640, 480),
                },
            ),
        ),
        RenderCameraOrderInput::new(
            20,
            ViewportCameraSnapshot {
                target: RenderCameraTarget::Texture(texture_a),
                hdr: true,
                ..camera_order_input(0, RenderCameraTarget::PrimarySurface)
            },
        ),
        RenderCameraOrderInput::new(
            50,
            camera_order_input(0, RenderCameraTarget::PrimarySurface),
        ),
    ]);

    assert!(!report.has_ambiguities());
    assert_eq!(
        report
            .cameras
            .iter()
            .map(|camera| camera.entity)
            .collect::<Vec<_>>(),
        vec![10, 50, 20, 40, 30]
    );
    assert_eq!(
        report
            .cameras
            .iter()
            .map(|camera| camera.sorted_camera_index_for_target)
            .collect::<Vec<_>>(),
        vec![0, 0, 0, 1, 1]
    );
}

#[test]
fn render_camera_ordering_reports_ambiguities_and_skips_inactive_cameras() {
    let report = super::render::sort_render_cameras([
        RenderCameraOrderInput::new(30, inactive_camera_order_input(1)),
        RenderCameraOrderInput::new(
            20,
            camera_order_input(1, RenderCameraTarget::PrimarySurface),
        ),
        RenderCameraOrderInput::new(
            40,
            camera_order_input(
                1,
                RenderCameraTarget::Headless {
                    size: UVec2::new(320, 240),
                },
            ),
        ),
        RenderCameraOrderInput::new(
            10,
            camera_order_input(1, RenderCameraTarget::PrimarySurface),
        ),
    ]);

    assert_eq!(
        report
            .cameras
            .iter()
            .map(|camera| camera.entity)
            .collect::<Vec<_>>(),
        vec![10, 20, 40]
    );
    assert_eq!(
        report.ambiguities,
        vec![RenderCameraOrderAmbiguity {
            order: 1,
            target: RenderCameraTargetOrderKey::PrimarySurface,
        }]
    );
}

fn camera_order_input(order: i32, target: RenderCameraTarget) -> ViewportCameraSnapshot {
    ViewportCameraSnapshot {
        order,
        target,
        ..ViewportCameraSnapshot::default()
    }
}

fn inactive_camera_order_input(order: i32) -> ViewportCameraSnapshot {
    ViewportCameraSnapshot {
        order,
        is_active: false,
        ..ViewportCameraSnapshot::default()
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

#[test]
fn time_framework_tracks_real_virtual_and_fixed_clocks() {
    let mut real = Time::<Real>::default();
    real.advance_by(Duration::from_millis(16));

    assert_eq!(real.delta(), Duration::from_millis(16));
    assert_eq!(real.elapsed(), Duration::from_millis(16));
    assert_eq!(real.frame_index(), 1);

    let mut virtual_time = Time::<Virtual>::default();
    virtual_time.advance_from_real_delta(Duration::from_millis(500));
    assert_eq!(virtual_time.delta(), Duration::from_millis(250));
    assert_eq!(virtual_time.elapsed(), Duration::from_millis(250));

    virtual_time.set_relative_speed_f64(0.5);
    virtual_time.advance_from_real_delta(Duration::from_millis(100));
    assert_eq!(virtual_time.delta(), Duration::from_millis(50));
    assert_eq!(virtual_time.elapsed(), Duration::from_millis(300));
    assert_eq!(virtual_time.effective_speed_f64(), 0.5);

    virtual_time.pause();
    virtual_time.advance_from_real_delta(Duration::from_millis(100));
    assert_eq!(virtual_time.delta(), Duration::ZERO);
    assert_eq!(virtual_time.elapsed(), Duration::from_millis(300));
    assert!(virtual_time.is_paused());
    assert_eq!(virtual_time.effective_speed_f64(), 0.0);

    let mut fixed = Time::<Fixed>::from_duration(Duration::from_millis(10));
    fixed.accumulate_overstep(Duration::from_millis(35));
    let plan = fixed.drain_steps(3);

    assert_eq!(plan.step_count, 3);
    assert_eq!(plan.consumed, Duration::from_millis(30));
    assert_eq!(plan.remaining_overstep, Duration::from_millis(5));
    assert_eq!(fixed.delta(), Duration::from_millis(10));
    assert_eq!(fixed.elapsed(), Duration::from_millis(30));
    assert_eq!(fixed.frame_index(), 3);
    assert_eq!(fixed.overstep(), Duration::from_millis(5));

    fixed.accumulate_overstep(Duration::from_millis(30));
    let capped = fixed.drain_steps(2);
    assert_eq!(capped.step_count, 2);
    assert_eq!(capped.remaining_overstep, Duration::from_millis(15));
    assert_eq!(fixed.elapsed(), Duration::from_millis(50));
    assert_eq!(fixed.frame_index(), 5);
}

#[test]
fn task_framework_contracts_describe_pools_status_and_poll_budget() {
    let compute = TaskPoolDescriptor::compute().with_worker_threads(0);
    let async_compute = TaskPoolDescriptor::async_compute().with_thread_name("async-streaming");
    let io = TaskPoolDescriptor::io();
    let handle = AsyncTaskHandle::new(42);
    let descriptor = AsyncTaskDescriptor::new(handle, TaskPoolKind::AsyncCompute, "mesh-import")
        .with_cancellation_policy(TaskCancellationPolicy::DetachOnDrop);

    assert_eq!(compute.kind, TaskPoolKind::Compute);
    assert_eq!(compute.worker_threads, Some(1));
    assert_eq!(async_compute.kind, TaskPoolKind::AsyncCompute);
    assert_eq!(async_compute.thread_name, "async-streaming");
    assert_eq!(io.thread_name, TaskPoolKind::Io.default_thread_name());
    assert_eq!(descriptor.handle.raw(), 42);
    assert_eq!(descriptor.pool, TaskPoolKind::AsyncCompute);
    assert_eq!(
        descriptor.cancellation_policy,
        TaskCancellationPolicy::DetachOnDrop
    );

    let mut status = AsyncTaskStatus::pending(handle);
    assert_eq!(status.state, AsyncTaskState::Pending);
    assert!(!status.is_terminal());

    status.mark_running();
    status.record_poll();
    status.record_poll();
    assert_eq!(status.state, AsyncTaskState::Running);
    assert_eq!(status.poll_count, 2);

    status.mark_failed("importer returned no artifact");
    assert_eq!(status.state, AsyncTaskState::Failed);
    assert!(status.is_terminal());
    assert_eq!(
        status.failure_message.as_deref(),
        Some("importer returned no artifact")
    );

    let budget = TaskPollBudget::default();
    assert_eq!(
        budget.remaining_after(40),
        Some(DEFAULT_MAIN_THREAD_POLLS_PER_FRAME - 40)
    );
    assert!(budget.is_exhausted_after(DEFAULT_MAIN_THREAD_POLLS_PER_FRAME));
    assert!(!TaskPollBudget::unlimited().is_exhausted_after(u32::MAX));
}

#[test]
fn task_framework_root_stays_structural_after_folder_split() {
    let tasks_mod = include_str!("tasks/mod.rs");

    for required in [
        "mod async_task_descriptor;",
        "mod async_task_handle;",
        "mod async_task_state;",
        "mod async_task_status;",
        "mod task_cancellation_policy;",
        "mod task_poll_budget;",
        "mod task_pool_descriptor;",
        "mod task_pool_kind;",
        "AsyncTaskDescriptor",
        "AsyncTaskHandle",
        "AsyncTaskState",
        "AsyncTaskStatus",
        "TaskCancellationPolicy",
        "TaskPollBudget",
        "TaskPoolDescriptor",
        "TaskPoolKind",
    ] {
        assert!(
            tasks_mod.contains(required),
            "tasks framework root should keep structural export `{required}`"
        );
    }

    for forbidden in [
        "pub struct AsyncTaskDescriptor",
        "pub struct AsyncTaskStatus",
        "pub struct TaskPoolDescriptor",
        "pub enum AsyncTaskState",
        "pub enum TaskPoolKind",
        "impl AsyncTaskStatus",
        "impl TaskPoolDescriptor",
    ] {
        assert!(
            !tasks_mod.contains(forbidden),
            "tasks framework root should not keep implementation detail `{forbidden}`"
        );
    }
}

#[test]
fn time_framework_root_stays_structural_after_folder_split() {
    let time_mod = include_str!("time/mod.rs");

    for required in [
        "mod clock;",
        "mod fixed;",
        "mod fixed_step_plan;",
        "mod real;",
        "mod virtual_clock;",
        "FixedStepPlan",
        "Fixed",
        "Real",
        "Time",
        "Virtual",
    ] {
        assert!(
            time_mod.contains(required),
            "time framework root should keep structural export `{required}`"
        );
    }

    for forbidden in [
        "pub struct Time",
        "pub struct FixedStepPlan",
        "pub struct Fixed",
        "pub struct Virtual",
        "impl Time<Virtual>",
        "impl Time<Fixed>",
    ] {
        assert!(
            !time_mod.contains(forbidden),
            "time framework root should not keep implementation detail `{forbidden}`"
        );
    }
}

#[test]
fn physics_framework_root_stays_structural_after_folder_split() {
    let physics_mod = include_str!("physics/mod.rs");

    for required in [
        "mod backend_state;",
        "mod backend_status;",
        "mod body_sync_state;",
        "mod body_type;",
        "mod collider_shape;",
        "mod collider_sync_state;",
        "mod combine_rule;",
        "mod contact_event;",
        "mod joint_sync_state;",
        "mod joint_type;",
        "mod manager;",
        "mod material_metadata;",
        "mod material_sync_state;",
        "mod query_filter;",
        "mod ray_cast_hit;",
        "mod ray_cast_query;",
        "mod settings;",
        "mod shape_cast_hit;",
        "mod shape_cast_query;",
        "mod shape_overlap_hit;",
        "mod shape_overlap_query;",
        "mod simulation_mode;",
        "mod trigger_event;",
        "mod trigger_event_kind;",
        "mod world_step_plan;",
        "mod world_sync_state;",
        "PhysicsBackendState",
        "PhysicsBackendStatus",
        "PhysicsBodySyncState",
        "PhysicsBodyType",
        "PhysicsColliderShape",
        "PhysicsColliderSyncState",
        "PhysicsCombineRule",
        "PhysicsContactEvent",
        "PhysicsJointSyncState",
        "PhysicsJointType",
        "PhysicsManager",
        "PhysicsMaterialMetadata",
        "PhysicsMaterialSyncState",
        "PhysicsQueryFilter",
        "PhysicsRayCastHit",
        "PhysicsRayCastQuery",
        "PhysicsSettings",
        "PhysicsShapeCastHit",
        "PhysicsShapeCastQuery",
        "PhysicsShapeOverlapHit",
        "PhysicsShapeOverlapQuery",
        "PhysicsSimulationMode",
        "PhysicsTriggerEvent",
        "PhysicsTriggerEventKind",
        "PhysicsWorldStepPlan",
        "PhysicsWorldSyncState",
    ] {
        assert!(
            physics_mod.contains(required),
            "physics framework root should keep structural export `{required}`"
        );
    }

    for forbidden in [
        "pub enum PhysicsCombineRule",
        "pub struct PhysicsMaterialMetadata",
        "pub struct PhysicsSettings",
        "pub trait PhysicsManager",
        "impl Default for PhysicsSettings",
        "impl Default for PhysicsWorldSyncState",
    ] {
        assert!(
            !physics_mod.contains(forbidden),
            "physics framework root should not keep implementation detail `{forbidden}`"
        );
    }
}

#[test]
fn animation_framework_root_stays_structural_after_folder_split() {
    let animation_mod = include_str!("animation/mod.rs");

    for required in [
        "mod graph_clip_instance;",
        "mod avatar_mask;",
        "mod event;",
        "mod gpu_skinning;",
        "mod graph_blend_mode;",
        "mod graph_evaluation;",
        "mod manager;",
        "mod parameter_map;",
        "mod parameter_value;",
        "mod playback_settings;",
        "mod pose_bone;",
        "mod pose_output;",
        "mod pose_source;",
        "mod sequence_apply_report;",
        "mod state_machine_evaluation;",
        "mod tick;",
        "mod track_path;",
        "mod track_path_error;",
        "AnimationAvatarMask",
        "AnimationEventRecord",
        "AnimationGpuSkinningReadiness",
        "AnimationSkinningBackend",
        "AnimationGraphClipInstance",
        "AnimationGraphBlendMode",
        "AnimationGraphEvaluation",
        "AnimationManager",
        "AnimationParameterMap",
        "AnimationParameterValue",
        "AnimationPlaybackSettings",
        "AnimationPoseBone",
        "AnimationPoseOutput",
        "AnimationPoseSource",
        "AnimationSequenceApplyReport",
        "AnimationStateMachineEvaluation",
        "AnimationTickReport",
        "AnimationTickRequest",
        "AnimationTrackPath",
        "AnimationTrackPathError",
    ] {
        assert!(
            animation_mod.contains(required),
            "animation framework root should keep structural export `{required}`"
        );
    }

    for forbidden in [
        "pub enum AnimationParameterValue",
        "pub struct AnimationTrackPath",
        "pub struct AnimationPlaybackSettings",
        "pub struct AnimationGraphClipInstance",
        "pub trait AnimationManager",
        "impl Default for AnimationPlaybackSettings",
    ] {
        assert!(
            !animation_mod.contains(forbidden),
            "animation framework root should not keep implementation detail `{forbidden}`"
        );
    }
}

#[test]
fn net_framework_root_stays_structural_after_folder_split() {
    let net_mod = include_str!("net/mod.rs");

    for required in [
        "mod endpoint;",
        "mod error;",
        "mod manager;",
        "mod packet;",
        "mod socket_id;",
        "NetEndpoint",
        "NetError",
        "NetManager",
        "NetPacket",
        "NetSocketId",
    ] {
        assert!(
            net_mod.contains(required),
            "net framework root should keep structural export `{required}`"
        );
    }

    for forbidden in [
        "pub struct NetEndpoint",
        "pub enum NetError",
        "pub trait NetManager",
        "pub struct NetPacket",
        "pub struct NetSocketId",
        "impl NetSocketId",
    ] {
        assert!(
            !net_mod.contains(forbidden),
            "net framework root should not keep implementation detail `{forbidden}`"
        );
    }
}

#[test]
fn render_frame_extract_roundtrip_preserves_split_light_lists() {
    let snapshot = RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera: ViewportCameraSnapshot::default(),
            meshes: Vec::new(),
            directional_lights: vec![RenderDirectionalLightSnapshot {
                node_id: 10,
                direction: Vec3::new(-0.4, -1.0, -0.2),
                color: Vec3::new(1.0, 0.9, 0.8),
                intensity: 3.0,
            }],
            point_lights: vec![RenderPointLightSnapshot {
                node_id: 20,
                position: Vec3::new(2.0, 3.0, 4.0),
                color: Vec3::new(0.2, 0.6, 1.0),
                intensity: 4.5,
                range: 9.0,
            }],
            spot_lights: vec![RenderSpotLightSnapshot {
                node_id: 30,
                position: Vec3::new(-1.0, 5.0, 2.0),
                direction: Vec3::new(0.0, -1.0, 0.3),
                color: Vec3::new(1.0, 0.7, 0.2),
                intensity: 8.0,
                range: 14.0,
                inner_angle_radians: 0.25,
                outer_angle_radians: 0.5,
            }],
            ambient_lights: vec![RenderAmbientLightSnapshot {
                color: Vec3::new(0.05, 0.06, 0.07),
                intensity: 0.2,
                renderer_degraded: true,
                degradation_reason: Some(
                    "ambient light renderer path is deferred after M5A".to_string(),
                ),
            }],
            rect_lights: vec![RenderRectLightSnapshot {
                node_id: 40,
                position: Vec3::new(1.0, 2.0, 3.0),
                direction: Vec3::new(0.0, -1.0, 0.0),
                color: Vec3::new(1.0, 0.8, 0.6),
                intensity: 6.0,
                range: 12.0,
                size: Vec2::new(2.0, 0.5),
                renderer_degraded: true,
                degradation_reason: Some(
                    "rect light renderer path is deferred after M5A".to_string(),
                ),
            }],
        },
        overlays: RenderOverlayExtract::default(),
        preview: PreviewEnvironmentExtract {
            lighting_enabled: true,
            skybox_enabled: true,
            fallback_skybox: FallbackSkyboxKind::ProceduralGradient,
            clear_color: Vec4::new(0.1, 0.2, 0.3, 1.0),
        },
        virtual_geometry_debug: None,
    };

    let extract = RenderFrameExtract::from_snapshot(WorldHandle::new(7).into(), snapshot.clone());

    assert_eq!(
        extract.lighting.directional_lights,
        snapshot.scene.directional_lights
    );
    assert_eq!(extract.lighting.point_lights, snapshot.scene.point_lights);
    assert_eq!(extract.lighting.spot_lights, snapshot.scene.spot_lights);
    assert_eq!(
        extract.lighting.ambient_lights,
        snapshot.scene.ambient_lights
    );
    assert_eq!(extract.lighting.rect_lights, snapshot.scene.rect_lights);

    let roundtrip = extract.to_scene_snapshot();
    assert_eq!(
        roundtrip.scene.directional_lights,
        snapshot.scene.directional_lights
    );
    assert_eq!(roundtrip.scene.point_lights, snapshot.scene.point_lights);
    assert_eq!(roundtrip.scene.spot_lights, snapshot.scene.spot_lights);
    assert_eq!(
        roundtrip.scene.ambient_lights,
        snapshot.scene.ambient_lights
    );
    assert_eq!(roundtrip.scene.rect_lights, snapshot.scene.rect_lights);
}

#[test]
fn render_product_pbr_lighting_extract_carries_ambient_and_rect_degradation_contracts() {
    let mut extract = RenderFrameExtract::from_snapshot(
        WorldHandle::new(8).into(),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: true,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    );
    extract
        .lighting
        .ambient_lights
        .push(RenderAmbientLightSnapshot {
            color: Vec3::new(0.1, 0.2, 0.3),
            intensity: 0.4,
            renderer_degraded: true,
            degradation_reason: Some(
                "ambient light renderer path is deferred after M5A".to_string(),
            ),
        });
    extract.lighting.rect_lights.push(RenderRectLightSnapshot {
        node_id: 80,
        position: Vec3::new(1.0, 2.0, 3.0),
        direction: Vec3::new(0.0, -1.0, 0.0),
        color: Vec3::new(1.0, 0.8, 0.6),
        intensity: 5.0,
        range: 12.0,
        size: Vec2::new(2.0, 0.5),
        renderer_degraded: true,
        degradation_reason: Some("rect light renderer path is deferred after M5A".to_string()),
    });

    assert_eq!(extract.lighting.ambient_lights.len(), 1);
    assert_eq!(extract.lighting.rect_lights.len(), 1);
    assert!(extract.lighting.ambient_lights[0].renderer_degraded);
    assert!(extract.lighting.rect_lights[0].renderer_degraded);
    assert!(extract.lighting.ambient_lights[0]
        .degradation_reason
        .as_deref()
        .unwrap()
        .contains("deferred"));
    assert!(extract.lighting.rect_lights[0]
        .degradation_reason
        .as_deref()
        .unwrap()
        .contains("deferred"));
}

#[test]
fn hybrid_gi_extract_defaults_to_public_settings_and_empty_internal_fixture() {
    let extract = RenderHybridGiExtract::default();

    assert!(!extract.enabled);
    assert_eq!(extract.quality, RenderHybridGiQuality::Medium);
    assert_eq!(extract.trace_budget, 0);
    assert_eq!(extract.card_budget, 0);
    assert_eq!(extract.voxel_budget, 0);
    assert_eq!(extract.debug_view, RenderHybridGiDebugView::None);
    assert_eq!(extract.probe_budget, 0);
    assert_eq!(extract.tracing_budget, 0);
    assert!(extract.probes.is_empty());
    assert!(extract.trace_regions.is_empty());
    assert!(!RenderFeatureQualitySettings::default().hybrid_global_illumination);
}

#[test]
fn render_profile_default_bundle_enables_basic_products_without_advanced_paths() {
    let bundle = RenderProfileBundle::default_render();

    assert_eq!(bundle.profile(), RenderProductProfile::DefaultRender);
    assert!(bundle.enables(RenderProductProfile::Render2d));
    assert!(bundle.enables(RenderProductProfile::Render3d));
    assert!(bundle.enables(RenderProductProfile::Ui));
    assert!(!bundle.enables(RenderProductProfile::AdvancedRender));
    assert!(!bundle.enables(RenderProductProfile::SolariExperimental));
    assert!(!bundle.has_feature(RenderProductFeature::VirtualGeometry));
    assert!(!bundle.has_feature(RenderProductFeature::HybridGlobalIllumination));
    assert!(!bundle.has_feature(RenderProductFeature::Solari));
    assert!(bundle.validate().is_ok());
}

#[test]
fn render_profile_validation_rejects_missing_2d_dependencies() {
    let bundle = RenderProfileBundle::new(RenderProductProfile::Render2d).with_features(
        RenderProfileBundle::render_2d().features_without(RenderProductFeature::Sprite),
    );

    assert_eq!(
        bundle.validate(),
        Err(RenderProfileValidationError::MissingRequiredFeature {
            profile: RenderProductProfile::Render2d,
            feature: RenderProductFeature::Sprite,
        })
    );
}

#[test]
fn render_profile_validation_rejects_missing_3d_dependencies() {
    let bundle = RenderProfileBundle::new(RenderProductProfile::Render3d).with_features(
        RenderProfileBundle::render_3d().features_without(RenderProductFeature::Pbr),
    );

    assert_eq!(
        bundle.validate(),
        Err(RenderProfileValidationError::MissingRequiredFeature {
            profile: RenderProductProfile::Render3d,
            feature: RenderProductFeature::Pbr,
        })
    );
}

#[test]
fn render_profile_validation_rejects_missing_ui_dependencies() {
    let bundle = RenderProfileBundle::new(RenderProductProfile::Ui).with_features(
        RenderProfileBundle::ui().features_without(RenderProductFeature::RenderTarget),
    );

    assert_eq!(
        bundle.validate(),
        Err(RenderProfileValidationError::MissingRequiredFeature {
            profile: RenderProductProfile::Ui,
            feature: RenderProductFeature::RenderTarget,
        })
    );
}

#[test]
fn render_profile_validation_rejects_unsatisfied_advanced_capabilities() {
    let bundle = RenderProfileBundle::advanced_render();
    let capabilities = RenderStats::default().capabilities;

    assert_eq!(
        bundle.validate_capabilities(&capabilities),
        Err(RenderProfileValidationError::MissingBackendCapability {
            profile: RenderProductProfile::AdvancedRender,
            detail: RenderCapabilityMismatchDetail::new(RenderCapabilityKind::VirtualGeometry),
        })
    );
}
