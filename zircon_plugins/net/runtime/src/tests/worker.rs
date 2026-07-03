use zircon_runtime::core::framework::net::{NetEndpoint, NetError, NetManager};

use crate::DefaultNetManager;

#[test]
fn worker_shutdown_leaves_no_tasks() {
    let net = DefaultNetManager::default();
    let socket = net.bind_udp(&NetEndpoint::new("127.0.0.1", 0)).unwrap();
    let listener = net.listen_tcp(&NetEndpoint::new("127.0.0.1", 0)).unwrap();

    let report = net.shutdown_worker_for_tests();

    assert_eq!(report.open_udp_sockets_closed, 1);
    assert_eq!(report.open_tcp_listeners_closed, 1);
    assert_eq!(report.open_tcp_connections_closed, 0);
    assert_eq!(report.open_handles_closed(), 2);
    assert!(net.worker_is_shutdown_for_tests());
    assert!(matches!(
        net.close_socket(socket).unwrap_err(),
        NetError::Io(_)
    ));
    assert!(matches!(
        net.close_listener(listener).unwrap_err(),
        NetError::Io(_)
    ));
}

#[test]
fn tcp_udp_service_paths_do_not_block_on_tokio_runtime() {
    for path in ["src/service_types/tcp.rs", "src/service_types/udp.rs"] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
        let source = std::fs::read_to_string(&path).expect("read net service source");
        assert!(
            !source.contains(".block_on("),
            "{} must route Tokio IO through worker commands instead of blocking the caller",
            path.display()
        );
    }
}
