use serde::{Deserialize, Serialize};

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
