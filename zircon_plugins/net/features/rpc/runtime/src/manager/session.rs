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
mod batched_transport_close_tests {
    use std::{collections::HashMap, hint::black_box, time::Instant};

    use zircon_runtime::core::framework::net::{
        NetConnectionId, NetConnectionState, NetEvent, NetSessionHandshakeState, NetSessionId,
        NetTransportKind,
    };

    use super::{NetRpcRuntimeManager, NetRpcSessionState, close_sessions_for_connections};

    const BENCHMARK_SESSION_COUNT: usize = 4_096;
    const BENCHMARK_CLOSE_EVENT_COUNT: usize = 256;
    const BENCHMARK_SAMPLE_COUNT: usize = 21;

    #[test]
    fn batched_transport_close_preserves_event_order_and_duplicates() {
        let manager = NetRpcRuntimeManager::new();
        let first_connection = NetConnectionId::new(11);
        let second_connection = NetConnectionId::new(22);
        let first_session = manager.begin_handshake_for_connection(first_connection);
        let second_session = manager.begin_handshake_for_connection(second_connection);

        let closed = manager.apply_transport_events([
            NetEvent::ConnectionClosed {
                connection: second_connection,
                transport: NetTransportKind::Tcp,
            },
            NetEvent::ConnectionStateChanged {
                connection: first_connection,
                transport: NetTransportKind::WebSocket,
                state: NetConnectionState::Closed,
            },
            NetEvent::ConnectionStateChanged {
                connection: second_connection,
                transport: NetTransportKind::Tcp,
                state: NetConnectionState::Failed,
            },
            NetEvent::ConnectionStateChanged {
                connection: NetConnectionId::new(33),
                transport: NetTransportKind::Tcp,
                state: NetConnectionState::Open,
            },
        ]);

        assert_eq!(
            closed
                .iter()
                .map(|session| session.session)
                .collect::<Vec<_>>(),
            vec![second_session, first_session, second_session]
        );
        assert!(
            closed
                .iter()
                .all(|session| session.state == NetSessionHandshakeState::Closed)
        );
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn batched_transport_close_release_benchmark_evidence() {
        let sessions = benchmark_sessions();
        let connections = (1..=BENCHMARK_CLOSE_EVENT_COUNT as u64)
            .map(NetConnectionId::new)
            .collect::<Vec<_>>();
        let mut legacy_equivalence = sessions.clone();
        let mut optimized_equivalence = sessions.clone();
        assert_eq!(
            legacy_close_sessions(&mut legacy_equivalence, &connections),
            close_sessions_for_connections(&mut optimized_equivalence, connections.clone())
        );
        assert!(legacy_equivalence.iter().all(|(session, legacy)| {
            optimized_equivalence.get(session).is_some_and(|optimized| {
                legacy.connection == optimized.connection
                    && legacy.handshake_state == optimized.handshake_state
                    && legacy.player_id == optimized.player_id
                    && legacy.netspeed_bytes_per_second == optimized.netspeed_bytes_per_second
            })
        }));

        let legacy_session_inspections = BENCHMARK_SESSION_COUNT * BENCHMARK_CLOSE_EVENT_COUNT;
        let optimized_session_inspections = BENCHMARK_SESSION_COUNT;
        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_legacy(&sessions, &connections));
                optimized_samples.push(measure_optimized(&sessions, &connections));
            } else {
                optimized_samples.push(measure_optimized(&sessions, &connections));
                legacy_samples.push(measure_legacy(&sessions, &connections));
            }
        }

        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        println!(
            "PERF_RESULT task=plugins10_batched_rpc_transport_close sessions={BENCHMARK_SESSION_COUNT} close_events={BENCHMARK_CLOSE_EVENT_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_session_inspections_per_sample={legacy_session_inspections} optimized_session_inspections_per_sample={optimized_session_inspections} threshold_percent=50 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_raw_ns={} optimized_raw_ns={}",
            raw_samples(&legacy_samples),
            raw_samples(&optimized_samples),
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(50),
            "batched close P95 {optimized_p95}ns did not improve legacy {legacy_p95}ns by 50%"
        );
    }

    fn benchmark_sessions() -> HashMap<NetSessionId, NetRpcSessionState> {
        (1..=BENCHMARK_SESSION_COUNT as u64)
            .map(|raw| {
                (
                    NetSessionId::new(raw),
                    NetRpcSessionState {
                        connection: Some(NetConnectionId::new(raw)),
                        handshake_state: NetSessionHandshakeState::Joined,
                        player_id: Some(format!("benchmark-player-{raw}")),
                        netspeed_bytes_per_second: Some(64_000),
                    },
                )
            })
            .collect()
    }

    fn legacy_close_sessions(
        sessions: &mut HashMap<NetSessionId, NetRpcSessionState>,
        connections: &[NetConnectionId],
    ) -> Vec<zircon_runtime::core::framework::net::NetSessionInfo> {
        let mut closed = Vec::new();
        for connection in connections {
            closed.extend(sessions.iter_mut().filter_map(|(session, session_state)| {
                (session_state.connection == Some(*connection)).then(|| {
                    session_state.handshake_state = NetSessionHandshakeState::Closed;
                    session_state.info(*session)
                })
            }));
        }
        closed
    }

    fn measure_legacy(
        seed: &HashMap<NetSessionId, NetRpcSessionState>,
        connections: &[NetConnectionId],
    ) -> u128 {
        let mut sessions = seed.clone();
        let start = Instant::now();
        let closed = legacy_close_sessions(black_box(&mut sessions), black_box(connections));
        let elapsed = start.elapsed().as_nanos();
        black_box(closed);
        elapsed
    }

    fn measure_optimized(
        seed: &HashMap<NetSessionId, NetRpcSessionState>,
        connections: &[NetConnectionId],
    ) -> u128 {
        let mut sessions = seed.clone();
        let close_connections = connections.to_vec();
        let start = Instant::now();
        let closed =
            close_sessions_for_connections(black_box(&mut sessions), black_box(close_connections));
        let elapsed = start.elapsed().as_nanos();
        black_box(closed);
        elapsed
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn raw_samples(samples: &[u128]) -> String {
        format!(
            "[{}]",
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}
