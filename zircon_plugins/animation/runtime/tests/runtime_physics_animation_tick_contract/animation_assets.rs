use zircon_runtime::asset::{AssetReference, AssetUri, ProjectAssetManager};
use zircon_runtime::core::framework::animation::AnimationParameterValue;
use zircon_runtime::core::framework::animation::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationClipAsset, AnimationClipBoneTrackAsset, AnimationConditionOperatorAsset,
    AnimationGraphAsset, AnimationGraphNodeAsset, AnimationGraphParameterAsset,
    AnimationInterpolationAsset, AnimationSequenceAsset, AnimationSequenceBindingAsset,
    AnimationSequenceTrackAsset, AnimationSkeletonAsset, AnimationSkeletonBoneAsset,
    AnimationStateAsset, AnimationStateMachineAsset, AnimationStateTransitionAsset,
    AnimationTransitionConditionAsset, AnimationTransitionInterruptionPolicyAsset,
};
use zircon_runtime::core::framework::scene::{ComponentPropertyPath, EntityPath};
use zircon_runtime::core::resource::{ResourceId, ResourceKind, ResourceRecord};

pub(super) fn sequence_asset_for_entity(entity_path: &str) -> AnimationSequenceAsset {
    AnimationSequenceAsset {
        name: Some("RuntimeSequenceTick".to_string()),
        duration_seconds: 1.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse(entity_path).unwrap(),
            target_id: Some(entity_path.to_string()),
            tracks: vec![AnimationSequenceTrackAsset {
                property_path: ComponentPropertyPath::parse("Transform.translation").unwrap(),
                channel: AnimationChannelAsset {
                    interpolation: AnimationInterpolationAsset::Hermite,
                    keys: vec![
                        AnimationChannelKeyAsset {
                            time_seconds: 0.0,
                            value: AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0]),
                            in_tangent: None,
                            out_tangent: Some(AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0])),
                        },
                        AnimationChannelKeyAsset {
                            time_seconds: 0.5,
                            value: AnimationChannelValueAsset::Vec3([2.0, 0.0, 0.0]),
                            in_tangent: Some(AnimationChannelValueAsset::Vec3([0.0, 0.0, 0.0])),
                            out_tangent: None,
                        },
                    ],
                },
            }],
        }],
    }
}

pub(super) fn register_animation_blend_assets(
    asset_manager: &ProjectAssetManager,
    skeleton_uri: &AssetUri,
    clip_a_uri: &AssetUri,
    clip_b_uri: &AssetUri,
) {
    let skeleton_id = ResourceId::from_locator(skeleton_uri);
    let clip_a_id = ResourceId::from_locator(clip_a_uri);
    let clip_b_id = ResourceId::from_locator(clip_b_uri);
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            skeleton_id,
            ResourceKind::AnimationSkeleton,
            skeleton_uri.clone(),
        ),
        two_bone_skeleton(),
    );
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(clip_a_id, ResourceKind::AnimationClip, clip_a_uri.clone()),
        single_hand_translation_clip(skeleton_uri, 0.0),
    );
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(clip_b_id, ResourceKind::AnimationClip, clip_b_uri.clone()),
        single_hand_translation_clip(skeleton_uri, 10.0),
    );
}

pub(super) fn register_single_clip_graph(
    asset_manager: &ProjectAssetManager,
    graph_uri: &AssetUri,
    clip_uri: &AssetUri,
) {
    asset_manager.resource_manager().register_ready(
        ResourceRecord::new(
            ResourceId::from_locator(graph_uri),
            ResourceKind::AnimationGraph,
            graph_uri.clone(),
        ),
        single_clip_graph(clip_uri),
    );
}

pub(super) fn two_bone_skeleton() -> AnimationSkeletonAsset {
    AnimationSkeletonAsset {
        name: Some("BlendSkeleton".to_string()),
        bones: vec![
            AnimationSkeletonBoneAsset {
                name: "Root".to_string(),
                parent_index: None,
                local_translation: [0.0, 0.0, 0.0],
                local_rotation: [0.0, 0.0, 0.0, 1.0],
                local_scale: [1.0, 1.0, 1.0],
            },
            AnimationSkeletonBoneAsset {
                name: "Hand".to_string(),
                parent_index: Some(0),
                local_translation: [0.0, 0.0, 0.0],
                local_rotation: [0.0, 0.0, 0.0, 1.0],
                local_scale: [1.0, 1.0, 1.0],
            },
        ],
    }
}

