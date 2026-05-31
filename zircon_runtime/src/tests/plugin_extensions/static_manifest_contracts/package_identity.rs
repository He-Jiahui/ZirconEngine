use std::path::Path;

use super::{for_each_optional_feature, for_each_static_plugin_manifest, non_empty_string_value};

#[test]
fn plugin_tomls_declare_package_ids_match_directories() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let package_id = non_empty_string_value(table, relative_path, "top-level", "id");
        assert_package_id_token(relative_path, package_id);
        assert_package_id_matches_manifest_directory(relative_path, package_id);
    });
}

#[test]
fn plugin_tomls_declare_optional_feature_ids_are_dot_namespaced() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
            let feature_id = non_empty_string_value(feature, relative_path, feature_context, "id");
            assert_dot_namespaced_feature_id(relative_path, feature_context, feature_id);
        });
    });
}

fn assert_package_id_token(relative_path: &Path, package_id: &str) {
    assert_eq!(
        package_id.trim(),
        package_id,
        "plugin manifest {relative_path:?} package id `{package_id}` should not have leading or trailing whitespace"
    );
    assert!(
        package_id
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase()),
        "plugin manifest {relative_path:?} package id `{package_id}` should start with a lowercase ASCII letter"
    );
    assert!(
        package_id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'),
        "plugin manifest {relative_path:?} package id `{package_id}` should contain only lowercase ASCII letters, digits, or underscores"
    );
    assert!(
        !package_id.ends_with('_') && !package_id.contains("__"),
        "plugin manifest {relative_path:?} package id `{package_id}` should not end with an underscore or contain repeated underscores"
    );
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

fn assert_package_id_matches_manifest_directory(relative_path: &Path, package_id: &str) {
    let directory_name = relative_path
        .parent()
        .and_then(Path::file_name)
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
