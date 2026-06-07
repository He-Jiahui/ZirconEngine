use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetError, NetEvent, NetSessionHandshakeState,
    NetSessionId, NetSessionInfo,
};

use super::{state::NetRpcSessionState, NetRpcRuntimeManager};

impl NetRpcRuntimeManager {
    pub fn begin_handshake(&self) -> NetSessionId {
        self.begin_session(None)
    }

    pub fn begin_handshake_for_connection(&self, connection: NetConnectionId) -> NetSessionId {
        self.begin_session(Some(connection))
    }

    fn begin_session(&self, connection: Option<NetConnectionId>) -> NetSessionId {
        let mut state = self.state.lock().expect("net RPC state mutex poisoned");
        state.next_session_id += 1;
        let session = NetSessionId::new(state.next_session_id);
        state
            .sessions
            .insert(session, NetRpcSessionState::new(connection));
        session
    }

    pub fn handshake_state(
        &self,
        session: NetSessionId,
    ) -> Result<NetSessionHandshakeState, NetError> {
        self.state
            .lock()
            .expect("net RPC state mutex poisoned")
            .sessions
            .get(&session)
            .map(|session_state| session_state.handshake_state)
            .ok_or(NetError::UnknownSession { session })
    }

    pub fn session_info(&self, session: NetSessionId) -> Result<NetSessionInfo, NetError> {
        self.state
            .lock()
            .expect("net RPC state mutex poisoned")
            .sessions
            .get(&session)
            .map(|session_state| session_state.info(session))
            .ok_or(NetError::UnknownSession { session })
    }

    pub fn close_session(&self, session: NetSessionId) -> Result<NetSessionInfo, NetError> {
        let mut state = self.state.lock().expect("net RPC state mutex poisoned");
        let session_state = state
            .sessions
            .get_mut(&session)
            .ok_or(NetError::UnknownSession { session })?;
        session_state.handshake_state = NetSessionHandshakeState::Closed;
        Ok(session_state.info(session))
    }

    pub fn close_connection_sessions(&self, connection: NetConnectionId) -> Vec<NetSessionInfo> {
        let mut state = self.state.lock().expect("net RPC state mutex poisoned");
        state
            .sessions
            .iter_mut()
            .filter_map(|(session, session_state)| {
                (session_state.connection == Some(connection)).then(|| {
                    session_state.handshake_state = NetSessionHandshakeState::Closed;
                    session_state.info(*session)
                })
            })
            .collect()
    }

    pub fn apply_transport_events(
        &self,
        events: impl IntoIterator<Item = NetEvent>,
    ) -> Vec<NetSessionInfo> {
        let mut closed = Vec::new();
        for event in events {
            match event {
                NetEvent::ConnectionClosed { connection, .. }
                | NetEvent::ConnectionStateChanged {
                    connection,
                    state: NetConnectionState::Closed | NetConnectionState::Failed,
                    ..
                } => closed.extend(self.close_connection_sessions(connection)),
                _ => {}
            }
        }
        closed
    }
}
