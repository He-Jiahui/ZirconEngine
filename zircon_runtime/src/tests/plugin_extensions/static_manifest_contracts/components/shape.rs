use std::path::Path;

pub(super) fn assert_lowercase_dot_namespace(
    relative_path: &Path,
    context: &str,
    field_name: &str,
    value: &str,
) {
    assert_trimmed(relative_path, context, field_name, value);
    let segments: Vec<_> = value.split('.').collect();
    assert!(
        segments.len() >= 2,
        "plugin manifest {relative_path:?} {context} {field_name} `{value}` should use lowercase dot namespace form"
    );
    for segment in segments {
        assert!(
            !segment.is_empty(),
            "plugin manifest {relative_path:?} {context} {field_name} `{value}` should not contain empty namespace segments"
        );
        assert!(
            segment
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'),
            "plugin manifest {relative_path:?} {context} {field_name} `{value}` should contain only lowercase ASCII letters, digits, underscores, and dots"
        );
    }
}

pub(super) fn assert_prefixed_by_package(
    relative_path: &Path,
    context: &str,
    field_name: &str,
    value: &str,
    package_id: &str,
) {
    let expected_prefix = format!("{package_id}.");
    assert!(
        value.starts_with(&expected_prefix),
        "plugin manifest {relative_path:?} {context} {field_name} `{value}` should stay under package namespace `{expected_prefix}`"
    );
}

pub(super) fn assert_package_path(
    relative_path: &Path,
    context: &str,
    field_name: &str,
    value: &str,
) {
    assert!(
        !value.starts_with('/') && !value.contains('\\'),
        "plugin manifest {relative_path:?} {context} {field_name} `{value}` should be a relative forward-slash package path"
    );
    for segment in value.split('/') {
        assert!(
            !matches!(segment, "" | "." | ".."),
            "plugin manifest {relative_path:?} {context} {field_name} `{value}` should not contain empty, current, or parent path segments"
        );
    }
}

pub(super) fn assert_trimmed(relative_path: &Path, context: &str, field_name: &str, value: &str) {
    assert_eq!(
        value.trim(),
        value,
        "plugin manifest {relative_path:?} {context} {field_name} `{value}` should not have leading or trailing whitespace"
    );
}
