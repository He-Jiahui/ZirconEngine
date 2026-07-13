use super::state_machine_interruption::{assert_hand_translation, spawn_state_machine_player, uri};
use super::*;
use zircon_runtime::core::framework::animation::{
    AnimationBlendSpace1DAsset, AnimationBlendSpace1DSampleAsset, AnimationBlendSpace2DAsset,
    AnimationBlendSpace2DSampleAsset, AnimationConditionOperatorAsset, AnimationStateAsset,
    AnimationStateKindAsset, AnimationStateMachineAsset, AnimationStateMachineLayerAsset,
    AnimationStateMachineLayerBlendModeAsset, AnimationStateTransitionAsset,
    AnimationTransitionConditionAsset,
};
use zircon_runtime::core::math::Vec2;

#[test]
fn blend_space_1d_state_blends_production_graph_poses() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let asset_manager = runtime_asset_manager(&runtime.handle());
    let skeleton_uri = uri("state-blend-space.skeleton");
    let idle_clip_uri = uri("state-blend-space-idle.clip");
    let run_clip_uri = uri("state-blend-space-run.clip");
    let idle_graph_uri = uri("state-blend-space-idle.graph");
    let run_graph_uri = uri("state-blend-space-run.graph");
    let machine_uri = uri("state-blend-space.machine");
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let machine_id = ResourceId::from_locator(&machine_uri);

    register_animation_blend_assets(&asset_manager, &skeleton_uri, &idle_clip_uri, &run_clip_uri);
    register_single_clip_graph(&asset_manager, &idle_graph_uri, &idle_clip_uri);
    register_single_clip_graph(&asset_manager, &run_graph_uri, &run_clip_uri);
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(machine_id, ResourceKind::AnimationStateMachine, machine_uri),
        AnimationStateMachineAsset {
            name: Some("BlendSpaceState".to_string()),
            entry_state: "Idle".to_string(),
            states: vec![AnimationStateAsset {
                name: "Idle".to_string(),
                kind: AnimationStateKindAsset::BlendSpace1D(AnimationBlendSpace1DAsset {
                    parameter: "speed".to_string(),
                    samples: vec![
                        AnimationBlendSpace1DSampleAsset {
                            position: 0.0,
                            graph: AssetReference::from_locator(idle_graph_uri),
                        },
                        AnimationBlendSpace1DSampleAsset {
                            position: 1.0,
                            graph: AssetReference::from_locator(run_graph_uri),
                        },
                    ],
                }),
            }],
            transitions: Vec::new(),
            layers: Vec::new(),
        },
    );

    let level = runtime.create_default_level().unwrap();
    let entity = spawn_state_machine_player(
        &level,
        skeleton_id,
        machine_id,
        BTreeMap::from([("speed".to_string(), AnimationParameterValue::Scalar(0.25))]),
    );
    runtime.tick_level_seconds(&level, 0.0).unwrap();

    assert_hand_translation(&level, entity, 2.5);
}

