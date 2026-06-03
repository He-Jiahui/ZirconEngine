use std::collections::BTreeMap;

use super::super::for_each_static_plugin_manifest;
use super::helpers::resolved_package_id;

#[test]
fn plugin_tomls_declare_unique_resolved_package_ids() {
    let mut resolved_package_ids = BTreeMap::new();

    for_each_static_plugin_manifest(|relative_path, table| {
        let resolved_package_id = resolved_package_id(table, relative_path);
        if let Some(previous_path) = resolved_package_ids.insert(
            resolved_package_id.clone(),
            relative_path.display().to_string(),
        ) {
            panic!(
                "resolved package id `{resolved_package_id}` should be globally unique; first declared by {previous_path}, repeated by {}",
                relative_path.display()
            );
        }
    });
}
