mod paused_event {
    use super::{
        drain_animation_clip_events, register_single_clip_graph, runtime_asset_manager,
        runtime_with_physics_animation_scene_asset, single_hand_translation_clip,
        subscribe_animation_clip_events, timed_transition_state_machine, two_bone_skeleton,
        AnimationEventTrackAsset, AnimationParameterValue, AnimationSkeletonComponent,
        AnimationSkeletonMarker, AnimationStateMachineMarker, AnimationStateMachinePlayerComponent,
        AssetUri, BTreeMap, NodeKind, Real, ResourceHandle, ResourceId, ResourceKind,
        ResourceRecord,
    };
    use zircon_plugin_animation_runtime::AnimationClipEvent;
    use zircon_runtime::core::framework::animation::AnimationEventRecord;
    use zircon_runtime::scene::AnimationStateTransitionRuntime;

    const STEP: Real = 0.125;
    const FADE: Real = 0.25;
    const POINT: Real = 0.1875;
    const EVENT: &str = "run_mid";
    const TARGET: &str = "Root/Hand";
    const PAYLOAD: &str = "pause-observer";

    #[derive(Debug)]
    struct Frame {
        replacement_epoch: u64,
        playing: Option<bool>,
        toggled: bool,
        active_state: Option<String>,
        machine_time: Option<Real>,
        transition: Option<AnimationStateTransitionRuntime>,
        backlog_before: Option<usize>,
        backlog_after: Option<usize>,
        typed: Vec<AnimationClipEvent>,
        neutral: Vec<AnimationEventRecord>,
    }

    #[test]
    fn pause_fade_notify_resume_does_not_replay_completed_window() {
        let frames = observe("resume", &[true, false, true]);
        assert_phase(&frames[0], 0.125, true);
        assert_eq!(frames[1].machine_time, Some(0.125));
        assert_phase(&frames[2], 0.25, false);

        // The no-replay contract alone does not choose a fade policy while paused.
        let counts = point_counts(&frames);
        assert_eq!(counts[0], 0);
        assert_eq!(counts.iter().sum::<usize>(), 1, "point counts: {counts:?}");
    }

    #[test]
    fn pause_fade_notify_unpaused_finishes_once() {
        let frames = observe("unpaused", &[true, true, true]);
        assert_phase(&frames[0], 0.125, true);
        assert_phase(&frames[1], 0.25, false);
        assert_phase(&frames[2], 0.375, false);
        assert_eq!(point_counts(&frames), [0, 1, 0]);
    }

    #[test]
    fn pause_fade_notify_pause_after_completion_does_not_replay() {
        let frames = observe("pause_after_completion", &[true, true, false, true]);
        assert_phase(&frames[0], 0.125, true);
        assert_phase(&frames[1], 0.25, false);
        assert_phase(&frames[2], 0.25, false);
        assert_phase(&frames[3], 0.375, false);
        assert_eq!(point_counts(&frames), [0, 1, 0, 0]);
    }

    #[test]
    fn pause_fade_notify_clean_pause_retains_transition_and_target_clock() {
        let frames = observe("hold_active_transition", &[true, false, false, true]);
        // This case pins the freeze policy, stronger than just counting points.
        // F3 performs no setter/asset mutation. Dropping the active transition
        // here and restarting it after resume must not qualify as a fix.
        for frame in &frames[..3] {
            assert_phase(frame, 0.125, true);
        }
        assert_phase(&frames[3], 0.25, false);
        assert_eq!(point_counts(&frames), [0, 0, 0, 1]);
    }

