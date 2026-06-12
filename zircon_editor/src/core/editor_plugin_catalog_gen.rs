use super::editor_plugin::EditorPluginDescriptor;

pub(crate) struct GeneratedEditorPluginCatalogEntry {
    pub package_id: &'static str,
    pub display_name: &'static str,
    pub crate_name: &'static str,
    pub category: &'static str,
    pub capabilities: &'static [&'static str],
}

include!(concat!(env!("OUT_DIR"), "/editor_plugin_catalog_gen.rs"));

pub(crate) fn builtin_editor_plugin_descriptors() -> Vec<EditorPluginDescriptor> {
    GENERATED_EDITOR_PLUGIN_CATALOG
        .iter()
        .map(|entry| {
            let mut descriptor =
                EditorPluginDescriptor::new(entry.package_id, entry.display_name, entry.crate_name)
                    .with_category(entry.category);
            for capability in entry.capabilities {
                descriptor = descriptor.with_capability(*capability);
            }
            descriptor
        })
        .collect()
}
