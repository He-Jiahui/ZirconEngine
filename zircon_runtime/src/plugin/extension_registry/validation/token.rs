pub(super) fn is_lowercase_plugin_token(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

pub(super) fn is_lowercase_plugin_package_id(value: &str) -> bool {
    !value.trim().is_empty()
        && value.trim() == value
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase())
        && value.split('.').all(is_lowercase_plugin_package_segment)
}

fn is_lowercase_plugin_package_segment(value: &str) -> bool {
    is_lowercase_plugin_token(value) && !value.ends_with('_') && !value.contains("__")
}
