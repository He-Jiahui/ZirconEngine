use super::plugin_event_catalog_namespace_from_module;

#[test]
fn exact_event_catalog_namespace_preserves_module_identity() {
    assert_eq!(
        plugin_event_catalog_namespace_from_module("weather.runtime"),
        Some("weather.events".to_string())
    );
    assert_eq!(
        plugin_event_catalog_namespace_from_module("weather"),
        Some("weather.events".to_string())
    );
    assert_eq!(plugin_event_catalog_namespace_from_module(".runtime"), None);
}
