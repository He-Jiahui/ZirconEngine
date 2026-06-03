mod asset_scene;
mod component;
mod plugin_metadata;

use crate::plugin::RuntimeExtensionRegistry;

use asset_scene::merge_asset_scene_descriptor_contributions;
use component::merge_component_descriptor_contributions;
use plugin_metadata::merge_plugin_metadata_descriptor_contributions;

pub(super) fn merge_descriptor_extension_registry_contributions(
    extensions: &RuntimeExtensionRegistry,
    registry: &mut RuntimeExtensionRegistry,
    diagnostics: &mut Vec<String>,
    fatal_diagnostics: &mut Vec<String>,
) {
    merge_component_descriptor_contributions(extensions, registry, diagnostics, fatal_diagnostics);
    merge_plugin_metadata_descriptor_contributions(
        extensions,
        registry,
        diagnostics,
        fatal_diagnostics,
    );
    merge_asset_scene_descriptor_contributions(
        extensions,
        registry,
        diagnostics,
        fatal_diagnostics,
    );
}
