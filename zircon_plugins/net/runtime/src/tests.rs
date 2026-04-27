use zircon_runtime::core::framework::net::{NetEndpoint, NetManager};
use zircon_runtime::RuntimePluginRegistrationReport;

use super::{runtime_plugin, DefaultNetManager, NET_MODULE_NAME};

#[test]
fn net_plugin_registration_contributes_runtime_module() {
    let report = RuntimePluginRegistrationReport::from_plugin(&runtime_plugin());

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == NET_MODULE_NAME));
    assert_eq!(
        report.package_manifest.modules[0].target_modes,
        vec![
            zircon_runtime::RuntimeTargetMode::ServerRuntime,
            zircon_runtime::RuntimeTargetMode::ClientRuntime,
        ]
    );
}

#[test]
fn default_net_manager_sends_udp_packet_to_bound_socket() {
    let net = DefaultNetManager::default();
    let socket = net.bind_udp(&NetEndpoint::new("127.0.0.1", 0)).unwrap();
    let endpoint = net.local_endpoint(socket).unwrap();

    assert_eq!(net.send_udp(socket, &endpoint, b"ping").unwrap(), 4);
    let packets = poll_until_packet(&net, socket);

    assert_eq!(packets[0].payload, b"ping");
    net.close_socket(socket).unwrap();
}

fn poll_until_packet(
    net: &DefaultNetManager,
    socket: zircon_runtime::core::framework::net::NetSocketId,
) -> Vec<zircon_runtime::core::framework::net::NetPacket> {
    for _ in 0..100 {
        let packets = net.poll_udp(socket, 4).unwrap();
        if !packets.is_empty() {
            return packets;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    panic!("expected loopback UDP packet");
}
