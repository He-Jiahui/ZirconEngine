use std::path::Path;

pub(super) fn assert_dependency_provider_id_token(
    relative_path: &Path,
    context: &str,
    field_name: &str,
    provider_id: &str,
) {
    assert_eq!(
        provider_id.trim(),
        provider_id,
        "plugin manifest {relative_path:?} {context} `{field_name}` `{provider_id}` should not have leading or trailing whitespace"
    );
    assert!(
        provider_id
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase()),
        "plugin manifest {relative_path:?} {context} `{field_name}` `{provider_id}` should start with a lowercase ASCII letter"
    );
    assert!(
        provider_id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'),
        "plugin manifest {relative_path:?} {context} `{field_name}` `{provider_id}` should contain only lowercase ASCII letters, digits, or underscores"
    );
    assert!(
        !provider_id.ends_with('_') && !provider_id.contains("__"),
        "plugin manifest {relative_path:?} {context} `{field_name}` `{provider_id}` should not end with an underscore or contain repeated underscores"
    );
}
