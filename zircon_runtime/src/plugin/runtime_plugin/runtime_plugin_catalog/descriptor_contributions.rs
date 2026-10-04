mod asset_scene;
mod component;
mod plugin_metadata;

use crate::plugin::RuntimeExtensionRegistry;

use asset_scene::merge_asset_scene_descriptor_contributions;
use component::merge_component_descriptor_contributions;
use plugin_metadata::merge_plugin_metadata_descriptor_contributions;

// 描述贡献按 component、plugin metadata、asset scene 固定顺序注册；单项错误记入两类诊断后继续处理后续项。
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
