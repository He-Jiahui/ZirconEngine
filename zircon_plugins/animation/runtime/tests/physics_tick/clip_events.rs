#[test]
fn level_tick_emits_animation_clip_event_tracks_crossed_by_player_time() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let core = runtime.handle();
    let asset_manager = runtime_asset_manager(&core);
    let skeleton_uri = AssetUri::parse("res://animation/event.skeleton.zranim").unwrap();
    let clip_uri = AssetUri::parse("res://animation/event.clip.zranim").unwrap();
    let clip_id = ResourceId::from_locator(&clip_uri);
    let mut clip = single_hand_translation_clip(&skeleton_uri, 0.0);
    clip.event_tracks = vec![
        AnimationEventTrackAsset {
            target_id: Some("Root/Hand".to_string()),
            event: "footstep".to_string(),
            time_seconds: 0.25,
            payload: Some("left".to_string()),
        },
        AnimationEventTrackAsset {
            target_id: None,
            event: "land".to_string(),
            time_seconds: 0.75,
            payload: None,
        },
    ];
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(clip_id, ResourceKind::AnimationClip, clip_uri),
        clip,
    );
    let level = runtime.create_default_level().unwrap();
    let entity = level.with_world_mut(|world| {
        let entity = world
            .spawn_node(NodeKind::Cube)
            .expect("test node creation must succeed");
        world
            .set_animation_player(
                entity,
                Some(AnimationPlayerComponent {
                    clip: ResourceHandle::<AnimationClipMarker>::new(clip_id),
                    playback_speed: 1.0,
                    time_seconds: 0.2,
                    weight: 1.0,
                    looping: false,
                    playing: true,
                }),
            )
            .unwrap();
        entity
    });

    let mut event_subscription = subscribe_animation_clip_events(&level);
    runtime.tick_level_seconds(&level, 0.1).unwrap();
    let events = drain_animation_clip_events(&level, &mut event_subscription);

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].entity, entity);
    assert_eq!(events[0].event, "footstep");
    assert_eq!(events[0].payload.as_deref(), Some("left"));
    assert_eq!(events[0].clip_time_seconds, 0.25);
    assert_eq!(events[0].playback_time_seconds, 0.25);
}

#[test]
fn negative_speed_looping_player_emits_reverse_events_across_the_loop_boundary() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let core = runtime.handle();
    let asset_manager = runtime_asset_manager(&core);
    let skeleton_uri = AssetUri::parse("res://animation/reverse-loop.skeleton.zranim").unwrap();
    let clip_uri = AssetUri::parse("res://animation/reverse-loop.clip.zranim").unwrap();
    let clip_id = ResourceId::from_locator(&clip_uri);
    let mut clip = single_hand_translation_clip(&skeleton_uri, 0.0);
    clip.event_tracks = vec![
        AnimationEventTrackAsset {
            target_id: None,
            event: "loop-start".to_string(),
            time_seconds: 0.0,
            payload: None,
        },
        AnimationEventTrackAsset {
            target_id: None,
            event: "loop-end".to_string(),
            time_seconds: 1.0,
            payload: None,
        },
        AnimationEventTrackAsset {
            target_id: None,
            event: "three-quarter".to_string(),
            time_seconds: 0.75,
            payload: None,
        },
    ];
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(clip_id, ResourceKind::AnimationClip, clip_uri),
        clip,
    );
    let level = runtime.create_default_level().unwrap();
    let entity = level.with_world_mut(|world| {
        let entity = world
            .spawn_node(NodeKind::Cube)
            .expect("test node creation must succeed");
        world
            .set_animation_player(
                entity,
                Some(AnimationPlayerComponent {
                    clip: ResourceHandle::<AnimationClipMarker>::new(clip_id),
                    playback_speed: -1.0,
                    time_seconds: 2.25,
                    weight: 1.0,
                    looping: true,
                    playing: true,
                }),
            )
            .unwrap();
        entity
    });

    let mut event_subscription = subscribe_animation_clip_events(&level);
    runtime.tick_level_seconds(&level, 0.5).unwrap();
    let events = drain_animation_clip_events(&level, &mut event_subscription);

    assert_eq!(
        events
            .iter()
            .map(|event| (
                event.entity,
                event.event.as_str(),
                event.clip_time_seconds,
                event.playback_time_seconds
            ))
            .collect::<Vec<_>>(),
        vec![
            (entity, "loop-start", 0.0, 2.0),
            (entity, "three-quarter", 0.75, 1.75),
        ]
    );
    assert_eq!(
        level.with_world(|world| world.animation_player(entity).unwrap().time_seconds),
        1.75
    );
}

