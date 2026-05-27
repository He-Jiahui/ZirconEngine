use std::any::TypeId;
use std::sync::{Arc, Mutex};

use crate::core::framework::render::{
    DisplayMode, ProjectionMode, RenderCameraClearColor, RenderExtractContext, RenderLayerSet,
    RenderMaterialAlphaMode, RenderPhase, RenderViewportRect, RenderVirtualGeometryDebugState,
    RenderWorldSnapshotHandle, SceneViewportExtractRequest, ViewportCameraSnapshot,
    ViewportRenderSettings,
};
use crate::core::math::{Transform, UVec2, Vec3};
use crate::core::{CoreRuntime, JobScheduler};
use crate::plugin::{
    RuntimeExtensionRegistry, SceneRuntimeHook, SceneRuntimeHookContext,
    SceneRuntimeHookDescriptor, SceneRuntimeHookRegistration,
};
use crate::scene::components::{CameraComponent, MeshRenderer, Mobility};
use crate::scene::ecs::{
    CommandsParam, Component, EventStore, Events, InternalSceneSystem, QueryState, ResMutParam,
    ResParam, Resource, ResourceStore, SceneSystemDescriptor, Schedule, ScheduleConflictGraph,
    ScheduleConflictNode, ScheduleParallelExecutor, ScheduleParallelExecutorError,
    ScheduleParallelTaskRegistry, SystemParamAccess, SystemParamConflictKind, SystemStage,
    SystemState, With, Without,
};
use crate::scene::{create_default_level, module_descriptor, NodeKind, World, SCENE_MODULE_NAME};

#[derive(Debug, PartialEq, Eq)]
struct DeferredMarker;

impl Component for DeferredMarker {}

#[derive(Debug, PartialEq, Eq)]
struct ScheduleHealth(u32);

impl Component for ScheduleHealth {}

#[derive(Debug, PartialEq, Eq)]
struct SchedulePlayer;

impl Component for SchedulePlayer {}

#[derive(Debug, PartialEq, Eq)]
struct ScheduleFrameCounter(u32);

impl Resource for ScheduleFrameCounter {}

#[derive(Debug, PartialEq, Eq)]
struct ScheduleHitEvent;

#[derive(Debug, PartialEq, Eq)]
struct ScheduleNoticeMessage;

#[test]
fn resource_store_keeps_resources_by_concrete_type() {
    #[derive(Debug, PartialEq, Eq)]
    struct SceneFrameCounter(u32);

    let mut resources = ResourceStore::default();

    assert!(resources.is_empty());
    assert_eq!(resources.insert(SceneFrameCounter(1)), None);
    assert_eq!(
        resources.get::<SceneFrameCounter>(),
        Some(&SceneFrameCounter(1))
    );
    assert_eq!(
        resources.insert(SceneFrameCounter(2)),
        Some(SceneFrameCounter(1))
    );
    resources.get_mut::<SceneFrameCounter>().unwrap().0 += 1;
    assert_eq!(
        resources.remove::<SceneFrameCounter>(),
        Some(SceneFrameCounter(3))
    );
    assert!(!resources.contains::<SceneFrameCounter>());
}

#[test]
fn typed_events_publish_on_update_and_keep_next_frame_events_separate() {
    let mut events = Events::<u32>::default();

    events.send(7);
    assert!(events.is_empty());

    events.update();
    assert_eq!(events.iter().copied().collect::<Vec<_>>(), vec![7]);

    events.send(9);
    assert_eq!(events.drain(), vec![7]);
    events.update();
    assert_eq!(events.drain(), vec![9]);
}

#[test]
fn apply_deferred_internal_system_flushes_queued_commands() {
    let mut world = crate::scene::World::empty();
    let entity = world.spawn_node(NodeKind::Mesh);
    let mut system = SystemState::<CommandsParam>::new(&mut world).unwrap();

    system.run(&mut world, |mut commands| {
        commands.entity(entity).insert((DeferredMarker,));
    });

    assert!(world.get::<DeferredMarker>(entity).is_none());
    world.run_internal_scene_system(InternalSceneSystem::ApplyDeferred);

    assert_eq!(world.get::<DeferredMarker>(entity), Some(&DeferredMarker));
}