#[test]
fn blend_space_2d_state_blends_production_graph_poses() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let asset_manager = runtime_asset_manager(&runtime.handle());
    let skeleton_uri = uri("state-blend-space-2d.skeleton");
    let idle_clip_uri = uri("state-blend-space-2d-idle.clip");
    let run_clip_uri = uri("state-blend-space-2d-run.clip");
    let jump_clip_uri = uri("state-blend-space-2d-jump.clip");
    let idle_graph_uri = uri("state-blend-space-2d-idle.graph");
    let run_graph_uri = uri("state-blend-space-2d-run.graph");
    let jump_graph_uri = uri("state-blend-space-2d-jump.graph");
    let machine_uri = uri("state-blend-space-2d.machine");
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let machine_id = ResourceId::from_locator(&machine_uri);

    register_animation_blend_assets(&asset_manager, &skeleton_uri, &idle_clip_uri, &run_clip_uri);
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            ResourceId::from_locator(&jump_clip_uri),
            ResourceKind::AnimationClip,
            jump_clip_uri.clone(),
        ),
        single_hand_translation_clip(&skeleton_uri, 20.0),
    );
    register_single_clip_graph(&asset_manager, &idle_graph_uri, &idle_clip_uri);
    register_single_clip_graph(&asset_manager, &run_graph_uri, &run_clip_uri);
    register_single_clip_graph(&asset_manager, &jump_graph_uri, &jump_clip_uri);
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(machine_id, ResourceKind::AnimationStateMachine, machine_uri),
        AnimationStateMachineAsset {
            name: Some("BlendSpace2DState".to_string()),
            entry_state: "Move".to_string(),
            states: vec![AnimationStateAsset {
                name: "Move".to_string(),
                kind: AnimationStateKindAsset::BlendSpace2D(AnimationBlendSpace2DAsset {
                    parameter: "direction".to_string(),
                    samples: vec![
                        AnimationBlendSpace2DSampleAsset {
                            position: Vec2::new(0.0, 0.0),
                            graph: AssetReference::from_locator(idle_graph_uri),
                        },
                        AnimationBlendSpace2DSampleAsset {
                            position: Vec2::new(1.0, 0.0),
                            graph: AssetReference::from_locator(run_graph_uri),
                        },
                        AnimationBlendSpace2DSampleAsset {
                            position: Vec2::new(0.0, 1.0),
                            graph: AssetReference::from_locator(jump_graph_uri),
                        },
                    ],
                }),
            }],
            transitions: Vec::new(),
            layers: Vec::new(),
        },
    );

    let level = runtime.create_default_level().unwrap();
    let entity = spawn_state_machine_player(
        &level,
        skeleton_id,
        machine_id,
        BTreeMap::from([(
            "direction".to_string(),
            AnimationParameterValue::Vec2([0.25, 0.25]),
        )]),
    );
    runtime.tick_level_seconds(&level, 0.0).unwrap();

    assert_hand_translation(&level, entity, 7.5);
}

#[test]
fn direct_clip_state_samples_production_pose() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let asset_manager = runtime_asset_manager(&runtime.handle());
    let skeleton_uri = uri("state-direct-clip.skeleton");
    let clip_uri = uri("state-direct-clip.clip");
    let unused_clip_uri = uri("state-direct-clip-unused.clip");
    let machine_uri = uri("state-direct-clip.machine");
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let machine_id = ResourceId::from_locator(&machine_uri);

    register_animation_blend_assets(&asset_manager, &skeleton_uri, &unused_clip_uri, &clip_uri);
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(machine_id, ResourceKind::AnimationStateMachine, machine_uri),
        AnimationStateMachineAsset {
            name: Some("DirectClipState".to_string()),
            entry_state: "Clip".to_string(),
            states: vec![AnimationStateAsset {
                name: "Clip".to_string(),
                kind: AnimationStateKindAsset::Clip {
                    clip: AssetReference::from_locator(clip_uri),
                },
            }],
            transitions: Vec::new(),
            layers: Vec::new(),
        },
    );

    let level = runtime.create_default_level().unwrap();
    let entity = spawn_state_machine_player(&level, skeleton_id, machine_id, BTreeMap::new());
    runtime.tick_level_seconds(&level, 0.0).unwrap();

    assert_hand_translation(&level, entity, 10.0);
}

#[test]
fn sub_machine_state_delegates_to_nested_production_machine() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let asset_manager = runtime_asset_manager(&runtime.handle());
    let skeleton_uri = uri("state-sub-machine.skeleton");
    let idle_clip_uri = uri("state-sub-machine-idle.clip");
    let run_clip_uri = uri("state-sub-machine-run.clip");
    let run_graph_uri = uri("state-sub-machine-run.graph");
    let child_uri = uri("state-sub-machine-child.machine");
    let parent_uri = uri("state-sub-machine-parent.machine");
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let child_id = ResourceId::from_locator(&child_uri);
    let parent_id = ResourceId::from_locator(&parent_uri);

    register_animation_blend_assets(&asset_manager, &skeleton_uri, &idle_clip_uri, &run_clip_uri);
    register_single_clip_graph(&asset_manager, &run_graph_uri, &run_clip_uri);
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            child_id,
            ResourceKind::AnimationStateMachine,
            child_uri.clone(),
        ),
        AnimationStateMachineAsset {
            name: Some("Child".to_string()),
            entry_state: "Run".to_string(),
            states: vec![AnimationStateAsset::graph_ref(
                "Run",
                AssetReference::from_locator(run_graph_uri),
            )],
            transitions: Vec::new(),
            layers: Vec::new(),
        },
    );
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(parent_id, ResourceKind::AnimationStateMachine, parent_uri),
        AnimationStateMachineAsset {
            name: Some("Parent".to_string()),
            entry_state: "Nested".to_string(),
            states: vec![AnimationStateAsset {
                name: "Nested".to_string(),
                kind: AnimationStateKindAsset::SubMachine {
                    state_machine: AssetReference::from_locator(child_uri),
                },
            }],
            transitions: Vec::new(),
            layers: Vec::new(),
        },
    );

    let level = runtime.create_default_level().unwrap();
    let entity = spawn_state_machine_player(&level, skeleton_id, parent_id, BTreeMap::new());
    runtime.tick_level_seconds(&level, 0.0).unwrap();

    assert_hand_translation(&level, entity, 10.0);
}