#[test]
fn reversing_a_clip_player_does_not_repeat_the_direction_change_endpoint() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let core = runtime.handle();
    let asset_manager = runtime_asset_manager(&core);
    let skeleton_uri = AssetUri::parse("res://animation/reverse-turn.skeleton.zranim").unwrap();
    let clip_uri = AssetUri::parse("res://animation/reverse-turn.clip.zranim").unwrap();
    let clip_id = ResourceId::from_locator(&clip_uri);
    let mut clip = single_hand_translation_clip(&skeleton_uri, 0.0);
    clip.event_tracks = vec![
        AnimationEventTrackAsset {
            target_id: None,
            event: "destination".to_string(),
            time_seconds: 0.25,
            payload: None,
        },
        AnimationEventTrackAsset {
            target_id: None,
            event: "turn".to_string(),
            time_seconds: 0.5,
            payload: None,
        },
    ];
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(clip_id, ResourceKind::AnimationClip, clip_uri),
        clip,
    );
    let level = runtime.create_default_level().unwrap();
    let entity = level.with_world_mut(|world| {
        let entity = world
            .spawn_node(NodeKind::Cube)
            .expect("test node creation must succeed");
        world
            .set_animation_player(
                entity,
                Some(AnimationPlayerComponent {
                    clip: ResourceHandle::<AnimationClipMarker>::new(clip_id),
                    playback_speed: 1.0,
                    time_seconds: 0.25,
                    weight: 1.0,
                    looping: false,
                    playing: true,
                }),
            )
            .unwrap();
        entity
    });

    let mut event_subscription = subscribe_animation_clip_events(&level);
    runtime.tick_level_seconds(&level, 0.25).unwrap();
    let forward = drain_animation_clip_events(&level, &mut event_subscription);
    assert_eq!(
        forward
            .iter()
            .map(|event| event.event.as_str())
            .collect::<Vec<_>>(),
        vec!["turn"]
    );

    level.with_world_mut(|world| {
        let mut player = world.animation_player(entity).unwrap().clone();
        player.playback_speed = -1.0;
        world.set_animation_player(entity, Some(player)).unwrap();
    });
    runtime.tick_level_seconds(&level, 0.25).unwrap();
    let reverse = drain_animation_clip_events(&level, &mut event_subscription);

    assert_eq!(
        reverse
            .iter()
            .map(|event| (event.event.as_str(), event.playback_time_seconds))
            .collect::<Vec<_>>(),
        vec![("destination", 0.25)]
    );
}

#[test]
fn graph_player_emits_clip_events_using_graph_clip_playback_speed() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let core = runtime.handle();
    let asset_manager = runtime_asset_manager(&core);
    let skeleton_uri = AssetUri::parse("res://animation/graph-event.skeleton.zranim").unwrap();
    let clip_uri = AssetUri::parse("res://animation/graph-event.clip.zranim").unwrap();
    let graph_uri = AssetUri::parse("res://animation/graph-event.graph.zranim").unwrap();
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let clip_id = ResourceId::from_locator(&clip_uri);
    let graph_id = ResourceId::from_locator(&graph_uri);
    let mut clip = single_hand_translation_clip(&skeleton_uri, 0.0);
    clip.event_tracks = vec![AnimationEventTrackAsset {
        target_id: Some("Root/Hand".to_string()),
        event: "graph_hit".to_string(),
        time_seconds: 0.5,
        payload: Some("fast".to_string()),
    }];
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            skeleton_id,
            ResourceKind::AnimationSkeleton,
            skeleton_uri.clone(),
        ),
        two_bone_skeleton(),
    );
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(clip_id, ResourceKind::AnimationClip, clip_uri.clone()),
        clip,
    );
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(graph_id, ResourceKind::AnimationGraph, graph_uri),
        AnimationGraphAsset {
            name: Some("GraphEvent".to_string()),
            parameters: Vec::new(),
            nodes: vec![
                AnimationGraphNodeAsset::Clip {
                    id: "clip".to_string(),
                    clip: AssetReference::from_locator(clip_uri),
                    playback_speed: 2.0,
                    looping: false,
                },
                AnimationGraphNodeAsset::Output {
                    source: "clip".to_string(),
                },
            ],
        },
    );
    let level = runtime.create_default_level().unwrap();
    let entity = level.with_world_mut(|world| {
        let entity = world
            .spawn_node(NodeKind::Cube)
            .expect("test node creation must succeed");
        world
            .set_animation_skeleton(
                entity,
                Some(AnimationSkeletonComponent {
                    skeleton: ResourceHandle::<AnimationSkeletonMarker>::new(skeleton_id),
                }),
            )
            .unwrap();
        world
            .set_animation_graph_player(
                entity,
                Some(AnimationGraphPlayerComponent {
                    graph: ResourceHandle::<AnimationGraphMarker>::new(graph_id),
                    parameters: BTreeMap::new().into(),
                    playing: true,
                }),
            )
            .unwrap();
        entity
    });

    let mut event_subscription = subscribe_animation_clip_events(&level);
    runtime.tick_level_seconds(&level, 0.3).unwrap();
    let events = drain_animation_clip_events(&level, &mut event_subscription);

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].entity, entity);
    assert_eq!(events[0].event, "graph_hit");
    assert_eq!(events[0].payload.as_deref(), Some("fast"));
    assert_eq!(events[0].clip_time_seconds, 0.5);
    assert_eq!(events[0].playback_time_seconds, 0.5);
}

