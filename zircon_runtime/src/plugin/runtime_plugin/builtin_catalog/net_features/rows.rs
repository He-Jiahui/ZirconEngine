use crate::builtin::RuntimeTargetMode;

pub(super) struct NetFeatureRow {
    pub id_suffix: &'static str,
    pub display_name: &'static str,
    pub capability: &'static str,
    pub runtime_crate: &'static str,
    pub target_modes: &'static [RuntimeTargetMode],
    pub extra_dependencies: &'static [NetFeatureDependencyRow],
}

pub(super) struct NetFeatureDependencyRow {
    pub provider_plugin_id: &'static str,
    pub capability: &'static str,
}

const SERVER_CLIENT_TARGETS: &[RuntimeTargetMode] = &[
    RuntimeTargetMode::ServerRuntime,
    RuntimeTargetMode::ClientRuntime,
];

const CLIENT_TARGETS: &[RuntimeTargetMode] = &[RuntimeTargetMode::ClientRuntime];

const CONTENT_DOWNLOAD_DEPENDENCIES: &[NetFeatureDependencyRow] = &[NetFeatureDependencyRow {
    provider_plugin_id: "net",
    capability: "runtime.feature.net.http",
}];

pub(super) const NET_FEATURE_ROWS: &[NetFeatureRow] = &[
    NetFeatureRow {
        id_suffix: "http",
        display_name: "HTTP(S)",
        capability: "runtime.feature.net.http",
        runtime_crate: "zircon_plugin_net_http_runtime",
        target_modes: SERVER_CLIENT_TARGETS,
        extra_dependencies: &[],
    },
    NetFeatureRow {
        id_suffix: "websocket",
        display_name: "WebSocket",
        capability: "runtime.feature.net.websocket",
        runtime_crate: "zircon_plugin_net_websocket_runtime",
        target_modes: SERVER_CLIENT_TARGETS,
        extra_dependencies: &[],
    },
    NetFeatureRow {
        id_suffix: "rpc",
        display_name: "Network RPC",
        capability: "runtime.feature.net.rpc",
        runtime_crate: "zircon_plugin_net_rpc_runtime",
        target_modes: SERVER_CLIENT_TARGETS,
        extra_dependencies: &[],
    },
    NetFeatureRow {
        id_suffix: "replication",
        display_name: "State Replication",
        capability: "runtime.feature.net.replication",
        runtime_crate: "zircon_plugin_net_replication_runtime",
        target_modes: SERVER_CLIENT_TARGETS,
        extra_dependencies: &[],
    },
    NetFeatureRow {
        id_suffix: "reliable_udp",
        display_name: "Reliable UDP",
        capability: "runtime.feature.net.reliable_udp",
        runtime_crate: "zircon_plugin_net_reliable_udp_runtime",
        target_modes: SERVER_CLIENT_TARGETS,
        extra_dependencies: &[],
    },
    NetFeatureRow {
        id_suffix: "content_download",
        display_name: "Content Download",
        capability: "runtime.feature.net.cdn_download",
        runtime_crate: "zircon_plugin_net_content_download_runtime",
        target_modes: CLIENT_TARGETS,
        extra_dependencies: CONTENT_DOWNLOAD_DEPENDENCIES,
    },
];
