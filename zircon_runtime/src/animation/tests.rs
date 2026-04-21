use crate::asset::assets::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationClipAsset, AnimationClipBoneTrackAsset, AnimationGraphAsset, AnimationGraphNodeAsset,
    AnimationGraphParameterAsset, AnimationInterpolationAsset, AnimationSequenceAsset,
    AnimationSequenceBindingAsset, AnimationSequenceTrackAsset, AnimationSkeletonAsset,
    AnimationSkeletonBoneAsset, AnimationStateAsset, AnimationStateMachineAsset,
    AnimationStateTransitionAsset, AnimationTransitionConditionAsset,
};
use crate::asset::{AnimationConditionOperatorAsset, AssetReference, AssetUri};
use crate::core::framework::animation::{
    AnimationManager, AnimationParameterValue, AnimationPlaybackSettings, AnimationPoseSource,
};
use crate::core::framework::scene::{ComponentPropertyPath, EntityPath};
use crate::core::manager::{resolve_animation_manager, resolve_config_manager};
use crate::core::math::{Quat, Vec3};
use crate::core::resource::{AnimationClipMarker, ResourceHandle, ResourceId};
use crate::core::CoreRuntime;
use crate::foundation::FOUNDATION_MODULE_NAME;
use crate::scene::components::{AnimationPlayerComponent, NodeKind};
use crate::scene::world::World;
use std::collections::BTreeMap;

#[test]
fn animation_root_stays_structural_after_module_split() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("animation")
            .join("mod.rs"),
    )
    .expect("animation mod source");

    for forbidden in [
        "pub struct AnimationConfig",
        "pub struct AnimationModule",
        "pub fn module_descriptor(",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected animation/mod.rs to stay structural after split, found `{forbidden}`"
        );
    }
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

    let report = super::apply_sequence_to_world(&mut world, &sequence, 0.5).unwrap();

    assert_eq!(report.applied_tracks.len(), 2);
    assert!(report.missing_tracks.is_empty());
    assert_eq!(
        world.find_node(hero).unwrap().transform.translation,
        Vec3::new(5.0, 0.0, 0.0)
    );
    assert_eq!(world.animation_player(hero).unwrap().weight, 0.5);
}

#[test]
fn animation_manager_persists_playback_settings_to_runtime_config() {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(crate::foundation::module_descriptor())
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

    let facade = resolve_animation_manager(&runtime.handle()).unwrap();
    assert_eq!(facade.playback_settings(), playback_settings);

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
fn animation_manager_samples_clip_pose_against_skeleton() {
    let manager = super::DefaultAnimationManager::default();
    let skeleton = AnimationSkeletonAsset {
        name: Some("HeroSkeleton".to_string()),
        bones: vec![
            AnimationSkeletonBoneAsset {
                name: "Root".to_string(),
                parent_index: None,
                local_translation: [0.0, 0.0, 0.0],
                local_rotation: Quat::IDENTITY.to_array(),
                local_scale: [1.0, 1.0, 1.0],
            },
            AnimationSkeletonBoneAsset {
                name: "Hand".to_string(),
                parent_index: Some(0),
                local_translation: [0.0, 1.0, 0.0],
                local_rotation: Quat::IDENTITY.to_array(),
                local_scale: [1.0, 1.0, 1.0],
            },
        ],
    };
    let clip = AnimationClipAsset {
        name: Some("Wave".to_string()),
        skeleton: asset_reference("res://animation/hero.skeleton.zranim"),
        duration_seconds: 1.0,
        tracks: vec![AnimationClipBoneTrackAsset {
            bone_name: "Hand".to_string(),
            translation: vec3_channel([(0.0, [0.0, 1.0, 0.0]), (1.0, [0.0, 2.0, 0.0])]),
            rotation: quaternion_channel([
                (0.0, Quat::IDENTITY.to_array()),
                (
                    1.0,
                    Quat::from_rotation_y(std::f32::consts::FRAC_PI_2).to_array(),
                ),
            ]),
            scale: vec3_channel([(0.0, [1.0, 1.0, 1.0]), (1.0, [2.0, 2.0, 2.0])]),
        }],
    };

    let pose = manager.sample_clip_pose(&skeleton, &clip, 0.5).unwrap();

    assert_eq!(pose.source, AnimationPoseSource::Clip);
    assert_eq!(pose.bones.len(), 2);
    let hand = pose
        .bones
        .iter()
        .find(|bone| bone.name == "Hand")
        .expect("missing hand pose");
    assert_eq!(hand.local_transform.translation, Vec3::new(0.0, 1.5, 0.0));
    assert_eq!(hand.local_transform.scale, Vec3::splat(1.5));
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
