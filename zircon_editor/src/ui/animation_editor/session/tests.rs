use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::assets::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationGraphAsset, AnimationGraphParameterAsset, AnimationInterpolationAsset,
    AnimationSequenceAsset, AnimationSequenceBindingAsset, AnimationSequenceTrackAsset,
    AnimationStateAsset, AnimationStateMachineAsset, AnimationStateTransitionAsset,
    AnimationTransitionConditionAsset,
};
use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::animation::{AnimationParameterValue, AnimationTrackPath};
use zircon_runtime::core::framework::scene::{ComponentPropertyPath, EntityPath};

use super::{AnimationConditionOperatorAsset, AnimationEditorDocument, AnimationEditorSession};

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}"))
}

fn write_state_machine_asset_with_transition(path: &Path) {
    let graph_reference =
        AssetReference::from_locator(AssetUri::parse("res://animation/hero.graph.zranim").unwrap());
    let asset = AnimationStateMachineAsset {
        name: Some("Hero State Machine".to_string()),
        entry_state: "Idle".to_string(),
        states: vec![
            AnimationStateAsset {
                name: "Idle".to_string(),
                graph: graph_reference.clone(),
            },
            AnimationStateAsset {
                name: "Run".to_string(),
                graph: graph_reference,
            },
        ],
        transitions: vec![AnimationStateTransitionAsset {
            from_state: "Idle".to_string(),
            to_state: "Run".to_string(),
            duration_seconds: 0.25,
            conditions: vec![AnimationTransitionConditionAsset {
                parameter: "speed".to_string(),
                operator: AnimationConditionOperatorAsset::GreaterEqual,
                value: Some(AnimationParameterValue::Scalar(1.0)),
            }],
        }],
    };
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, asset.to_bytes().unwrap()).unwrap();
}

fn write_state_machine_asset_with_vec2_transition(path: &Path) {
    let graph_reference =
        AssetReference::from_locator(AssetUri::parse("res://animation/hero.graph.zranim").unwrap());
    let asset = AnimationStateMachineAsset {
        name: Some("Hero State Machine".to_string()),
        entry_state: "Idle".to_string(),
        states: vec![
            AnimationStateAsset {
                name: "Idle".to_string(),
                graph: graph_reference.clone(),
            },
            AnimationStateAsset {
                name: "Run".to_string(),
                graph: graph_reference,
            },
        ],
        transitions: vec![AnimationStateTransitionAsset {
            from_state: "Idle".to_string(),
            to_state: "Run".to_string(),
            duration_seconds: 0.25,
            conditions: vec![AnimationTransitionConditionAsset {
                parameter: "velocity".to_string(),
                operator: AnimationConditionOperatorAsset::Equal,
                value: Some(AnimationParameterValue::Vec2([1.0, 2.0])),
            }],
        }],
    };
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, asset.to_bytes().unwrap()).unwrap();
}

fn write_graph_asset_with_bool_parameter(path: &Path) {
    let asset = AnimationGraphAsset {
        name: Some("Hero Graph".to_string()),
        parameters: vec![AnimationGraphParameterAsset {
            name: "grounded".to_string(),
            default_value: AnimationParameterValue::Bool(true),
        }],
        nodes: Vec::new(),
    };
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, asset.to_bytes().unwrap()).unwrap();
}

fn write_graph_asset_with_vec2_parameter(path: &Path) {
    let asset = AnimationGraphAsset {
        name: Some("Hero Graph".to_string()),
        parameters: vec![AnimationGraphParameterAsset {
            name: "velocity".to_string(),
            default_value: AnimationParameterValue::Vec2([1.0, 2.0]),
        }],
        nodes: Vec::new(),
    };
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, asset.to_bytes().unwrap()).unwrap();
}

fn write_graph_asset_with_scalar_parameter(path: &Path) {
    let asset = AnimationGraphAsset {
        name: Some("Hero Graph".to_string()),
        parameters: vec![AnimationGraphParameterAsset {
            name: "speed".to_string(),
            default_value: AnimationParameterValue::Scalar(1.0),
        }],
        nodes: Vec::new(),
    };
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, asset.to_bytes().unwrap()).unwrap();
}