    fn observe(case: &str, schedule: &[bool]) -> Vec<Frame> {
        let runtime = runtime_with_physics_animation_scene_asset();
        let assets = runtime_asset_manager(&runtime.handle());
        let uri = |name: &str| {
            AssetUri::parse(&format!("res://animation/pause-observer-{name}.zranim")).unwrap()
        };
        let skeleton = uri("skeleton");
        let idle_clip = uri("idle-clip");
        let run_clip = uri("run-clip");
        let idle_graph = uri("idle-graph");
        let run_graph = uri("run-graph");
        let machine = uri("machine");
        let skeleton_id = ResourceId::from_locator(&skeleton);
        let machine_id = ResourceId::from_locator(&machine);
        let resources = assets.resource_manager();
        resources.register_ready(
            ResourceRecord::new(
                skeleton_id,
                ResourceKind::AnimationSkeleton,
                skeleton.clone(),
            ),
            two_bone_skeleton(),
        );
        resources.register_ready(
            ResourceRecord::new(
                ResourceId::from_locator(&idle_clip),
                ResourceKind::AnimationClip,
                idle_clip.clone(),
            ),
            single_hand_translation_clip(&skeleton, 0.0),
        );
        let mut run = single_hand_translation_clip(&skeleton, 10.0);
        run.event_tracks.push(AnimationEventTrackAsset {
            target_id: Some(TARGET.into()),
            event: EVENT.into(),
            time_seconds: POINT,
            payload: Some(PAYLOAD.into()),
        });
        resources.register_ready(
            ResourceRecord::new(
                ResourceId::from_locator(&run_clip),
                ResourceKind::AnimationClip,
                run_clip.clone(),
            ),
            run,
        );
        register_single_clip_graph(&assets, &idle_graph, &idle_clip);
        register_single_clip_graph(&assets, &run_graph, &run_clip);
        let mut state_machine = timed_transition_state_machine(&idle_graph, &run_graph);
        state_machine.transitions[0].duration_seconds = FADE;
        resources.register_ready(
            ResourceRecord::new(machine_id, ResourceKind::AnimationStateMachine, machine),
            state_machine,
        );

        let level = runtime.create_default_level().unwrap();
        let entity = level.with_world_mut(|world| {
            let entity = world.spawn_node(NodeKind::Cube).unwrap();
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
                        state_machine: ResourceHandle::<AnimationStateMachineMarker>::new(
                            machine_id,
                        ),
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
        let epoch = level.capture_world_replacement_epoch();
        let mut clip_subscription = subscribe_animation_clip_events(&level);
        let mut neutral_subscription = level.with_world_mut(|world| {
            let mut subscription =
                world.register_dormant_event_subscription::<AnimationEventRecord>();
            assert!(world.connect_event_subscription(&mut subscription));
            subscription
        });

        let mut frames = Vec::with_capacity(schedule.len());
        for (index, &playing) in schedule.iter().enumerate() {
            let toggled = level.with_world_mut(|world| {
                let mut player = world
                    .animation_state_machine_player(entity)
                    .cloned()
                    .unwrap();
                if player.playing == playing {
                    return false;
                }
                player.playing = playing;
                world
                    .set_animation_state_machine_player(entity, Some(player))
                    .unwrap()
            });
            let backlog_before = level.animation_clip_event_backlog_len(epoch);
            runtime.tick_level_seconds(&level, f64::from(STEP)).unwrap();
            let backlog_after = level.animation_clip_event_backlog_len(epoch);
            let (machine_time, transition) = level.animation_playback_times(epoch).map_or(
                (None, None),
                |(_, times, transitions)| {
                    (
                        times.get(&entity).copied(),
                        transitions.get(&entity).cloned(),
                    )
                },
            );
            let (playing_now, active_state) = level.with_world(|world| {
                world
                    .animation_state_machine_player(entity)
                    .map_or((None, None), |player| {
                        (Some(player.playing), player.active_state.clone())
                    })
            });
            let typed = drain_animation_clip_events(&level, &mut clip_subscription);
            let neutral = level.with_world_mut(|world| {
                world.update_events::<AnimationEventRecord>();
                world
                    .read_event_subscription(&mut neutral_subscription)
                    .cloned()
                    .collect()
            });
            let frame = Frame {
                replacement_epoch: level.capture_world_replacement_epoch(),
                playing: playing_now,
                toggled,
                active_state,
                machine_time,
                transition,
                backlog_before,
                backlog_after,
                typed,
                neutral,
            };
            println!(
                "PAUSE_FADE_NOTIFY_V1 case={case} frame={} request_playing={playing} {frame:?}",
                index + 1
            );
            frames.push(frame);
        }

        // Capture both streams and all frames before inspecting any behavior.
        let typed_point = AnimationClipEvent {
            entity,
            target_id: Some(TARGET.into()),
            event: EVENT.into(),
            payload: Some(PAYLOAD.into()),
            clip_time_seconds: POINT,
            playback_time_seconds: POINT,
        };
        let neutral_point = AnimationEventRecord::new(entity, EVENT)
            .with_target_id(TARGET)
            .with_payload(PAYLOAD)
            .at_times(POINT, POINT);
        let mut previous = true;
        for (index, (frame, &request)) in frames.iter().zip(schedule).enumerate() {
            let frame_number = index + 1;
            assert_eq!(frame.replacement_epoch, epoch, "frame {frame_number}");
            assert_eq!(frame.playing, Some(request), "frame {frame_number}");
            assert_eq!(frame.toggled, request != previous, "frame {frame_number}");
            previous = request;
            assert_eq!(frame.backlog_before, Some(0), "frame {frame_number}");
            assert_eq!(frame.backlog_after, Some(0), "frame {frame_number}");
            assert_eq!(
                frame.typed.len(),
                frame.neutral.len(),
                "frame {frame_number}"
            );
            for evt in &frame.typed {
                assert_eq!(*evt, typed_point, "frame {frame_number}");
            }
            for evt in &frame.neutral {
                assert_eq!(*evt, neutral_point, "frame {frame_number}");
            }
        }
        frames
    }

    fn assert_phase(frame: &Frame, machine_time: Real, in_flight: bool) {
        assert_eq!(frame.machine_time, Some(machine_time));
        assert_eq!(
            frame.active_state.as_deref(),
            Some(if in_flight { "Idle" } else { "Run" })
        );
        let expected = in_flight.then(|| AnimationStateTransitionRuntime {
            from_state: "Idle".into(),
            to_state: "Run".into(),
            duration_seconds: 0.25,
            elapsed_seconds: 0.125,
            from_time_seconds: 0.125,
            to_time_seconds: 0.125,
        });
        assert_eq!(frame.transition, expected);
    }

    fn point_counts(frames: &[Frame]) -> Vec<usize> {
        frames.iter().map(|frame| frame.typed.len()).collect()
    }
}
