use crate::ui::binding::{
    AnimationCommand, EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind, EditorUiRouter,
};

#[derive(Clone, Debug, PartialEq, Eq)]
enum MockEditorCommand {
    AddAnimationFrame { track_path: String, frame: u32 },
}

#[test]
fn animation_clip_binding_formats_as_stable_native_binding() {
    let binding = EditorUiBinding::new(
        "AnimationClipEditorView",
        "AddFrameButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::add_animation_key("root/child:transform.translation", 24),
    );

    assert_eq!(
        binding.native_binding(),
        r#"AnimationClipEditorView/AddFrameButton:onClick(AnimationCommand.AddKey("root/child:transform.translation",24))"#
    );
    assert_eq!(
        EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap(),
        binding
    );
}

#[test]
fn editor_ui_router_dispatches_animation_binding_headlessly() {
    let binding = EditorUiBinding::new(
        "AnimationClipEditorView",
        "AddFrameButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::add_animation_key("root/child:transform.translation", 24),
    );
    let mut router = EditorUiRouter::<MockEditorCommand>::default();
    router.register_exact(
        binding.path().clone(),
        |binding: &EditorUiBinding| match binding.payload() {
            EditorUiBindingPayload::AnimationCommand(AnimationCommand::AddKey {
                track_path,
                frame,
            }) => MockEditorCommand::AddAnimationFrame {
                track_path: track_path.clone(),
                frame: *frame,
            },
            payload => panic!("unexpected payload {payload:?}"),
        },
    );

    assert_eq!(
        router.dispatch(&binding),
        vec![MockEditorCommand::AddAnimationFrame {
            track_path: "root/child:transform.translation".to_string(),
            frame: 24,
        }]
    );
}

#[test]
fn animation_command_bindings_roundtrip_for_track_lifecycle_and_playback() {
    let bindings = [
        (
            EditorUiBinding::new(
                "AnimationSequenceEditorView",
                "CreateTrackButton",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::animation_command(AnimationCommand::CreateTrack {
                    track_path: "root/child:AnimationPlayer.weight".to_string(),
                }),
            ),
            r#"AnimationSequenceEditorView/CreateTrackButton:onClick(AnimationCommand.CreateTrack("root/child:AnimationPlayer.weight"))"#,
        ),
        (
            EditorUiBinding::new(
                "AnimationSequenceEditorView",
                "Scrubber",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::animation_command(AnimationCommand::ScrubTimeline {
                    frame: 48,
                }),
            ),
            r#"AnimationSequenceEditorView/Scrubber:onChange(AnimationCommand.ScrubTimeline(48))"#,
        ),
        (
            EditorUiBinding::new(
                "AnimationSequenceEditorView",
                "PlaybackToggle",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::animation_command(AnimationCommand::SetPlayback {
                    playing: true,
                    looping: false,
                    speed: 1.25,
                }),
            ),
            r#"AnimationSequenceEditorView/PlaybackToggle:onClick(AnimationCommand.SetPlayback(true,false,1.25))"#,
        ),
    ];

    for (binding, expected) in bindings {
        assert_eq!(binding.native_binding(), expected);
        assert_eq!(
            EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap(),
            binding
        );
    }
}