#[test]
fn state_machine_player_emits_active_graph_clip_events() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let core = runtime.handle();
    let asset_manager = runtime_asset_manager(&core);
    let skeleton_uri = AssetUri::parse("res://animation/state-event.skeleton.zranim").unwrap();
    let clip_uri = AssetUri::parse("res://animation/state-event.clip.zranim").unwrap();
    let graph_uri = AssetUri::parse("res://animation/state-event.graph.zranim").unwrap();
    let machine_uri = AssetUri::parse("res://animation/state-event.machine.zranim").unwrap();
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let clip_id = ResourceId::from_locator(&clip_uri);
    let machine_id = ResourceId::from_locator(&machine_uri);

    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            skeleton_id,
            ResourceKind::AnimationSkeleton,
            skeleton_uri.clone(),
        ),
        two_bone_skeleton(),
    );
    let mut clip = single_hand_translation_clip(&skeleton_uri, 0.0);
    clip.event_tracks = vec![AnimationEventTrackAsset {
        target_id: Some("Root/Hand".to_string()),
        event: "state_hit".to_string(),
        time_seconds: 0.4,
        payload: Some("idle".to_string()),
    }];
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(clip_id, ResourceKind::AnimationClip, clip_uri.clone()),
        clip,
    );
    register_single_clip_graph(&asset_manager, &graph_uri, &clip_uri);
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            machine_id,
            ResourceKind::AnimationStateMachine,
            machine_uri.clone(),
        ),
        single_state_machine(&graph_uri),
    );

    let level = runtime.create_default_level().unwrap();
    let entity = level.with_world_mut(|world| {
        let entity = world
            .spawn_node(NodeKind::Cube)
            .expect("test node creation must succeed");
        world
            .set_animation_skeleton(
                entity,
                Some(AnimationSkeletonComponent {
                    skeleton: ResourceHandle::<AnimationSkeletonMarker>::new(skeleton_id),
                }),
            )
            .unwrap();
        world
            .set_animation_state_machine_player(
                entity,
                Some(AnimationStateMachinePlayerComponent {
                    state_machine: ResourceHandle::<AnimationStateMachineMarker>::new(machine_id),
                    parameters: BTreeMap::new().into(),
                    active_state: Some("Idle".to_string()),
                    playing: true,
                }),
            )
            .unwrap();
        entity
    });

    let mut event_subscription = subscribe_animation_clip_events(&level);
    runtime.tick_level_seconds(&level, 0.5).unwrap();
    let events = drain_animation_clip_events(&level, &mut event_subscription);

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].entity, entity);
    assert_eq!(events[0].target_id.as_deref(), Some("Root/Hand"));
    assert_eq!(events[0].event, "state_hit");
    assert_eq!(events[0].payload.as_deref(), Some("idle"));
    assert_eq!(events[0].clip_time_seconds, 0.4);
    assert_eq!(events[0].playback_time_seconds, 0.4);
}

