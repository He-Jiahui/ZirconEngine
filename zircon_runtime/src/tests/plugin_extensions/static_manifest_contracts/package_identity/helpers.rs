use std::path::Path;

pub(super) fn assert_package_id_token(relative_path: &Path, package_id: &str) {
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