fn scalar_channel(value: f32) -> AnimationChannelAsset {
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

fn write_sequence_asset_with_track(path: &Path) {
    write_sequence_asset_with_track_timing(path, 2.0, 30.0);
}

fn write_sequence_asset_with_track_timing(
    path: &Path,
    duration_seconds: f32,
    frames_per_second: f32,
) {
    let asset = AnimationSequenceAsset {
        name: Some("Hero Sequence".to_string()),
        duration_seconds,
        frames_per_second,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Root/Hero").unwrap(),
            tracks: vec![AnimationSequenceTrackAsset {
                property_path: ComponentPropertyPath::parse("Transform.translation").unwrap(),
                channel: scalar_channel(1.0),
            }],
        }],
    };
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, asset.to_bytes().unwrap()).unwrap();
}

#[test]
fn set_transition_condition_ignores_unknown_operator() {
    let asset_path = unique_temp_dir("zircon_animation_session_unknown_operator")
        .join("hero.state_machine.zranim");
    write_state_machine_asset_with_transition(&asset_path);

    let mut session = AnimationEditorSession::from_path(&asset_path).unwrap();
    let changed = session
        .set_transition_condition("Idle", "Run", "speed", "approximately", "2.5")
        .unwrap();

    assert!(
        !changed,
        "unknown condition operators should remain a no-op instead of mutating the transition"
    );
    assert!(
        !session.is_dirty(),
        "unknown condition operators should not dirty the session"
    );

    let AnimationEditorDocument::StateMachine(asset) = &session.document else {
        panic!("expected state-machine session");
    };
    assert_eq!(asset.transitions.len(), 1);
    assert_eq!(asset.transitions[0].conditions.len(), 1);
    assert_eq!(
        asset.transitions[0].conditions[0],
        AnimationTransitionConditionAsset {
            parameter: "speed".to_string(),
            operator: AnimationConditionOperatorAsset::GreaterEqual,
            value: Some(AnimationParameterValue::Scalar(1.0)),
        },
        "unknown condition operators should preserve the original transition condition"
    );
}

#[test]
fn set_transition_condition_ignores_invalid_value_literal() {
    let asset_path = unique_temp_dir("zircon_animation_session_invalid_condition_value")
        .join("hero.state_machine.zranim");
    write_state_machine_asset_with_transition(&asset_path);

    let mut session = AnimationEditorSession::from_path(&asset_path).unwrap();
    let changed = session
        .set_transition_condition("Idle", "Run", "speed", "greater_equal", "fast")
        .unwrap();

    assert!(
        !changed,
        "invalid condition literals should remain a no-op instead of overwriting the transition value"
    );
    assert!(
        !session.is_dirty(),
        "invalid condition literals should not dirty the session"
    );

    let AnimationEditorDocument::StateMachine(asset) = &session.document else {
        panic!("expected state-machine session");
    };
    assert_eq!(asset.transitions.len(), 1);
    assert_eq!(asset.transitions[0].conditions.len(), 1);
    assert_eq!(
        asset.transitions[0].conditions[0],
        AnimationTransitionConditionAsset {
            parameter: "speed".to_string(),
            operator: AnimationConditionOperatorAsset::GreaterEqual,
            value: Some(AnimationParameterValue::Scalar(1.0)),
        },
        "invalid condition literals should preserve the original transition condition value"
    );
}

