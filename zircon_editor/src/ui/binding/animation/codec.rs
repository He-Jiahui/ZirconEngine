use zircon_runtime_interface::ui::{binding::UiBindingCall, binding::UiBindingValue};

use super::AnimationCommand;
use crate::ui::binding::core::{
    required_bool_argument, required_f32_argument, required_string_argument, required_u32_argument,
    EditorUiBindingError,
};

impl AnimationCommand {
    pub(crate) fn to_call(&self) -> UiBindingCall {
        match self {
            Self::AddKey { track_path, frame } => UiBindingCall::new("AnimationCommand.AddKey")
                .with_argument(UiBindingValue::string(track_path))
                .with_argument(UiBindingValue::unsigned(*frame)),
            Self::RemoveKey { track_path, frame } => {
                UiBindingCall::new("AnimationCommand.RemoveKey")
                    .with_argument(UiBindingValue::string(track_path))
                    .with_argument(UiBindingValue::unsigned(*frame))
            }
            Self::CreateTrack { track_path } => UiBindingCall::new("AnimationCommand.CreateTrack")
                .with_argument(UiBindingValue::string(track_path)),
            Self::RemoveTrack { track_path } => UiBindingCall::new("AnimationCommand.RemoveTrack")
                .with_argument(UiBindingValue::string(track_path)),
            Self::RebindTrack {
                from_track_path,
                to_track_path,
            } => UiBindingCall::new("AnimationCommand.RebindTrack")
                .with_argument(UiBindingValue::string(from_track_path))
                .with_argument(UiBindingValue::string(to_track_path)),
            Self::ScrubTimeline { frame } => UiBindingCall::new("AnimationCommand.ScrubTimeline")
                .with_argument(UiBindingValue::unsigned(*frame)),
            Self::SetTimelineRange {
                start_frame,
                end_frame,
            } => UiBindingCall::new("AnimationCommand.SetTimelineRange")
                .with_argument(UiBindingValue::unsigned(*start_frame))
                .with_argument(UiBindingValue::unsigned(*end_frame)),
            Self::SelectTimelineSpan {
                track_path,
                start_frame,
                end_frame,
            } => UiBindingCall::new("AnimationCommand.SelectTimelineSpan")
                .with_argument(UiBindingValue::string(track_path))
                .with_argument(UiBindingValue::unsigned(*start_frame))
                .with_argument(UiBindingValue::unsigned(*end_frame)),
            Self::SetPlayback {
                playing,
                looping,
                speed,
            } => UiBindingCall::new("AnimationCommand.SetPlayback")
                .with_argument(UiBindingValue::Bool(*playing))
                .with_argument(UiBindingValue::Bool(*looping))
                .with_argument(UiBindingValue::Float(*speed as f64)),
            Self::AddGraphNode {
                graph_path,
                node_id,
                node_kind,
            } => UiBindingCall::new("AnimationCommand.AddGraphNode")
                .with_argument(UiBindingValue::string(graph_path))
                .with_argument(UiBindingValue::string(node_id))
                .with_argument(UiBindingValue::string(node_kind)),
            Self::RemoveGraphNode {
                graph_path,
                node_id,
            } => UiBindingCall::new("AnimationCommand.RemoveGraphNode")
                .with_argument(UiBindingValue::string(graph_path))
                .with_argument(UiBindingValue::string(node_id)),
            Self::ConnectGraphNodes {
                graph_path,
                from_node_id,
                to_node_id,
            } => UiBindingCall::new("AnimationCommand.ConnectGraphNodes")
                .with_argument(UiBindingValue::string(graph_path))
                .with_argument(UiBindingValue::string(from_node_id))
                .with_argument(UiBindingValue::string(to_node_id)),
            Self::DisconnectGraphNodes {
                graph_path,
                from_node_id,
                to_node_id,
            } => UiBindingCall::new("AnimationCommand.DisconnectGraphNodes")
                .with_argument(UiBindingValue::string(graph_path))
                .with_argument(UiBindingValue::string(from_node_id))
                .with_argument(UiBindingValue::string(to_node_id)),
            Self::SetGraphParameter {
                graph_path,
                parameter_name,
                value_literal,
            } => UiBindingCall::new("AnimationCommand.SetGraphParameter")
                .with_argument(UiBindingValue::string(graph_path))
                .with_argument(UiBindingValue::string(parameter_name))
                .with_argument(UiBindingValue::string(value_literal)),
            Self::CreateState {
                state_machine_path,
                state_name,
                graph_path,
            } => UiBindingCall::new("AnimationCommand.CreateState")
                .with_argument(UiBindingValue::string(state_machine_path))
                .with_argument(UiBindingValue::string(state_name))
                .with_argument(UiBindingValue::string(graph_path)),
            Self::RemoveState {
                state_machine_path,
                state_name,
            } => UiBindingCall::new("AnimationCommand.RemoveState")
                .with_argument(UiBindingValue::string(state_machine_path))
                .with_argument(UiBindingValue::string(state_name)),
            Self::SetEntryState {
                state_machine_path,
                state_name,
            } => UiBindingCall::new("AnimationCommand.SetEntryState")
                .with_argument(UiBindingValue::string(state_machine_path))
                .with_argument(UiBindingValue::string(state_name)),
            Self::CreateTransition {
                state_machine_path,
                from_state,
                to_state,
                duration_frames,
            } => UiBindingCall::new("AnimationCommand.CreateTransition")
                .with_argument(UiBindingValue::string(state_machine_path))
                .with_argument(UiBindingValue::string(from_state))
                .with_argument(UiBindingValue::string(to_state))
                .with_argument(UiBindingValue::unsigned(*duration_frames)),
            Self::RemoveTransition {
                state_machine_path,
                from_state,
                to_state,
            } => UiBindingCall::new("AnimationCommand.RemoveTransition")
                .with_argument(UiBindingValue::string(state_machine_path))
                .with_argument(UiBindingValue::string(from_state))
                .with_argument(UiBindingValue::string(to_state)),
            Self::SetTransitionCondition {
                state_machine_path,
                from_state,
                to_state,
                parameter_name,
                operator,
                value_literal,
            } => UiBindingCall::new("AnimationCommand.SetTransitionCondition")
                .with_argument(UiBindingValue::string(state_machine_path))
                .with_argument(UiBindingValue::string(from_state))
                .with_argument(UiBindingValue::string(to_state))
                .with_argument(UiBindingValue::string(parameter_name))
                .with_argument(UiBindingValue::string(operator))
                .with_argument(UiBindingValue::string(value_literal)),
        }
    }

