use std::collections::BTreeMap;
use zircon_runtime::asset::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationGraphAsset, AnimationGraphNodeAsset, AnimationGraphParameterAsset,
    AnimationInterpolationAsset, AnimationSequenceAsset, AnimationSequenceBindingAsset,
    AnimationSequenceTrackAsset, AnimationStateAsset, AnimationStateMachineAsset,
    AnimationStateTransitionAsset, AnimationTransitionConditionAsset,
};
use zircon_runtime::asset::{AnimationConditionOperatorAsset, AssetReference, AssetUri};
use zircon_runtime::core::framework::animation::{
    AnimationManager, AnimationParameterValue, AnimationPlaybackSettings,
};
use zircon_runtime::core::framework::scene::{ComponentPropertyPath, EntityPath};
use zircon_runtime::core::manager::{resolve_animation_manager, resolve_config_manager};
use zircon_runtime::core::math::{Quat, Vec3};
use zircon_runtime::core::resource::{AnimationClipMarker, ResourceHandle, ResourceId};
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::foundation::FOUNDATION_MODULE_NAME;
use zircon_runtime::scene::components::{AnimationPlayerComponent, NodeKind};
use zircon_runtime::scene::world::World;

mod clip_pose_guards;

#[test]
fn animation_plugin_registration_contributes_runtime_module() {
    let report = super::plugin_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == super::ANIMATION_MODULE_NAME));
    assert_eq!(
        report.package_manifest.modules[0].target_modes,
        vec![
            zircon_runtime::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::RuntimeTargetMode::EditorHost,
        ]
    );
}

#[test]
fn apply_sequence_to_world_resolves_track_paths_and_updates_scene_properties() {
    let mut world = World::new();
    let root = world.spawn_node(NodeKind::Cube);
    world.rename_node(root, "Root").unwrap();
    let hero = world.spawn_node(NodeKind::Mesh);
    world.rename_node(hero, "Hero").unwrap();
    world.set_parent_checked(hero, Some(root)).unwrap();
    world
        .set_animation_player(
            hero,
            Some(AnimationPlayerComponent {
                clip: ResourceHandle::<AnimationClipMarker>::new(ResourceId::from_stable_label(
                    "res://animation/hero.clip.zranim",
                )),
                playback_speed: 1.0,
                time_seconds: 0.0,
                weight: 0.0,
                looping: true,
                playing: true,
            }),
        )
        .unwrap();

    let sequence = AnimationSequenceAsset {
        name: Some("Locomotion".to_string()),
        duration_seconds: 1.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Root/Hero").unwrap(),
            tracks: vec![
                AnimationSequenceTrackAsset {
                    property_path: ComponentPropertyPath::parse("Transform.translation").unwrap(),
                    channel: AnimationChannelAsset {
                        interpolation: AnimationInterpolationAsset::Hermite,
                        keys: vec![
                            AnimationChannelKeyAsset {
                                time_seconds: 0.0,
                                value: AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0]),
                                in_tangent: Some(AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0])),
                                out_tangent: Some(AnimationChannelValueAsset::Vec3([
                                    0.0, 0.0, 0.0,
                                ])),
                            },
                            AnimationChannelKeyAsset {
                                time_seconds: 1.0,
                                value: AnimationChannelValueAsset::Vec3([10.0, 0.0, 0.0]),
                                in_tangent: Some(AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0])),
                                out_tangent: Some(AnimationChannelValueAsset::Vec3([
                                    0.0, 0.0, 0.0,
                                ])),
                            },
                        ],
                    },
                },
                AnimationSequenceTrackAsset {
                    property_path: ComponentPropertyPath::parse("AnimationPlayer.weight").unwrap(),
                    channel: AnimationChannelAsset {
                        interpolation: AnimationInterpolationAsset::Hermite,
                        keys: vec![
                            AnimationChannelKeyAsset {
                                time_seconds: 0.0,
                                value: AnimationChannelValueAsset::Scalar(0.0),
                                in_tangent: Some(AnimationChannelValueAsset::Scalar(0.0)),
                                out_tangent: Some(AnimationChannelValueAsset::Scalar(0.0)),
                            },
                            AnimationChannelKeyAsset {
                                time_seconds: 1.0,
                                value: AnimationChannelValueAsset::Scalar(1.0),
                                in_tangent: Some(AnimationChannelValueAsset::Scalar(0.0)),
                                out_tangent: Some(AnimationChannelValueAsset::Scalar(0.0)),
                            },
                        ],
                    },
                },
            ],
        }],
    };

    let report = super::apply_sequence_to_world(&mut world, &sequence, 0.5, false).unwrap();

    assert_eq!(report.applied_tracks.len(), 2);
    assert!(report.missing_tracks.is_empty());
    assert_eq!(
        world.find_node(hero).unwrap().transform.translation,
        Vec3::new(5.0, 0.0, 0.0)
    );
    assert_eq!(world.animation_player(hero).unwrap().weight, 0.5);
}

