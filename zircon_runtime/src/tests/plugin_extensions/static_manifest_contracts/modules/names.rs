use std::path::Path;

use super::super::{
    for_each_module_row, for_each_optional_feature, for_each_static_plugin_manifest,
    non_empty_string_value, visit_module_rows,
};

#[test]
fn plugin_tomls_declare_module_names_under_owner_namespace() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let package_id = non_empty_string_value(table, relative_path, "top-level", "id");
        let package_prefix = format!("{package_id}.");

        visit_module_rows(
            table.get("modules"),
            relative_path,
            "package",
            &mut |module, module_context| {
                let module_name =
                    non_empty_string_value(module, relative_path, module_context, "name");
                assert_module_name_prefix(
                    relative_path,
                    module_context,
                    module_name,
                    &package_prefix,
                );
            },
        );

        for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
            let feature_id = non_empty_string_value(feature, relative_path, feature_context, "id");
            let feature_prefix = format!("{feature_id}.");
            visit_module_rows(
                feature.get("modules"),
                relative_path,
                feature_context,
                &mut |module, module_context| {
                    let module_name =
                        non_empty_string_value(module, relative_path, module_context, "name");
                    assert_module_name_prefix(
                        relative_path,
                        module_context,
                        module_name,
                        &feature_prefix,
                    );
                },
            );
        });
    });
}

fn assert_module_name_prefix(
    relative_path: &Path,
    context: &str,
    module_name: &str,
    expected_prefix: &str,
) {
    assert!(
        module_name.starts_with(expected_prefix),
        "plugin manifest {relative_path:?} {context} module name `{module_name}` should stay under namespace `{expected_prefix}`"
    );
}

fn assert_dot_namespaced_module_name(relative_path: &Path, context: &str, module_name: &str) {
    assert_eq!(
        module_name.trim(),
        module_name,
        "plugin manifest {relative_path:?} {context} module name `{module_name}` should not have leading or trailing whitespace"
    );

    let segments: Vec<_> = module_name.split('.').collect();
    assert!(
        segments.len() >= 2,
        "plugin manifest {relative_path:?} {context} module name `{module_name}` should use package.module dot namespace form"
    );

    for segment in segments {
        assert!(
            !segment.is_empty(),
            "plugin manifest {relative_path:?} {context} module name `{module_name}` should not contain empty namespace segments"
        );
        assert!(
            segment
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'),
            "plugin manifest {relative_path:?} {context} module name `{module_name}` should contain only lowercase ASCII letters, digits, underscores, and dots"
        );
    }
}

#[test]
fn plugin_tomls_declare_module_names_are_dot_namespaced() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for_each_module_row(table, relative_path, &mut |module, module_context| {
            let module_name = non_empty_string_value(module, relative_path, module_context, "name");
            assert_dot_namespaced_module_name(relative_path, module_context, module_name);
        });
    });
}

#[test]
fn plugin_tomls_declare_module_names_match_module_kind() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for_each_module_row(table, relative_path, &mut |module, module_context| {
            let module_name = non_empty_string_value(module, relative_path, module_context, "name");
            let module_kind = non_empty_string_value(module, relative_path, module_context, "kind");
            let expected_suffix = match module_kind {
                "runtime" => ".runtime",
                "editor" => ".editor",
                _ => return,
            };

            assert!(
                module_name.ends_with(expected_suffix),
                "plugin manifest {relative_path:?} {module_context} module name `{module_name}` with kind `{module_kind}` should end with `{expected_suffix}`"
            );
        });
    });
}