    pub(crate) fn from_call(call: UiBindingCall) -> Result<Option<Self>, EditorUiBindingError> {
        let command = match call.symbol.as_str() {
            "AnimationCommand.AddKey" => Self::AddKey {
                track_path: required_string_argument(&call, 0, "AnimationCommand.AddKey")?,
                frame: required_u32_argument(&call, 1, "AnimationCommand.AddKey")?,
            },
            "AnimationCommand.RemoveKey" => Self::RemoveKey {
                track_path: required_string_argument(&call, 0, "AnimationCommand.RemoveKey")?,
                frame: required_u32_argument(&call, 1, "AnimationCommand.RemoveKey")?,
            },
            "AnimationCommand.CreateTrack" => Self::CreateTrack {
                track_path: required_string_argument(&call, 0, "AnimationCommand.CreateTrack")?,
            },
            "AnimationCommand.RemoveTrack" => Self::RemoveTrack {
                track_path: required_string_argument(&call, 0, "AnimationCommand.RemoveTrack")?,
            },
            "AnimationCommand.RebindTrack" => Self::RebindTrack {
                from_track_path: required_string_argument(
                    &call,
                    0,
                    "AnimationCommand.RebindTrack",
                )?,
                to_track_path: required_string_argument(&call, 1, "AnimationCommand.RebindTrack")?,
            },
            "AnimationCommand.ScrubTimeline" => Self::ScrubTimeline {
                frame: required_u32_argument(&call, 0, "AnimationCommand.ScrubTimeline")?,
            },
            "AnimationCommand.SetTimelineRange" => Self::SetTimelineRange {
                start_frame: required_u32_argument(&call, 0, "AnimationCommand.SetTimelineRange")?,
                end_frame: required_u32_argument(&call, 1, "AnimationCommand.SetTimelineRange")?,
            },
            "AnimationCommand.SelectTimelineSpan" => Self::SelectTimelineSpan {
                track_path: required_string_argument(
                    &call,
                    0,
                    "AnimationCommand.SelectTimelineSpan",
                )?,
                start_frame: required_u32_argument(
                    &call,
                    1,
                    "AnimationCommand.SelectTimelineSpan",
                )?,
                end_frame: required_u32_argument(&call, 2, "AnimationCommand.SelectTimelineSpan")?,
            },
            "AnimationCommand.SetPlayback" => Self::SetPlayback {
                playing: required_bool_argument(&call, 0, "AnimationCommand.SetPlayback")?,
                looping: required_bool_argument(&call, 1, "AnimationCommand.SetPlayback")?,
                speed: required_f32_argument(&call, 2, "AnimationCommand.SetPlayback")?,
            },
            "AnimationCommand.AddGraphNode" => Self::AddGraphNode {
                graph_path: required_string_argument(&call, 0, "AnimationCommand.AddGraphNode")?,
                node_id: required_string_argument(&call, 1, "AnimationCommand.AddGraphNode")?,
                node_kind: required_string_argument(&call, 2, "AnimationCommand.AddGraphNode")?,
            },
            "AnimationCommand.RemoveGraphNode" => Self::RemoveGraphNode {
                graph_path: required_string_argument(&call, 0, "AnimationCommand.RemoveGraphNode")?,
                node_id: required_string_argument(&call, 1, "AnimationCommand.RemoveGraphNode")?,
            },
            "AnimationCommand.ConnectGraphNodes" => Self::ConnectGraphNodes {
                graph_path: required_string_argument(
                    &call,
                    0,
                    "AnimationCommand.ConnectGraphNodes",
                )?,
                from_node_id: required_string_argument(
                    &call,
                    1,
                    "AnimationCommand.ConnectGraphNodes",
                )?,
                to_node_id: required_string_argument(
                    &call,
                    2,
                    "AnimationCommand.ConnectGraphNodes",
                )?,
            },
            "AnimationCommand.DisconnectGraphNodes" => Self::DisconnectGraphNodes {
                graph_path: required_string_argument(
                    &call,
                    0,
                    "AnimationCommand.DisconnectGraphNodes",
                )?,
                from_node_id: required_string_argument(
                    &call,
                    1,
                    "AnimationCommand.DisconnectGraphNodes",
                )?,
                to_node_id: required_string_argument(
                    &call,
                    2,
                    "AnimationCommand.DisconnectGraphNodes",
                )?,
            },
            "AnimationCommand.SetGraphParameter" => Self::SetGraphParameter {
                graph_path: required_string_argument(
                    &call,
                    0,
                    "AnimationCommand.SetGraphParameter",
                )?,
                parameter_name: required_string_argument(
                    &call,
                    1,
                    "AnimationCommand.SetGraphParameter",
                )?,
                value_literal: required_string_argument(
                    &call,
                    2,
                    "AnimationCommand.SetGraphParameter",
                )?,
            },
            "AnimationCommand.CreateState" => Self::CreateState {
                state_machine_path: required_string_argument(
                    &call,
                    0,
                    "AnimationCommand.CreateState",
                )?,
                state_name: required_string_argument(&call, 1, "AnimationCommand.CreateState")?,
                graph_path: required_string_argument(&call, 2, "AnimationCommand.CreateState")?,
            },
            "AnimationCommand.RemoveState" => Self::RemoveState {
                state_machine_path: required_string_argument(
                    &call,
                    0,
                    "AnimationCommand.RemoveState",
                )?,
                state_name: required_string_argument(&call, 1, "AnimationCommand.RemoveState")?,
            },
            "AnimationCommand.SetEntryState" => Self::SetEntryState {
                state_machine_path: required_string_argument(
                    &call,
                    0,
                    "AnimationCommand.SetEntryState",
                )?,
                state_name: required_string_argument(&call, 1, "AnimationCommand.SetEntryState")?,
            },
            "AnimationCommand.CreateTransition" => Self::CreateTransition {
                state_machine_path: required_string_argument(
                    &call,
                    0,
                    "AnimationCommand.CreateTransition",
                )?,
                from_state: required_string_argument(
                    &call,
                    1,
                    "AnimationCommand.CreateTransition",
                )?,
                to_state: required_string_argument(&call, 2, "AnimationCommand.CreateTransition")?,
                duration_frames: required_u32_argument(
                    &call,
                    3,
                    "AnimationCommand.CreateTransition",
                )?,
            },
            "AnimationCommand.RemoveTransition" => Self::RemoveTransition {
                state_machine_path: required_string_argument(
                    &call,
                    0,
                    "AnimationCommand.RemoveTransition",
                )?,
                from_state: required_string_argument(
                    &call,
                    1,
                    "AnimationCommand.RemoveTransition",
                )?,
                to_state: required_string_argument(&call, 2, "AnimationCommand.RemoveTransition")?,
            },
            "AnimationCommand.SetTransitionCondition" => Self::SetTransitionCondition {
                state_machine_path: required_string_argument(
                    &call,
                    0,
                    "AnimationCommand.SetTransitionCondition",
                )?,
                from_state: required_string_argument(
                    &call,
                    1,
                    "AnimationCommand.SetTransitionCondition",
                )?,
                to_state: required_string_argument(
                    &call,
                    2,
                    "AnimationCommand.SetTransitionCondition",
                )?,
                parameter_name: required_string_argument(
                    &call,
                    3,
                    "AnimationCommand.SetTransitionCondition",
                )?,
                operator: required_string_argument(
                    &call,
                    4,
                    "AnimationCommand.SetTransitionCondition",
                )?,
                value_literal: required_string_argument(
                    &call,
                    5,
                    "AnimationCommand.SetTransitionCondition",
                )?,
            },
            _ => return Ok(None),
        };
        Ok(Some(command))
    }
}
