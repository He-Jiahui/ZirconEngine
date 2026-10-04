//! Build-generated descriptor adapter for the core plugin catalog.

use super::descriptor::EditorPluginDescriptor;

pub(crate) struct GeneratedEditorPluginCatalogEntry {
    pub package_id: &'static str,
    pub display_name: &'static str,
    pub crate_name: &'static str,
    pub category: &'static str,
    pub capabilities: &'static [&'static str],
}

include!(concat!(env!("OUT_DIR"), "/plugin_catalog_generated.rs"));

// 将构建期目录条目转换为编辑器可登记描述符；只能描述内置包，运行期项目插件走独立发现与登记。
pub(crate) fn builtin_editor_plugin_descriptors() -> Vec<EditorPluginDescriptor> {
    let mut descriptors = Vec::with_capacity(GENERATED_EDITOR_PLUGIN_CATALOG.len());
    for entry in GENERATED_EDITOR_PLUGIN_CATALOG.iter() {
        let mut descriptor =
            EditorPluginDescriptor::new(entry.package_id, entry.display_name, entry.crate_name)
                .with_category(entry.category);
        for capability in entry.capabilities {
            descriptor = descriptor.with_capability(*capability);
        }
        descriptors.push(descriptor);
    }
    descriptors
}

#[cfg(test)]
#[path = "tests/catalog_gen_optimization_batch_20260830bw_editor_tests.rs"]
mod optimization_batch_20260830bw_editor_tests;
