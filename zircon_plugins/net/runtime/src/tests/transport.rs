use std::time::Duration;

use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetEvent, NetTransportKind,
};

use crate::transport::{ReconnectPolicy, TransportStateMachine};

#[test]
fn reconnect_backoff_timing_sequence() {
    let policy = ReconnectPolicy::new(
        Duration::from_millis(100),
        Duration::from_millis(750),
        Duration::from_millis(0),
        5,
    );

    assert_eq!(
        policy.delays().collect::<Vec<_>>(),
        vec![
            Duration::from_millis(100),
            Duration::from_millis(200),
            Duration::from_millis(400),
            Duration::from_millis(750),
            Duration::from_millis(750),
        ]
    );
    assert_eq!(policy.delay_for_attempt(5), None);
}

#[test]
fn state_changes_emit_events() {
    let connection = NetConnectionId::new(42);
    let mut state = TransportStateMachine::new(
        connection,
        NetTransportKind::Tcp,
        NetConnectionState::Connecting,
    );

    assert_eq!(state.transition(NetConnectionState::Connecting), None);
    assert_eq!(
        state.transition(NetConnectionState::Open),
        Some(NetEvent::ConnectionStateChanged {
            connection,
            transport: NetTransportKind::Tcp,
            state: NetConnectionState::Open,
        })
    );
    assert_eq!(state.state(), NetConnectionState::Open);
    assert_eq!(
        state.transition(NetConnectionState::Closing),
        Some(NetEvent::ConnectionStateChanged {
            connection,
            transport: NetTransportKind::Tcp,
            state: NetConnectionState::Closing,
        })
    );
    assert_eq!(
        state.transition(NetConnectionState::Closed),
        Some(NetEvent::ConnectionStateChanged {
            connection,
            transport: NetTransportKind::Tcp,
            state: NetConnectionState::Closed,
        })
    );

    let failed_connection = NetConnectionId::new(43);
    let mut failed = TransportStateMachine::new(
        failed_connection,
        NetTransportKind::Tcp,
        NetConnectionState::Connecting,
    );
    assert_eq!(
        failed.transition(NetConnectionState::Failed),
        Some(NetEvent::ConnectionStateChanged {
            connection: failed_connection,
            transport: NetTransportKind::Tcp,
            state: NetConnectionState::Failed,
        })
    );
}
