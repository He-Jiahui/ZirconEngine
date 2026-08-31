mod graph;
mod sequence;
mod state_machine;

use zircon_runtime::core::framework::animation::AnimationTrackPath;

use super::{
    AnimationAuthoringAsset, AnimationAuthoringDocumentError, AnimationDocumentMutationError,
    AnimationGraphNodeKind,
};

/// A semantic source mutation prepared from an editor event before it enters history.
#[derive(Clone, Debug)]
pub(crate) enum AnimationDocumentMutation {
    AddKey {
        track_path: AnimationTrackPath,
        frame: u32,
    },
    RemoveKey {
        track_path: AnimationTrackPath,
        frame: u32,
    },
    CreateTrack {
        track_path: AnimationTrackPath,
    },
    RemoveTrack {
        track_path: AnimationTrackPath,
    },
    RebindTrack {
        from_track_path: AnimationTrackPath,
        to_track_path: AnimationTrackPath,
    },
    AddGraphNode {
        node_id: String,
        node_kind: AnimationGraphNodeKind,
    },
    RemoveGraphNode {
        node_id: String,
    },
    ConnectGraphNodes {
        from_node_id: String,
        to_node_id: String,
    },
    DisconnectGraphNodes {
        from_node_id: String,
        to_node_id: String,
    },
    SetGraphParameter {
        parameter_name: String,
        value_literal: String,
    },
    CreateState {
        state_name: String,
        graph_locator: String,
    },
    RemoveState {
        state_name: String,
    },
    SetEntryState {
        state_name: String,
    },
    CreateTransition {
        from_state: String,
        to_state: String,
        duration_frames: u32,
    },
    RemoveTransition {
        from_state: String,
        to_state: String,
    },
    SetTransitionCondition {
        from_state: String,
        to_state: String,
        parameter_name: String,
        operator: String,
        value_literal: String,
    },
}

