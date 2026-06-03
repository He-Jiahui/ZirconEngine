use super::super::{
    for_each_static_plugin_manifest, non_empty_string_array_values, non_empty_string_value,
};
use super::traversal::capability_status_array;

#[test]
fn plugin_tomls_declare_capability_status_target_modes_within_package_targets() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let package_targets =
            non_empty_string_array_values(table, relative_path, "top-level", "supported_targets");
        let Some(statuses) = capability_status_array(table, relative_path) else {
            return;
        };

        for status in statuses {
            let status = status.as_table().unwrap_or_else(|| {
                panic!("plugin manifest {relative_path:?} capability status should be a table")
            });
            let capability =
                non_empty_string_value(status, relative_path, "capability status", "capability");
            let context = format!("capability status `{capability}`");
            if status.get("target_modes").is_none() {
                continue;
            }

            for target_mode in
                non_empty_string_array_values(status, relative_path, &context, "target_modes")
            {
                assert!(
                    package_targets.contains(&target_mode),
                    "plugin manifest {relative_path:?} {context} target mode `{target_mode}` should be covered by package supported_targets"
                );
            }
        }
    });
}