#[test]
fn event_store_tracks_each_event_type_independently() {
    #[derive(Debug, PartialEq, Eq)]
    struct Spawned(&'static str);
    #[derive(Debug, PartialEq, Eq)]
    struct Despawned(u64);

    let mut store = EventStore::default();
    store.send(Spawned("cube"));
    store.send(Despawned(42));

    assert!(store.events::<Spawned>().unwrap().is_empty());
    store.update::<Spawned>();
    assert_eq!(store.drain::<Spawned>(), vec![Spawned("cube")]);
    assert!(store.events::<Despawned>().unwrap().is_empty());

    store.update::<Despawned>();
    assert_eq!(store.drain::<Despawned>(), vec![Despawned(42)]);
    assert_eq!(store.registered_type_names().len(), 2);
}

#[test]
fn schedule_uses_bevy_style_stage_order_and_builtin_post_update_systems() {
    let schedule = Schedule::default();

    assert_eq!(
        schedule.stages,
        vec![
            SystemStage::First,
            SystemStage::PreUpdate,
            SystemStage::FixedUpdate,
            SystemStage::Update,
            SystemStage::PostUpdate,
            SystemStage::Last,
            SystemStage::RenderExtract,
        ]
    );
    assert!(schedule
        .systems()
        .iter()
        .all(|system| system.stage != SystemStage::First));

    let post_update = schedule
        .systems_for_stage(SystemStage::PostUpdate)
        .map(|system| system.system())
        .collect::<Vec<_>>();
    assert_eq!(
        post_update,
        vec![
            InternalSceneSystem::HierarchyValidity,
            InternalSceneSystem::ActiveHierarchy,
            InternalSceneSystem::WorldTransform,
            InternalSceneSystem::NodeCache,
        ]
    );

    let render_extract = schedule
        .systems_for_stage(SystemStage::RenderExtract)
        .map(|system| system.system())
        .collect::<Vec<_>>();
    assert_eq!(
        render_extract,
        vec![InternalSceneSystem::RenderExtractPrepare]
    );
}

#[test]
fn schedule_rejects_duplicate_and_blank_system_ids() {
    let mut schedule = Schedule::default();

    let duplicate = schedule
        .register_system(SceneSystemDescriptor::new(
            "zircon.scene.node_cache",
            SystemStage::PostUpdate,
            InternalSceneSystem::NodeCache,
        ))
        .unwrap_err();
    assert!(duplicate
        .to_string()
        .contains("system zircon.scene.node_cache already registered"));

    let blank = schedule
        .register_system(SceneSystemDescriptor::new(
            " ",
            SystemStage::Update,
            InternalSceneSystem::NodeCache,
        ))
        .unwrap_err();
    assert_eq!(blank.to_string(), "system id cannot be empty");
}

#[test]
fn schedule_rejects_duplicate_native_and_builtin_system_ids() {
    let mut world = crate::scene::World::empty();
    let duplicate_builtin = world
        .register_native_system::<(), _>("zircon.scene.node_cache", SystemStage::Update, 0, |_| {})
        .unwrap_err();
    assert!(duplicate_builtin
        .to_string()
        .contains("system zircon.scene.node_cache already registered"));

    world
        .register_native_system::<(), _>("gameplay.first", SystemStage::Update, 0, |_| {})
        .unwrap();
    let duplicate_native = world
        .register_native_system::<(), _>("gameplay.first", SystemStage::Update, 1, |_| {})
        .unwrap_err();
    assert!(duplicate_native
        .to_string()
        .contains("system gameplay.first already registered"));
}

#[test]
fn native_system_registration_reports_missing_required_resources() {
    let mut world = crate::scene::World::empty();
    let error = world
        .register_native_system::<crate::scene::ecs::ResParam<MissingScheduleResource>, _>(
            "gameplay.requires_missing_resource",
            SystemStage::Update,
            0,
            |_: crate::scene::ecs::Res<'_, MissingScheduleResource>| {},
        )
        .unwrap_err();

    assert!(error
        .to_string()
        .contains("system gameplay.requires_missing_resource failed to initialize params"));
    assert!(error
        .to_string()
        .contains(std::any::type_name::<MissingScheduleResource>()));
}

#[derive(Debug, PartialEq, Eq)]
struct MissingScheduleResource;

impl crate::scene::ecs::Resource for MissingScheduleResource {}

#[test]
fn schedule_conflict_graph_reports_component_write_conflicts_in_same_stage() {
    let mut world = World::empty();
    world.spawn((ScheduleHealth(1), SchedulePlayer)).unwrap();
    let read_health = SystemState::<QueryState<&'static ScheduleHealth>>::new(&mut world).unwrap();
    let write_health =
        SystemState::<QueryState<&'static mut ScheduleHealth>>::new(&mut world).unwrap();
    let health_component = world.component_id::<ScheduleHealth>();

    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new(
            "read.health",
            SystemStage::Update,
            read_health.access().clone(),
        ),
        ScheduleConflictNode::new(
            "write.health",
            SystemStage::Update,
            write_health.access().clone(),
        ),
    ]);

    assert_eq!(graph.nodes().len(), 2);
    assert!(graph.has_conflicts());
    let edge = &graph.edges()[0];
    assert_eq!(edge.left_system_id(), "read.health");
    assert_eq!(edge.right_system_id(), "write.health");
    assert_eq!(edge.stage(), SystemStage::Update);
    assert_eq!(
        edge.conflicts(),
        &[SystemParamConflictKind::Component(health_component)]
    );
    assert_eq!(graph.conflicts_for("read.health").count(), 1);
}