#[test]
fn set_transition_condition_preserves_existing_condition_value_type() {
    let asset_path = unique_temp_dir("zircon_animation_session_condition_type_preservation")
        .join("hero.state_machine.zranim");
    write_state_machine_asset_with_transition(&asset_path);

    let mut session = AnimationEditorSession::from_path(&asset_path).unwrap();
    let changed = session
        .set_transition_condition("Idle", "Run", "speed", "greater_equal", "true")
        .unwrap();

    assert!(
        !changed,
        "typed transition conditions should reject mismatched literals instead of changing value type"
    );
    assert!(
        !session.is_dirty(),
        "typed transition conditions should not dirty the session on mismatched literals"
    );

    let AnimationEditorDocument::StateMachine(asset) = &session.document else {
        panic!("expected state-machine session");
    };
    assert_eq!(asset.transitions.len(), 1);
    assert_eq!(asset.transitions[0].conditions.len(), 1);
    assert_eq!(
        asset.transitions[0].conditions[0],
        AnimationTransitionConditionAsset {
            parameter: "speed".to_string(),
            operator: AnimationConditionOperatorAsset::GreaterEqual,
            value: Some(AnimationParameterValue::Scalar(1.0)),
        },
        "typed transition conditions should preserve the original scalar value when the new literal has the wrong type"
    );
}

#[test]
fn set_transition_condition_ignores_nan_scalar_literal() {
    let asset_path = unique_temp_dir("zircon_animation_session_nan_condition_value")
        .join("hero.state_machine.zranim");
    write_state_machine_asset_with_transition(&asset_path);

    let mut session = AnimationEditorSession::from_path(&asset_path).unwrap();
    let changed = session
        .set_transition_condition("Idle", "Run", "speed", "greater_equal", "NaN")
        .unwrap();

    assert!(
        !changed,
        "non-finite condition literals should remain a no-op instead of mutating the transition"
    );
    assert!(
        !session.is_dirty(),
        "non-finite condition literals should not dirty the session"
    );

    let AnimationEditorDocument::StateMachine(asset) = &session.document else {
        panic!("expected state-machine session");
    };
    assert_eq!(asset.transitions.len(), 1);
    assert_eq!(asset.transitions[0].conditions.len(), 1);
    assert_eq!(
        asset.transitions[0].conditions[0],
        AnimationTransitionConditionAsset {
            parameter: "speed".to_string(),
            operator: AnimationConditionOperatorAsset::GreaterEqual,
            value: Some(AnimationParameterValue::Scalar(1.0)),
        },
        "non-finite condition literals should preserve the original scalar transition value"
    );
}

#[test]
fn set_transition_condition_ignores_non_finite_vector_component() {
    let asset_path = unique_temp_dir("zircon_animation_session_nan_condition_vector")
        .join("hero.state_machine.zranim");
    write_state_machine_asset_with_vec2_transition(&asset_path);

    let mut session = AnimationEditorSession::from_path(&asset_path).unwrap();
    let changed = session
        .set_transition_condition("Idle", "Run", "velocity", "equal", "1.0, NaN")
        .unwrap();

    assert!(
        !changed,
        "non-finite vector condition literals should remain a no-op instead of mutating the transition"
    );
    assert!(
        !session.is_dirty(),
        "non-finite vector condition literals should not dirty the session"
    );

    let AnimationEditorDocument::StateMachine(asset) = &session.document else {
        panic!("expected state-machine session");
    };
    assert_eq!(asset.transitions.len(), 1);
    assert_eq!(asset.transitions[0].conditions.len(), 1);
    assert_eq!(
        asset.transitions[0].conditions[0],
        AnimationTransitionConditionAsset {
            parameter: "velocity".to_string(),
            operator: AnimationConditionOperatorAsset::Equal,
            value: Some(AnimationParameterValue::Vec2([1.0, 2.0])),
        },
        "non-finite vector condition literals should preserve the original vector transition value"
    );
}

#[test]
fn set_graph_parameter_ignores_invalid_literal_for_existing_bool_parameter() {
    let asset_path = unique_temp_dir("zircon_animation_session_invalid_graph_parameter")
        .join("hero.graph.zranim");
    write_graph_asset_with_bool_parameter(&asset_path);

    let mut session = AnimationEditorSession::from_path(&asset_path).unwrap();
    let changed = session
        .set_graph_parameter("grounded", "definitely")
        .unwrap();

    assert!(
        !changed,
        "invalid graph parameter literals should remain a no-op instead of coercing typed parameters"
    );
    assert!(
        !session.is_dirty(),
        "invalid graph parameter literals should not dirty the session"
    );

    let AnimationEditorDocument::Graph(asset) = &session.document else {
        panic!("expected graph session");
    };
    assert_eq!(asset.parameters.len(), 1);
    assert_eq!(
        asset.parameters[0],
        AnimationGraphParameterAsset {
            name: "grounded".to_string(),
            default_value: AnimationParameterValue::Bool(true),
        },
        "invalid graph parameter literals should preserve the original typed default value"
    );
}

