mod capability_statuses;
mod maturity;

use self::capability_statuses::capability_statuses_from_plugin_toml;
use self::maturity::static_maturity_from_plugin_toml;

pub(in crate::tests::manifest) struct StaticSoundPluginMetadata {
    pub(in crate::tests::manifest) maturity: zircon_runtime::plugin::PluginMaturity,
    pub(in crate::tests::manifest) capability_statuses:
        Vec<zircon_runtime::plugin::CapabilityStatusManifest>,
}

pub(super) fn static_plugin_metadata(manifest: &str) -> StaticSoundPluginMetadata {
    StaticSoundPluginMetadata {
        maturity: static_maturity_from_plugin_toml(manifest),
        capability_statuses: capability_statuses_from_plugin_toml(manifest),
    }
}