#[test]
fn schedule_conflict_graph_respects_disjoint_query_filters() {
    let mut world = World::empty();
    type PlayerHealth = QueryState<&'static mut ScheduleHealth, With<SchedulePlayer>>;
    type NonPlayerHealth = QueryState<&'static mut ScheduleHealth, Without<SchedulePlayer>>;
    let player_health = SystemState::<PlayerHealth>::new(&mut world).unwrap();
    let non_player_health = SystemState::<NonPlayerHealth>::new(&mut world).unwrap();

    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new(
            "write.player-health",
            SystemStage::Update,
            player_health.access().clone(),
        ),
        ScheduleConflictNode::new(
            "write.non-player-health",
            SystemStage::Update,
            non_player_health.access().clone(),
        ),
    ]);

    assert!(!graph.has_conflicts());
    assert!(graph.edges().is_empty());
}

#[test]
fn schedule_conflict_graph_keeps_different_stages_independent() {
    let mut world = World::empty();
    let read_health = SystemState::<QueryState<&'static ScheduleHealth>>::new(&mut world).unwrap();
    let write_health =
        SystemState::<QueryState<&'static mut ScheduleHealth>>::new(&mut world).unwrap();

    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new(
            "read.health",
            SystemStage::PreUpdate,
            read_health.access().clone(),
        ),
        ScheduleConflictNode::new(
            "write.health",
            SystemStage::PostUpdate,
            write_health.access().clone(),
        ),
    ]);

    assert!(!graph.has_conflicts());
}

#[test]
fn schedule_conflict_graph_reports_resource_write_conflicts() {
    let mut world = World::empty();
    world.insert_resource(ScheduleFrameCounter(0));
    let read_counter = SystemState::<ResParam<ScheduleFrameCounter>>::new(&mut world).unwrap();
    let write_counter = SystemState::<ResMutParam<ScheduleFrameCounter>>::new(&mut world).unwrap();
    let counter_resource = world.resource_id::<ScheduleFrameCounter>();

    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new(
            "read.frame-counter",
            SystemStage::Update,
            read_counter.access().clone(),
        ),
        ScheduleConflictNode::new(
            "write.frame-counter",
            SystemStage::Update,
            write_counter.access().clone(),
        ),
    ]);

    assert!(read_counter.access().conflicts_with(write_counter.access()));
    let edge = &graph.edges()[0];
    assert_eq!(
        edge.conflicts(),
        &[SystemParamConflictKind::Resource(counter_resource)]
    );
}

#[test]
fn schedule_conflict_graph_reports_event_and_message_write_conflicts() {
    let mut event_reader = SystemParamAccess::default();
    event_reader.add_event_read::<ScheduleHitEvent>().unwrap();
    let mut event_writer = SystemParamAccess::default();
    event_writer.add_event_write::<ScheduleHitEvent>().unwrap();
    let mut message_reader = SystemParamAccess::default();
    message_reader
        .add_message_read::<ScheduleNoticeMessage>()
        .unwrap();
    let mut message_writer = SystemParamAccess::default();
    message_writer
        .add_message_write::<ScheduleNoticeMessage>()
        .unwrap();

    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new("read.event", SystemStage::Update, event_reader),
        ScheduleConflictNode::new("write.event", SystemStage::Update, event_writer),
        ScheduleConflictNode::new("read.message", SystemStage::Update, message_reader),
        ScheduleConflictNode::new("write.message", SystemStage::Update, message_writer),
    ]);
    let event_type = TypeId::of::<ScheduleHitEvent>();
    let message_type = TypeId::of::<ScheduleNoticeMessage>();

    assert_eq!(graph.edges().len(), 2);
    assert!(graph.edges().iter().any(|edge| {
        edge.conflicts()
            .contains(&SystemParamConflictKind::Event(event_type))
    }));
    assert!(graph.edges().iter().any(|edge| {
        edge.conflicts()
            .contains(&SystemParamConflictKind::Message(message_type))
    }));
}

#[test]
fn schedule_conflict_graph_builds_conservative_parallel_batches() {
    let mut world = World::empty();
    world.spawn((ScheduleHealth(1), SchedulePlayer)).unwrap();
    world.insert_resource(ScheduleFrameCounter(0));
    let read_health = SystemState::<QueryState<&'static ScheduleHealth>>::new(&mut world).unwrap();
    let read_counter = SystemState::<ResParam<ScheduleFrameCounter>>::new(&mut world).unwrap();
    let write_health =
        SystemState::<QueryState<&'static mut ScheduleHealth>>::new(&mut world).unwrap();

    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new(
            "read.health",
            SystemStage::Update,
            read_health.access().clone(),
        ),
        ScheduleConflictNode::new(
            "read.counter",
            SystemStage::Update,
            read_counter.access().clone(),
        ),
        ScheduleConflictNode::new(
            "write.health",
            SystemStage::Update,
            write_health.access().clone(),
        ),
    ]);

    assert!(graph.systems_conflict("read.health", "write.health"));
    assert!(!graph.systems_conflict("read.counter", "write.health"));

    let batches = graph.conservative_parallel_batches();
    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0].stage(), SystemStage::Update);
    assert_eq!(
        batches[0]
            .system_ids()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["read.health", "read.counter"]
    );
    assert_eq!(
        batches[1]
            .system_ids()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["write.health"]
    );
}

