use crate::plugin::RuntimeExtensionRegistry;

use super::super::contributions::push_runtime_extension_result;

pub(super) fn merge_asset_scene_descriptor_contributions(
    extensions: &RuntimeExtensionRegistry,
    registry: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    for importer in extensions.asset_importers().importers() {
        push_runtime_extension_result(
            registry.register_asset_importer_arc(importer),
            diagnostics,
            fatal_diagnostics,
        );
    }
    for hook in extensions.scene_hooks() {
        push_runtime_extension_result(
            registry.register_scene_hook(hook.clone()),
            diagnostics,
            fatal_diagnostics,
        );
    }
}
