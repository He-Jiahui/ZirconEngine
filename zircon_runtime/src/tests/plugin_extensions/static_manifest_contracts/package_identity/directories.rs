use super::super::{for_each_static_plugin_manifest, non_empty_string_value};
use super::package_id_tokens::assert_package_id_token;

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

    let materialized_package_id = package_id.replace('.', "_");
    let package_leaf = package_id
        .rsplit('.')
        .next()
        .expect("package ids split into at least one segment");
    let accepted_directory_names = [package_id, materialized_package_id.as_str(), package_leaf];

    assert!(
        accepted_directory_names.contains(&directory_name),
        "plugin manifest {relative_path:?} top-level id `{package_id}` should match package directory `{directory_name}` by exact id, underscore materialization, or dot-namespace leaf"
    );
}