#[test]
fn apply_sequence_to_world_clamps_non_finite_timing_to_start() {
    let mut world = World::new();
    let hero = world.spawn_node(NodeKind::Mesh);
    world.rename_node(hero, "Hero").unwrap();

    let mut sequence = AnimationSequenceAsset {
        name: Some("Locomotion".to_string()),
        duration_seconds: f32::NAN,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Hero").unwrap(),
            tracks: vec![AnimationSequenceTrackAsset {
                property_path: ComponentPropertyPath::parse("Transform.translation").unwrap(),
                channel: AnimationChannelAsset {
                    interpolation: AnimationInterpolationAsset::Hermite,
                    keys: vec![
                        AnimationChannelKeyAsset {
                            time_seconds: 0.0,
                            value: AnimationChannelValueAsset::Vec3([0.0, 1.0, 0.0]),
                            in_tangent: Some(AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0])),
                            out_tangent: Some(AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0])),
                        },
                        AnimationChannelKeyAsset {
                            time_seconds: 1.0,
                            value: AnimationChannelValueAsset::Vec3([0.0, 2.0, 0.0]),
                            in_tangent: Some(AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0])),
                            out_tangent: Some(AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0])),
                        },
                    ],
                },
            }],
        }],
    };

    super::apply_sequence_to_world(&mut world, &sequence, 0.75, true).unwrap();
    assert_eq!(
        world.find_node(hero).unwrap().transform.translation,
        Vec3::new(0.0, 1.0, 0.0)
    );

    sequence.duration_seconds = 1.0;

    super::apply_sequence_to_world(&mut world, &sequence, f32::INFINITY, true).unwrap();
    assert_eq!(
        world.find_node(hero).unwrap().transform.translation,
        Vec3::new(0.0, 1.0, 0.0)
    );
}

#[test]
fn apply_sequence_to_world_rejects_non_finite_channel_values() {
    let mut world = World::new();
    let hero = world.spawn_node(NodeKind::Mesh);
    world.rename_node(hero, "Hero").unwrap();

    let sequence = AnimationSequenceAsset {
        name: Some("BadLocomotion".to_string()),
        duration_seconds: 1.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Hero").unwrap(),
            tracks: vec![AnimationSequenceTrackAsset {
                property_path: ComponentPropertyPath::parse("Transform.translation").unwrap(),
                channel: AnimationChannelAsset {
                    interpolation: AnimationInterpolationAsset::Hermite,
                    keys: vec![
                        AnimationChannelKeyAsset {
                            time_seconds: 0.0,
                            value: AnimationChannelValueAsset::Vec3([0.0, f32::NAN, 0.0]),
                            in_tangent: Some(AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0])),
                            out_tangent: Some(AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0])),
                        },
                        AnimationChannelKeyAsset {
                            time_seconds: 1.0,
                            value: AnimationChannelValueAsset::Vec3([0.0, 2.0, 0.0]),
                            in_tangent: Some(AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0])),
                            out_tangent: Some(AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0])),
                        },
                    ],
                },
            }],
        }],
    };

    match super::apply_sequence_to_world(&mut world, &sequence, 0.0, true) {
        Ok(report) => {
            panic!("expected non-finite sequence channel value to be rejected, got {report:?}")
        }
        Err(error) => assert!(error.contains("non-finite"), "{error}"),
    }
    assert_eq!(
        world.find_node(hero).unwrap().transform.translation,
        Vec3::ZERO
    );
}