#[test]
fn set_graph_parameter_ignores_non_finite_vector_component() {
    let asset_path = unique_temp_dir("zircon_animation_session_nan_graph_parameter_vec2")
        .join("hero.graph.zranim");
    write_graph_asset_with_vec2_parameter(&asset_path);

    let mut session = AnimationEditorSession::from_path(&asset_path).unwrap();
    let changed = session.set_graph_parameter("velocity", "1.0, NaN").unwrap();

    assert!(
        !changed,
        "non-finite vector graph parameter literals should remain a no-op instead of mutating the session"
    );
    assert!(
        !session.is_dirty(),
        "non-finite vector graph parameter literals should not dirty the session"
    );

    let AnimationEditorDocument::Graph(asset) = &session.document else {
        panic!("expected graph session");
    };
    assert_eq!(asset.parameters.len(), 1);
    assert_eq!(
        asset.parameters[0],
        AnimationGraphParameterAsset {
            name: "velocity".to_string(),
            default_value: AnimationParameterValue::Vec2([1.0, 2.0]),
        },
        "non-finite vector graph parameter literals should preserve the original vector default value"
    );
}

#[test]
fn set_graph_parameter_ignores_nan_literal_for_existing_scalar_parameter() {
    let asset_path =
        unique_temp_dir("zircon_animation_session_nan_graph_parameter").join("hero.graph.zranim");
    write_graph_asset_with_scalar_parameter(&asset_path);

    let mut session = AnimationEditorSession::from_path(&asset_path).unwrap();
    let changed = session.set_graph_parameter("speed", "NaN").unwrap();

    assert!(
        !changed,
        "non-finite graph parameter literals should remain a no-op instead of mutating the session"
    );
    assert!(
        !session.is_dirty(),
        "non-finite graph parameter literals should not dirty the session"
    );

    let AnimationEditorDocument::Graph(asset) = &session.document else {
        panic!("expected graph session");
    };
    assert_eq!(asset.parameters.len(), 1);
    assert_eq!(
        asset.parameters[0],
        AnimationGraphParameterAsset {
            name: "speed".to_string(),
            default_value: AnimationParameterValue::Scalar(1.0),
        },
        "non-finite graph parameter literals should preserve the original scalar value"
    );
}

#[test]
fn select_timeline_span_clamps_to_current_timeline_range() {
    let asset_path =
        unique_temp_dir("zircon_animation_session_select_span_clamps").join("hero.sequence.zranim");
    write_sequence_asset_with_track(&asset_path);
    let track_path = AnimationTrackPath::parse("Root/Hero:Transform.translation").unwrap();

    let mut session = AnimationEditorSession::from_path(&asset_path).unwrap();
    session.set_timeline_range(10, 20).unwrap();
    let changed = session.select_timeline_span(&track_path, 4, 28).unwrap();

    assert!(
        changed,
        "selecting a new timeline span should report a change"
    );
    assert!(
        !session.is_dirty(),
        "timeline selection should stay editor-local and not dirty the sequence asset"
    );

    let AnimationEditorDocument::Sequence(document) = &session.document else {
        panic!("expected sequence session");
    };
    assert_eq!(
        document.selected_span,
        Some((track_path, 10, 20)),
        "timeline selection should clamp to the active visible range instead of storing out-of-range endpoints"
    );
}

