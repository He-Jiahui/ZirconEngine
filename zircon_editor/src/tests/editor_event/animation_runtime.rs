use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use zircon_runtime::asset::assets::{
    AnimationChannelAsset, AnimationChannelKeyAsset, AnimationChannelValueAsset,
    AnimationConditionOperatorAsset, AnimationGraphAsset, AnimationGraphNodeAsset,
    AnimationInterpolationAsset, AnimationSequenceAsset, AnimationSequenceBindingAsset,
    AnimationSequenceTrackAsset, AnimationStateAsset, AnimationStateMachineAsset,
    AnimationStateTransitionAsset, AnimationTransitionConditionAsset,
};
use zircon_runtime::asset::{AssetReference, AssetUri};
use zircon_runtime::core::framework::animation::AnimationParameterValue;
use zircon_runtime::core::framework::scene::{ComponentPropertyPath, EntityPath};

use crate::core::editor_event::{EditorAssetEvent, EditorEvent, EditorEventSource};
use crate::ui::binding::{
    AnimationCommand, EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind,
};
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;
use crate::ui::workbench::view::ViewDescriptorId;

use super::support::{env_lock, EventRuntimeHarness};

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}"))
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

fn write_sequence_asset(path: &Path) {
    let asset = AnimationSequenceAsset {
        name: Some("Hero Sequence".to_string()),
        duration_seconds: 2.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Root/Hero").unwrap(),
            tracks: vec![AnimationSequenceTrackAsset {
                property_path: ComponentPropertyPath::parse("AnimationPlayer.weight").unwrap(),
                channel: scalar_channel(1.0),
            }],
        }],
    };
    fs::write(path, asset.to_bytes().unwrap()).unwrap();
}

fn write_sequence_asset_with_multiple_tracks(path: &Path) {
    let asset = AnimationSequenceAsset {
        name: Some("Hero Sequence".to_string()),
        duration_seconds: 2.0,
        frames_per_second: 30.0,
        bindings: vec![AnimationSequenceBindingAsset {
            entity_path: EntityPath::parse("Root/Hero").unwrap(),
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

fn write_state_machine_asset(path: &Path) {
    let graph_reference = zircon_runtime::asset::AssetReference::from_locator(
        AssetUri::parse("res://animation/hero.graph.zranim").unwrap(),
    );
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
        transitions: Vec::new(),
    };
    fs::write(path, asset.to_bytes().unwrap()).unwrap();
}

fn write_state_machine_asset_with_transition(path: &Path) {
    let graph_reference = zircon_runtime::asset::AssetReference::from_locator(
        AssetUri::parse("res://animation/hero.graph.zranim").unwrap(),
    );
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
    fs::write(path, asset.to_bytes().unwrap()).unwrap();
}

fn write_graph_asset(path: &Path) {
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

#[test]
fn animation_sequence_binding_marks_active_sequence_editor_dirty_and_updates_session_state() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_editor_event_animation_sequence_dirty");
    let asset_path = unique_temp_dir("zircon_editor_event_animation_sequence_asset")
        .join("hero.sequence.zranim");
    fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
    write_sequence_asset(&asset_path);

    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: asset_path.to_string_lossy().into_owned(),
            }),
        )
        .unwrap();

    let binding = EditorUiBinding::new(
        "AnimationSequenceEditorView",
        "CreateTrackButton",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::animation_command(AnimationCommand::CreateTrack {
            track_path: "Root/Hero:Transform.translation".to_string(),
        }),
    );
    harness
        .runtime
        .dispatch_binding(binding, EditorEventSource::Headless)
        .unwrap();

    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = harness
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| {
            instance.descriptor_id == ViewDescriptorId::new("editor.animation_sequence")
        })
        .expect("sequence editor view should stay open");
    let pane = manager
        .animation_editor_pane_presentation(&instance.instance_id)
        .expect("sequence session should be queryable after command");

    assert!(
        instance.dirty,
        "animation authoring command should mark instance dirty"
    );
    assert!(pane
        .track_items
        .contains(&"Root/Hero:Transform.translation".to_string()));
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Created animation track Root/Hero:Transform.translation"
    );
}

