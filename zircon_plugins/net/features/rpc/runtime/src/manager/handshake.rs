use zircon_runtime::core::framework::net::{
    NetControlMessage, NetError, NetSessionControlReport, NetSessionHandshakePolicy,
    NetSessionHandshakeState, NetSessionId,
};

use crate::NET_RPC_FEATURE_CAPABILITY;

use super::NetRpcRuntimeManager;

pub const RPC_HANDSHAKE_MAGIC: u32 = 0x4350_525A; // little-endian bytes: ZRPC
pub const RPC_HANDSHAKE_CAPABILITY_NET_RPC: u64 = 1 << 0;

const RPC_HANDSHAKE_HEADER_BYTES: usize = 4 + 2 + 8 + 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetRpcHandshakeFrame {
    pub protocol_version: u16,
    pub capabilities: u64,
    pub token: Vec<u8>,
}

impl NetRpcHandshakeFrame {
    pub fn new(protocol_version: u16, capabilities: u64, token: Vec<u8>) -> Self {
        Self {
            protocol_version,
            capabilities,
            token,
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, NetError> {
        let token_len = u16::try_from(self.token.len())
            .map_err(|_| NetError::Io("rpc handshake token too large".to_string()))?;
        let mut bytes = Vec::with_capacity(RPC_HANDSHAKE_HEADER_BYTES + self.token.len());
        bytes.extend_from_slice(&RPC_HANDSHAKE_MAGIC.to_le_bytes());
        bytes.extend_from_slice(&self.protocol_version.to_le_bytes());
        bytes.extend_from_slice(&self.capabilities.to_le_bytes());
        bytes.extend_from_slice(&token_len.to_le_bytes());
        bytes.extend_from_slice(&self.token);
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, NetError> {
        if bytes.len() < RPC_HANDSHAKE_HEADER_BYTES {
            return Err(NetError::Io("rpc handshake frame is truncated".to_string()));
        }

        let magic = u32::from_le_bytes(bytes[0..4].try_into().expect("slice length checked"));
        if magic != RPC_HANDSHAKE_MAGIC {
            return Err(NetError::Io("rpc handshake magic mismatch".to_string()));
        }

        let protocol_version =
            u16::from_le_bytes(bytes[4..6].try_into().expect("slice length checked"));
        let capabilities =
            u64::from_le_bytes(bytes[6..14].try_into().expect("slice length checked"));
        let token_len =
            u16::from_le_bytes(bytes[14..16].try_into().expect("slice length checked")) as usize;
        let expected_len = RPC_HANDSHAKE_HEADER_BYTES + token_len;
        if bytes.len() != expected_len {
            return Err(NetError::Io(
                "rpc handshake token length mismatch".to_string(),
            ));
        }

        Ok(Self {
            protocol_version,
            capabilities,
            token: bytes[RPC_HANDSHAKE_HEADER_BYTES..].to_vec(),
        })
    }

    pub fn supports_net_rpc(&self) -> bool {
        self.capabilities & RPC_HANDSHAKE_CAPABILITY_NET_RPC != 0
    }

    fn into_hello_message(self) -> NetControlMessage {
        let mut runtime_features = Vec::new();
        if self.supports_net_rpc() {
            runtime_features.push(NET_RPC_FEATURE_CAPABILITY.to_string());
        }
        NetControlMessage::Hello {
            protocol_version: u32::from(self.protocol_version),
            runtime_features,
        }
    }
}

impl NetRpcRuntimeManager {
    pub fn process_handshake_frame(
        &self,
        session: NetSessionId,
        frame_bytes: &[u8],
    ) -> Result<NetSessionControlReport, NetError> {
        let frame = NetRpcHandshakeFrame::decode(frame_bytes)?;
        self.process_control_message(session, frame.into_hello_message())
    }

    pub fn process_control_message(
        &self,
        session: NetSessionId,
        message: NetControlMessage,
    ) -> Result<NetSessionControlReport, NetError> {
        let mut state = self.state.lock().expect("net RPC state mutex poisoned");
        let current = state
            .sessions
            .get(&session)
            .map(|session_state| session_state.handshake_state)
            .ok_or(NetError::UnknownSession { session })?;
        let login_player_id = match &message {
            NetControlMessage::Login { player_id, .. } => Some(player_id.clone()),
            _ => None,
        };
        let netspeed_bytes_per_second = match &message {
            NetControlMessage::NetSpeed { bytes_per_second } => Some(*bytes_per_second),
            _ => None,
        };
        let (next, response) =
            Self::advance_handshake(current, &state.handshake_policy, session, message);
        let session_state = state
            .sessions
            .get_mut(&session)
            .expect("session should exist after control lookup");
        session_state.handshake_state = next;
        if matches!(
            next,
            NetSessionHandshakeState::Welcomed | NetSessionHandshakeState::Joined
        ) {
            if let Some(player_id) = login_player_id {
                session_state.player_id = Some(player_id);
            }
            if let Some(bytes_per_second) = netspeed_bytes_per_second {
                session_state.netspeed_bytes_per_second = Some(bytes_per_second);
            }
        }
        Ok(NetSessionControlReport::new(session, next, response))
    }

    fn advance_handshake(
        current: NetSessionHandshakeState,
        policy: &NetSessionHandshakePolicy,
        session: NetSessionId,
        message: NetControlMessage,
    ) -> (NetSessionHandshakeState, Option<NetControlMessage>) {
        match (current, message) {
            (
                NetSessionHandshakeState::AwaitingHello,
                NetControlMessage::Hello {
                    protocol_version,
                    runtime_features,
                },
            ) => Self::accept_hello(policy, protocol_version, &runtime_features),
            (
                NetSessionHandshakeState::AwaitingLogin,
                NetControlMessage::Login {
                    player_id,
                    challenge_response,
                },
            ) => Self::accept_login(policy, session, &player_id, &challenge_response),
            (NetSessionHandshakeState::Welcomed, NetControlMessage::NetSpeed { .. }) => {
                (NetSessionHandshakeState::Welcomed, None)
            }
            (NetSessionHandshakeState::Welcomed, NetControlMessage::Join) => {
                (NetSessionHandshakeState::Joined, None)
            }
            (NetSessionHandshakeState::Joined, NetControlMessage::NetSpeed { .. })
            | (NetSessionHandshakeState::Joined, NetControlMessage::Join) => {
                (NetSessionHandshakeState::Joined, None)
            }
            (NetSessionHandshakeState::Failed, _) => (NetSessionHandshakeState::Failed, None),
            (NetSessionHandshakeState::Closed, _) => (NetSessionHandshakeState::Closed, None),
            _ => Self::failure("unexpected control message"),
        }
    }

    fn accept_hello(
        policy: &NetSessionHandshakePolicy,
        protocol_version: u32,
        runtime_features: &[String],
    ) -> (NetSessionHandshakeState, Option<NetControlMessage>) {
        if protocol_version != policy.protocol_version {
            return Self::failure("protocol version mismatch");
        }

        if let Some(feature) = policy
            .required_features
            .iter()
            .find(|required| !runtime_features.contains(required))
        {
            return Self::failure(format!("missing required feature: {feature}"));
        }

        (
            NetSessionHandshakeState::AwaitingLogin,
            Some(NetControlMessage::Challenge {
                nonce: policy.challenge_nonce.clone(),
            }),
        )
    }

    fn accept_login(
        policy: &NetSessionHandshakePolicy,
        session: NetSessionId,
        player_id: &str,
        challenge_response: &str,
    ) -> (NetSessionHandshakeState, Option<NetControlMessage>) {
        if player_id.trim().is_empty() || challenge_response != policy.challenge_nonce {
            return Self::failure("challenge response rejected");
        }

        (
            NetSessionHandshakeState::Welcomed,
            Some(NetControlMessage::Welcome {
                session_id: session.raw().to_string(),
                map: policy.welcome_map.clone(),
            }),
        )
    }

    fn failure(reason: impl Into<String>) -> (NetSessionHandshakeState, Option<NetControlMessage>) {
        (
            NetSessionHandshakeState::Failed,
            Some(NetControlMessage::Failure {
                reason: reason.into(),
            }),
        )
    }
}
