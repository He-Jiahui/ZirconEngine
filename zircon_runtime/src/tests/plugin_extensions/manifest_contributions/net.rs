use super::*;

#[test]
fn net_plugin_toml_declares_content_download_http_dependency() {
    let plugins_root = plugins_workspace_root();
    let manifest = read_plugin_manifest(&plugins_root, "net");
    let encoded = toml::to_string(&manifest).expect("net plugin manifest toml");
    let decoded: PluginPackageManifest =
        toml::from_str(&encoded).expect("net plugin manifest roundtrip");
    let runtime_module = manifest
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Runtime)
        .expect("net plugin should declare a runtime module");
    let descriptor = RuntimePluginDescriptor::builtin_catalog()
        .into_iter()
        .find(|descriptor| descriptor.runtime_id() == RuntimePluginId::Net)
        .expect("net plugin should be in the runtime catalog");
    let expected_targets = vec![
        RuntimeTargetMode::ServerRuntime,
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ];
    let expected_capabilities = vec!["runtime.plugin.net".to_string()];

    assert_eq!(decoded, manifest);
    assert_eq!(manifest.sdk_api_version, "0.1.0");
    assert_eq!(manifest.category, "runtime");
    assert_eq!(manifest.maturity, crate::plugin::PluginMaturity::Beta);
    assert_eq!(manifest.supported_targets, expected_targets);
    assert_eq!(manifest.capabilities, expected_capabilities);
    assert_eq!(runtime_module.target_modes, manifest.supported_targets);
    assert_eq!(runtime_module.capabilities, manifest.capabilities);
    assert_eq!(
        descriptor.target_modes(),
        manifest.supported_targets.as_slice()
    );
    assert_eq!(descriptor.capabilities(), manifest.capabilities.as_slice());
    assert!(manifest.capability_statuses.iter().any(|status| {
        status.capability == "runtime.plugin.net" && status.status == CapabilityStatus::Partial
    }));
    let content_download = manifest
        .optional_features
        .iter()
        .find(|feature| feature.id == "net.content_download")
        .expect("content download optional feature");

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
fn builtin_net_catalog_declares_layered_optional_features() {
    let descriptor = RuntimePluginDescriptor::builtin_catalog()
        .into_iter()
        .find(|descriptor| descriptor.package_id() == "net")
        .expect("net catalog entry");

    assert_eq!(
        descriptor
            .optional_features()
            .iter()
            .map(|feature| feature.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "net.http",
            "net.websocket",
            "net.rpc",
            "net.replication",
            "net.reliable_udp",
            "net.content_download",
        ]
    );

    for (feature_id, capability, runtime_crate, target_modes) in [
        (
            "net.http",
            "runtime.feature.net.http",
            "zircon_plugin_net_http_runtime",
            vec![
                RuntimeTargetMode::ServerRuntime,
                RuntimeTargetMode::ClientRuntime,
            ],
        ),
        (
            "net.websocket",
            "runtime.feature.net.websocket",
            "zircon_plugin_net_websocket_runtime",
            vec![
                RuntimeTargetMode::ServerRuntime,
                RuntimeTargetMode::ClientRuntime,
            ],
        ),
        (
            "net.rpc",
            "runtime.feature.net.rpc",
            "zircon_plugin_net_rpc_runtime",
            vec![
                RuntimeTargetMode::ServerRuntime,
                RuntimeTargetMode::ClientRuntime,
            ],
        ),
        (
            "net.replication",
            "runtime.feature.net.replication",
            "zircon_plugin_net_replication_runtime",
            vec![
                RuntimeTargetMode::ServerRuntime,
                RuntimeTargetMode::ClientRuntime,
            ],
        ),
        (
            "net.reliable_udp",
            "runtime.feature.net.reliable_udp",
            "zircon_plugin_net_reliable_udp_runtime",
            vec![
                RuntimeTargetMode::ServerRuntime,
                RuntimeTargetMode::ClientRuntime,
            ],
        ),
        (
            "net.content_download",
            "runtime.feature.net.cdn_download",
            "zircon_plugin_net_content_download_runtime",
            vec![RuntimeTargetMode::ClientRuntime],
        ),
    ] {
        let feature = descriptor
            .optional_features()
            .iter()
            .find(|feature| feature.id == feature_id)
            .expect("net optional feature should be present in the built-in catalog");

        assert!(feature.capabilities.contains(&capability.to_string()));
        assert!(feature.dependencies.iter().any(|dependency| {
            dependency.plugin_id == "net"
                && dependency.capability == "runtime.plugin.net"
                && dependency.primary
        }));
        assert!(feature.modules.iter().any(|module| {
            module.kind == PluginModuleKind::Runtime
                && module.name == format!("{feature_id}.runtime")
                && module.crate_name == runtime_crate
                && module.target_modes == target_modes
                && module.capabilities.contains(&capability.to_string())
        }));
    }

    let content_download = descriptor
        .optional_features()
        .iter()
        .find(|feature| feature.id == "net.content_download")
        .expect("content download feature");
    assert!(content_download.dependencies.iter().any(|dependency| {
        dependency.plugin_id == "net"
            && dependency.capability == "runtime.feature.net.http"
            && !dependency.primary
    }));
}