#[test]
fn sub_machine_transition_state_persists_across_production_ticks() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let asset_manager = runtime_asset_manager(&runtime.handle());
    let skeleton_uri = uri("state-sub-transition.skeleton");
    let idle_clip_uri = uri("state-sub-transition-idle.clip");
    let run_clip_uri = uri("state-sub-transition-run.clip");
    let idle_graph_uri = uri("state-sub-transition-idle.graph");
    let run_graph_uri = uri("state-sub-transition-run.graph");
    let child_uri = uri("state-sub-transition-child.machine");
    let parent_uri = uri("state-sub-transition-parent.machine");
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let child_id = ResourceId::from_locator(&child_uri);
    let parent_id = ResourceId::from_locator(&parent_uri);

    register_animation_blend_assets(&asset_manager, &skeleton_uri, &idle_clip_uri, &run_clip_uri);
    register_single_clip_graph(&asset_manager, &idle_graph_uri, &idle_clip_uri);
    register_single_clip_graph(&asset_manager, &run_graph_uri, &run_clip_uri);
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            child_id,
            ResourceKind::AnimationStateMachine,
            child_uri.clone(),
        ),
        AnimationStateMachineAsset {
            name: Some("TransitionChild".to_string()),
            entry_state: "Idle".to_string(),
            states: vec![
                AnimationStateAsset::graph_ref(
                    "Idle",
                    AssetReference::from_locator(idle_graph_uri),
                ),
                AnimationStateAsset::graph_ref("Run", AssetReference::from_locator(run_graph_uri)),
            ],
            transitions: vec![AnimationStateTransitionAsset {
                from_state: "Idle".to_string(),
                to_state: "Run".to_string(),
                duration_seconds: 0.2,
                exit_time: None,
                interruption: Default::default(),
                conditions: vec![AnimationTransitionConditionAsset {
                    parameter: "go".to_string(),
                    operator: AnimationConditionOperatorAsset::Equal,
                    value: Some(AnimationParameterValue::Bool(true)),
                }],
            }],
            layers: Vec::new(),
        },
    );
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(parent_id, ResourceKind::AnimationStateMachine, parent_uri),
        AnimationStateMachineAsset {
            name: Some("TransitionParent".to_string()),
            entry_state: "Nested".to_string(),
            states: vec![AnimationStateAsset {
                name: "Nested".to_string(),
                kind: AnimationStateKindAsset::SubMachine {
                    state_machine: AssetReference::from_locator(child_uri),
                },
            }],
            transitions: Vec::new(),
            layers: Vec::new(),
        },
    );

    let level = runtime.create_default_level().unwrap();
    let entity = spawn_state_machine_player(
        &level,
        skeleton_id,
        parent_id,
        BTreeMap::from([("go".to_string(), AnimationParameterValue::Bool(true))]),
    );
    runtime.tick_level_seconds(&level, 0.1).unwrap();
    assert_hand_translation(&level, entity, 5.0);

    runtime.tick_level_seconds(&level, 0.1).unwrap();
    assert_hand_translation(&level, entity, 10.0);
}