#[test]
fn schedule_conflict_graph_keeps_parallel_batches_inside_stage_boundaries() {
    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new(
            "update.a",
            SystemStage::Update,
            SystemParamAccess::default(),
        ),
        ScheduleConflictNode::new(
            "post-update.b",
            SystemStage::PostUpdate,
            SystemParamAccess::default(),
        ),
    ]);

    let batches = graph.conservative_parallel_batches();
    assert_eq!(batches.len(), 2);
    assert_eq!(batches[0].stage(), SystemStage::Update);
    assert_eq!(batches[1].stage(), SystemStage::PostUpdate);
}

#[test]
fn schedule_parallel_executor_runs_registered_batches_through_job_scheduler() {
    let mut world = World::empty();
    world.spawn((ScheduleHealth(1), SchedulePlayer)).unwrap();
    world.insert_resource(ScheduleFrameCounter(0));
    let read_health = SystemState::<QueryState<&'static ScheduleHealth>>::new(&mut world).unwrap();
    let read_counter = SystemState::<ResParam<ScheduleFrameCounter>>::new(&mut world).unwrap();
    let write_health =
        SystemState::<QueryState<&'static mut ScheduleHealth>>::new(&mut world).unwrap();
    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new(
            "read.health",
            SystemStage::Update,
            read_health.access().clone(),
        ),
        ScheduleConflictNode::new(
            "read.counter",
            SystemStage::Update,
            read_counter.access().clone(),
        ),
        ScheduleConflictNode::new(
            "write.health",
            SystemStage::Update,
            write_health.access().clone(),
        ),
    ]);
    let batches = graph.conservative_parallel_batches();
    let observed = Arc::new(Mutex::new(Vec::new()));
    let mut registry = ScheduleParallelTaskRegistry::<&'static str>::new();
    for system_id in ["read.health", "read.counter", "write.health"] {
        let observed = observed.clone();
        registry.register(system_id, move || {
            observed.lock().unwrap().push(system_id);
            Ok(())
        });
    }
    let executor = ScheduleParallelExecutor::new(JobScheduler::default());

    executor.run_batches(&batches, &registry).unwrap();

    assert!(executor.scheduler().parallelism() >= 1);
    assert!(registry.contains("write.health"));
    let observed = observed.lock().unwrap();
    assert_eq!(observed.len(), 3);
    let mut first_batch = observed[..2].to_vec();
    first_batch.sort_unstable();
    assert_eq!(first_batch, vec!["read.counter", "read.health"]);
    assert_eq!(observed[2], "write.health");
}

#[test]
fn schedule_parallel_executor_reports_missing_tasks_before_running_batch() {
    let graph = ScheduleConflictGraph::from_nodes([ScheduleConflictNode::new(
        "missing.task",
        SystemStage::Update,
        SystemParamAccess::default(),
    )]);
    let batches = graph.conservative_parallel_batches();
    let registry = ScheduleParallelTaskRegistry::<&'static str>::new();
    let executor = ScheduleParallelExecutor::new(JobScheduler::default());

    let error = executor.run_batches(&batches, &registry).unwrap_err();

    assert_eq!(
        error,
        ScheduleParallelExecutorError::MissingTask {
            system_id: "missing.task".to_string(),
        }
    );
}

#[test]
fn schedule_parallel_executor_reports_task_failure_by_batch_order() {
    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new("ok.task", SystemStage::Update, SystemParamAccess::default()),
        ScheduleConflictNode::new(
            "fail.task",
            SystemStage::Update,
            SystemParamAccess::default(),
        ),
    ]);
    let batches = graph.conservative_parallel_batches();
    let mut registry = ScheduleParallelTaskRegistry::<&'static str>::new();
    registry.register("ok.task", || Ok(()));
    registry.register("fail.task", || Err("boom"));
    let executor = ScheduleParallelExecutor::new(JobScheduler::default());

    let error = executor.run_batches(&batches, &registry).unwrap_err();

    assert_eq!(
        error,
        ScheduleParallelExecutorError::TaskFailed {
            system_id: "fail.task".to_string(),
            error: "boom",
        }
    );
}

