use std::path::Path;

use super::{for_each_static_plugin_manifest, non_empty_string_value};

#[test]
fn plugin_tomls_declare_semantic_package_versions() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for field_name in ["version", "sdk_api_version"] {
            let value = non_empty_string_value(table, relative_path, "top-level", field_name);
            assert_semver_core(relative_path, "top-level", field_name, value);
        }
    });
}

fn assert_semver_core(relative_path: &Path, context: &str, field_name: &str, value: &str) {
    assert_eq!(
        value.trim(),
        value,
        "plugin manifest {relative_path:?} {context} `{field_name}` value `{value}` should not have leading or trailing whitespace"
    );

    let mut segments = value.split('.');
    for component_name in ["major", "minor", "patch"] {
        let segment = segments.next().unwrap_or_else(|| {
            panic!(
                "plugin manifest {relative_path:?} {context} `{field_name}` value `{value}` should use MAJOR.MINOR.PATCH form"
            )
        });
        assert_semver_segment(
            relative_path,
            context,
            field_name,
            value,
            component_name,
            segment,
        );
    }

    assert!(
        segments.next().is_none(),
        "plugin manifest {relative_path:?} {context} `{field_name}` value `{value}` should use MAJOR.MINOR.PATCH form"
    );
}

fn assert_semver_segment(
    relative_path: &Path,
    context: &str,
    field_name: &str,
    value: &str,
    component_name: &str,
    segment: &str,
) {
    assert!(
        !segment.is_empty(),
        "plugin manifest {relative_path:?} {context} `{field_name}` value `{value}` has an empty {component_name} component"
    );
    assert!(
        segment.chars().all(|character| character.is_ascii_digit()),
        "plugin manifest {relative_path:?} {context} `{field_name}` value `{value}` {component_name} component `{segment}` should contain only ASCII digits"
    );
    assert!(
        segment == "0" || !segment.starts_with('0'),
        "plugin manifest {relative_path:?} {context} `{field_name}` value `{value}` {component_name} component `{segment}` should not use leading zeroes"
    );

    segment.parse::<u32>().unwrap_or_else(|error| {
        panic!(
            "plugin manifest {relative_path:?} {context} `{field_name}` value `{value}` {component_name} component `{segment}` should fit in u32: {error}"
        )
    });
}
