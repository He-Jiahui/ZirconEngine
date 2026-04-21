use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AnimationCommand {
    AddKey {
        track_path: String,
        frame: u32,
    },
    RemoveKey {
        track_path: String,
        frame: u32,
    },
    CreateTrack {
        track_path: String,
    },
    RemoveTrack {
        track_path: String,
    },
    RebindTrack {
        from_track_path: String,
        to_track_path: String,
    },
    ScrubTimeline {
        frame: u32,
    },
    SetTimelineRange {
        start_frame: u32,
        end_frame: u32,
    },
    SelectTimelineSpan {
        track_path: String,
        start_frame: u32,
        end_frame: u32,
    },
    SetPlayback {
        playing: bool,
        looping: bool,
        speed: f32,
    },
    AddGraphNode {
        graph_path: String,
        node_id: String,
        node_kind: String,
    },
    RemoveGraphNode {
        graph_path: String,
        node_id: String,
    },
    ConnectGraphNodes {
        graph_path: String,
        from_node_id: String,
        to_node_id: String,
    },
    DisconnectGraphNodes {
        graph_path: String,
        from_node_id: String,
        to_node_id: String,
    },
    SetGraphParameter {
        graph_path: String,
        parameter_name: String,
        value_literal: String,
    },
    CreateState {
        state_machine_path: String,
        state_name: String,
        graph_path: String,
    },
    RemoveState {
        state_machine_path: String,
        state_name: String,
    },
    SetEntryState {
        state_machine_path: String,
        state_name: String,
    },
    CreateTransition {
        state_machine_path: String,
        from_state: String,
        to_state: String,
        duration_frames: u32,
    },
    RemoveTransition {
        state_machine_path: String,
        from_state: String,
        to_state: String,
    },
    SetTransitionCondition {
        state_machine_path: String,
        from_state: String,
        to_state: String,
        parameter_name: String,
        operator: String,
        value_literal: String,
    },
}