#[test]
fn world_mutations_mark_derived_state_dirty_until_post_update_systems_flush() {
    let mut world = crate::scene::World::new();
    let parent = world.spawn_node(NodeKind::Cube);
    let child = world.spawn_node(NodeKind::Mesh);

    world
        .update_transform(
            parent,
            Transform::from_translation(Vec3::new(5.0, 0.0, 0.0)),
        )
        .unwrap();
    world
        .update_transform(child, Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)))
        .unwrap();
    world.set_parent_checked(child, Some(parent)).unwrap();
    world.set_active_self(parent, false).unwrap();

    assert!(world.nodes().iter().all(|node| node.id != parent));
    assert!(world.node_records().iter().any(|node| node.id == parent));

    assert!(world.has_pending_scene_systems());
    assert_eq!(
        world.world_transform(child).unwrap().translation,
        Vec3::new(7.0, 0.0, 0.0)
    );
    assert_eq!(world.active_in_hierarchy(child), Some(false));
    assert!(world.has_pending_scene_systems());

    world.run_internal_scene_systems_for_stage(SystemStage::PostUpdate);

    assert!(world.has_pending_scene_systems());
    assert_eq!(
        world.world_transform(child).unwrap().translation,
        Vec3::new(7.0, 0.0, 0.0)
    );
    assert_eq!(world.active_in_hierarchy(child), Some(false));

    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    assert!(!world.has_pending_scene_systems());
}

#[test]
fn render_extract_prepare_flushes_parent_reorder_and_active_changes() {
    let mut world = crate::scene::World::new();
    let first_parent = world.spawn_node(NodeKind::Cube);
    let second_parent = world.spawn_node(NodeKind::Cube);
    let child = world.spawn_node(NodeKind::Mesh);

    world
        .update_transform(
            first_parent,
            Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
        )
        .unwrap();
    world
        .update_transform(
            second_parent,
            Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
        )
        .unwrap();
    world
        .update_transform(child, Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)))
        .unwrap();
    world.set_parent_checked(child, Some(first_parent)).unwrap();
    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    world
        .set_parent_checked(child, Some(second_parent))
        .unwrap();
    world.set_active_self(second_parent, false).unwrap();

    assert!(world
        .nodes()
        .iter()
        .find(|node| node.id == child)
        .is_some_and(|node| node.parent == Some(first_parent)));
    assert!(world
        .node_records()
        .iter()
        .find(|node| node.id == child)
        .is_some_and(|node| node.parent == Some(second_parent)));
    assert_eq!(world.active_in_hierarchy(child), Some(false));
    assert_eq!(
        world.world_transform(child).unwrap().translation,
        Vec3::new(12.0, 0.0, 0.0)
    );
    assert!(world.has_pending_scene_systems());
    assert!(world
        .to_render_extract()
        .scene
        .meshes
        .iter()
        .all(|mesh| mesh.node_id != child));
    assert!(world.has_pending_scene_systems());

    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);

    assert!(!world.has_pending_scene_systems());
}

#[test]
fn canonical_render_frame_extract_populates_scene_sections_directly() {
    let mut world = crate::scene::World::new();
    let camera = world.active_camera();
    let mesh = world.spawn_node(NodeKind::Mesh);
    world.set_render_layer_mask(camera, 0b1010).unwrap();
    world.set_render_layer_mask(mesh, 0b1010).unwrap();
    world
        .update_transform(mesh, Transform::from_translation(Vec3::new(4.0, 5.0, 6.0)))
        .unwrap();
    world.set_mobility(mesh, Mobility::Static).unwrap();
    let debug = RenderVirtualGeometryDebugState {
        forced_mip: Some(3),
        visualize_bvh: true,
        ..RenderVirtualGeometryDebugState::default()
    };
    let context = RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(55),
        SceneViewportExtractRequest {
            settings: ViewportRenderSettings {
                display_mode: DisplayMode::WireOnly,
                preview_lighting: false,
                preview_skybox: false,
                ..ViewportRenderSettings::default()
            },
            active_camera_override: None,
            camera: None,
            viewport_size: Some(UVec2::new(1280, 720)),
            virtual_geometry_debug: Some(debug),
        },
    );

    let extract = world.build_prepared_render_frame_extract(&context);

    assert_eq!(extract.world.raw(), 55);
    assert_eq!(extract.view.camera.aspect_ratio, 1280.0 / 720.0);
    assert!(extract.geometry.meshes.iter().any(|snapshot| {
        snapshot.node_id == mesh
            && snapshot.transform.translation == Vec3::new(4.0, 5.0, 6.0)
            && snapshot.mobility == Mobility::Static
            && snapshot.render_layer_mask == 0b1010
    }));
    assert_eq!(extract.geometry.virtual_geometry_debug, Some(debug));
    assert!(extract.geometry.virtual_geometry.is_some());
    assert!(extract
        .lighting
        .hybrid_global_illumination
        .as_ref()
        .is_some_and(|hybrid_gi| !hybrid_gi.enabled));
    assert_eq!(extract.post_process.display_mode, DisplayMode::WireOnly);
    assert!(!extract.post_process.preview.lighting_enabled);
    assert!(!extract.post_process.preview.skybox_enabled);
    assert_eq!(
        extract.visibility.renderables.len(),
        extract.geometry.meshes.len()
    );
    assert!(extract
        .visibility
        .static_entities
        .iter()
        .any(|entity| *entity == mesh));
    assert!(!world.has_pending_scene_systems());
}

