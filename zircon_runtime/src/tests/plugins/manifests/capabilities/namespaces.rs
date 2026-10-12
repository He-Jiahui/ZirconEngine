use std::path::Path;

use super::super::{
    for_each_feature_extension, for_each_module_row, for_each_optional_feature,
    for_each_static_plugin_manifest, non_empty_string_array_values,
    visit_asset_importer_required_capabilities, visit_feature_dependency_rows,
    visit_option_required_capabilities, visit_package_dependency_rows,
};
use super::traversal::visit_capability_status_rows;

#[test]
fn plugin_tomls_declare_capability_namespaces() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for capability in
            non_empty_string_array_values(table, relative_path, "top-level", "capabilities")
        {
            assert_capability_namespace(relative_path, "top-level capabilities", capability);
        }

        visit_capability_status_rows(table, relative_path, &mut |capability, context| {
            assert_capability_namespace(relative_path, &context, capability);
        });
        visit_package_dependency_rows(table, relative_path, &mut |dependency_id, capability| {
            let context = format!("top-level dependency `{dependency_id}` capability");
            assert_capability_namespace(relative_path, &context, capability);
        });
        visit_asset_importer_required_capabilities(
            table,
            relative_path,
            &mut |importer_id, capability| {
                let context = format!("asset importer `{importer_id}`");
                assert_capability_namespace(relative_path, &context, capability);
            },
        );
        visit_option_required_capabilities(table, relative_path, &mut |key, capability| {
            let context = format!("plugin option `{key}` capability");
            assert_capability_namespace(relative_path, &context, capability);
        });

        for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
            for capability in non_empty_string_array_values(
                feature,
                relative_path,
                feature_context,
                "capabilities",
            ) {
                assert_capability_namespace(relative_path, feature_context, capability);
            }
            visit_feature_dependency_rows(
                feature,
                relative_path,
                feature_context,
                &mut |dependency_plugin, capability| {
                    let context =
                        format!("{feature_context} dependency `{dependency_plugin}` capability");
                    assert_capability_namespace(relative_path, &context, capability);
                },
            );
        });

        for_each_feature_extension(table, relative_path, &mut |feature, feature_context| {
            for capability in non_empty_string_array_values(
                feature,
                relative_path,
                feature_context,
                "capabilities",
            ) {
                assert_capability_namespace(relative_path, feature_context, capability);
            }
            visit_feature_dependency_rows(
                feature,
                relative_path,
                feature_context,
                &mut |dependency_plugin, capability| {
                    let context =
                        format!("{feature_context} dependency `{dependency_plugin}` capability");
                    assert_capability_namespace(relative_path, &context, capability);
                },
            );
        });

        for_each_module_row(table, relative_path, &mut |module, module_context| {
            let module_name = module
                .get("name")
                .and_then(toml::Value::as_str)
                .unwrap_or("<unknown>");
            let module_context = format!("{module_context} module `{module_name}`");
            for capability in non_empty_string_array_values(
                module,
                relative_path,
                &module_context,
                "capabilities",
            ) {
                assert_capability_namespace(relative_path, &module_context, capability);
            }
        });
    });
}

fn assert_capability_namespace(relative_path: &Path, context: &str, capability: &str) {
    assert_eq!(
        capability.trim(),
        capability,
        "plugin manifest {relative_path:?} {context} capability `{capability}` should not have leading or trailing whitespace"
    );

    let segments: Vec<_> = capability.split('.').collect();
    assert!(
        segments.len() >= 2,
        "plugin manifest {relative_path:?} {context} capability `{capability}` should use at least two dot-separated namespace segments"
    );

    for segment in segments {
        assert!(
            !segment.is_empty(),
            "plugin manifest {relative_path:?} {context} capability `{capability}` should not contain empty namespace segments"
        );
        assert!(
            segment
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_'),
            "plugin manifest {relative_path:?} {context} capability `{capability}` segment `{segment}` should use lowercase ASCII letters, digits, or underscores"
        );
    }
}
