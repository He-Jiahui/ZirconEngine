use std::path::Path;

use super::super::{
    for_each_optional_feature, for_each_static_plugin_manifest, non_empty_string_value,
};

#[test]
fn plugin_tomls_declare_optional_feature_ids_are_dot_namespaced() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
            let feature_id = non_empty_string_value(feature, relative_path, feature_context, "id");
            assert_dot_namespaced_feature_id(relative_path, feature_context, feature_id);
        });
    });
}

#[test]
fn plugin_tomls_declare_optional_feature_ids_under_owner_namespace() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let package_id = non_empty_string_value(table, relative_path, "top-level", "id");
        let expected_prefix = format!("{package_id}.");

        for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
            let feature_id = non_empty_string_value(feature, relative_path, feature_context, "id");
            assert!(
                feature_id.starts_with(&expected_prefix),
                "plugin manifest {relative_path:?} {feature_context} id `{feature_id}` should stay under owner namespace `{expected_prefix}`"
            );
        });
    });
}

fn assert_dot_namespaced_feature_id(relative_path: &Path, context: &str, feature_id: &str) {
    assert_eq!(
        feature_id.trim(),
        feature_id,
        "plugin manifest {relative_path:?} {context} id `{feature_id}` should not have leading or trailing whitespace"
    );

    let segments: Vec<_> = feature_id.split('.').collect();
    assert!(
        segments.len() >= 2,
        "plugin manifest {relative_path:?} {context} id `{feature_id}` should use owner.feature dot namespace form"
    );

    for segment in segments {
        assert!(
            !segment.is_empty(),
            "plugin manifest {relative_path:?} {context} id `{feature_id}` should not contain empty namespace segments"
        );
        assert!(
            segment
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'),
            "plugin manifest {relative_path:?} {context} id `{feature_id}` should contain only lowercase ASCII letters, digits, underscores, and dots"
        );
    }
}