#[test]
fn prepared_render_frame_extract_queues_meshes_from_mesh_renderer_alpha_hints() {
    let mut world = crate::scene::World::new();
    let alpha_mask_mesh = world.spawn_node(NodeKind::Mesh);
    let transparent_mesh = world.spawn_node(NodeKind::Mesh);
    world
        .get_mut::<MeshRenderer>(alpha_mask_mesh)
        .unwrap()
        .material_alpha_mode = RenderMaterialAlphaMode::Mask { cutoff: 0.37 };
    world
        .get_mut::<MeshRenderer>(transparent_mesh)
        .unwrap()
        .material_alpha_mode = RenderMaterialAlphaMode::Blend;
    world
        .update_transform(
            transparent_mesh,
            Transform::from_translation(Vec3::new(0.0, 0.0, 9.0)),
        )
        .unwrap();
    let context = RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(56),
        SceneViewportExtractRequest::default(),
    );

    let extract = world.build_prepared_render_frame_extract(&context);
    let alpha_mask_index = extract
        .geometry
        .meshes
        .iter()
        .position(|mesh| mesh.node_id == alpha_mask_mesh)
        .expect("alpha-mask mesh should be extracted");
    let transparent_index = extract
        .geometry
        .meshes
        .iter()
        .position(|mesh| mesh.node_id == transparent_mesh)
        .expect("transparent mesh should be extracted");

    assert!(extract.geometry.phase_inputs.iter().any(|input| {
        input.entity == alpha_mask_mesh
            && input.mesh_index == alpha_mask_index
            && input.material_alpha_mode == RenderMaterialAlphaMode::Mask { cutoff: 0.37 }
    }));
    assert!(extract.geometry.phase_inputs.iter().any(|input| {
        input.entity == transparent_mesh
            && input.mesh_index == transparent_index
            && input.material_alpha_mode == RenderMaterialAlphaMode::Blend
            && input.depth == 9.0
    }));
    assert!(extract
        .geometry
        .phase_queue
        .items_for_phase(RenderPhase::AlphaMask3d)
        .any(|item| item.entity == alpha_mask_mesh));
    assert!(extract
        .geometry
        .phase_queue
        .items_for_phase(RenderPhase::Transparent3d)
        .any(|item| item.entity == transparent_mesh));
}

#[test]
fn render_extract_filters_meshes_by_active_camera_layers() {
    let mut world = crate::scene::World::new();
    let camera = world.active_camera();
    let visible_mesh = world.spawn_node(NodeKind::Mesh);
    let hidden_mesh = world.spawn_node(NodeKind::Mesh);
    world.set_render_layer_mask(camera, 0b0010).unwrap();
    world.set_render_layer_mask(visible_mesh, 0b0010).unwrap();
    world.set_render_layer_mask(hidden_mesh, 0b0100).unwrap();

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(57),
        SceneViewportExtractRequest::default(),
    ));

    assert!(extract
        .geometry
        .meshes
        .iter()
        .any(|mesh| mesh.node_id == visible_mesh));
    assert!(extract
        .geometry
        .meshes
        .iter()
        .all(|mesh| mesh.node_id != hidden_mesh));
    assert!(extract
        .geometry
        .meshes
        .iter()
        .all(|mesh| mesh.render_layer_mask & 0b0010 != 0));
    assert!(extract
        .view
        .camera
        .render_layers
        .intersects_legacy_mask(0b0010));
}

#[test]
fn explicit_render_camera_snapshot_layers_override_scene_camera_layers() {
    let mut world = crate::scene::World::new();
    let camera = world.active_camera();
    let visible_mesh = world.spawn_node(NodeKind::Mesh);
    let hidden_mesh = world.spawn_node(NodeKind::Mesh);
    world.set_render_layer_mask(camera, 0b0010).unwrap();
    world.set_render_layer_mask(visible_mesh, 0b0100).unwrap();
    world.set_render_layer_mask(hidden_mesh, 0b0010).unwrap();

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(58),
        SceneViewportExtractRequest {
            camera: Some(ViewportCameraSnapshot {
                render_layers: RenderLayerSet::from_legacy_mask(0b0100),
                ..ViewportCameraSnapshot::default()
            }),
            ..SceneViewportExtractRequest::default()
        },
    ));

    assert!(extract
        .geometry
        .meshes
        .iter()
        .any(|mesh| mesh.node_id == visible_mesh));
    assert!(extract
        .geometry
        .meshes
        .iter()
        .all(|mesh| mesh.node_id != hidden_mesh));
    assert!(extract
        .view
        .camera
        .render_layers
        .intersects_legacy_mask(0b0100));
}

