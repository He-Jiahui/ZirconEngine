mod dependencies;
mod event_catalogs;
mod modules;

use self::dependencies::dependencies_from_plugin_toml;
use self::event_catalogs::event_catalogs_from_plugin_toml;
use self::modules::modules_from_plugin_toml;

type StaticDependency = (String, bool, Option<String>);
type StaticEventCatalog = (String, u32);
type StaticModule = (
    String,
    zircon_runtime::plugin::PluginModuleKind,
    String,
    Vec<zircon_runtime::RuntimeTargetMode>,
    Vec<String>,
);

pub(in crate::tests::manifest) struct StaticSoundContributions {
    pub(in crate::tests::manifest) dependencies: Vec<StaticDependency>,
    pub(in crate::tests::manifest) event_catalogs: Vec<StaticEventCatalog>,
    pub(in crate::tests::manifest) modules: Vec<StaticModule>,
}

pub(super) fn static_sound_contributions(manifest: &str) -> StaticSoundContributions {
    let mut dependencies = dependencies_from_plugin_toml(manifest);
    let mut event_catalogs = event_catalogs_from_plugin_toml(manifest);
    let mut modules = modules_from_plugin_toml(manifest)
        .into_iter()
        .filter(|module| module.1 == zircon_runtime::plugin::PluginModuleKind::Runtime)
        .collect::<Vec<_>>();
    dependencies.sort_unstable();
    event_catalogs.sort_unstable();
    modules.sort_unstable_by_key(|module| module.0.clone());

    StaticSoundContributions {
        dependencies,
        event_catalogs,
        modules,
    }
}
