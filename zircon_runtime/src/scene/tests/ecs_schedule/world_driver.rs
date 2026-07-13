use super::*;

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
    crate::scene::install_scene_runtime_hooks(
        &runtime.handle(),
        registry.scene_hooks().iter().cloned(),
    )
    .unwrap();

    let advance = runtime.advance_time_by(Duration::from_secs_f32(1.0 / 60.0), 8);
    level.tick(&runtime.handle(), advance).unwrap();

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
fn world_driver_consumes_runtime_time_advance_without_advancing_clocks_again() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    runtime.set_fixed_timestep(Duration::from_millis(10));
    let level = create_default_level(&runtime.handle()).unwrap();

    let advance = runtime.advance_time_by(Duration::from_millis(25), 8);
    assert_eq!(advance.fixed_step_plan().step_count, 2);

    level.tick(&runtime.handle(), advance).unwrap();

    let clocks = runtime.time_clocks();
    assert_eq!(clocks.real().frame_index(), 1);
    assert_eq!(clocks.fixed().frame_index(), 2);
    assert_eq!(clocks.fixed().overstep(), Duration::from_millis(5));
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
    crate::scene::install_scene_runtime_hooks(
        &runtime.handle(),
        registry.scene_hooks().iter().cloned(),
    )
    .unwrap();

    let advance = runtime.advance_time_by(Duration::from_secs_f32(1.0 / 60.0), 8);
    level.tick(&runtime.handle(), advance).unwrap();

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
    crate::scene::install_scene_runtime_hooks(
        &runtime.handle(),
        registry.scene_hooks().iter().cloned(),
    )
    .unwrap();

    let advance = runtime.advance_time_by(Duration::from_secs_f32(1.0 / 60.0), 8);
    level.tick(&runtime.handle(), advance).unwrap();

    assert_eq!(
        *events.lock().unwrap(),
        vec![
            "native-before-hook".to_string(),
            "hook".to_string(),
            "native-after-hook".to_string(),
        ]
    );
}

#[test]
fn world_driver_runs_runtime_scene_systems_in_schedule_order() {
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
                    "gameplay.native.before-runtime",
                    SystemStage::Update,
                    -1,
                    move |()| {
                        events
                            .lock()
                            .unwrap()
                            .push("native-before-runtime".to_string())
                    },
                )
            })
            .unwrap();
    }
    {
        let events = events.clone();
        let system = FunctionRuntimeSceneSystem::new(
            SceneSystemMetadata::new("gameplay.runtime.context", SystemStage::Update, 0),
            move |context| {
                context.level.with_world(|_| {
                    events
                        .lock()
                        .unwrap()
                        .push(format!("runtime-delta={:.3}", context.delta_seconds));
                });
                assert!(context
                    .core
                    .resolve_driver::<crate::scene::WorldDriver>(crate::scene::WORLD_DRIVER_NAME)
                    .is_ok());
                Ok(())
            },
        );
        level
            .with_world_mut(|world| world.register_boxed_runtime_scene_system(Box::new(system)))
            .unwrap();
    }
    {
        let events = events.clone();
        level
            .with_world_mut(|world| {
                world.register_native_system::<(), _>(
                    "gameplay.native.after-runtime",
                    SystemStage::Update,
                    1,
                    move |()| {
                        events
                            .lock()
                            .unwrap()
                            .push("native-after-runtime".to_string())
                    },
                )
            })
            .unwrap();
    }

    let advance = runtime.advance_time_by(Duration::from_millis(16), 8);
    level.tick(&runtime.handle(), advance).unwrap();

    assert_eq!(
        *events.lock().unwrap(),
        vec![
            "native-before-runtime".to_string(),
            "runtime-delta=0.016".to_string(),
            "native-after-runtime".to_string(),
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