#[test]
fn animation_sequence_ignores_timeline_selection_for_missing_track() {
    let _guard = env_lock().lock().unwrap();
    let harness =
        EventRuntimeHarness::new("zircon_editor_event_animation_sequence_missing_selection");
    let asset_path = unique_temp_dir("zircon_editor_event_animation_sequence_missing_selection")
        .join("hero.sequence.zranim");
    fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
    write_sequence_asset(&asset_path);

    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: asset_path.to_string_lossy().into_owned(),
            }),
        )
        .unwrap();
    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Animation(
                crate::core::editor_event::EditorAnimationEvent::SelectTimelineSpan {
                    track_path:
                        zircon_runtime::core::framework::animation::AnimationTrackPath::parse(
                            "Root/Hero:Transform.rotation",
                        )
                        .unwrap(),
                    start_frame: 24,
                    end_frame: 48,
                },
            ),
        )
        .unwrap();

    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = harness
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| {
            instance.descriptor_id == ViewDescriptorId::new("editor.animation_sequence")
        })
        .expect("sequence editor view should stay open");
    let pane = manager
        .animation_editor_pane_presentation(&instance.instance_id)
        .expect("sequence session should remain queryable after invalid selection");

    assert!(
        pane.selection_summary.is_empty(),
        "missing-track selection should not create a phantom timeline selection"
    );
    assert!(
        !instance.dirty,
        "missing-track selection should remain a no-op for the document"
    );
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Ignored animation command because it did not change the current document"
    );
}

#[test]
fn animation_sequence_removing_selected_track_clears_selection_summary() {
    let _guard = env_lock().lock().unwrap();
    let harness =
        EventRuntimeHarness::new("zircon_editor_event_animation_sequence_remove_selected");
    let asset_path = unique_temp_dir("zircon_editor_event_animation_sequence_remove_selected")
        .join("hero.sequence.zranim");
    fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
    write_sequence_asset_with_multiple_tracks(&asset_path);

    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: asset_path.to_string_lossy().into_owned(),
            }),
        )
        .unwrap();
    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Animation(
                crate::core::editor_event::EditorAnimationEvent::SelectTimelineSpan {
                    track_path:
                        zircon_runtime::core::framework::animation::AnimationTrackPath::parse(
                            "Root/Hero:Transform.translation",
                        )
                        .unwrap(),
                    start_frame: 24,
                    end_frame: 48,
                },
            ),
        )
        .unwrap();
    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Animation(
                crate::core::editor_event::EditorAnimationEvent::RemoveTrack {
                    track_path:
                        zircon_runtime::core::framework::animation::AnimationTrackPath::parse(
                            "Root/Hero:Transform.translation",
                        )
                        .unwrap(),
                },
            ),
        )
        .unwrap();

    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = harness
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| {
            instance.descriptor_id == ViewDescriptorId::new("editor.animation_sequence")
        })
        .expect("sequence editor view should stay open");
    let pane = manager
        .animation_editor_pane_presentation(&instance.instance_id)
        .expect("sequence session should remain queryable after removing a selected track");

    assert!(
        pane.track_items
            .iter()
            .all(|item| item != "Root/Hero:Transform.translation"),
        "removed track should disappear from the sequence session"
    );
    assert!(
        pane.selection_summary.is_empty(),
        "removing the selected track should clear the stale timeline selection"
    );
    assert!(
        instance.dirty,
        "removing the selected track should mark the document dirty"
    );
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Removed animation track Root/Hero:Transform.translation"
    );
}

