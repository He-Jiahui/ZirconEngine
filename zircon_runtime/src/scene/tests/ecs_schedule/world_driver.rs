use super::*;

#[test]
fn world_driver_defers_runtime_system_mutations_until_builtin_post_update_systems_run() {
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
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    let events_for_system = Arc::clone(&events);
    registry
        .register_runtime_scene_system(
            owner,
            "weather.scene.post-update",
            SystemStage::PostUpdate,
            move || {
                let events = Arc::clone(&events_for_system);
                move |context| {
                    context.level.with_world_mut(|world| {
                        let before = world
                            .world_transform(cube)
                            .map(|transform| transform.translation.x as i32)
                            .unwrap_or_default();
                        events
                            .lock()
                            .unwrap()
                            .push(format!("runtime-before-transform={before}"));
                        world
                            .update_transform(
                                cube,
                                Transform::from_translation(Vec3::new(9.0, 0.0, 0.0)),
                            )
                            .expect("runtime system may update local transform before PostUpdate");
                        events.lock().unwrap().push(format!(
                            "runtime-after-local-update-pending={}",
                            world.has_pending_scene_systems()
                        ));
                    });
                    Ok(())
                }
            },
        )
        .with_order(0)
        .register()
        .unwrap();
    apply_runtime_scene_systems(&level, &registry);

    let advance = runtime.advance_time_by(Duration::from_secs_f32(1.0 / 60.0), 8);
    level.tick(&runtime.handle(), advance).unwrap();

    assert_eq!(
        *events.lock().unwrap(),
        vec![
            "runtime-before-transform=0".to_string(),
            "runtime-after-local-update-pending=true".to_string(),
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
fn world_driver_runs_native_render_extract_system_before_runtime_scene_systems() {
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
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    let events_for_system = Arc::clone(&events);
    registry
        .register_runtime_scene_system(
            owner,
            "weather.scene.render-extract",
            SystemStage::RenderExtract,
            move || {
                let events = Arc::clone(&events_for_system);
                move |context| {
                    context.level.with_world(|world| {
                        events.lock().unwrap().push(format!(
                            "render-extract-runtime-pending={}",
                            world.has_pending_scene_systems()
                        ));
                    });
                    Ok(())
                }
            },
        )
        .with_order(0)
        .register()
        .unwrap();
    apply_runtime_scene_systems(&level, &registry);

    let advance = runtime.advance_time_by(Duration::from_secs_f32(1.0 / 60.0), 8);
    level.tick(&runtime.handle(), advance).unwrap();

    assert_eq!(
        *events.lock().unwrap(),
        vec!["render-extract-runtime-pending=false".to_string()]
    );
    assert!(!level.with_world(|world| world.has_pending_scene_systems()));
}

#[test]
fn world_driver_orders_native_systems_with_plugin_runtime_systems() {
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

    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("weather.runtime").unwrap();
    let events_for_system = Arc::clone(&events);
    registry
        .register_runtime_scene_system(
            owner,
            "weather.scene.update",
            SystemStage::Update,
            move || {
                let events = Arc::clone(&events_for_system);
                move |context| {
                    context.level.with_world_mut(|_| {
                        events.lock().unwrap().push("runtime".to_string());
                    });
                    Ok(())
                }
            },
        )
        .with_order(0)
        .register()
        .unwrap();
    apply_runtime_scene_systems(&level, &registry);

    let advance = runtime.advance_time_by(Duration::from_secs_f32(1.0 / 60.0), 8);
    level.tick(&runtime.handle(), advance).unwrap();

    assert_eq!(
        *events.lock().unwrap(),
        vec![
            "native-before-runtime".to_string(),
            "runtime".to_string(),
            "native-after-runtime".to_string(),
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
                assert!(
                    context
                        .core
                        .resolve_driver::<crate::scene::WorldDriver>(
                            crate::scene::WORLD_DRIVER_NAME
                        )
                        .is_ok()
                );
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

#[derive(Debug, PartialEq, Eq)]
struct WorldDriverTickEvent(u32);

#[test]
fn world_driver_rotates_event_generations_once_per_tick() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    let mut subscription = level.with_world_mut(|world| {
        let mut subscription = world.register_dormant_event_subscription::<WorldDriverTickEvent>();
        assert!(world.connect_event_subscription(&mut subscription));
        subscription
    });
    let mut frame = 0;
    let system = FunctionRuntimeSceneSystem::new(
        SceneSystemMetadata::new("gameplay.runtime.event-generation", SystemStage::Update, 0),
        move |context| {
            frame += 1;
            context
                .level
                .with_world_mut(|world| world.send_event(WorldDriverTickEvent(frame)));
            Ok(())
        },
    );
    level
        .with_world_mut(|world| world.register_boxed_runtime_scene_system(Box::new(system)))
        .unwrap();

    let advance = runtime.advance_time_by(Duration::from_millis(16), 8);
    level.tick(&runtime.handle(), advance).unwrap();
    let first_generation = level.with_world(|world| {
        world
            .read_event_subscription(&mut subscription)
            .map(|event| event.0)
            .collect::<Vec<_>>()
    });
    assert!(first_generation.is_empty());

    let advance = runtime.advance_time_by(Duration::from_millis(16), 8);
    level.tick(&runtime.handle(), advance).unwrap();
    let second_generation = level.with_world(|world| {
        world
            .read_event_subscription(&mut subscription)
            .map(|event| event.0)
            .collect::<Vec<_>>()
    });
    assert_eq!(second_generation, vec![1]);
}

fn apply_runtime_scene_systems(
    level: &crate::scene::LevelSystem,
    registry: &RuntimeExtensionRegistry,
) {
    let plan = registry.world_runtime_extension_plan().unwrap();
    level
        .with_world_mut(|world| plan.apply_to_world(world))
        .unwrap();
}

#[test]
fn world_driver_pauses_virtual_systems_and_runs_explicit_real_time_systems() {
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

    {
        let events = events.clone();
        level
            .with_world_mut(|world| {
                world.register_native_system::<(), _>(
                    "gameplay.native.virtual-default",
                    SystemStage::Update,
                    -1,
                    move |()| events.lock().unwrap().push("native-virtual".to_string()),
                )
            })
            .unwrap();
    }
    {
        let events = events.clone();
        let system = FunctionRuntimeSceneSystem::new(
            SceneSystemMetadata::new("gameplay.runtime.virtual-delta", SystemStage::Update, 0),
            move |context| {
                events
                    .lock()
                    .unwrap()
                    .push(format!("runtime-delta={:.3}", context.delta_seconds));
                Ok(())
            },
        );
        level
            .with_world_mut(|world| world.register_boxed_runtime_scene_system(Box::new(system)))
            .unwrap();
    }
    {
        let events = events.clone();
        let mut registry = RuntimeExtensionRegistry::default();
        let owner = registry.intern_plugin_module("diagnostic.runtime").unwrap();
        registry
            .register_runtime_scene_system(
                owner,
                "diagnostic.runtime.real-delta",
                SystemStage::Update,
                move || {
                    let events = events.clone();
                    move |context| {
                        events
                            .lock()
                            .unwrap()
                            .push(format!("real-delta={:.3}", context.delta_seconds));
                        Ok(())
                    }
                },
            )
            .with_order(1)
            .with_clock_domain(SceneSystemClockDomain::Real)
            .register()
            .unwrap();
        apply_runtime_scene_systems(&level, &registry);
    }
    level
        .with_world_mut(|world| {
            world.update_transform(cube, Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)))
        })
        .unwrap();
    assert!(level.with_world(|world| world.has_pending_scene_systems()));

    runtime.pause_virtual_time();
    let paused = runtime.advance_time_by(Duration::from_millis(16), 8);
    assert_eq!(paused.real_delta(), Duration::from_millis(16));
    assert_eq!(paused.virtual_delta(), Duration::ZERO);
    level.tick(&runtime.handle(), paused).unwrap();
    assert_eq!(
        *events.lock().unwrap(),
        vec!["real-delta=0.016".to_string()]
    );
    assert!(level.with_world(|world| world.has_pending_scene_systems()));

    events.lock().unwrap().clear();
    runtime.unpause_virtual_time();
    runtime.set_virtual_time_relative_speed_f64(0.5);
    let scaled = runtime.advance_time_by(Duration::from_millis(16), 8);
    assert_eq!(scaled.virtual_delta(), Duration::from_millis(8));
    level.tick(&runtime.handle(), scaled).unwrap();
    assert_eq!(
        *events.lock().unwrap(),
        vec![
            "native-virtual".to_string(),
            "runtime-delta=0.008".to_string(),
            "real-delta=0.016".to_string(),
        ]
    );
    assert!(!level.with_world(|world| world.has_pending_scene_systems()));
    println!(
        "PERF_RESULT runtime22_clock_domain paused_virtual_callbacks=0 paused_real_callbacks=1 paused_virtual_work_reduction_percent=100 scaled_virtual_delta_ms=8 scaled_real_delta_ms=16"
    );
}
