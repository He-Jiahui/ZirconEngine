use zircon_runtime::plugin::{CapabilityStatus, PluginMaturity, RuntimePlugin};

use crate::{
    runtime_plugin, NET_DIST_CRATE_NAME, NET_DIST_RUNTIME_ENTRY, NET_FLUSH_EGRESS_SYSTEM,
    NET_MAIN_SYSTEM_SET, NET_POLL_INGRESS_SYSTEM, NET_TRANSPORT_SYSTEM_SET, RUNTIME_CAPABILITIES,
};

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
    assert_eq!(
        manifest.modules[0].system_sets,
        [NET_MAIN_SYSTEM_SET, NET_TRANSPORT_SYSTEM_SET].map(str::to_string)
    );
    assert_eq!(
        manifest.modules[0].system_anchors,
        vec![
            NET_POLL_INGRESS_SYSTEM.to_string(),
            NET_FLUSH_EGRESS_SYSTEM.to_string()
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

#[test]
fn net_package_manifest_declares_dist_contract() {
    let manifest = runtime_plugin().package_manifest();

    assert!(manifest.default_packaging.contains(
        &zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic
    ));

    let distribution = manifest
        .distribution
        .as_ref()
        .expect("net distribution manifest");
    assert_eq!(distribution.forms, vec!["dist".to_string()]);
    assert_eq!(
        distribution.default_packaging,
        vec![zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.engine_compat, ">=0.1, <0.2");
    assert_eq!(distribution.dist_crate, NET_DIST_CRATE_NAME);
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert_eq!(distribution.runtime_entry, NET_DIST_RUNTIME_ENTRY);

    let native_module = manifest
        .modules
        .iter()
        .find(|module| module.name == "net.dist")
        .expect("net native dist module");
    assert_eq!(
        native_module.kind,
        zircon_runtime::plugin::PluginModuleKind::Native
    );
    assert_eq!(native_module.crate_name, NET_DIST_CRATE_NAME);
    assert_eq!(
        native_module.target_modes,
        vec![
            zircon_runtime::core::framework::platform::RuntimeTargetMode::ServerRuntime,
            zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost,
        ]
    );
    for capability in RUNTIME_CAPABILITIES {
        assert!(native_module.capabilities.contains(&capability.to_string()));
    }
}