#[test]
fn animation_state_machine_event_marks_open_graph_editor_dirty_and_updates_transition_summary() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_editor_event_animation_state_machine_dirty");
    let asset_path = unique_temp_dir("zircon_editor_event_animation_state_machine_asset")
        .join("hero.state_machine.zranim");
    fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
    write_state_machine_asset(&asset_path);

    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: asset_path.to_string_lossy().into_owned(),
            }),
        )
        .unwrap();
    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Animation(
                crate::core::editor_event::EditorAnimationEvent::CreateTransition {
                    state_machine_path: asset_path.to_string_lossy().into_owned(),
                    from_state: "Idle".to_string(),
                    to_state: "Run".to_string(),
                    duration_frames: 8,
                },
            ),
        )
        .unwrap();

    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = harness
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.animation_graph"))
        .expect("graph editor view should stay open");
    let pane = manager
        .animation_editor_pane_presentation(&instance.instance_id)
        .expect("graph editor session should be queryable after command");

    assert!(
        instance.dirty,
        "state-machine authoring command should mark instance dirty"
    );
    assert_eq!(pane.transition_items, vec!["Idle -> Run"]);
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        format!(
            "Created animation transition Idle -> Run in {} (8 frames)",
            asset_path.to_string_lossy()
        )
    );
}

#[test]
fn animation_state_machine_ignores_missing_entry_state_requests() {
    let _guard = env_lock().lock().unwrap();
    let harness =
        EventRuntimeHarness::new("zircon_editor_event_animation_state_machine_invalid_entry");
    let asset_path = unique_temp_dir("zircon_editor_event_animation_state_machine_invalid_entry")
        .join("hero.state_machine.zranim");
    fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
    write_state_machine_asset(&asset_path);

    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: asset_path.to_string_lossy().into_owned(),
            }),
        )
        .unwrap();
    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Animation(
                crate::core::editor_event::EditorAnimationEvent::SetEntryState {
                    state_machine_path: asset_path.to_string_lossy().into_owned(),
                    state_name: "Jump".to_string(),
                },
            ),
        )
        .unwrap();

    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = harness
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.animation_graph"))
        .expect("graph editor view should stay open");
    let pane = manager
        .animation_editor_pane_presentation(&instance.instance_id)
        .expect("graph editor session should remain queryable after invalid entry request");

    assert_eq!(
        pane.selection_summary, "Idle",
        "invalid entry-state request should preserve the current entry state"
    );
    assert!(
        !instance.dirty,
        "invalid entry-state request should remain a no-op"
    );
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Ignored animation command because it did not change the current document"
    );
}

#[test]
fn animation_state_machine_ignores_transition_requests_with_missing_states() {
    let _guard = env_lock().lock().unwrap();
    let harness =
        EventRuntimeHarness::new("zircon_editor_event_animation_state_machine_invalid_transition");
    let asset_path =
        unique_temp_dir("zircon_editor_event_animation_state_machine_invalid_transition")
            .join("hero.state_machine.zranim");
    fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
    write_state_machine_asset(&asset_path);

    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: asset_path.to_string_lossy().into_owned(),
            }),
        )
        .unwrap();
    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Animation(
                crate::core::editor_event::EditorAnimationEvent::CreateTransition {
                    state_machine_path: asset_path.to_string_lossy().into_owned(),
                    from_state: "Idle".to_string(),
                    to_state: "Jump".to_string(),
                    duration_frames: 8,
                },
            ),
        )
        .unwrap();

    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = harness
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.animation_graph"))
        .expect("graph editor view should stay open");
    let pane = manager
        .animation_editor_pane_presentation(&instance.instance_id)
        .expect("graph editor session should remain queryable after invalid transition request");

    assert!(
        pane.transition_items.is_empty(),
        "invalid transition request should not create orphaned transitions"
    );
    assert!(
        !instance.dirty,
        "invalid transition request should remain a no-op"
    );
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Ignored animation command because it did not change the current document"
    );
}

#[test]
fn animation_state_machine_ignores_condition_requests_for_missing_transitions() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new(
        "zircon_editor_event_animation_state_machine_missing_transition_condition",
    );
    let asset_path =
        unique_temp_dir("zircon_editor_event_animation_state_machine_missing_transition")
            .join("hero.state_machine.zranim");
    fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
    write_state_machine_asset(&asset_path);

    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: asset_path.to_string_lossy().into_owned(),
            }),
        )
        .unwrap();
    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Animation(
                crate::core::editor_event::EditorAnimationEvent::SetTransitionCondition {
                    state_machine_path: asset_path.to_string_lossy().into_owned(),
                    from_state: "Idle".to_string(),
                    to_state: "Run".to_string(),
                    parameter_name: "speed".to_string(),
                    operator: "greater_equal".to_string(),
                    value_literal: "1.0".to_string(),
                },
            ),
        )
        .unwrap();

    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = harness
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.animation_graph"))
        .expect("graph editor view should stay open");
    let pane = manager
        .animation_editor_pane_presentation(&instance.instance_id)
        .expect("graph editor session should remain queryable after missing-transition request");

    assert!(
        pane.transition_items.is_empty(),
        "condition authoring should not create implicit transitions"
    );
    assert!(
        !instance.dirty,
        "missing-transition condition request should remain a no-op"
    );
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Ignored animation command because it did not change the current document"
    );
}