pub(super) fn single_hand_translation_clip(
    skeleton_uri: &AssetUri,
    translation_x: f32,
) -> AnimationClipAsset {
    AnimationClipAsset {
        name: Some(format!("Hand{translation_x}")),
        skeleton: AssetReference::from_locator(skeleton_uri.clone()),
        duration_seconds: 1.0,
        tracks: vec![AnimationClipBoneTrackAsset {
            bone_name: "Hand".to_string(),
            target_id: Some("Root/Hand".to_string()),
            translation: constant_vec3_channel([translation_x, 0.0, 0.0]),
            rotation: constant_quaternion_channel([0.0, 0.0, 0.0, 1.0]),
            scale: constant_vec3_channel([1.0, 1.0, 1.0]),
        }],
        event_tracks: Vec::new(),
    }
}

pub(super) fn two_clip_blend_graph(
    clip_a_uri: &AssetUri,
    clip_b_uri: &AssetUri,
    blend_weight: f32,
) -> AnimationGraphAsset {
    AnimationGraphAsset {
        name: Some("TwoClipBlend".to_string()),
        parameters: vec![AnimationGraphParameterAsset {
            name: "blend".to_string(),
            default_value: AnimationParameterValue::Scalar(blend_weight),
        }],
        nodes: vec![
            AnimationGraphNodeAsset::Clip {
                id: "a".to_string(),
                clip: AssetReference::from_locator(clip_a_uri.clone()),
                playback_speed: 1.0,
                looping: false,
            },
            AnimationGraphNodeAsset::Clip {
                id: "b".to_string(),
                clip: AssetReference::from_locator(clip_b_uri.clone()),
                playback_speed: 1.0,
                looping: false,
            },
            AnimationGraphNodeAsset::Blend {
                id: "blend".to_string(),
                inputs: vec!["a".to_string(), "b".to_string()],
                weight_parameter: Some("blend".to_string()),
            },
            AnimationGraphNodeAsset::Output {
                source: "blend".to_string(),
            },
        ],
    }
}

pub(super) fn additive_mask_graph(
    base_uri: &AssetUri,
    additive_uri: &AssetUri,
) -> AnimationGraphAsset {
    AnimationGraphAsset {
        name: Some("AdditiveMaskGraph".to_string()),
        parameters: vec![AnimationGraphParameterAsset {
            name: "additive_weight".to_string(),
            default_value: AnimationParameterValue::Scalar(1.0),
        }],
        nodes: vec![
            AnimationGraphNodeAsset::Clip {
                id: "base".to_string(),
                clip: AssetReference::from_locator(base_uri.clone()),
                playback_speed: 1.0,
                looping: false,
            },
            AnimationGraphNodeAsset::Clip {
                id: "add".to_string(),
                clip: AssetReference::from_locator(additive_uri.clone()),
                playback_speed: 1.0,
                looping: false,
            },
            AnimationGraphNodeAsset::Additive {
                id: "additive".to_string(),
                base: "base".to_string(),
                additive: "add".to_string(),
                weight_parameter: Some("additive_weight".to_string()),
            },
            AnimationGraphNodeAsset::Mask {
                id: "masked".to_string(),
                input: "additive".to_string(),
                target_ids: vec!["Root/Hand".to_string()],
            },
            AnimationGraphNodeAsset::Output {
                source: "masked".to_string(),
            },
        ],
    }
}

fn single_clip_graph(clip_uri: &AssetUri) -> AnimationGraphAsset {
    AnimationGraphAsset {
        name: Some("SingleClipGraph".to_string()),
        parameters: Vec::new(),
        nodes: vec![
            AnimationGraphNodeAsset::Clip {
                id: "clip".to_string(),
                clip: AssetReference::from_locator(clip_uri.clone()),
                playback_speed: 1.0,
                looping: false,
            },
            AnimationGraphNodeAsset::Output {
                source: "clip".to_string(),
            },
        ],
    }
}

