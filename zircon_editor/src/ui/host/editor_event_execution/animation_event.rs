use crate::core::editor_event::{EditorAnimationEvent, EditorEventEffect};

use super::execution_outcome::ExecutionOutcome;
use crate::core::editor_event::runtime::editor_event_runtime_inner::EditorEventRuntimeInner;

pub(super) fn execute_animation_event(
    inner: &mut EditorEventRuntimeInner,
    event: &EditorAnimationEvent,
) -> Result<ExecutionOutcome, String> {
    let changed = match inner.manager.apply_animation_event(event) {
        Ok(changed) => changed,
        Err(error) if should_tolerate_missing_animation_target(&error.to_string()) => {
            inner
                .state
                .set_status_line(ignored_status_line_for_error(&error.to_string()));
            return Ok(ExecutionOutcome {
                changed: false,
                effects: vec![
                    EditorEventEffect::PresentationChanged,
                    EditorEventEffect::ReflectionChanged,
                ],
            });
        }
        Err(error) => return Err(error.to_string()),
    };
    inner.state.set_status_line(if changed {
        status_line_for_event(event)
    } else {
        ignored_status_line_for_no_change()
    });
    Ok(ExecutionOutcome {
        changed,
        effects: vec![
            EditorEventEffect::PresentationChanged,
            EditorEventEffect::ReflectionChanged,
        ],
    })
}

fn should_tolerate_missing_animation_target(message: &str) -> bool {
    message == "no active animation sequence editor"
        || message == "active center tab is not an animation sequence editor"
        || message == "no active animation graph editor"
        || message == "active center tab is not an animation graph editor"
        || message.starts_with("missing active animation sequence view ")
        || message.starts_with("missing active animation graph view ")
}

fn ignored_status_line_for_error(message: &str) -> String {
    format!("Ignored animation command because {message}")
}

fn ignored_status_line_for_no_change() -> String {
    "Ignored animation command because it did not change the current document".to_string()
}

fn status_line_for_event(event: &EditorAnimationEvent) -> String {
    match event {
        EditorAnimationEvent::AddKey { track_path, frame } => {
            format!("Added animation key {track_path} at frame {frame}")
        }
        EditorAnimationEvent::RemoveKey { track_path, frame } => {
            format!("Removed animation key {track_path} at frame {frame}")
        }
        EditorAnimationEvent::CreateTrack { track_path } => {
            format!("Created animation track {track_path}")
        }
        EditorAnimationEvent::RemoveTrack { track_path } => {
            format!("Removed animation track {track_path}")
        }
        EditorAnimationEvent::RebindTrack {
            from_track_path,
            to_track_path,
        } => format!("Rebound animation track {from_track_path} -> {to_track_path}"),
        EditorAnimationEvent::ScrubTimeline { frame } => {
            format!("Scrubbed animation timeline to frame {frame}")
        }
        EditorAnimationEvent::SetTimelineRange {
            start_frame,
            end_frame,
        } => format!("Set animation timeline range to {start_frame}..{end_frame}"),
        EditorAnimationEvent::SelectTimelineSpan {
            track_path,
            start_frame,
            end_frame,
        } => format!(
            "Selected animation timeline span {track_path} [{start_frame}..{end_frame}]"
        ),
        EditorAnimationEvent::SetPlayback {
            playing,
            looping,
            speed,
        } => {
            format!("Updated animation playback playing={playing} looping={looping} speed={speed}")
        }
        EditorAnimationEvent::AddGraphNode {
            graph_path,
            node_id,
            node_kind,
        } => format!("Added animation graph node {node_id} ({node_kind}) to {graph_path}"),
        EditorAnimationEvent::RemoveGraphNode {
            graph_path,
            node_id,
        } => format!("Removed animation graph node {node_id} from {graph_path}"),
        EditorAnimationEvent::ConnectGraphNodes {
            graph_path,
            from_node_id,
            to_node_id,
        } => format!(
            "Connected animation graph nodes {from_node_id} -> {to_node_id} in {graph_path}"
        ),
        EditorAnimationEvent::DisconnectGraphNodes {
            graph_path,
            from_node_id,
            to_node_id,
        } => format!(
            "Disconnected animation graph nodes {from_node_id} -> {to_node_id} in {graph_path}"
        ),
        EditorAnimationEvent::SetGraphParameter {
            graph_path,
            parameter_name,
            value_literal,
        } => format!(
            "Set animation graph parameter {parameter_name}={value_literal} in {graph_path}"
        ),
        EditorAnimationEvent::CreateState {
            state_machine_path,
            state_name,
            graph_path,
        } => format!(
            "Created animation state {state_name} in {state_machine_path} using {graph_path}"
        ),
        EditorAnimationEvent::RemoveState {
            state_machine_path,
            state_name,
        } => format!("Removed animation state {state_name} from {state_machine_path}"),
        EditorAnimationEvent::SetEntryState {
            state_machine_path,
            state_name,
        } => format!("Set animation entry state {state_name} in {state_machine_path}"),
        EditorAnimationEvent::CreateTransition {
            state_machine_path,
            from_state,
            to_state,
            duration_frames,
        } => format!(
            "Created animation transition {from_state} -> {to_state} in {state_machine_path} ({duration_frames} frames)"
        ),
        EditorAnimationEvent::RemoveTransition {
            state_machine_path,
            from_state,
            to_state,
        } => format!(
            "Removed animation transition {from_state} -> {to_state} from {state_machine_path}"
        ),
        EditorAnimationEvent::SetTransitionCondition {
            state_machine_path,
            from_state,
            to_state,
            parameter_name,
            operator,
            value_literal,
        } => format!(
            "Set animation transition condition {from_state} -> {to_state} in {state_machine_path}: {parameter_name} {operator} {value_literal}"
        ),
    }
}
