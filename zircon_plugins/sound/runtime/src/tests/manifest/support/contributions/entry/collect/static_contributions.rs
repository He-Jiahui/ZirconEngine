use super::super::super::{
    dependencies::dependencies_from_plugin_toml, event_catalogs::event_catalogs_from_plugin_toml,
    modules::modules_from_plugin_toml, StaticSoundContributions,
};
use super::runtime_modules::runtime_modules_from_static_modules;

pub(in crate::tests::manifest::support::contributions::entry) fn static_sound_contributions_from_plugin_toml(
    manifest: &str,
) -> StaticSoundContributions {
    StaticSoundContributions {
        dependencies: dependencies_from_plugin_toml(manifest),
        event_catalogs: event_catalogs_from_plugin_toml(manifest),
        modules: runtime_modules_from_static_modules(modules_from_plugin_toml(manifest)),
    }
}
