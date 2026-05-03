use zircon_runtime::{
    plugin::PluginEventCatalogManifest, plugin::PluginFeatureBundleManifest,
    plugin::PluginFeatureDependency, plugin::PluginModuleManifest, plugin::PluginOptionManifest,
    plugin::PluginPackageManifest, RuntimeTargetMode,
};

pub const NET_RUNTIME_EVENT_NAMESPACE: &str = "net.runtime_events";

pub fn attach_net_manifest_contributions(manifest: PluginPackageManifest) -> PluginPackageManifest {
    net_event_catalogs().into_iter().fold(
        net_options().into_iter().fold(
            net_optional_features()
                .into_iter()
                .fold(manifest, |manifest, feature| {
                    manifest.with_optional_feature(feature)
                }),
            |manifest, option| manifest.with_option(option),
        ),
        |manifest, event_catalog| manifest.with_event_catalog(event_catalog),
    )
}

pub fn net_optional_features() -> Vec<PluginFeatureBundleManifest> {
    vec![
        feature(
            "net.http",
            "HTTP(S)",
            "runtime.feature.net.http",
            "zircon_plugin_net_http_runtime",
            &[
                RuntimeTargetMode::ServerRuntime,
                RuntimeTargetMode::ClientRuntime,
            ],
        ),
        feature(
            "net.websocket",
            "WebSocket",
            "runtime.feature.net.websocket",
            "zircon_plugin_net_websocket_runtime",
            &[
                RuntimeTargetMode::ServerRuntime,
                RuntimeTargetMode::ClientRuntime,
            ],
        ),
        feature(
            "net.rpc",
            "Network RPC",
            "runtime.feature.net.rpc",
            "zircon_plugin_net_rpc_runtime",
            &[
                RuntimeTargetMode::ServerRuntime,
                RuntimeTargetMode::ClientRuntime,
            ],
        ),
        feature(
            "net.replication",
            "State Replication",
            "runtime.feature.net.replication",
            "zircon_plugin_net_replication_runtime",
            &[
                RuntimeTargetMode::ServerRuntime,
                RuntimeTargetMode::ClientRuntime,
            ],
        ),
        feature(
            "net.reliable_udp",
            "Reliable UDP",
            "runtime.feature.net.reliable_udp",
            "zircon_plugin_net_reliable_udp_runtime",
            &[
                RuntimeTargetMode::ServerRuntime,
                RuntimeTargetMode::ClientRuntime,
            ],
        ),
        feature(
            "net.content_download",
            "Content Download",
            "runtime.feature.net.cdn_download",
            "zircon_plugin_net_content_download_runtime",
            &[RuntimeTargetMode::ClientRuntime],
        ),
    ]
}

pub fn net_options() -> Vec<PluginOptionManifest> {
    vec![
        PluginOptionManifest::new("net.runtime_mode", "Runtime Mode", "enum", "client"),
        PluginOptionManifest::new(
            "net.tcp_poll_budget_bytes",
            "TCP Poll Budget",
            "integer",
            "65536",
        ),
        PluginOptionManifest::new(
            "net.udp_poll_budget_packets",
            "UDP Poll Budget",
            "integer",
            "64",
        ),
        PluginOptionManifest::new("net.http_timeout_ms", "HTTP Timeout", "integer", "30000")
            .with_required_capability("runtime.feature.net.http"),
        PluginOptionManifest::new(
            "net.websocket_message_budget",
            "WebSocket Message Budget",
            "integer",
            "256",
        )
        .with_required_capability("runtime.feature.net.websocket"),
        PluginOptionManifest::new(
            "net.rpc_max_calls_per_second",
            "RPC Rate Limit",
            "integer",
            "60",
        )
        .with_required_capability("runtime.feature.net.rpc"),
    ]
}

pub fn net_event_catalogs() -> Vec<PluginEventCatalogManifest> {
    vec![PluginEventCatalogManifest::empty(
        NET_RUNTIME_EVENT_NAMESPACE,
        1,
    )]
}

fn feature(
    id: &str,
    display_name: &str,
    capability: &str,
    crate_name: &str,
    target_modes: &[RuntimeTargetMode],
) -> PluginFeatureBundleManifest {
    PluginFeatureBundleManifest::new(id, display_name, "net")
        .with_dependency(PluginFeatureDependency::primary(
            "net",
            "runtime.plugin.net",
        ))
        .with_capability(capability)
        .with_runtime_module(
            PluginModuleManifest::runtime(format!("{id}.runtime"), crate_name)
                .with_target_modes(target_modes.iter().copied())
                .with_capabilities([capability.to_string()]),
        )
}