impl AnimationDocumentMutation {
    pub(crate) const fn label(&self) -> &'static str {
        match self {
            Self::AddKey { .. } => "Add Animation Key",
            Self::RemoveKey { .. } => "Remove Animation Key",
            Self::CreateTrack { .. } => "Create Animation Track",
            Self::RemoveTrack { .. } => "Remove Animation Track",
            Self::RebindTrack { .. } => "Rebind Animation Track",
            Self::AddGraphNode { .. } => "Add Animation Graph Node",
            Self::RemoveGraphNode { .. } => "Remove Animation Graph Node",
            Self::ConnectGraphNodes { .. } => "Connect Animation Graph Nodes",
            Self::DisconnectGraphNodes { .. } => "Disconnect Animation Graph Nodes",
            Self::SetGraphParameter { .. } => "Set Animation Graph Parameter",
            Self::CreateState { .. } => "Create Animation State",
            Self::RemoveState { .. } => "Remove Animation State",
            Self::SetEntryState { .. } => "Set Animation Entry State",
            Self::CreateTransition { .. } => "Create Animation Transition",
            Self::RemoveTransition { .. } => "Remove Animation Transition",
            Self::SetTransitionCondition { .. } => "Set Animation Transition Condition",
        }
    }

    pub(crate) fn apply(
        &self,
        asset: &mut AnimationAuthoringAsset,
    ) -> Result<bool, AnimationDocumentMutationError> {
        match (self, asset) {
            (Self::AddKey { track_path, frame }, AnimationAuthoringAsset::Sequence(asset)) => {
                sequence::add_key(asset, track_path, *frame)
            }
            (Self::RemoveKey { track_path, frame }, AnimationAuthoringAsset::Sequence(asset)) => {
                sequence::remove_key(asset, track_path, *frame)
            }
            (Self::CreateTrack { track_path }, AnimationAuthoringAsset::Sequence(asset)) => {
                sequence::create_track(asset, track_path)
            }
            (Self::RemoveTrack { track_path }, AnimationAuthoringAsset::Sequence(asset)) => {
                sequence::remove_track(asset, track_path)
            }
            (
                Self::RebindTrack {
                    from_track_path,
                    to_track_path,
                },
                AnimationAuthoringAsset::Sequence(asset),
            ) => sequence::rebind_track(asset, from_track_path, to_track_path),
            (Self::AddGraphNode { node_id, node_kind }, AnimationAuthoringAsset::Graph(asset)) => {
                graph::add_node(asset, node_id, *node_kind)
            }
            (Self::RemoveGraphNode { node_id }, AnimationAuthoringAsset::Graph(asset)) => {
                Ok(graph::remove_node(asset, node_id))
            }
            (
                Self::ConnectGraphNodes {
                    from_node_id,
                    to_node_id,
                },
                AnimationAuthoringAsset::Graph(asset),
            ) => Ok(graph::connect_nodes(asset, from_node_id, to_node_id)),
            (
                Self::DisconnectGraphNodes {
                    from_node_id,
                    to_node_id,
                },
                AnimationAuthoringAsset::Graph(asset),
            ) => Ok(graph::disconnect_nodes(asset, from_node_id, to_node_id)),
            (
                Self::SetGraphParameter {
                    parameter_name,
                    value_literal,
                },
                AnimationAuthoringAsset::Graph(asset),
            ) => Ok(graph::set_parameter(asset, parameter_name, value_literal)),
            (
                Self::CreateState {
                    state_name,
                    graph_locator,
                },
                AnimationAuthoringAsset::StateMachine(asset),
            ) => state_machine::create_state(asset, state_name, graph_locator),
            (Self::RemoveState { state_name }, AnimationAuthoringAsset::StateMachine(asset)) => {
                Ok(state_machine::remove_state(asset, state_name))
            }
            (Self::SetEntryState { state_name }, AnimationAuthoringAsset::StateMachine(asset)) => {
                Ok(state_machine::set_entry_state(asset, state_name))
            }
            (
                Self::CreateTransition {
                    from_state,
                    to_state,
                    duration_frames,
                },
                AnimationAuthoringAsset::StateMachine(asset),
            ) => Ok(state_machine::create_transition(
                asset,
                from_state,
                to_state,
                *duration_frames,
            )),
            (
                Self::RemoveTransition {
                    from_state,
                    to_state,
                },
                AnimationAuthoringAsset::StateMachine(asset),
            ) => Ok(state_machine::remove_transition(
                asset, from_state, to_state,
            )),
            (
                Self::SetTransitionCondition {
                    from_state,
                    to_state,
                    parameter_name,
                    operator,
                    value_literal,
                },
                AnimationAuthoringAsset::StateMachine(asset),
            ) => Ok(state_machine::set_transition_condition(
                asset,
                from_state,
                to_state,
                parameter_name,
                operator,
                value_literal,
            )),
            (_, asset) => Err(AnimationAuthoringDocumentError::wrong_kind(
                expected_kind(self),
                asset.kind(),
            )
            .into()),
        }
    }
}

fn expected_kind(mutation: &AnimationDocumentMutation) -> super::AnimationAuthoringDocumentKind {
    match mutation {
        AnimationDocumentMutation::AddKey { .. }
        | AnimationDocumentMutation::RemoveKey { .. }
        | AnimationDocumentMutation::CreateTrack { .. }
        | AnimationDocumentMutation::RemoveTrack { .. }
        | AnimationDocumentMutation::RebindTrack { .. } => {
            super::AnimationAuthoringDocumentKind::Sequence
        }
        AnimationDocumentMutation::AddGraphNode { .. }
        | AnimationDocumentMutation::RemoveGraphNode { .. }
        | AnimationDocumentMutation::ConnectGraphNodes { .. }
        | AnimationDocumentMutation::DisconnectGraphNodes { .. }
        | AnimationDocumentMutation::SetGraphParameter { .. } => {
            super::AnimationAuthoringDocumentKind::Graph
        }
        AnimationDocumentMutation::CreateState { .. }
        | AnimationDocumentMutation::RemoveState { .. }
        | AnimationDocumentMutation::SetEntryState { .. }
        | AnimationDocumentMutation::CreateTransition { .. }
        | AnimationDocumentMutation::RemoveTransition { .. }
        | AnimationDocumentMutation::SetTransitionCondition { .. } => {
            super::AnimationAuthoringDocumentKind::StateMachine
        }
    }
}