#[test]
fn state_machine_transition_emits_from_and_to_graph_clip_events() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let core = runtime.handle();
    let asset_manager = runtime_asset_manager(&core);
    let skeleton_uri = AssetUri::parse("res://animation/transition-event.skeleton.zranim").unwrap();
    let idle_clip_uri =
        AssetUri::parse("res://animation/transition-event-idle.clip.zranim").unwrap();
    let run_clip_uri = AssetUri::parse("res://animation/transition-event-run.clip.zranim").unwrap();
    let idle_graph_uri =
        AssetUri::parse("res://animation/transition-event-idle.graph.zranim").unwrap();
    let run_graph_uri =
        AssetUri::parse("res://animation/transition-event-run.graph.zranim").unwrap();
    let machine_uri = AssetUri::parse("res://animation/transition-event.machine.zranim").unwrap();
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let idle_clip_id = ResourceId::from_locator(&idle_clip_uri);
    let run_clip_id = ResourceId::from_locator(&run_clip_uri);
    let machine_id = ResourceId::from_locator(&machine_uri);

    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            skeleton_id,
            ResourceKind::AnimationSkeleton,
            skeleton_uri.clone(),
        ),
        two_bone_skeleton(),
    );
    let mut idle_clip = single_hand_translation_clip(&skeleton_uri, 0.0);
    idle_clip.event_tracks = vec![AnimationEventTrackAsset {
        target_id: None,
        event: "idle_exit".to_string(),
        time_seconds: 0.05,
        payload: None,
    }];
    let mut run_clip = single_hand_translation_clip(&skeleton_uri, 10.0);
    run_clip.event_tracks = vec![AnimationEventTrackAsset {
        target_id: None,
        event: "run_enter".to_string(),
        time_seconds: 0.05,
        payload: None,
    }];
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            idle_clip_id,
            ResourceKind::AnimationClip,
            idle_clip_uri.clone(),
        ),
        idle_clip,
    );
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            run_clip_id,
            ResourceKind::AnimationClip,
            run_clip_uri.clone(),
        ),
        run_clip,
    );
    register_single_clip_graph(&asset_manager, &idle_graph_uri, &idle_clip_uri);
    register_single_clip_graph(&asset_manager, &run_graph_uri, &run_clip_uri);
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            machine_id,
            ResourceKind::AnimationStateMachine,
            machine_uri.clone(),
        ),
        timed_transition_state_machine(&idle_graph_uri, &run_graph_uri),
    );

    let level = runtime.create_default_level().unwrap();
    let entity = level.with_world_mut(|world| {
        let entity = world
            .spawn_node(NodeKind::Cube)
            .expect("test node creation must succeed");
        world
            .set_animation_skeleton(
                entity,
                Some(AnimationSkeletonComponent {
                    skeleton: ResourceHandle::<AnimationSkeletonMarker>::new(skeleton_id),
                }),
            )
            .unwrap();
        world
            .set_animation_state_machine_player(
                entity,
                Some(AnimationStateMachinePlayerComponent {
                    state_machine: ResourceHandle::<AnimationStateMachineMarker>::new(machine_id),
                    parameters: BTreeMap::from([(
                        "advance".to_string(),
                        AnimationParameterValue::Bool(true),
                    )])
                    .into(),
                    active_state: Some("Idle".to_string()),
                    playing: true,
                }),
            )
            .unwrap();
        entity
    });

    let mut event_subscription = subscribe_animation_clip_events(&level);
    runtime.tick_level_seconds(&level, 0.1).unwrap();
    let mut events = drain_animation_clip_events(&level, &mut event_subscription);
    events.sort_by(|a, b| a.event.cmp(&b.event));

    assert_eq!(events.len(), 2);
    assert!(events.iter().all(|event| event.entity == entity));
    assert_eq!(
        events
            .iter()
            .map(|event| (event.event.as_str(), event.clip_time_seconds))
            .collect::<Vec<_>>(),
        vec![("idle_exit", 0.05), ("run_enter", 0.05)]
    );
}

fn subscribe_animation_clip_events(
    level: &zircon_runtime::scene::LevelSystem,
) -> zircon_runtime::scene::EventSubscription<zircon_plugin_animation_runtime::AnimationClipEvent> {
    level.with_world_mut(|world| {
        let mut subscription = world
            .register_dormant_event_subscription::<
                zircon_plugin_animation_runtime::AnimationClipEvent,
            >();
        assert!(world.connect_event_subscription(&mut subscription));
        subscription
    })
}

fn drain_animation_clip_events(
    level: &zircon_runtime::scene::LevelSystem,
    subscription: &mut zircon_runtime::scene::EventSubscription<
        zircon_plugin_animation_runtime::AnimationClipEvent,
    >,
) -> Vec<zircon_plugin_animation_runtime::AnimationClipEvent> {
    level.with_world_mut(|world| {
        world.update_events::<zircon_plugin_animation_runtime::AnimationClipEvent>();
        world
            .read_event_subscription(subscription)
            .cloned()
            .collect()
    })
}
