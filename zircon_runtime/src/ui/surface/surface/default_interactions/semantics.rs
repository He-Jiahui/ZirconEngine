use zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata;
use zircon_runtime_interface::ui::widget::UI_WIDGET_COMPONENT_ROLE_ATTRIBUTE;

pub(super) fn component_role(metadata: &UiTemplateNodeMetadata) -> Option<&str> {
    metadata
        .attributes
        .get(UI_WIDGET_COMPONENT_ROLE_ATTRIBUTE)
        .and_then(toml::Value::as_str)
}

pub(super) fn component_role_is(metadata: &UiTemplateNodeMetadata, role: &str) -> bool {
    component_role(metadata) == Some(role)
}

pub(super) fn component_role_is_one_of(metadata: &UiTemplateNodeMetadata, roles: &[&str]) -> bool {
    component_role(metadata).is_some_and(|role| roles.contains(&role))
}

#[cfg(test)]
mod tests {
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
}
