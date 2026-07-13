use crate::ui::binding::{AnimationCommand, EditorUiBinding, EditorUiBindingPayload};
use zircon_runtime::core::framework::animation::AnimationTrackPath;

use super::super::error::EditorBindingDispatchError;
use super::animation_host_event::AnimationHostEvent;

pub fn dispatch_animation_binding(
    binding: &EditorUiBinding,
) -> Result<AnimationHostEvent, EditorBindingDispatchError> {
    match binding.payload() {
        EditorUiBindingPayload::AnimationCommand(command) => dispatch_animation_command(command),
        _ => Err(EditorBindingDispatchError::UnsupportedPayload),
    }
}

fn dispatch_animation_command(
    command: &AnimationCommand,
) -> Result<AnimationHostEvent, EditorBindingDispatchError> {
    match command {
        AnimationCommand::AddKey { track_path, frame } => Ok(AnimationHostEvent::AddKey {
            track_path: parse_track_path(track_path)?,
            frame: *frame,
        }),
        AnimationCommand::RemoveKey { track_path, frame } => Ok(AnimationHostEvent::RemoveKey {
            track_path: parse_track_path(track_path)?,
            frame: *frame,
        }),
        AnimationCommand::CreateTrack { track_path } => Ok(AnimationHostEvent::CreateTrack {
            track_path: parse_track_path(track_path)?,
        }),
        AnimationCommand::RemoveTrack { track_path } => Ok(AnimationHostEvent::RemoveTrack {
            track_path: parse_track_path(track_path)?,
        }),
        AnimationCommand::RebindTrack {
            from_track_path,
            to_track_path,
        } => Ok(AnimationHostEvent::RebindTrack {
            from_track_path: parse_track_path(from_track_path)?,
            to_track_path: parse_track_path(to_track_path)?,
        }),
        AnimationCommand::ScrubTimeline { frame } => {
            Ok(AnimationHostEvent::ScrubTimeline { frame: *frame })
        }
        AnimationCommand::SetTimelineRange {
            start_frame,
            end_frame,
        } => Ok(AnimationHostEvent::SetTimelineRange {
            start_frame: *start_frame,
            end_frame: *end_frame,
        }),
        AnimationCommand::SelectTimelineSpan {
            track_path,
            start_frame,
            end_frame,
        } => Ok(AnimationHostEvent::SelectTimelineSpan {
            track_path: parse_track_path(track_path)?,
            start_frame: *start_frame,
            end_frame: *end_frame,
        }),
        AnimationCommand::SetPlayback {
            playing,
            looping,
            speed,
        } => Ok(AnimationHostEvent::SetPlayback {
            playing: *playing,
            looping: *looping,
            speed: *speed,
        }),
        AnimationCommand::AddGraphNode {
            graph_locator,
            node_id,
            node_kind,
        } => Ok(AnimationHostEvent::AddGraphNode {
            graph_locator: graph_locator.clone(),
            node_id: node_id.clone(),
            node_kind: node_kind.clone(),
        }),
        AnimationCommand::RemoveGraphNode {
            graph_locator,
            node_id,
        } => Ok(AnimationHostEvent::RemoveGraphNode {
            graph_locator: graph_locator.clone(),
            node_id: node_id.clone(),
        }),
        AnimationCommand::ConnectGraphNodes {
            graph_locator,
            from_node_id,
            to_node_id,
        } => Ok(AnimationHostEvent::ConnectGraphNodes {
            graph_locator: graph_locator.clone(),
            from_node_id: from_node_id.clone(),
            to_node_id: to_node_id.clone(),
        }),
        AnimationCommand::DisconnectGraphNodes {
            graph_locator,
            from_node_id,
            to_node_id,
        } => Ok(AnimationHostEvent::DisconnectGraphNodes {
            graph_locator: graph_locator.clone(),
            from_node_id: from_node_id.clone(),
            to_node_id: to_node_id.clone(),
        }),
        AnimationCommand::SetGraphParameter {
            graph_locator,
            parameter_name,
            value_literal,
        } => Ok(AnimationHostEvent::SetGraphParameter {
            graph_locator: graph_locator.clone(),
            parameter_name: parameter_name.clone(),
            value_literal: value_literal.clone(),
        }),
        AnimationCommand::CreateState {
            state_machine_locator,
            state_name,
            graph_locator,
        } => Ok(AnimationHostEvent::CreateState {
            state_machine_locator: state_machine_locator.clone(),
            state_name: state_name.clone(),
            graph_locator: graph_locator.clone(),
        }),
        AnimationCommand::RemoveState {
            state_machine_locator,
            state_name,
        } => Ok(AnimationHostEvent::RemoveState {
            state_machine_locator: state_machine_locator.clone(),
            state_name: state_name.clone(),
        }),
        AnimationCommand::SetEntryState {
            state_machine_locator,
            state_name,
        } => Ok(AnimationHostEvent::SetEntryState {
            state_machine_locator: state_machine_locator.clone(),
            state_name: state_name.clone(),
        }),
        AnimationCommand::CreateTransition {
            state_machine_locator,
            from_state,
            to_state,
            duration_frames,
        } => Ok(AnimationHostEvent::CreateTransition {
            state_machine_locator: state_machine_locator.clone(),
            from_state: from_state.clone(),
            to_state: to_state.clone(),
            duration_frames: *duration_frames,
        }),
        AnimationCommand::RemoveTransition {
            state_machine_locator,
            from_state,
            to_state,
        } => Ok(AnimationHostEvent::RemoveTransition {
            state_machine_locator: state_machine_locator.clone(),
            from_state: from_state.clone(),
            to_state: to_state.clone(),
        }),
        AnimationCommand::SetTransitionCondition {
            state_machine_locator,
            from_state,
            to_state,
            parameter_name,
            operator,
            value_literal,
        } => Ok(AnimationHostEvent::SetTransitionCondition {
            state_machine_locator: state_machine_locator.clone(),
            from_state: from_state.clone(),
            to_state: to_state.clone(),
            parameter_name: parameter_name.clone(),
            operator: operator.clone(),
            value_literal: value_literal.clone(),
        }),
    }
}

fn parse_track_path(raw: &str) -> Result<AnimationTrackPath, EditorBindingDispatchError> {
    AnimationTrackPath::parse(raw)
        .map_err(|error| EditorBindingDispatchError::InvalidAnimationTrackPath(error.to_string()))
}