#[test]
fn parent_transition_can_leave_sub_machine_state() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let asset_manager = runtime_asset_manager(&runtime.handle());
    let skeleton_uri = uri("state-sub-parent-transition.skeleton");
    let idle_clip_uri = uri("state-sub-parent-transition-idle.clip");
    let run_clip_uri = uri("state-sub-parent-transition-run.clip");
    let run_graph_uri = uri("state-sub-parent-transition-run.graph");
    let child_uri = uri("state-sub-parent-transition-child.machine");
    let parent_uri = uri("state-sub-parent-transition-parent.machine");
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let child_id = ResourceId::from_locator(&child_uri);
    let parent_id = ResourceId::from_locator(&parent_uri);

    register_animation_blend_assets(&asset_manager, &skeleton_uri, &idle_clip_uri, &run_clip_uri);
    register_single_clip_graph(&asset_manager, &run_graph_uri, &run_clip_uri);
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            child_id,
            ResourceKind::AnimationStateMachine,
            child_uri.clone(),
        ),
        AnimationStateMachineAsset {
            name: Some("ParentTransitionChild".to_string()),
            entry_state: "Run".to_string(),
            states: vec![AnimationStateAsset::graph_ref(
                "Run",
                AssetReference::from_locator(run_graph_uri),
            )],
            transitions: Vec::new(),
            layers: Vec::new(),
        },
    );
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(parent_id, ResourceKind::AnimationStateMachine, parent_uri),
        AnimationStateMachineAsset {
            name: Some("ParentTransitionParent".to_string()),
            entry_state: "Nested".to_string(),
            states: vec![
                AnimationStateAsset {
                    name: "Nested".to_string(),
                    kind: AnimationStateKindAsset::SubMachine {
                        state_machine: AssetReference::from_locator(child_uri),
                    },
                },
                AnimationStateAsset {
                    name: "Exit".to_string(),
                    kind: AnimationStateKindAsset::Clip {
                        clip: AssetReference::from_locator(idle_clip_uri),
                    },
                },
            ],
            transitions: vec![AnimationStateTransitionAsset {
                from_state: "Nested".to_string(),
                to_state: "Exit".to_string(),
                duration_seconds: 0.2,
                exit_time: None,
                interruption: Default::default(),
                conditions: vec![AnimationTransitionConditionAsset {
                    parameter: "leave".to_string(),
                    operator: AnimationConditionOperatorAsset::Equal,
                    value: Some(AnimationParameterValue::Bool(true)),
                }],
            }],
            layers: Vec::new(),
        },
    );

    let level = runtime.create_default_level().unwrap();
    let entity = spawn_state_machine_player(
        &level,
        skeleton_id,
        parent_id,
        BTreeMap::from([("leave".to_string(), AnimationParameterValue::Bool(true))]),
    );
    runtime.tick_level_seconds(&level, 0.1).unwrap();

    assert_hand_translation(&level, entity, 5.0);
}

#[test]
fn layered_state_machine_blends_production_pose_with_dense_mask() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let asset_manager = runtime_asset_manager(&runtime.handle());
    let skeleton_uri = uri("state-layer.skeleton");
    let idle_clip_uri = uri("state-layer-idle.clip");
    let run_clip_uri = uri("state-layer-run.clip");
    let layer_uri = uri("state-layer-upper.machine");
    let base_uri = uri("state-layer-base.machine");
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let layer_id = ResourceId::from_locator(&layer_uri);
    let base_id = ResourceId::from_locator(&base_uri);

    register_animation_blend_assets(&asset_manager, &skeleton_uri, &idle_clip_uri, &run_clip_uri);
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            layer_id,
            ResourceKind::AnimationStateMachine,
            layer_uri.clone(),
        ),
        AnimationStateMachineAsset {
            name: Some("UpperLayer".to_string()),
            entry_state: "Run".to_string(),
            states: vec![AnimationStateAsset {
                name: "Run".to_string(),
                kind: AnimationStateKindAsset::Clip {
                    clip: AssetReference::from_locator(run_clip_uri),
                },
            }],
            transitions: Vec::new(),
            layers: Vec::new(),
        },
    );
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(base_id, ResourceKind::AnimationStateMachine, base_uri),
        AnimationStateMachineAsset {
            name: Some("LayeredBase".to_string()),
            entry_state: "Idle".to_string(),
            states: vec![AnimationStateAsset {
                name: "Idle".to_string(),
                kind: AnimationStateKindAsset::Clip {
                    clip: AssetReference::from_locator(idle_clip_uri),
                },
            }],
            transitions: Vec::new(),
            layers: vec![AnimationStateMachineLayerAsset {
                name: "upper".to_string(),
                state_machine: AssetReference::from_locator(layer_uri),
                weight: 0.5,
                blend_mode: AnimationStateMachineLayerBlendModeAsset::Override,
                mask_weights: vec![0.0, 1.0],
            }],
        },
    );

    let level = runtime.create_default_level().unwrap();
    let entity = spawn_state_machine_player(&level, skeleton_id, base_id, BTreeMap::new());
    runtime.tick_level_seconds(&level, 0.0).unwrap();

    assert_hand_translation(&level, entity, 5.0);
}

