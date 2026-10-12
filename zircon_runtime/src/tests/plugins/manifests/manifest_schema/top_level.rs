use super::super::for_each_static_plugin_manifest;
use super::field_sets::KNOWN_TOP_LEVEL_FIELDS;

#[test]
fn plugin_tomls_declare_known_top_level_fields() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for field_name in table.keys() {
            assert!(
                KNOWN_TOP_LEVEL_FIELDS.contains(&field_name.as_str()),
                "plugin manifest {relative_path:?} top-level field `{field_name}` is not a known PluginPackageManifest field"
            );
        }
    });
}
