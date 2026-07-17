pub(super) fn runtime_plugin_package_event_catalog_has_owner(
    package_id: &str,
    event_catalog_namespace: &str,
) -> bool {
    event_catalog_namespace
        .strip_prefix(package_id)
        .is_some_and(|suffix| suffix.starts_with('.'))
}

#[cfg(test)]
mod tests {
    #[test]
    fn event_catalog_owner_check_does_not_format_a_prefix() {
        let source = include_str!("prefix.rs");
        let formatted_prefix = ["format!(\"", "{package_id}.", "\")"].concat();
        assert!(!source.contains(&formatted_prefix));
    }

    #[test]
    fn event_catalog_owner_check_preserves_the_dot_boundary() {
        assert!(super::runtime_plugin_package_event_catalog_has_owner(
            "rendering",
            "rendering.events"
        ));
        assert!(!super::runtime_plugin_package_event_catalog_has_owner(
            "render",
            "rendering.events"
        ));
    }
}
