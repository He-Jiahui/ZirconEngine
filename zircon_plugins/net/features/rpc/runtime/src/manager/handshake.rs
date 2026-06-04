use zircon_runtime::core::framework::net::{
    NetControlMessage, NetError, NetSessionControlReport, NetSessionHandshakePolicy,
    NetSessionHandshakeState, NetSessionId,
};

use super::NetRpcRuntimeManager;

impl NetRpcRuntimeManager {
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