#[test]
fn animation_state_machine_ignores_unknown_transition_condition_operator() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new(
        "zircon_editor_event_animation_state_machine_unknown_condition_operator",
    );
    let asset_path =
        unique_temp_dir("zircon_editor_event_animation_state_machine_unknown_condition_operator")
            .join("hero.state_machine.zranim");
    fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
    write_state_machine_asset_with_transition(&asset_path);

    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: asset_path.to_string_lossy().into_owned(),
            }),
        )
        .unwrap();
    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Animation(
                crate::core::editor_event::EditorAnimationEvent::SetTransitionCondition {
                    state_machine_path: asset_path.to_string_lossy().into_owned(),
                    from_state: "Idle".to_string(),
                    to_state: "Run".to_string(),
                    parameter_name: "speed".to_string(),
                    operator: "approximately".to_string(),
                    value_literal: "2.5".to_string(),
                },
            ),
        )
        .unwrap();

    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = harness
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.animation_graph"))
        .expect("graph editor view should stay open");
    let pane = manager
        .animation_editor_pane_presentation(&instance.instance_id)
        .expect("graph editor session should remain queryable after invalid condition operator");

    assert_eq!(
        pane.transition_items,
        vec!["Idle -> Run"],
        "unknown condition operator should preserve the existing transition summary"
    );
    assert!(
        !instance.dirty,
        "unknown condition operator should remain a no-op for the document"
    );
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Ignored animation command because it did not change the current document"
    );
}

#[test]
fn animation_graph_ignores_duplicate_output_node_requests() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_editor_event_animation_graph_duplicate_output");
    let asset_path = unique_temp_dir("zircon_editor_event_animation_graph_duplicate_output")
        .join("hero.graph.zranim");
    fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
    write_graph_asset(&asset_path);

    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: asset_path.to_string_lossy().into_owned(),
            }),
        )
        .unwrap();
    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Animation(
                crate::core::editor_event::EditorAnimationEvent::AddGraphNode {
                    graph_path: asset_path.to_string_lossy().into_owned(),
                    node_id: "output_2".to_string(),
                    node_kind: "output".to_string(),
                },
            ),
        )
        .unwrap();

    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = harness
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.animation_graph"))
        .expect("graph editor view should stay open");
    let pane = manager
        .animation_editor_pane_presentation(&instance.instance_id)
        .expect("graph editor session should remain queryable after duplicate output request");

    assert_eq!(
        pane.node_items
            .iter()
            .filter(|item| item.starts_with("Output <-"))
            .count(),
        1,
        "graph editor should preserve a single output node"
    );
    assert!(
        !instance.dirty,
        "duplicate output node request should remain a no-op"
    );
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Ignored animation command because it did not change the current document"
    );
}

#[test]
fn animation_graph_removes_output_node_when_requested() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_editor_event_animation_graph_remove_output");
    let asset_path = unique_temp_dir("zircon_editor_event_animation_graph_remove_output")
        .join("hero.graph.zranim");
    fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
    write_graph_asset(&asset_path);

    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: asset_path.to_string_lossy().into_owned(),
            }),
        )
        .unwrap();
    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Animation(
                crate::core::editor_event::EditorAnimationEvent::RemoveGraphNode {
                    graph_path: asset_path.to_string_lossy().into_owned(),
                    node_id: "output".to_string(),
                },
            ),
        )
        .unwrap();

    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = harness
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.animation_graph"))
        .expect("graph editor view should stay open");
    let pane = manager
        .animation_editor_pane_presentation(&instance.instance_id)
        .expect("graph editor session should remain queryable after removing output");

    assert!(
        pane.node_items
            .iter()
            .all(|item| !item.starts_with("Output <-")),
        "removing the output node should clear it from the graph session"
    );
    assert!(
        instance.dirty,
        "removing the output node should mark the document dirty"
    );
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        format!(
            "Removed animation graph node output from {}",
            asset_path.to_string_lossy()
        )
    );
}