#[test]
fn apply_sequence_to_world_skips_channel_with_non_finite_key_times() {
    let mut world = World::new();
    let hero = world.spawn_node(NodeKind::Mesh);
    world.rename_node(hero, "Hero").unwrap();

    let sequence = AnimationSequenceAsset {
        name: Some("BadLocomotion".to_string()),
        duration_seconds: 1.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Hero").unwrap(),
            tracks: vec![AnimationSequenceTrackAsset {
                property_path: ComponentPropertyPath::parse("Transform.translation").unwrap(),
                channel: AnimationChannelAsset {
                    interpolation: AnimationInterpolationAsset::Step,
                    keys: vec![
                        AnimationChannelKeyAsset {
                            time_seconds: f32::NAN,
                            value: AnimationChannelValueAsset::Vec3([0.0, 9.0, 0.0]),
                            in_tangent: None,
                            out_tangent: None,
                        },
                        AnimationChannelKeyAsset {
                            time_seconds: 1.0,
                            value: AnimationChannelValueAsset::Vec3([0.0, 2.0, 0.0]),
                            in_tangent: None,
                            out_tangent: None,
                        },
                    ],
                },
            }],
        }],
    };

    let report = super::apply_sequence_to_world(&mut world, &sequence, 0.5, true).unwrap();

    assert!(report.applied_tracks.is_empty());
    assert_eq!(report.missing_tracks.len(), 1);
    assert_eq!(
        world.find_node(hero).unwrap().transform.translation,
        Vec3::ZERO
    );
}

#[test]
fn apply_sequence_to_world_rejects_zero_length_quaternion_channel_values() {
    let mut world = World::new();
    let hero = world.spawn_node(NodeKind::Mesh);
    world.rename_node(hero, "Hero").unwrap();

    let sequence = AnimationSequenceAsset {
        name: Some("BadRotation".to_string()),
        duration_seconds: 1.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Hero").unwrap(),
            tracks: vec![AnimationSequenceTrackAsset {
                property_path: ComponentPropertyPath::parse("Transform.rotation").unwrap(),
                channel: AnimationChannelAsset {
                    interpolation: AnimationInterpolationAsset::Hermite,
                    keys: vec![
                        AnimationChannelKeyAsset {
                            time_seconds: 0.0,
                            value: AnimationChannelValueAsset::Quaternion([0.0, 0.0, 0.0, 0.0]),
                            in_tangent: None,
                            out_tangent: None,
                        },
                        AnimationChannelKeyAsset {
                            time_seconds: 1.0,
                            value: AnimationChannelValueAsset::Quaternion([0.0, 0.0, 0.0, 0.0]),
                            in_tangent: None,
                            out_tangent: None,
                        },
                    ],
                },
            }],
        }],
    };

    match super::apply_sequence_to_world(&mut world, &sequence, 0.0, true) {
        Ok(report) => {
            panic!("expected zero-length sequence quaternion to be rejected, got {report:?}")
        }
        Err(error) => assert!(error.contains("zero-length"), "{error}"),
    }
    assert_eq!(
        world.find_node(hero).unwrap().transform.rotation,
        Quat::IDENTITY
    );
}

#[test]
fn animation_manager_persists_playback_settings_to_runtime_config() {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(zircon_runtime::foundation::module_descriptor())
        .unwrap();
    runtime.register_module(super::module_descriptor()).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    runtime
        .activate_module(super::ANIMATION_MODULE_NAME)
        .unwrap();

    let manager = runtime
        .handle()
        .resolve_manager::<super::DefaultAnimationManager>(super::DEFAULT_ANIMATION_MANAGER_NAME)
        .unwrap();
    let playback_settings = AnimationPlaybackSettings {
        enabled: true,
        property_tracks: true,
        skeletal_clips: false,
        graphs: false,
        state_machines: true,
    };

    manager
        .store_playback_settings(playback_settings.clone())
        .unwrap();

    let resolved_manager = resolve_animation_manager(&runtime.handle()).unwrap();
    assert_eq!(resolved_manager.playback_settings(), playback_settings);

    let config = resolve_config_manager(&runtime.handle()).unwrap();
    assert!(config
        .get_value(super::ANIMATION_PLAYBACK_CONFIG_KEY)
        .is_some());
}

