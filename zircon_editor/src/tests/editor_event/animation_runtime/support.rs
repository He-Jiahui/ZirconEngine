use std::fs;
use std::path::Path;

use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::animation::AnimationParameterValue;
use zircon_runtime::core::framework::animation::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationConditionOperatorAsset, AnimationGraphAsset, AnimationGraphNodeAsset,
    AnimationInterpolationAsset, AnimationSequenceAsset, AnimationSequenceBindingAsset,
    AnimationSequenceTrackAsset, AnimationStateAsset, AnimationStateMachineAsset,
    AnimationStateTransitionAsset, AnimationTransitionConditionAsset,
};
use zircon_runtime::core::framework::scene::{ComponentPropertyPath, EntityPath};

use crate::tests::editor_event::support::EventRuntimeHarness;

pub(super) fn open_indexed_animation_asset(
    harness: &mut EventRuntimeHarness,
    prefix: &str,
    locator: &str,
    write_asset: impl FnOnce(&Path),
) -> String {
    harness.register_animation_asset_toolkits();
    let catalog = harness.open_project_with_assets(prefix, |project| {
        write_asset(&project.source_path(locator));
    });
    assert!(
        catalog.assets.iter().any(|asset| asset.locator == locator),
        "animation fixture asset {locator} should be indexed by the real project catalog"
    );
    locator.to_string()
}

pub(super) fn scalar_channel(value: f32) -> AnimationChannelAsset {
    AnimationChannelAsset {
        interpolation: AnimationInterpolationAsset::Step,
        keys: vec![AnimationChannelKeyAsset {
            time_seconds: 0.0,
            value: AnimationChannelValueAsset::Scalar(value),
            in_tangent: None,
            out_tangent: None,
        }],
    }
}

pub(super) fn write_sequence_asset(path: &Path) {
    let asset = AnimationSequenceAsset {
        name: Some("Hero Sequence".to_string()),
        duration_seconds: 2.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Root/Hero").unwrap(),
            target_id: None,
            tracks: vec![AnimationSequenceTrackAsset {
                property_path: ComponentPropertyPath::parse("AnimationPlayer.weight").unwrap(),
                channel: scalar_channel(1.0),
            }],
        }],
    };
    fs::write(path, asset.to_bytes().unwrap()).unwrap();
}

pub(super) fn write_sequence_asset_with_multiple_tracks(path: &Path) {
    let asset = AnimationSequenceAsset {
        name: Some("Hero Sequence".to_string()),
        duration_seconds: 2.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Root/Hero").unwrap(),
            target_id: None,
            tracks: vec![
                AnimationSequenceTrackAsset {
                    property_path: ComponentPropertyPath::parse("AnimationPlayer.weight").unwrap(),
                    channel: scalar_channel(1.0),
                },
                AnimationSequenceTrackAsset {
                    property_path: ComponentPropertyPath::parse("Transform.translation").unwrap(),
                    channel: scalar_channel(2.0),
                },
            ],
        }],
    };
    fs::write(path, asset.to_bytes().unwrap()).unwrap();
}

pub(super) fn write_state_machine_asset(path: &Path) {
    let graph_reference = zircon_runtime::asset::AssetReference::from_locator(
        AssetUri::parse("res://animation/hero.graph.zranim").unwrap(),
    );
    let asset = AnimationStateMachineAsset {
        name: Some("Hero State Machine".to_string()),
        entry_state: "Idle".to_string(),
        states: vec![
            AnimationStateAsset::graph_ref("Idle", graph_reference.clone()),
            AnimationStateAsset::graph_ref("Run", graph_reference),
        ],
        transitions: Vec::new(),
        layers: Vec::new(),
    };
    fs::write(path, asset.to_bytes().unwrap()).unwrap();
}

pub(super) fn write_state_machine_asset_with_transition(path: &Path) {
    let graph_reference = zircon_runtime::asset::AssetReference::from_locator(
        AssetUri::parse("res://animation/hero.graph.zranim").unwrap(),
    );
    let asset = AnimationStateMachineAsset {
        name: Some("Hero State Machine".to_string()),
        entry_state: "Idle".to_string(),
        states: vec![
            AnimationStateAsset::graph_ref("Idle", graph_reference.clone()),
            AnimationStateAsset::graph_ref("Run", graph_reference),
        ],
        transitions: vec![AnimationStateTransitionAsset {
            from_state: "Idle".to_string(),
            to_state: "Run".to_string(),
            duration_seconds: 0.25,
            exit_time: None,
            interruption: Default::default(),
            conditions: vec![AnimationTransitionConditionAsset {
                parameter: "speed".to_string(),
                operator: AnimationConditionOperatorAsset::GreaterEqual,
                value: Some(AnimationParameterValue::Scalar(1.0)),
            }],
        }],
        layers: Vec::new(),
    };
    fs::write(path, asset.to_bytes().unwrap()).unwrap();
}

pub(super) fn write_graph_asset(path: &Path) {
    let clip_reference =
        AssetReference::from_locator(AssetUri::parse("res://animation/hero.clip.zranim").unwrap());
    let asset = AnimationGraphAsset {
        name: Some("Hero Graph".to_string()),
        parameters: Vec::new(),
        nodes: vec![
            AnimationGraphNodeAsset::Clip {
                id: "idle".to_string(),
                clip: clip_reference,
                playback_speed: 1.0,
                looping: true,
            },
            AnimationGraphNodeAsset::Blend {
                id: "locomotion".to_string(),
                inputs: Vec::new(),
                weight_parameter: Some("speed".to_string()),
            },
            AnimationGraphNodeAsset::Output {
                source: "idle".to_string(),
            },
        ],
    };
    fs::write(path, asset.to_bytes().unwrap()).unwrap();
}
