use std::path::Path;

pub(super) fn assert_package_id_token(relative_path: &Path, package_id: &str) {
    assert_eq!(
        package_id.trim(),
        package_id,
        "plugin manifest {relative_path:?} package id `{package_id}` should not have leading or trailing whitespace"
    );

    for segment in package_id.split('.') {
        assert!(
            !segment.is_empty(),
            "plugin manifest {relative_path:?} package id `{package_id}` should not contain empty dot-namespace segments"
        );
        assert!(
            segment
                .bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_lowercase()),
            "plugin manifest {relative_path:?} package id `{package_id}` segment `{segment}` should start with a lowercase ASCII letter"
        );
        assert!(
            segment
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'),
            "plugin manifest {relative_path:?} package id `{package_id}` should contain only lowercase ASCII letters, digits, underscores, or dots"
        );
        assert!(
            !segment.ends_with('_') && !segment.contains("__"),
            "plugin manifest {relative_path:?} package id `{package_id}` segment `{segment}` should not end with an underscore or contain repeated underscores"
        );
    }
}
