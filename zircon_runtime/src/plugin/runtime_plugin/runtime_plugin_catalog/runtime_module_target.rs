use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::{PluginModuleKind, PluginModuleManifest};

// 只提取 Runtime kind 且目标匹配的模块名；空 target_modes 在过滤条件中表示不限制目标。
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