#[test]
fn animation_command_bindings_roundtrip_for_timeline_graph_and_state_machine_authoring() {
    let bindings = [
        (
            EditorUiBinding::new(
                "AnimationSequenceEditorView",
                "TimelineRange",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::animation_command(AnimationCommand::SetTimelineRange {
                    start_frame: 12,
                    end_frame: 96,
                }),
            ),
            r#"AnimationSequenceEditorView/TimelineRange:onChange(AnimationCommand.SetTimelineRange(12,96))"#,
        ),
        (
            EditorUiBinding::new(
                "AnimationSequenceEditorView",
                "TimelineSelection",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::animation_command(AnimationCommand::SelectTimelineSpan {
                    track_path: "Root/Hero:Transform.translation".to_string(),
                    start_frame: 24,
                    end_frame: 48,
                }),
            ),
            r#"AnimationSequenceEditorView/TimelineSelection:onChange(AnimationCommand.SelectTimelineSpan("Root/Hero:Transform.translation",24,48))"#,
        ),
        (
            EditorUiBinding::new(
                "AnimationGraphEditorView",
                "AddBlendNode",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::animation_command(AnimationCommand::AddGraphNode {
                    graph_path: "res://animation/hero.graph.zranim".to_string(),
                    node_id: "blend_walk_run".to_string(),
                    node_kind: "blend".to_string(),
                }),
            ),
            r#"AnimationGraphEditorView/AddBlendNode:onClick(AnimationCommand.AddGraphNode("res://animation/hero.graph.zranim","blend_walk_run","blend"))"#,
        ),
        (
            EditorUiBinding::new(
                "AnimationGraphEditorView",
                "SetGraphParameter",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::animation_command(AnimationCommand::SetGraphParameter {
                    graph_path: "res://animation/hero.graph.zranim".to_string(),
                    parameter_name: "speed".to_string(),
                    value_literal: "1.5".to_string(),
                }),
            ),
            r#"AnimationGraphEditorView/SetGraphParameter:onChange(AnimationCommand.SetGraphParameter("res://animation/hero.graph.zranim","speed","1.5"))"#,
        ),
        (
            EditorUiBinding::new(
                "AnimationGraphEditorView",
                "CreateTransition",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::animation_command(AnimationCommand::CreateTransition {
                    state_machine_path: "res://animation/hero.state_machine.zranim".to_string(),
                    from_state: "Idle".to_string(),
                    to_state: "Run".to_string(),
                    duration_frames: 8,
                }),
            ),
            r#"AnimationGraphEditorView/CreateTransition:onClick(AnimationCommand.CreateTransition("res://animation/hero.state_machine.zranim","Idle","Run",8))"#,
        ),
        (
            EditorUiBinding::new(
                "AnimationGraphEditorView",
                "TransitionCondition",
                EditorUiEventKind::Change,
                EditorUiBindingPayload::animation_command(
                    AnimationCommand::SetTransitionCondition {
                        state_machine_path: "res://animation/hero.state_machine.zranim".to_string(),
                        from_state: "Idle".to_string(),
                        to_state: "Run".to_string(),
                        parameter_name: "speed".to_string(),
                        operator: "greater_equal".to_string(),
                        value_literal: "1.0".to_string(),
                    },
                ),
            ),
            r#"AnimationGraphEditorView/TransitionCondition:onChange(AnimationCommand.SetTransitionCondition("res://animation/hero.state_machine.zranim","Idle","Run","speed","greater_equal","1.0"))"#,
        ),
    ];

    for (binding, expected) in bindings {
        assert_eq!(binding.native_binding(), expected);
        assert_eq!(
            EditorUiBinding::parse_native_binding(&binding.native_binding()).unwrap(),
            binding
        );
    }
}

#[test]
fn animation_binding_payload_uses_shared_framework_track_path() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let payload_source = std::fs::read_to_string(crate_root.join("src/ui/binding/core/payload.rs"))
        .unwrap_or_default();
    let payload_codec_source =
        std::fs::read_to_string(crate_root.join("src/ui/binding/core/payload_codec.rs"))
            .unwrap_or_default();
    let dispatch_source =
        std::fs::read_to_string(crate_root.join("src/ui/binding_dispatch/animation/dispatch.rs"))
            .unwrap_or_default();
    let event_source = std::fs::read_to_string(
        crate_root.join("src/ui/binding_dispatch/animation/animation_host_event.rs"),
    )
    .unwrap_or_default();

    for required in [
        "AnimationTrackPath",
        "zircon_runtime::core::framework::animation",
    ] {
        assert!(
            payload_source.contains(required)
                || payload_codec_source.contains(required)
                || dispatch_source.contains(required)
                || event_source.contains(required),
            "editor animation binding should route through shared framework track path `{required}`"
        );
    }
}