#[test]
fn render_extract_projects_scene_camera_component_product_fields() {
    let mut world = crate::scene::World::new();
    let camera = world.active_camera();
    *world.get_mut::<CameraComponent>(camera).unwrap() = CameraComponent {
        projection_mode: ProjectionMode::Orthographic,
        fov_y_radians: 0.85,
        ortho_size: 14.0,
        z_near: 0.05,
        z_far: 750.0,
        viewport: Some(RenderViewportRect::new(
            UVec2::new(16, 32),
            UVec2::new(400, 200),
        )),
        order: 4,
        is_active: false,
        hdr: true,
        exposure_ev100: 12.0,
        clear_color: RenderCameraClearColor::None,
        msaa_samples: 4,
        ..CameraComponent::default()
    };

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(59),
        SceneViewportExtractRequest {
            viewport_size: Some(UVec2::new(1280, 720)),
            ..SceneViewportExtractRequest::default()
        },
    ));

    assert_eq!(
        extract.view.camera.projection_mode,
        ProjectionMode::Orthographic
    );
    assert_eq!(extract.view.camera.fov_y_radians, 0.85);
    assert_eq!(extract.view.camera.ortho_size, 14.0);
    assert_eq!(extract.view.camera.z_near, 0.05);
    assert_eq!(extract.view.camera.z_far, 750.0);
    assert_eq!(extract.view.camera.aspect_ratio, 2.0);
    assert_eq!(extract.view.camera.order, 4);
    assert!(!extract.view.camera.is_active);
    assert!(extract.view.camera.hdr);
    assert_eq!(extract.view.camera.exposure_ev100, 12.0);
    assert_eq!(
        extract.view.camera.clear_color,
        RenderCameraClearColor::None
    );
    assert_eq!(extract.view.camera.msaa_samples, 4);
}

#[test]
fn inactive_render_camera_extracts_no_scene_renderables() {
    let mut world = crate::scene::World::new();
    let camera = world.active_camera();
    world.get_mut::<CameraComponent>(camera).unwrap().is_active = false;
    world.spawn_node(NodeKind::Mesh);
    world.spawn_node(NodeKind::DirectionalLight);

    let extract = world.build_prepared_render_frame_extract(&RenderExtractContext::new(
        RenderWorldSnapshotHandle::new(60),
        SceneViewportExtractRequest::default(),
    ));

    assert!(!extract.view.camera.is_active);
    assert!(extract.geometry.meshes.is_empty());
    assert!(extract.geometry.phase_inputs.is_empty());
    assert!(extract.visibility.renderable_entities.is_empty());
    assert!(extract.visibility.renderables.is_empty());
    assert!(extract.lighting.directional_lights.is_empty());

    let packet = world.build_viewport_render_packet(&SceneViewportExtractRequest::default());
    assert!(!packet.scene.camera.is_active);
    assert!(packet.scene.meshes.is_empty());
    assert!(packet.scene.directional_lights.is_empty());
}

#[test]
fn mobility_changes_are_node_cache_dirty_without_transform_flush() {
    let mut world = crate::scene::World::new();
    let entity = world.spawn_node(NodeKind::Mesh);

    world.run_internal_scene_systems_for_stage(SystemStage::RenderExtract);
    assert!(!world.has_pending_scene_systems());
    assert!(world.set_mobility(entity, Mobility::Static).unwrap());

    assert!(world.has_pending_scene_systems());
    assert_eq!(world.world_transform(entity).unwrap(), Transform::default());
    assert!(world.has_pending_scene_systems());
}

#[test]
fn world_driver_defers_hook_mutations_until_builtin_post_update_systems_run() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    let cube = level.with_world(|world| {
        world
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube))
            .unwrap()
            .id
    });
    let events = Arc::new(Mutex::new(Vec::new()));

    let mut registry = RuntimeExtensionRegistry::default();
    registry
        .register_scene_hook(SceneRuntimeHookRegistration::new(
            SceneRuntimeHookDescriptor::new(
                "weather.scene.post-update",
                "weather",
                SystemStage::PostUpdate,
            )
            .with_order(0),
            RecordingPostUpdateHook {
                cube,
                events: events.clone(),
            },
        ))
        .unwrap();
    runtime.install_scene_runtime_hooks(&registry).unwrap();

    level.tick(&runtime.handle(), 1.0 / 60.0).unwrap();

    assert_eq!(
        *events.lock().unwrap(),
        vec![
            "hook-before-transform=0".to_string(),
            "hook-after-local-update-pending=true".to_string(),
        ]
    );
    assert_eq!(
        level.with_world(|world| world.world_transform(cube).unwrap().translation),
        Vec3::new(9.0, 0.0, 0.0)
    );
}

