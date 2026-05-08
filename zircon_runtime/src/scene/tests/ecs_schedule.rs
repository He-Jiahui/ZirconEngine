use std::sync::{Arc, Mutex};

use crate::core::framework::render::{
    DisplayMode, RenderExtractContext, RenderVirtualGeometryDebugState, RenderWorldSnapshotHandle,
    SceneViewportExtractRequest, ViewportRenderSettings,
};
use crate::core::math::{Transform, UVec2, Vec3};
use crate::core::CoreRuntime;
use crate::plugin::{
    RuntimeExtensionRegistry, SceneRuntimeHook, SceneRuntimeHookContext,
    SceneRuntimeHookDescriptor, SceneRuntimeHookRegistration,
};
use crate::scene::components::Mobility;
use crate::scene::ecs::{
    EventStore, Events, InternalSceneSystem, ResourceStore, SceneSystemDescriptor, Schedule,
    SystemStage,
};
use crate::scene::{create_default_level, module_descriptor, NodeKind, SCENE_MODULE_NAME};

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
    let mesh = world.spawn_node(NodeKind::Mesh);
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

#[derive(Debug)]
struct RecordingPostUpdateHook {
    cube: u64,
    events: Arc<Mutex<Vec<String>>>,
}

#[derive(Debug)]
struct RecordingRenderExtractHook {
    events: Arc<Mutex<Vec<String>>>,
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
