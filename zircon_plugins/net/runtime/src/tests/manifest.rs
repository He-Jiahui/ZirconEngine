use zircon_runtime::plugin::{CapabilityStatus, PluginMaturity, RuntimePlugin};

use crate::runtime_plugin;

#[test]
fn net_plugin_manifest_advertises_layered_optional_features() {
    let manifest = runtime_plugin().package_manifest();

    assert_eq!(manifest.category, "runtime");
    assert_eq!(manifest.maturity, PluginMaturity::Beta);
    assert!(manifest.capability_statuses.iter().any(|status| {
        status.capability == "runtime.plugin.net" && status.status == CapabilityStatus::Partial
    }));
    let feature_ids = manifest
        .optional_features
        .iter()
        .map(|feature| feature.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        feature_ids,
        vec![
            "net.http",
            "net.websocket",
            "net.rpc",
            "net.replication",
            "net.reliable_udp",
            "net.content_download",
        ]
    );
    let runtime_mode = manifest
        .options
        .iter()
        .find(|option| option.key == "net.runtime_mode")
        .expect("net runtime mode option");
    assert_eq!(runtime_mode.value_type, "enum");
    assert_eq!(runtime_mode.default_value, "client");
    assert_eq!(
        runtime_mode.enum_values,
        vec![
            "client".to_string(),
            "listen_server".to_string(),
            "dedicated_server".to_string(),
        ]
    );
    let event_catalog = manifest
        .event_catalogs
        .iter()
        .find(|catalog| catalog.namespace == "net.runtime_events")
        .expect("net runtime event catalog");
    assert_eq!(
        event_catalog
            .events
            .iter()
            .map(|event| event.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "net.runtime_events.listener_started",
            "net.runtime_events.connection_state_changed",
            "net.runtime_events.http_route_registered",
            "net.runtime_events.websocket_frame_queued",
        ]
    );

    let content_download = manifest
        .optional_features
        .iter()
        .find(|feature| feature.id == "net.content_download")
        .expect("content download feature");
    assert!(content_download.dependencies.iter().any(|dependency| {
        dependency.plugin_id == "net"
            && dependency.capability == "runtime.plugin.net"
            && dependency.primary
    }));
    assert!(content_download.dependencies.iter().any(|dependency| {
        dependency.plugin_id == "net"
            && dependency.capability == "runtime.feature.net.http"
            && !dependency.primary
    }));
}
