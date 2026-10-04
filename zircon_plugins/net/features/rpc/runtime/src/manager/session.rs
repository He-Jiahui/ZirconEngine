//! 以 session ID 保存握手状态和可选连接 ID，并根据 transport 关闭事件将关联 session 关闭。
//! 该方法需要外部显式转发 NetEvent；当前 feature factory 不会自动订阅根网络事件。

use std::collections::HashMap;

use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetError, NetEvent, NetSessionHandshakeState,
    NetSessionId, NetSessionInfo,
};

use super::{NetRpcRuntimeManager, state::NetRpcSessionState};

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

    /// 将根网络的关闭/失败事件显式投影到本 manager 的 session；宿主需保证事件与连接 ID 属于同一生命周期。
    pub fn apply_transport_events(
        &self,
        events: impl IntoIterator<Item = NetEvent>,
    ) -> Vec<NetSessionInfo> {
        let close_connections = events
            .into_iter()
            .filter_map(|event| match event {
                NetEvent::ConnectionClosed { connection, .. }
                | NetEvent::ConnectionStateChanged {
                    connection,
                    state: NetConnectionState::Closed | NetConnectionState::Failed,
                    ..
                } => Some(connection),
                _ => None,
            })
            .collect::<Vec<_>>();
        if close_connections.is_empty() {
            return Vec::new();
        }

        let mut state = self.state.lock().expect("net RPC state mutex poisoned");
        close_sessions_for_connections(&mut state.sessions, close_connections)
    }
}

fn close_sessions_for_connections(
    sessions: &mut HashMap<NetSessionId, NetRpcSessionState>,
    close_connections: Vec<NetConnectionId>,
) -> Vec<NetSessionInfo> {
    let mut remaining_occurrences = HashMap::with_capacity(close_connections.len());
    for connection in &close_connections {
        *remaining_occurrences.entry(*connection).or_insert(0usize) += 1;
    }

    let mut sessions_by_connection =
        HashMap::<NetConnectionId, Vec<NetSessionInfo>>::with_capacity(remaining_occurrences.len());
    for (session, session_state) in sessions {
        let Some(connection) = session_state.connection else {
            continue;
        };
        if remaining_occurrences.contains_key(&connection) {
            session_state.handshake_state = NetSessionHandshakeState::Closed;
            sessions_by_connection
                .entry(connection)
                .or_default()
                .push(session_state.info(*session));
        }
    }

    let output_count = sessions_by_connection
        .iter()
        .try_fold(0usize, |count, (connection, sessions)| {
            let occurrences = remaining_occurrences
                .get(connection)
                .copied()
                .unwrap_or_default();
            count.checked_add(sessions.len().checked_mul(occurrences)?)
        })
        .expect("closed RPC session count should fit usize");
    let mut closed = Vec::with_capacity(output_count);
    for connection in close_connections {
        let is_last_occurrence = {
            let occurrences = remaining_occurrences
                .get_mut(&connection)
                .expect("close connection occurrence should be indexed");
            *occurrences -= 1;
            *occurrences == 0
        };
        if is_last_occurrence {
            remaining_occurrences.remove(&connection);
            if let Some(sessions) = sessions_by_connection.remove(&connection) {
                closed.extend(sessions);
            }
        } else if let Some(sessions) = sessions_by_connection.get(&connection) {
            closed.extend(sessions.iter().cloned());
        }
    }
    closed
}

#[cfg(test)]
#[path = "tests/session_batched_transport_close_tests.rs"]
mod batched_transport_close_tests;
