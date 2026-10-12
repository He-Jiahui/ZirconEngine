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

pub(super) fn assert_package_token(
    relative_path: &Path,
    context: &str,
    field_name: &str,
    value: &str,
) {
    assert_trimmed(relative_path, context, field_name, value);
    assert!(
        value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase()),
        "plugin manifest {relative_path:?} {context} {field_name} `{value}` should start with a lowercase ASCII letter"
    );
    assert!(
        value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'),
        "plugin manifest {relative_path:?} {context} {field_name} `{value}` should contain only lowercase ASCII letters, digits, or underscores"
    );
    assert!(
        !value.ends_with('_') && !value.contains("__"),
        "plugin manifest {relative_path:?} {context} {field_name} `{value}` should not end with an underscore or contain repeated underscores"
    );
}

pub(super) fn assert_crate_name_shape(relative_path: &Path, context: &str, crate_name: &str) {
    assert_trimmed(relative_path, context, "crate_name", crate_name);
    assert!(
        crate_name.starts_with("zircon_plugin_")
            && crate_name
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'),
        "plugin manifest {relative_path:?} {context} crate_name `{crate_name}` should use `zircon_plugin_` prefix and lowercase snake case"
    );
    assert!(
        !crate_name.ends_with('_') && !crate_name.contains("__"),
        "plugin manifest {relative_path:?} {context} crate_name `{crate_name}` should not end with an underscore or contain repeated underscores"
    );
}

pub(super) fn assert_trimmed(relative_path: &Path, context: &str, field_name: &str, value: &str) {
    assert_eq!(
        value.trim(),
        value,
        "plugin manifest {relative_path:?} {context} {field_name} `{value}` should not have leading or trailing whitespace"
    );
}