#[test]
fn layered_state_machine_interruption_preserves_blended_pose_continuity() {
    let runtime = runtime_with_physics_animation_scene_asset();
    let asset_manager = runtime_asset_manager(&runtime.handle());
    let skeleton_uri = uri("layer-interrupt.skeleton");
    let idle_clip_uri = uri("layer-interrupt-idle.clip");
    let run_clip_uri = uri("layer-interrupt-run.clip");
    let sprint_clip_uri = uri("layer-interrupt-sprint.clip");
    let idle_graph_uri = uri("layer-interrupt-idle.graph");
    let run_graph_uri = uri("layer-interrupt-run.graph");
    let sprint_graph_uri = uri("layer-interrupt-sprint.graph");
    let layer_uri = uri("layer-interrupt-upper.machine");
    let base_uri = uri("layer-interrupt-base.machine");
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    let layer_id = ResourceId::from_locator(&layer_uri);
    let base_id = ResourceId::from_locator(&base_uri);

    register_animation_blend_assets(&asset_manager, &skeleton_uri, &idle_clip_uri, &run_clip_uri);
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            ResourceId::from_locator(&sprint_clip_uri),
            ResourceKind::AnimationClip,
            sprint_clip_uri.clone(),
        ),
        single_hand_translation_clip(&skeleton_uri, 20.0),
    );
    for (graph, clip) in [
        (&idle_graph_uri, &idle_clip_uri),
        (&run_graph_uri, &run_clip_uri),
        (&sprint_graph_uri, &sprint_clip_uri),
    ] {
        register_single_clip_graph(&asset_manager, graph, clip);
    }
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            layer_id,
            ResourceKind::AnimationStateMachine,
            layer_uri.clone(),
        ),
        interruptible_transition_state_machine(&idle_graph_uri, &run_graph_uri, &sprint_graph_uri),
    );
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(base_id, ResourceKind::AnimationStateMachine, base_uri),
        AnimationStateMachineAsset {
            name: Some("LayerInterruptionBase".to_string()),
            entry_state: "Idle".to_string(),
            states: vec![AnimationStateAsset {
                name: "Idle".to_string(),
                kind: AnimationStateKindAsset::Clip {
                    clip: AssetReference::from_locator(idle_clip_uri),
                },
            }],
            transitions: Vec::new(),
            layers: vec![AnimationStateMachineLayerAsset {
                name: "upper".to_string(),
                state_machine: AssetReference::from_locator(layer_uri),
                weight: 1.0,
                blend_mode: AnimationStateMachineLayerBlendModeAsset::Override,
                mask_weights: vec![0.0, 1.0],
            }],
        },
    );

    let level = runtime.create_default_level().unwrap();
    let entity = spawn_state_machine_player(
        &level,
        skeleton_id,
        base_id,
        BTreeMap::from([
            ("start".to_string(), AnimationParameterValue::Bool(true)),
            ("interrupt".to_string(), AnimationParameterValue::Bool(true)),
        ]),
    );

    runtime.tick_level_seconds(&level, 0.5).unwrap();
    assert_hand_translation(&level, entity, 5.0);

    runtime.tick_level_seconds(&level, 0.0).unwrap();
    assert_hand_translation(&level, entity, 5.0);

    runtime.tick_level_seconds(&level, 0.5).unwrap();
    assert_hand_translation(&level, entity, 12.5);
}