#[test]
fn animation_manager_evaluates_graphs_and_parameter_overrides() {
    let manager = super::DefaultAnimationManager::default();
    let graph = AnimationGraphAsset {
        name: Some("HeroGraph".to_string()),
        parameters: vec![
            AnimationGraphParameterAsset {
                name: "speed".to_string(),
                default_value: AnimationParameterValue::Scalar(0.25),
            },
            AnimationGraphParameterAsset {
                name: "grounded".to_string(),
                default_value: AnimationParameterValue::Bool(true),
            },
        ],
        nodes: vec![
            AnimationGraphNodeAsset::Clip {
                id: "idle".to_string(),
                clip: asset_reference("res://animation/hero_idle.clip.zranim"),
                playback_speed: 1.0,
                looping: true,
            },
            AnimationGraphNodeAsset::Output {
                source: "idle".to_string(),
            },
        ],
    };

    let mut parameters = manager.parameter_defaults(&graph);
    assert_eq!(
        manager.parameter_value(&parameters, "speed"),
        Some(AnimationParameterValue::Scalar(0.25))
    );

    manager.set_parameter(
        &mut parameters,
        "speed",
        AnimationParameterValue::Scalar(0.75),
    );
    let evaluation = manager.evaluate_graph(&graph, &parameters);

    assert_eq!(
        evaluation.parameters.get("speed"),
        Some(&AnimationParameterValue::Scalar(0.75))
    );
    assert_eq!(evaluation.output_node.as_deref(), Some("idle"));
    assert_eq!(evaluation.clips.len(), 1);
    assert_eq!(
        evaluation.clips[0].clip.locator.to_string(),
        "res://animation/hero_idle.clip.zranim"
    );
    assert_eq!(evaluation.clips[0].weight, 1.0);
}

#[test]
fn animation_manager_evaluates_graph_blend_with_non_finite_weight_as_default() {
    let manager = super::DefaultAnimationManager::default();
    let graph = AnimationGraphAsset {
        name: Some("HeroGraph".to_string()),
        parameters: vec![AnimationGraphParameterAsset {
            name: "blend".to_string(),
            default_value: AnimationParameterValue::Scalar(0.0),
        }],
        nodes: vec![
            AnimationGraphNodeAsset::Clip {
                id: "idle".to_string(),
                clip: asset_reference("res://animation/hero_idle.clip.zranim"),
                playback_speed: 1.0,
                looping: true,
            },
            AnimationGraphNodeAsset::Clip {
                id: "run".to_string(),
                clip: asset_reference("res://animation/hero_run.clip.zranim"),
                playback_speed: 1.0,
                looping: true,
            },
            AnimationGraphNodeAsset::Blend {
                id: "blend_node".to_string(),
                inputs: vec!["idle".to_string(), "run".to_string()],
                weight_parameter: Some("blend".to_string()),
            },
            AnimationGraphNodeAsset::Output {
                source: "blend_node".to_string(),
            },
        ],
    };

    let evaluation = manager.evaluate_graph(
        &graph,
        &BTreeMap::from([(
            "blend".to_string(),
            AnimationParameterValue::Scalar(f32::NAN),
        )]),
    );

    assert_eq!(evaluation.clips.len(), 2);
    assert!(evaluation.clips.iter().all(|clip| clip.weight.is_finite()));
    assert_eq!(evaluation.clips[0].weight, 1.0);
    assert_eq!(evaluation.clips[1].weight, 0.0);
}

