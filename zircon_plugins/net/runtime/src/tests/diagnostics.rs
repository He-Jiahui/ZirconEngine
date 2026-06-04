use zircon_runtime::core::framework::net::{
    NetEndpoint, NetError, NetEvent, NetManager, NetRuntimeMode,
};

use crate::DefaultNetManager;

#[test]
fn net_runtime_manager_reports_mode_diagnostics_and_events() {
    let net = DefaultNetManager::for_mode(NetRuntimeMode::DedicatedServer);
    let listener = net.listen_tcp(&NetEndpoint::new("127.0.0.1", 0)).unwrap();

    let diagnostics = net.diagnostics();
    assert_eq!(diagnostics.mode, NetRuntimeMode::DedicatedServer);
    assert_eq!(diagnostics.open_tcp_listeners, 1);
    assert_eq!(diagnostics.open_tcp_connections, 0);

    let events = net.drain_events(8);
    assert!(events.iter().any(|event| matches!(
        event,
        NetEvent::ListenerStarted {
            listener: started,
            transport,
            ..
        } if *started == listener && transport.is_tcp()
    )));
}

#[test]
fn net_runtime_manager_closes_listeners_across_transports() {
    let net = DefaultNetManager::for_mode(NetRuntimeMode::DedicatedServer);
    let listener = net.listen_tcp(&NetEndpoint::new("127.0.0.1", 0)).unwrap();
    assert_eq!(net.diagnostics().open_tcp_listeners, 1);

    net.close_listener(listener).unwrap();

    assert_eq!(net.diagnostics().open_tcp_listeners, 0);
    assert_eq!(
        net.listener_endpoint(listener).unwrap_err(),
        NetError::UnknownListener { listener }
    );
}
