use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::{PluginModuleKind, PluginModuleManifest};

pub(super) fn runtime_module_names_for_target(
    modules: &[PluginModuleManifest],
    target: RuntimeTargetMode,
) -> impl Iterator<Item = &str> {
    modules
        .iter()
        .filter(move |module| {
            module.kind == PluginModuleKind::Runtime
                && (module.target_modes.is_empty() || module.target_modes.contains(&target))
        })
        .map(|module| module.name.as_str())
}
