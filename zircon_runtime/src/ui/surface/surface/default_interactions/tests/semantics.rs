use std::collections::BTreeMap;

use super::*;

fn metadata(component: &str, role: Option<&str>) -> UiTemplateNodeMetadata {
    let mut attributes = BTreeMap::new();
    if let Some(role) = role {
        attributes.insert(
            UI_WIDGET_COMPONENT_ROLE_ATTRIBUTE.to_string(),
            toml::Value::String(role.to_string()),
        );
    }
    UiTemplateNodeMetadata {
        component: component.to_string(),
        attributes,
        ..Default::default()
    }
}

#[test]
fn semantic_role_does_not_fall_back_to_component_name() {
    assert!(!component_role_is(&metadata("DataGrid", None), "data-grid"));
    assert!(component_role_is(
        &metadata("ProductTable", Some("data-grid")),
        "data-grid"
    ));
}