#[test]
fn animation_graph_ignores_connections_from_missing_source_nodes() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_editor_event_animation_graph_missing_source");
    let asset_path = unique_temp_dir("zircon_editor_event_animation_graph_missing_source")
        .join("hero.graph.zranim");
    fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
    write_graph_asset(&asset_path);

    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: asset_path.to_string_lossy().into_owned(),
            }),
        )
        .unwrap();
    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Animation(
                crate::core::editor_event::EditorAnimationEvent::ConnectGraphNodes {
                    graph_path: asset_path.to_string_lossy().into_owned(),
                    from_node_id: "ghost".to_string(),
                    to_node_id: "locomotion".to_string(),
                },
            ),
        )
        .unwrap();

    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = harness
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.animation_graph"))
        .expect("graph editor view should stay open");
    let pane = manager
        .animation_editor_pane_presentation(&instance.instance_id)
        .expect("graph editor session should remain queryable after invalid connection request");

    assert!(
        pane.node_items
            .iter()
            .any(|item| item == "Blend locomotion"),
        "invalid source connection should preserve the blend node's original inputs"
    );
    assert!(
        pane.node_items.iter().all(|item| !item.contains("ghost")),
        "invalid source connection should not write dangling node references"
    );
    assert!(
        !instance.dirty,
        "invalid source connection should remain a no-op"
    );
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Ignored animation command because it did not change the current document"
    );
}

#[test]
fn animation_graph_ignores_self_referential_connections() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_editor_event_animation_graph_self_cycle");
    let asset_path = unique_temp_dir("zircon_editor_event_animation_graph_self_cycle")
        .join("hero.graph.zranim");
    fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
    write_graph_asset(&asset_path);

    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: asset_path.to_string_lossy().into_owned(),
            }),
        )
        .unwrap();
    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Animation(
                crate::core::editor_event::EditorAnimationEvent::ConnectGraphNodes {
                    graph_path: asset_path.to_string_lossy().into_owned(),
                    from_node_id: "locomotion".to_string(),
                    to_node_id: "locomotion".to_string(),
                },
            ),
        )
        .unwrap();

    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = harness
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.animation_graph"))
        .expect("graph editor view should stay open");
    let pane = manager
        .animation_editor_pane_presentation(&instance.instance_id)
        .expect("graph editor session should remain queryable after self-cycle request");

    assert!(
        pane.node_items
            .iter()
            .all(|item| item != "Blend locomotion • locomotion"),
        "self-referential graph connections should not be written into the graph session"
    );
    assert!(
        !instance.dirty,
        "self-referential graph connections should remain a no-op"
    );
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Ignored animation command because it did not change the current document"
    );
}

#[test]
fn animation_graph_ignores_unknown_node_kinds() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_editor_event_animation_graph_unknown_kind");
    let asset_path = unique_temp_dir("zircon_editor_event_animation_graph_unknown_kind")
        .join("hero.graph.zranim");
    fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
    write_graph_asset(&asset_path);

    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: asset_path.to_string_lossy().into_owned(),
            }),
        )
        .unwrap();
    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Animation(
                crate::core::editor_event::EditorAnimationEvent::AddGraphNode {
                    graph_path: asset_path.to_string_lossy().into_owned(),
                    node_id: "run".to_string(),
                    node_kind: "clip".to_string(),
                },
            ),
        )
        .unwrap();

    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = harness
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.animation_graph"))
        .expect("graph editor view should stay open");
    let pane = manager
        .animation_editor_pane_presentation(&instance.instance_id)
        .expect("graph editor session should remain queryable after unknown-node request");

    assert!(
        pane.node_items.iter().all(|item| item != "Blend run"),
        "unsupported graph node kinds should not silently degrade into blend nodes"
    );
    assert!(
        !instance.dirty,
        "unknown graph node kinds should remain a no-op"
    );
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Ignored animation command because it did not change the current document"
    );
}