pub(super) fn timed_transition_state_machine(
    idle_graph_uri: &AssetUri,
    run_graph_uri: &AssetUri,
) -> AnimationStateMachineAsset {
    AnimationStateMachineAsset {
        name: Some("TimedTransition".to_string()),
        entry_state: "Idle".to_string(),
        states: vec![
            AnimationStateAsset::graph_ref(
                "Idle",
                AssetReference::from_locator(idle_graph_uri.clone()),
            ),
            AnimationStateAsset::graph_ref(
                "Run",
                AssetReference::from_locator(run_graph_uri.clone()),
            ),
        ],
        layers: Vec::new(),
        transitions: vec![AnimationStateTransitionAsset {
            from_state: "Idle".to_string(),
            to_state: "Run".to_string(),
            duration_seconds: 0.2,
            exit_time: None,
            interruption: Default::default(),
            conditions: vec![AnimationTransitionConditionAsset {
                parameter: "advance".to_string(),
                operator: AnimationConditionOperatorAsset::Equal,
                value: Some(AnimationParameterValue::Bool(true)),
            }],
        }],
    }
}

pub(super) fn interruptible_transition_state_machine(
    idle_graph_uri: &AssetUri,
    run_graph_uri: &AssetUri,
    sprint_graph_uri: &AssetUri,
) -> AnimationStateMachineAsset {
    AnimationStateMachineAsset {
        name: Some("InterruptibleTransition".to_string()),
        entry_state: "Idle".to_string(),
        states: vec![
            state("Idle", idle_graph_uri),
            state("Run", run_graph_uri),
            state("Sprint", sprint_graph_uri),
        ],
        transitions: vec![
            transition(
                "Idle",
                "Run",
                "start",
                AnimationTransitionInterruptionPolicyAsset::Both,
            ),
            transition(
                "Run",
                "Sprint",
                "interrupt",
                AnimationTransitionInterruptionPolicyAsset::None,
            ),
        ],
        layers: Vec::new(),
    }
}

fn state(name: &str, graph_uri: &AssetUri) -> AnimationStateAsset {
    AnimationStateAsset::graph_ref(name, AssetReference::from_locator(graph_uri.clone()))
}

fn transition(
    from_state: &str,
    to_state: &str,
    parameter: &str,
    interruption: AnimationTransitionInterruptionPolicyAsset,
) -> AnimationStateTransitionAsset {
    AnimationStateTransitionAsset {
        from_state: from_state.to_string(),
        to_state: to_state.to_string(),
        duration_seconds: 1.0,
        exit_time: None,
        interruption,
        conditions: vec![AnimationTransitionConditionAsset {
            parameter: parameter.to_string(),
            operator: AnimationConditionOperatorAsset::Equal,
            value: Some(AnimationParameterValue::Bool(true)),
        }],
    }
}

pub(super) fn single_state_machine(graph_uri: &AssetUri) -> AnimationStateMachineAsset {
    AnimationStateMachineAsset {
        name: Some("SingleState".to_string()),
        entry_state: "Idle".to_string(),
        states: vec![AnimationStateAsset::graph_ref(
            "Idle",
            AssetReference::from_locator(graph_uri.clone()),
        )],
        transitions: Vec::new(),
        layers: Vec::new(),
    }
}

fn constant_vec3_channel(value: [f32; 3]) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Step,
        keys: vec![AnimationChannelKeyAsset {
            time_seconds: 0.0,
            value: AnimationChannelValueAsset::Vec3(value),
            in_tangent: None,
            out_tangent: None,
        }],
    }
}

fn constant_quaternion_channel(value: [f32; 4]) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Step,
        keys: vec![AnimationChannelKeyAsset {
            time_seconds: 0.0,
            value: AnimationChannelValueAsset::Quaternion(value),
            in_tangent: None,
            out_tangent: None,
        }],
    }
}
