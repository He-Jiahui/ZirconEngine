use zircon_runtime::core::framework::animation::AnimationTrackPath;

use crate::ui::binding::{
    AnimationCommand, EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind,
};
use crate::ui::binding_dispatch::{dispatch_animation_binding, AnimationHostEvent};

#[test]
fn animation_binding_dispatches_into_host_event() {
    let binding = EditorUiBinding::new(
        "AnimationClipEditorView",
        "AddFrameButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::add_animation_key("root/child:transform.translation", 24),
    );

    assert_eq!(
        dispatch_animation_binding(&binding).unwrap(),
        AnimationHostEvent::AddKey {
            track_path: AnimationTrackPath::parse("root/child:transform.translation").unwrap(),
            frame: 24,
        }
    );
}

#[test]
fn animation_track_lifecycle_and_playback_bindings_dispatch_into_host_events() {
    let create_track = EditorUiBinding::new(
        "AnimationSequenceEditorView",
        "CreateTrackButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::animation_command(AnimationCommand::CreateTrack {
            track_path: "root/child:AnimationPlayer.weight".to_string(),
        }),
    );
    assert_eq!(
        dispatch_animation_binding(&create_track).unwrap(),
        AnimationHostEvent::CreateTrack {
            track_path: AnimationTrackPath::parse("root/child:AnimationPlayer.weight").unwrap(),
        }
    );

    let rebind = EditorUiBinding::new(
        "AnimationSequenceEditorView",
        "RebindTrackButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::animation_command(AnimationCommand::RebindTrack {
            from_track_path: "root/child:Transform.translation".to_string(),
            to_track_path: "root/child:AnimationPlayer.weight".to_string(),
        }),
    );
    assert_eq!(
        dispatch_animation_binding(&rebind).unwrap(),
        AnimationHostEvent::RebindTrack {
            from_track_path: AnimationTrackPath::parse("root/child:Transform.translation").unwrap(),
            to_track_path: AnimationTrackPath::parse("root/child:AnimationPlayer.weight").unwrap(),
        }
    );

    let playback = EditorUiBinding::new(
        "AnimationSequenceEditorView",
        "PlaybackToggle",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::animation_command(AnimationCommand::SetPlayback {
            playing: true,
            looping: false,
            speed: 1.25,
        }),
    );
    assert_eq!(
        dispatch_animation_binding(&playback).unwrap(),
        AnimationHostEvent::SetPlayback {
            playing: true,
            looping: false,
            speed: 1.25,
        }
    );
}

#[test]
fn animation_timeline_graph_and_state_machine_bindings_dispatch_into_host_events() {
    let timeline_range = EditorUiBinding::new(
        "AnimationSequenceEditorView",
        "TimelineRange",
        EditorUiEventKind::Change,
        EditorUiBindingPayload::animation_command(AnimationCommand::SetTimelineRange {
            start_frame: 12,
            end_frame: 96,
        }),
    );
    assert_eq!(
        dispatch_animation_binding(&timeline_range).unwrap(),
        AnimationHostEvent::SetTimelineRange {
            start_frame: 12,
            end_frame: 96,
        }
    );

    let timeline_selection = EditorUiBinding::new(
        "AnimationSequenceEditorView",
        "TimelineSelection",
        EditorUiEventKind::Change,
        EditorUiBindingPayload::animation_command(AnimationCommand::SelectTimelineSpan {
            track_path: "Root/Hero:Transform.translation".to_string(),
            start_frame: 24,
            end_frame: 48,
        }),
    );
    assert_eq!(
        dispatch_animation_binding(&timeline_selection).unwrap(),
        AnimationHostEvent::SelectTimelineSpan {
            track_path: AnimationTrackPath::parse("Root/Hero:Transform.translation").unwrap(),
            start_frame: 24,
            end_frame: 48,
        }
    );

    let graph_node = EditorUiBinding::new(
        "AnimationGraphEditorView",
        "AddBlendNode",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::animation_command(AnimationCommand::AddGraphNode {
            graph_locator: "res://animation/hero.graph.zranim".to_string(),
            node_id: "blend_walk_run".to_string(),
            node_kind: "blend".to_string(),
        }),
    );
    assert_eq!(
        dispatch_animation_binding(&graph_node).unwrap(),
        AnimationHostEvent::AddGraphNode {
            graph_locator: "res://animation/hero.graph.zranim".to_string(),
            node_id: "blend_walk_run".to_string(),
            node_kind: "blend".to_string(),
        }
    );

    let state_transition = EditorUiBinding::new(
        "AnimationGraphEditorView",
        "CreateTransition",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::animation_command(AnimationCommand::CreateTransition {
            state_machine_locator: "res://animation/hero.state_machine.zranim".to_string(),
            from_state: "Idle".to_string(),
            to_state: "Run".to_string(),
            duration_frames: 8,
        }),
    );
    assert_eq!(
        dispatch_animation_binding(&state_transition).unwrap(),
        AnimationHostEvent::CreateTransition {
            state_machine_locator: "res://animation/hero.state_machine.zranim".to_string(),
            from_state: "Idle".to_string(),
            to_state: "Run".to_string(),
            duration_frames: 8,
        }
    );

    let condition = EditorUiBinding::new(
        "AnimationGraphEditorView",
        "TransitionCondition",
        EditorUiEventKind::Change,
        EditorUiBindingPayload::animation_command(AnimationCommand::SetTransitionCondition {
            state_machine_locator: "res://animation/hero.state_machine.zranim".to_string(),
            from_state: "Idle".to_string(),
            to_state: "Run".to_string(),
            parameter_name: "speed".to_string(),
            operator: "greater_equal".to_string(),
            value_literal: "1.0".to_string(),
        }),
    );
    assert_eq!(
        dispatch_animation_binding(&condition).unwrap(),
        AnimationHostEvent::SetTransitionCondition {
            state_machine_locator: "res://animation/hero.state_machine.zranim".to_string(),
            from_state: "Idle".to_string(),
            to_state: "Run".to_string(),
            parameter_name: "speed".to_string(),
            operator: "greater_equal".to_string(),
            value_literal: "1.0".to_string(),
        }
    );
}