#[test]
fn animation_rebind_to_existing_track_keeps_original_sequence_tracks_intact() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_editor_event_animation_rebind_duplicate");
    let asset_path = unique_temp_dir("zircon_editor_event_animation_rebind_duplicate_asset")
        .join("hero.sequence.zranim");
    fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
    write_sequence_asset_with_multiple_tracks(&asset_path);

    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: asset_path.to_string_lossy().into_owned(),
            }),
        )
        .unwrap();
    harness
        .runtime
        .dispatch_binding(
            EditorUiBinding::new(
                "AnimationSequenceEditorView",
                "RebindTrackButton",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::animation_command(AnimationCommand::RebindTrack {
                    from_track_path: "Root/Hero:AnimationPlayer.weight".to_string(),
                    to_track_path: "Root/Hero:Transform.translation".to_string(),
                }),
            ),
            EditorEventSource::Headless,
        )
        .unwrap();

    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = harness
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| {
            instance.descriptor_id == ViewDescriptorId::new("editor.animation_sequence")
        })
        .expect("sequence editor view should stay open");
    let pane = manager
        .animation_editor_pane_presentation(&instance.instance_id)
        .expect("sequence session should remain queryable after duplicate rebind");

    assert_eq!(
        pane.track_items,
        vec![
            "Root/Hero:AnimationPlayer.weight".to_string(),
            "Root/Hero:Transform.translation".to_string(),
        ],
        "duplicate rebind should not delete the original source track"
    );
    assert!(
        !instance.dirty,
        "duplicate rebind should remain a no-op instead of marking the document dirty"
    );
}

#[test]
fn animation_rebind_updates_selected_timeline_span_to_new_track_path() {
    let _guard = env_lock().lock().unwrap();
    let harness =
        EventRuntimeHarness::new("zircon_editor_event_animation_rebind_updates_selection");
    let asset_path = unique_temp_dir("zircon_editor_event_animation_rebind_updates_selection")
        .join("hero.sequence.zranim");
    fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
    write_sequence_asset_with_multiple_tracks(&asset_path);

    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_path: asset_path.to_string_lossy().into_owned(),
            }),
        )
        .unwrap();
    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Animation(
                crate::core::editor_event::EditorAnimationEvent::SelectTimelineSpan {
                    track_path:
                        zircon_runtime::core::framework::animation::AnimationTrackPath::parse(
                            "Root/Hero:AnimationPlayer.weight",
                        )
                        .unwrap(),
                    start_frame: 24,
                    end_frame: 48,
                },
            ),
        )
        .unwrap();
    harness
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Animation(
                crate::core::editor_event::EditorAnimationEvent::RebindTrack {
                    from_track_path:
                        zircon_runtime::core::framework::animation::AnimationTrackPath::parse(
                            "Root/Hero:AnimationPlayer.weight",
                        )
                        .unwrap(),
                    to_track_path:
                        zircon_runtime::core::framework::animation::AnimationTrackPath::parse(
                            "Root/Hero:Transform.rotation",
                        )
                        .unwrap(),
                },
            ),
        )
        .unwrap();

    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let instance = harness
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| {
            instance.descriptor_id == ViewDescriptorId::new("editor.animation_sequence")
        })
        .expect("sequence editor view should stay open");
    let pane = manager
        .animation_editor_pane_presentation(&instance.instance_id)
        .expect("sequence session should remain queryable after rebind");

    assert!(
        pane.track_items
            .iter()
            .any(|item| item == "Root/Hero:Transform.rotation"),
        "rebound track should appear under the destination track path"
    );
    assert_eq!(
        pane.selection_summary, "Root/Hero:Transform.rotation [24..48]",
        "rebind should migrate the selected timeline span to the destination path"
    );
    assert!(
        instance.dirty,
        "successful rebind should mark the document dirty"
    );
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Rebound animation track Root/Hero:AnimationPlayer.weight -> Root/Hero:Transform.rotation"
    );
}