#[test]
fn animation_manager_evaluates_graph_clip_with_non_finite_playback_speed_as_default() {
    let manager = super::DefaultAnimationManager::default();
    let graph = AnimationGraphAsset {
        name: Some("MalformedGraph".to_string()),
        parameters: Vec::new(),
        nodes: vec![
            AnimationGraphNodeAsset::Clip {
                id: "idle".to_string(),
                clip: asset_reference("res://animation/hero_idle.clip.zranim"),
                playback_speed: f32::NAN,
                looping: true,
            },
            AnimationGraphNodeAsset::Clip {
                id: "run".to_string(),
                clip: asset_reference("res://animation/hero_run.clip.zranim"),
                playback_speed: f32::INFINITY,
                looping: true,
            },
            AnimationGraphNodeAsset::Blend {
                id: "blend_node".to_string(),
                inputs: vec!["idle".to_string(), "run".to_string()],
                weight_parameter: None,
            },
            AnimationGraphNodeAsset::Output {
                source: "blend_node".to_string(),
            },
        ],
    };

    let evaluation = manager.evaluate_graph(&graph, &BTreeMap::new());

    assert_eq!(evaluation.clips.len(), 2);
    assert!(evaluation
        .clips
        .iter()
        .all(|clip| clip.playback_speed == 1.0));
}

#[test]
fn animation_manager_rejects_non_finite_parameter_set() {
    let manager = super::DefaultAnimationManager::default();
    let mut parameters =
        BTreeMap::from([("speed".to_string(), AnimationParameterValue::Scalar(0.5))]);

    manager.set_parameter(
        &mut parameters,
        "speed",
        AnimationParameterValue::Scalar(f32::NAN),
    );
    manager.set_parameter(
        &mut parameters,
        "blend",
        AnimationParameterValue::Scalar(f32::INFINITY),
    );

    assert_eq!(
        manager.parameter_value(&parameters, "speed"),
        Some(AnimationParameterValue::Scalar(0.5))
    );
    assert_eq!(manager.parameter_value(&parameters, "blend"), None);
}

#[test]
fn animation_manager_rejects_non_finite_parameter_defaults() {
    let manager = super::DefaultAnimationManager::default();
    let graph = AnimationGraphAsset {
        name: Some("MalformedGraph".to_string()),
        parameters: vec![
            AnimationGraphParameterAsset {
                name: "speed".to_string(),
                default_value: AnimationParameterValue::Scalar(f32::NAN),
            },
            AnimationGraphParameterAsset {
                name: "direction".to_string(),
                default_value: AnimationParameterValue::Vec3([0.0, f32::INFINITY, 1.0]),
            },
            AnimationGraphParameterAsset {
                name: "grounded".to_string(),
                default_value: AnimationParameterValue::Bool(true),
            },
        ],
        nodes: Vec::new(),
    };

    let parameters = manager.parameter_defaults(&graph);

    assert_eq!(manager.parameter_value(&parameters, "speed"), None);
    assert_eq!(manager.parameter_value(&parameters, "direction"), None);
    assert_eq!(
        manager.parameter_value(&parameters, "grounded"),
        Some(AnimationParameterValue::Bool(true))
    );
}

#[test]
fn animation_manager_resolves_state_machine_transitions_from_parameters() {
    let manager = super::DefaultAnimationManager::default();
    let state_machine = AnimationStateMachineAsset {
        name: Some("HeroStateMachine".to_string()),
        entry_state: "Idle".to_string(),
        states: vec![
            AnimationStateAsset {
                name: "Idle".to_string(),
                graph: asset_reference("res://animation/idle.graph.zranim"),
            },
            AnimationStateAsset {
                name: "Jump".to_string(),
                graph: asset_reference("res://animation/jump.graph.zranim"),
            },
        ],
        transitions: vec![AnimationStateTransitionAsset {
            from_state: "Idle".to_string(),
            to_state: "Jump".to_string(),
            duration_seconds: 0.1,
            conditions: vec![AnimationTransitionConditionAsset {
                parameter: "grounded".to_string(),
                operator: AnimationConditionOperatorAsset::Equal,
                value: Some(AnimationParameterValue::Bool(false)),
            }],
        }],
    };

    let evaluation = manager.evaluate_state_machine(
        &state_machine,
        Some("Idle"),
        &BTreeMap::from([("grounded".to_string(), AnimationParameterValue::Bool(false))]),
    );

    assert_eq!(evaluation.active_state.as_deref(), Some("Jump"));
    assert!(evaluation.transitioned);
    assert_eq!(
        evaluation
            .graph
            .as_ref()
            .map(|graph| graph.locator.to_string())
            .as_deref(),
        Some("res://animation/jump.graph.zranim")
    );
}

