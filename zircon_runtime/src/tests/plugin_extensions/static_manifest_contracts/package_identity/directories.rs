use super::super::{for_each_static_plugin_manifest, non_empty_string_value};
use super::helpers::assert_package_id_token;

#[test]
fn plugin_tomls_declare_package_ids_match_directories() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let package_id = non_empty_string_value(table, relative_path, "top-level", "id");
        assert_package_id_token(relative_path, package_id);
        assert_package_id_matches_manifest_directory(relative_path, package_id);
    });
}

fn assert_package_id_matches_manifest_directory(relative_path: &std::path::Path, package_id: &str) {
    let directory_name = relative_path
        .parent()
        .and_then(std::path::Path::file_name)
        .and_then(|name| name.to_str())
        .unwrap_or_else(|| {
            panic!(
                "plugin manifest {relative_path:?} should live under zircon_plugins/<plugin_id>/plugin.toml"
            )
        });

    assert_eq!(
        directory_name, package_id,
        "plugin manifest {relative_path:?} top-level id `{package_id}` should match package directory `{directory_name}`"
    );
}
