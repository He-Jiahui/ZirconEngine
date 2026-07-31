use zircon_runtime::core::framework::net::{
    NetConnectionId, NetEndpoint, NetError, NetManager, NetRuntimeMode, NetSocketId,
};

use crate::DefaultNetManager;

#[test]
fn poisoned_event_queue_recovers_for_public_manager_reads() {
    let net = DefaultNetManager::for_mode(NetRuntimeMode::Client);
    net.poison_events_for_test();

    assert_eq!(net.diagnostics().queued_events, 0);
    assert!(net.drain_events(1).is_empty());
}

#[test]
fn poisoned_fallible_manager_state_returns_typed_error() {
    let net = DefaultNetManager::for_mode(NetRuntimeMode::Client);
    net.poison_udp_sockets_for_test();

    assert_eq!(
        net.local_endpoint(NetSocketId::new(1)).unwrap_err(),
        NetError::SharedStatePoisoned {
            resource: "net.udp_sockets".to_string(),
        }
    );
    assert_eq!(
        net.bind_udp(&NetEndpoint::new("127.0.0.1", 0)).unwrap_err(),
        NetError::SharedStatePoisoned {
            resource: "net.udp_sockets".to_string(),
        }
    );
    assert_eq!(net.diagnostics().open_udp_sockets, 0);
    assert!(net.drain_events(usize::MAX).is_empty());
}

#[test]
fn poisoned_transport_tables_fail_before_send_or_poll_io() {
    let udp = DefaultNetManager::for_mode(NetRuntimeMode::Client);
    udp.poison_udp_sockets_for_test();
    let socket = NetSocketId::new(1);
    let destination = NetEndpoint::new("127.0.0.1", 9);
    let udp_error = NetError::SharedStatePoisoned {
        resource: "net.udp_sockets".to_string(),
    };
    assert_eq!(
        udp.send_udp(socket, &destination, b"blocked").unwrap_err(),
        udp_error
    );
    assert_eq!(udp.poll_udp(socket, 1).unwrap_err(), udp_error);

    let tcp = DefaultNetManager::for_mode(NetRuntimeMode::Client);
    tcp.poison_tcp_connections_for_test();
    let connection = NetConnectionId::new(1);
    let tcp_error = NetError::SharedStatePoisoned {
        resource: "net.tcp_connections".to_string(),
    };
    assert_eq!(tcp.send_tcp(connection, b"blocked").unwrap_err(), tcp_error);
    assert_eq!(tcp.poll_tcp(connection, 1).unwrap_err(), tcp_error);
}

#[test]
fn poisoned_worker_thread_fails_before_shutdown_side_effects() {
    let net = DefaultNetManager::for_mode(NetRuntimeMode::Client);
    net.poison_worker_thread_for_test();

    assert_eq!(
        net.shutdown_worker_result_for_tests().unwrap_err(),
        NetError::SharedStatePoisoned {
            resource: "net.worker_thread".to_string(),
        }
    );
    assert!(!net.worker_is_shutdown_for_tests());
}

#[test]
fn failed_worker_shutdown_remains_retryable_until_join_completes() {
    let net = DefaultNetManager::for_mode(NetRuntimeMode::Client);
    net.fail_next_worker_shutdown_after_submit_for_test();

    assert!(matches!(
        net.shutdown_worker_result_for_tests(),
        Err(NetError::Io(detail)) if detail.contains("injected")
    ));
    assert!(!net.worker_is_shutdown_for_tests());

    net.shutdown_worker_result_for_tests().unwrap();
    assert!(net.worker_is_shutdown_for_tests());
}