#[test]
fn animation_manager_falls_back_to_entry_state_when_current_state_is_stale() {
    let manager = super::DefaultAnimationManager::default();
    let state_machine = AnimationStateMachineAsset {
        name: Some("HeroStateMachine".to_string()),
        entry_state: "Idle".to_string(),
        states: vec![
            AnimationStateAsset {
                name: "Idle".to_string(),
                graph: asset_reference("res://animation/idle.graph.zranim"),
            },
            AnimationStateAsset {
                name: "Jump".to_string(),
                graph: asset_reference("res://animation/jump.graph.zranim"),
            },
        ],
        transitions: Vec::new(),
    };

    let evaluation =
        manager.evaluate_state_machine(&state_machine, Some("RemovedState"), &BTreeMap::new());

    assert_eq!(evaluation.active_state.as_deref(), Some("Idle"));
    assert!(!evaluation.transitioned);
    assert_eq!(
        evaluation
            .graph
            .as_ref()
            .map(|graph| graph.locator.to_string())
            .as_deref(),
        Some("res://animation/idle.graph.zranim")
    );
}

#[test]
fn animation_manager_ignores_transition_when_target_state_is_missing() {
    let manager = super::DefaultAnimationManager::default();
    let state_machine = AnimationStateMachineAsset {
        name: Some("HeroStateMachine".to_string()),
        entry_state: "Idle".to_string(),
        states: vec![AnimationStateAsset {
            name: "Idle".to_string(),
            graph: asset_reference("res://animation/idle.graph.zranim"),
        }],
        transitions: vec![AnimationStateTransitionAsset {
            from_state: "Idle".to_string(),
            to_state: "RemovedState".to_string(),
            duration_seconds: 0.1,
            conditions: vec![AnimationTransitionConditionAsset {
                parameter: "jump".to_string(),
                operator: AnimationConditionOperatorAsset::Triggered,
                value: None,
            }],
        }],
    };

    let evaluation = manager.evaluate_state_machine(
        &state_machine,
        Some("Idle"),
        &BTreeMap::from([("jump".to_string(), AnimationParameterValue::Trigger)]),
    );

    assert_eq!(evaluation.active_state.as_deref(), Some("Idle"));
    assert!(!evaluation.transitioned);
    assert_eq!(
        evaluation
            .graph
            .as_ref()
            .map(|graph| graph.locator.to_string())
            .as_deref(),
        Some("res://animation/idle.graph.zranim")
    );
}

#[test]
fn animation_manager_ignores_state_machine_transition_when_condition_parameter_is_missing() {
    let manager = super::DefaultAnimationManager::default();
    let state_machine = AnimationStateMachineAsset {
        name: Some("HeroStateMachine".to_string()),
        entry_state: "Idle".to_string(),
        states: vec![
            AnimationStateAsset {
                name: "Idle".to_string(),
                graph: asset_reference("res://animation/idle.graph.zranim"),
            },
            AnimationStateAsset {
                name: "Run".to_string(),
                graph: asset_reference("res://animation/run.graph.zranim"),
            },
        ],
        transitions: vec![AnimationStateTransitionAsset {
            from_state: "Idle".to_string(),
            to_state: "Run".to_string(),
            duration_seconds: 0.1,
            conditions: vec![AnimationTransitionConditionAsset {
                parameter: "speed".to_string(),
                operator: AnimationConditionOperatorAsset::NotEqual,
                value: Some(AnimationParameterValue::Scalar(0.0)),
            }],
        }],
    };

    let evaluation = manager.evaluate_state_machine(&state_machine, Some("Idle"), &BTreeMap::new());

    assert_eq!(evaluation.active_state.as_deref(), Some("Idle"));
    assert!(!evaluation.transitioned);
    assert_eq!(
        evaluation
            .graph
            .as_ref()
            .map(|graph| graph.locator.to_string())
            .as_deref(),
        Some("res://animation/idle.graph.zranim")
    );
}

