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
    let key = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    if key.is_empty() {
        "target".to_string()
    } else {
        key
    }
}
