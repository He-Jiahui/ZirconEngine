pub(super) fn build_export_target_id(
    platform_id: &str,
    profile_name: &str,
    duplicate_platform: bool,
) -> String {
    if duplicate_platform {
        format!("{platform_id}.{}", build_export_key(profile_name))
    } else {
        platform_id.to_string()
    }
}

pub(super) fn build_export_key(value: &str) -> String {
    let mut key = String::with_capacity(value.len());
    let mut started = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            key.push(ch.to_ascii_lowercase());
            started = true;
        } else if started {
            key.push('_');
        }
    }
    while key.ends_with('_') {
        key.pop();
    }
    if key.is_empty() {
        "target".to_string()
    } else {
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_export_key_preserves_internal_separator_runs_without_a_trim_copy() {
        assert_eq!(build_export_key("  Desktop--Windows  "), "desktop__windows");
        assert_eq!(build_export_key("---"), "target");
    }
}
