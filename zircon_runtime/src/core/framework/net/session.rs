use serde::{Deserialize, Serialize};

use super::{NetConnectionId, NetSessionId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetRuntimeMode {
    DedicatedServer,
    Client,
    ListenServer,
}

impl Default for NetRuntimeMode {
    fn default() -> Self {
        Self::Client
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetControlMessage {
    Hello {
        protocol_version: u32,
        runtime_features: Vec<String>,
    },
    Challenge {
        nonce: String,
    },
    Login {
        player_id: String,
        challenge_response: String,
    },
    Welcome {
        session_id: String,
        map: String,
    },
    NetSpeed {
        bytes_per_second: u32,
    },
    Failure {
        reason: String,
    },
    Join,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetSessionHandshakeState {
    AwaitingHello,
    AwaitingLogin,
    Welcomed,
    Joined,
    Failed,
    Closed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetSessionInfo {
    pub session: NetSessionId,
    pub connection: Option<NetConnectionId>,
    pub state: NetSessionHandshakeState,
    pub player_id: Option<String>,
    pub netspeed_bytes_per_second: Option<u32>,
}

impl NetSessionInfo {
    pub fn new(
        session: NetSessionId,
        connection: Option<NetConnectionId>,
        state: NetSessionHandshakeState,
        player_id: Option<String>,
        netspeed_bytes_per_second: Option<u32>,
    ) -> Self {
        Self {
            session,
            connection,
            state,
            player_id,
            netspeed_bytes_per_second,
        }
    }
}

/// DTO-only policy for runtimes that validate control-message handshakes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetSessionHandshakePolicy {
    pub protocol_version: u32,
    pub required_features: Vec<String>,
    pub challenge_nonce: String,
    pub welcome_map: String,
}

impl NetSessionHandshakePolicy {
    pub fn new(protocol_version: u32) -> Self {
        Self {
            protocol_version,
            required_features: Vec::new(),
            challenge_nonce: "zircon-rpc-challenge".to_string(),
            welcome_map: "zircon-default".to_string(),
        }
    }

    pub fn with_required_feature(mut self, feature: impl Into<String>) -> Self {
        self.required_features.push(feature.into());
        self
    }

    pub fn with_challenge_nonce(mut self, nonce: impl Into<String>) -> Self {
        self.challenge_nonce = nonce.into();
        self
    }

    pub fn with_welcome_map(mut self, map: impl Into<String>) -> Self {
        self.welcome_map = map.into();
        self
    }
}

/// Copied result of applying one control message to a session handshake.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetSessionControlReport {
    pub session: NetSessionId,
    pub state: NetSessionHandshakeState,
    pub response: Option<NetControlMessage>,
}

impl NetSessionControlReport {
    pub fn new(
        session: NetSessionId,
        state: NetSessionHandshakeState,
        response: Option<NetControlMessage>,
    ) -> Self {
        Self {
            session,
            state,
            response,
        }
    }
}
