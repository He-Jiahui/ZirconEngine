use zircon_runtime::{PluginModuleKind, PluginPackageManifest, ProjectPluginSelection};

use super::module_crate_lookup::module_crate;

pub(in crate::ui::host::editor_manager_plugins_export) fn native_project_selection(
    package: &PluginPackageManifest,
) -> ProjectPluginSelection {
    let mut target_modes = Vec::new();
    for target_mode in package
        .modules
        .iter()
        .flat_map(|module| module.target_modes.iter().copied())
    {
        if !target_modes.contains(&target_mode) {
            target_modes.push(target_mode);
        }
    }
    ProjectPluginSelection {
        id: package.id.clone(),
        enabled: false,
        required: false,
        target_modes,
        packaging: zircon_runtime::ExportPackagingStrategy::NativeDynamic,
        runtime_crate: module_crate(package, PluginModuleKind::Runtime),
        editor_crate: module_crate(package, PluginModuleKind::Editor),
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime::{PluginModuleManifest, RuntimeTargetMode};

    use super::*;

    #[test]
    fn native_selection_aggregates_runtime_and_editor_module_target_modes() {
        let package = PluginPackageManifest::new("split_target_tool", "Split Target Tool")
            .with_runtime_module(
                PluginModuleManifest::runtime(
                    "split_target_tool.runtime",
                    "zircon_plugin_split_target_tool_runtime",
                )
                .with_target_modes([RuntimeTargetMode::ClientRuntime]),
            )
            .with_editor_module(
                PluginModuleManifest::editor(
                    "split_target_tool.editor",
                    "zircon_plugin_split_target_tool_editor",
                )
                .with_target_modes([RuntimeTargetMode::EditorHost]),
            );

        let selection = native_project_selection(&package);

        assert_eq!(
            selection.target_modes,
            vec![
                RuntimeTargetMode::ClientRuntime,
                RuntimeTargetMode::EditorHost
            ]
        );
    }
}
