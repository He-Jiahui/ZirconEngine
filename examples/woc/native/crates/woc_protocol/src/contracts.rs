use serde::{Deserialize, Serialize};

use crate::generated::message_kind_value;
use crate::{MovementInputFlags, ProtocolError};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct WocReferenceIdentity {
    pub source_commit: &'static str,
    pub contract_schema_fingerprint: &'static str,
    pub command_catalog_sha256: &'static str,
    pub command_payload_schema_sha256: &'static str,
    pub world_state_format: &'static str,
    pub world_state_schema_version: u16,
    pub simulation_hz: u32,
    pub presentation_hz: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct EntityRef {
    pub id: u64,
    pub generation: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FixedTickInput {
    pub tick: u64,
    pub commands: Vec<Command>,
    pub wall_time_forbidden: bool,
    pub committed_state: Vec<u8>,
    pub committed_state_digest: u32,
    pub generation: u64,
    pub movement_frames: Vec<MovementFrame>,
    pub offline_bootstrap: Option<OfflineSessionBootstrap>,
}

/// Source-derived inputs that construct one fresh standard offline simulation.
/// This is a first-tick envelope, never an authoritative gameplay command.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OfflineSessionBootstrap {
    pub launch_version: u16,
    pub world_seed: u32,
    pub player_class: u8,
    pub player_name: String,
    pub skin_variant: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct MovementFrame {
    pub actor: EntityRef,
    pub sequence: u32,
    pub flags: MovementInputFlags,
    pub facing: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Command {
    pub command_id: u16,
    pub actor: EntityRef,
    pub sequence: u32,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Event {
    pub event_id: u16,
    pub sequence: u32,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct WorldSnapshot {
    pub tick: u64,
    pub state_digest: u32,
    pub event_digest: u32,
    pub state: Vec<u8>,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SaveState {
    pub schema_fingerprint: [u8; 32],
    pub generation: u64,
    pub tick: u64,
    pub state: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NetworkEnvelope {
    pub protocol_version: u16,
    pub kind: MessageKind,
    pub sequence: u64,
    pub acknowledgement: u64,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RlObservationBatch {
    pub tick: u64,
    pub environment_ids: Vec<u32>,
    pub offsets: Vec<u32>,
    pub observations: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RlActionBatch {
    pub tick: u64,
    pub environment_ids: Vec<u32>,
    pub offsets: Vec<u32>,
    pub actions: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[repr(u16)]
pub enum MessageKind {
    FixedTickInput = message_kind_value::FIXED_TICK_INPUT,
    Command = message_kind_value::COMMAND,
    Event = message_kind_value::EVENT,
    WorldSnapshot = message_kind_value::WORLD_SNAPSHOT,
    SaveState = message_kind_value::SAVE_STATE,
    NetworkEnvelope = message_kind_value::NETWORK_ENVELOPE,
    RlObservationBatch = message_kind_value::RL_OBSERVATION_BATCH,
    RlActionBatch = message_kind_value::RL_ACTION_BATCH,
}

impl TryFrom<u16> for MessageKind {
    type Error = ProtocolError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            message_kind_value::FIXED_TICK_INPUT => Ok(Self::FixedTickInput),
            message_kind_value::COMMAND => Ok(Self::Command),
            message_kind_value::EVENT => Ok(Self::Event),
            message_kind_value::WORLD_SNAPSHOT => Ok(Self::WorldSnapshot),
            message_kind_value::SAVE_STATE => Ok(Self::SaveState),
            message_kind_value::NETWORK_ENVELOPE => Ok(Self::NetworkEnvelope),
            message_kind_value::RL_OBSERVATION_BATCH => Ok(Self::RlObservationBatch),
            message_kind_value::RL_ACTION_BATCH => Ok(Self::RlActionBatch),
            unknown => Err(ProtocolError::UnknownMessageKind(unknown)),
        }
    }
}