#[test]
fn animation_manager_ignores_state_machine_transition_when_comparison_value_is_missing() {
    let manager = super::DefaultAnimationManager::default();
    let state_machine = AnimationStateMachineAsset {
        name: Some("HeroStateMachine".to_string()),
        entry_state: "Idle".to_string(),
        states: vec![
            AnimationStateAsset {
                name: "Idle".to_string(),
                graph: asset_reference("res://animation/idle.graph.zranim"),
            },
            AnimationStateAsset {
                name: "Run".to_string(),
                graph: asset_reference("res://animation/run.graph.zranim"),
            },
        ],
        transitions: vec![AnimationStateTransitionAsset {
            from_state: "Idle".to_string(),
            to_state: "Run".to_string(),
            duration_seconds: 0.1,
            conditions: vec![AnimationTransitionConditionAsset {
                parameter: "speed".to_string(),
                operator: AnimationConditionOperatorAsset::NotEqual,
                value: None,
            }],
        }],
    };

    let evaluation = manager.evaluate_state_machine(
        &state_machine,
        Some("Idle"),
        &BTreeMap::from([("speed".to_string(), AnimationParameterValue::Scalar(1.0))]),
    );

    assert_eq!(evaluation.active_state.as_deref(), Some("Idle"));
    assert!(!evaluation.transitioned);
    assert_eq!(
        evaluation
            .graph
            .as_ref()
            .map(|graph| graph.locator.to_string())
            .as_deref(),
        Some("res://animation/idle.graph.zranim")
    );
}

#[test]
fn animation_manager_ignores_state_machine_transition_with_non_finite_parameter() {
    let manager = super::DefaultAnimationManager::default();
    let state_machine = AnimationStateMachineAsset {
        name: Some("HeroStateMachine".to_string()),
        entry_state: "Idle".to_string(),
        states: vec![
            AnimationStateAsset {
                name: "Idle".to_string(),
                graph: asset_reference("res://animation/idle.graph.zranim"),
            },
            AnimationStateAsset {
                name: "Run".to_string(),
                graph: asset_reference("res://animation/run.graph.zranim"),
            },
        ],
        transitions: vec![AnimationStateTransitionAsset {
            from_state: "Idle".to_string(),
            to_state: "Run".to_string(),
            duration_seconds: 0.1,
            conditions: vec![AnimationTransitionConditionAsset {
                parameter: "speed".to_string(),
                operator: AnimationConditionOperatorAsset::NotEqual,
                value: Some(AnimationParameterValue::Scalar(0.0)),
            }],
        }],
    };

    let evaluation = manager.evaluate_state_machine(
        &state_machine,
        Some("Idle"),
        &BTreeMap::from([(
            "speed".to_string(),
            AnimationParameterValue::Scalar(f32::NAN),
        )]),
    );

    assert_eq!(evaluation.active_state.as_deref(), Some("Idle"));
    assert!(!evaluation.transitioned);
    assert_eq!(
        evaluation
            .graph
            .as_ref()
            .map(|graph| graph.locator.to_string())
            .as_deref(),
        Some("res://animation/idle.graph.zranim")
    );
}

fn asset_reference(locator: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(locator).unwrap())
}

fn vec3_channel(keys: [(f32, [f32; 3]); 2]) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Hermite,
        keys: keys
            .into_iter()
            .map(|(time_seconds, value)| AnimationChannelKeyAsset {
                time_seconds,
                value: AnimationChannelValueAsset::Vec3(value),
                in_tangent: Some(AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0])),
                out_tangent: Some(AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0])),
            })
            .collect(),
    }
}

fn quaternion_channel(keys: [(f32, [f32; 4]); 2]) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Hermite,
        keys: keys
            .into_iter()
            .map(|(time_seconds, value)| AnimationChannelKeyAsset {
                time_seconds,
                value: AnimationChannelValueAsset::Quaternion(value),
                in_tangent: None,
                out_tangent: None,
            })
            .collect(),
    }
}