#[test]
fn world_driver_runs_native_render_extract_system_before_render_extract_hooks() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    let cube = level.with_world(|world| {
        world
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube))
            .unwrap()
            .id
    });
    level
        .with_world_mut(|world| world.set_render_layer_mask(cube, 0b11))
        .unwrap();
    assert!(level.with_world(|world| world.has_pending_scene_systems()));
    let events = Arc::new(Mutex::new(Vec::new()));

    let mut registry = RuntimeExtensionRegistry::default();
    registry
        .register_scene_hook(SceneRuntimeHookRegistration::new(
            SceneRuntimeHookDescriptor::new(
                "weather.scene.render-extract",
                "weather",
                SystemStage::RenderExtract,
            )
            .with_order(0),
            RecordingRenderExtractHook {
                events: events.clone(),
            },
        ))
        .unwrap();
    runtime.install_scene_runtime_hooks(&registry).unwrap();

    level.tick(&runtime.handle(), 1.0 / 60.0).unwrap();

    assert_eq!(
        *events.lock().unwrap(),
        vec!["render-extract-hook-pending=false".to_string()]
    );
    assert!(!level.with_world(|world| world.has_pending_scene_systems()));
}

#[test]
fn world_driver_orders_native_systems_with_plugin_hooks() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));

    {
        let events = events.clone();
        level
            .with_world_mut(|world| {
                world.register_native_system::<(), _>(
                    "gameplay.native.before-hook",
                    SystemStage::Update,
                    -1,
                    move |()| {
                        events
                            .lock()
                            .unwrap()
                            .push("native-before-hook".to_string())
                    },
                )
            })
            .unwrap();
    }
    {
        let events = events.clone();
        level
            .with_world_mut(|world| {
                world.register_native_system::<(), _>(
                    "gameplay.native.after-hook",
                    SystemStage::Update,
                    1,
                    move |()| events.lock().unwrap().push("native-after-hook".to_string()),
                )
            })
            .unwrap();
    }

    let mut registry = RuntimeExtensionRegistry::default();
    registry
        .register_scene_hook(SceneRuntimeHookRegistration::new(
            SceneRuntimeHookDescriptor::new("weather.scene.update", "weather", SystemStage::Update)
                .with_order(0),
            RecordingUpdateHook {
                events: events.clone(),
            },
        ))
        .unwrap();
    runtime.install_scene_runtime_hooks(&registry).unwrap();

    level.tick(&runtime.handle(), 1.0 / 60.0).unwrap();

    assert_eq!(
        *events.lock().unwrap(),
        vec![
            "native-before-hook".to_string(),
            "hook".to_string(),
            "native-after-hook".to_string(),
        ]
    );
}

#[derive(Debug)]
struct RecordingPostUpdateHook {
    cube: u64,
    events: Arc<Mutex<Vec<String>>>,
}

#[derive(Debug)]
struct RecordingRenderExtractHook {
    events: Arc<Mutex<Vec<String>>>,
}

#[derive(Debug)]
struct RecordingUpdateHook {
    events: Arc<Mutex<Vec<String>>>,
}

impl SceneRuntimeHook for RecordingUpdateHook {
    fn run(&self, context: SceneRuntimeHookContext<'_>) -> Result<(), crate::core::CoreError> {
        context.level.with_world_mut(|_| {
            self.events.lock().unwrap().push("hook".to_string());
        });
        Ok(())
    }
}

impl SceneRuntimeHook for RecordingRenderExtractHook {
    fn run(&self, context: SceneRuntimeHookContext<'_>) -> Result<(), crate::core::CoreError> {
        context.level.with_world(|world| {
            self.events.lock().unwrap().push(format!(
                "render-extract-hook-pending={}",
                world.has_pending_scene_systems()
            ));
        });
        Ok(())
    }
}

impl SceneRuntimeHook for RecordingPostUpdateHook {
    fn run(&self, context: SceneRuntimeHookContext<'_>) -> Result<(), crate::core::CoreError> {
        context.level.with_world_mut(|world| {
            let before = world
                .world_transform(self.cube)
                .map(|transform| transform.translation.x as i32)
                .unwrap_or_default();
            self.events
                .lock()
                .unwrap()
                .push(format!("hook-before-transform={before}"));
            world
                .update_transform(
                    self.cube,
                    Transform::from_translation(Vec3::new(9.0, 0.0, 0.0)),
                )
                .expect("hook may update local transform before built-in PostUpdate systems");
            self.events.lock().unwrap().push(format!(
                "hook-after-local-update-pending={}",
                world.has_pending_scene_systems()
            ));
        });
        Ok(())
    }
}
