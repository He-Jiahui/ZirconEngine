pub(super) fn maturity_from_plugin_toml(value: &str) -> zircon_runtime::plugin::PluginMaturity {
    match value {
        "core" => zircon_runtime::plugin::PluginMaturity::Core,
        "stable" => zircon_runtime::plugin::PluginMaturity::Stable,
        "beta" => zircon_runtime::plugin::PluginMaturity::Beta,
        "experimental" => zircon_runtime::plugin::PluginMaturity::Experimental,
        "externalized" => zircon_runtime::plugin::PluginMaturity::Externalized,
        "stub" => zircon_runtime::plugin::PluginMaturity::Stub,
        "deprecated" => zircon_runtime::plugin::PluginMaturity::Deprecated,
        _ => panic!("unknown sound plugin maturity {value}"),
    }
}
