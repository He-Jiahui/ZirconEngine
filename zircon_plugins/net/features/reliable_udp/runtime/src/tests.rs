use zircon_runtime::core::framework::net::{
    ReliableDatagramAck, ReliableDatagramConfig, ReliableDatagramSendStatus,
};

use super::{
    plugin_feature_registration, NetReliableUdpRuntimeManager, NET_RELIABLE_UDP_FEATURE_CAPABILITY,
    NET_RELIABLE_UDP_FEATURE_ID, NET_RELIABLE_UDP_FEATURE_MANAGER_NAME,
    NET_RELIABLE_UDP_FEATURE_MODULE_NAME,
};

#[test]
fn reliable_udp_feature_registration_contributes_runtime_module_and_manager() {
    let report = plugin_feature_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.manifest.id, NET_RELIABLE_UDP_FEATURE_ID);
    assert!(report
        .manifest
        .capabilities
        .iter()
        .any(|capability| capability == NET_RELIABLE_UDP_FEATURE_CAPABILITY));
    let module = report
        .extensions
        .modules()
        .iter()
        .find(|module| module.name == NET_RELIABLE_UDP_FEATURE_MODULE_NAME)
        .expect("reliable UDP feature module should be registered");
    assert_eq!(
        module.managers[0].name.to_string(),
        NET_RELIABLE_UDP_FEATURE_MANAGER_NAME
    );
}

#[test]
fn reliable_udp_manager_fragments_tracks_pending_and_acknowledges_sequence() {
    let manager = NetReliableUdpRuntimeManager::new(ReliableDatagramConfig {
        mtu_bytes: 4,
        ..ReliableDatagramConfig::default()
    });

    let report = manager.enqueue_reliable_datagram("state", b"abcdef".to_vec());

    assert_eq!(report.status, ReliableDatagramSendStatus::Fragmented);
    assert_eq!(report.packets.len(), 2);
    assert_eq!(manager.pending_packets().len(), 2);
    assert_eq!(manager.acknowledge(ReliableDatagramAck::new(1)), 2);
    assert!(manager.pending_packets().is_empty());
}