#[test]
fn set_timeline_range_clamps_existing_selected_span() {
    let asset_path = unique_temp_dir("zircon_animation_session_range_clamps_selection")
        .join("hero.sequence.zranim");
    write_sequence_asset_with_track(&asset_path);
    let track_path = AnimationTrackPath::parse("Root/Hero:Transform.translation").unwrap();

    let mut session = AnimationEditorSession::from_path(&asset_path).unwrap();
    session.select_timeline_span(&track_path, 4, 28).unwrap();
    let changed = session.set_timeline_range(10, 20).unwrap();

    assert!(
        changed,
        "changing the timeline range should report a change"
    );
    assert!(
        !session.is_dirty(),
        "changing the visible timeline range should not dirty the sequence asset"
    );

    let AnimationEditorDocument::Sequence(document) = &session.document else {
        panic!("expected sequence session");
    };
    assert_eq!(
        document.selected_span,
        Some((track_path, 10, 20)),
        "shrinking the timeline range should clamp any existing selection into the new range"
    );
}

#[test]
fn sequence_session_sanitizes_infinite_frames_per_second() {
    let asset_path =
        unique_temp_dir("zircon_animation_session_infinite_fps").join("hero.sequence.zranim");
    write_sequence_asset_with_track_timing(&asset_path, 2.0, f32::INFINITY);

    let session = AnimationEditorSession::from_path(&asset_path).unwrap();
    let pane = session.pane_presentation();

    assert_eq!(
        pane.timeline_end_frame, 60,
        "non-finite frames_per_second should fall back to the default timeline scale instead of exploding the visible range"
    );
}

#[test]
fn sequence_session_sanitizes_infinite_duration() {
    let asset_path = unique_temp_dir("zircon_animation_session_infinite_duration")
        .join("hero.sequence.zranim");
    write_sequence_asset_with_track_timing(&asset_path, f32::INFINITY, 30.0);

    let session = AnimationEditorSession::from_path(&asset_path).unwrap();
    let pane = session.pane_presentation();

    assert_eq!(
        pane.timeline_end_frame, 0,
        "non-finite duration_seconds should collapse to a safe zero-length timeline instead of exploding the visible range"
    );
}

#[test]
fn set_playback_ignores_nan_speed() {
    let asset_path =
        unique_temp_dir("zircon_animation_session_playback_nan").join("hero.sequence.zranim");
    write_sequence_asset_with_track(&asset_path);

    let mut session = AnimationEditorSession::from_path(&asset_path).unwrap();
    let changed = session.set_playback(false, false, f32::NAN).unwrap();

    assert!(
        !changed,
        "non-finite playback speeds should remain a no-op instead of mutating the session"
    );
    assert!(
        !session.is_dirty(),
        "playback controls should stay editor-local and must not dirty the sequence asset"
    );

    let AnimationEditorDocument::Sequence(document) = &session.document else {
        panic!("expected sequence session");
    };
    assert_eq!(document.speed, 1.0);
    assert!(
        document.speed.is_finite(),
        "NaN playback speeds should not leak into the sequence session"
    );
    assert_eq!(
        session.pane_presentation().playback_label,
        "Paused • loop=false • speed=1.00",
        "invalid playback speeds should preserve the current presentation state"
    );
}

#[test]
fn set_playback_ignores_infinite_speed() {
    let asset_path =
        unique_temp_dir("zircon_animation_session_playback_inf").join("hero.sequence.zranim");
    write_sequence_asset_with_track(&asset_path);

    let mut session = AnimationEditorSession::from_path(&asset_path).unwrap();
    let changed = session.set_playback(true, true, f32::INFINITY).unwrap();

    assert!(
        !changed,
        "infinite playback speeds should remain a no-op instead of mutating the session"
    );
    assert!(
        !session.is_dirty(),
        "invalid playback speeds should not dirty the sequence asset"
    );

    let AnimationEditorDocument::Sequence(document) = &session.document else {
        panic!("expected sequence session");
    };
    assert_eq!(document.playing, false);
    assert_eq!(document.looping, false);
    assert_eq!(document.speed, 1.0);
    assert!(
        document.speed.is_finite(),
        "infinite playback speeds should not leak into the sequence session"
    );
    assert_eq!(
        session.pane_presentation().playback_label,
        "Paused • loop=false • speed=1.00",
        "invalid playback speeds should preserve the current playback presentation"
    );
}
