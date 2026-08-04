use std::collections::BTreeMap;

use zircon_runtime_interface::ui::component::UiValue;

use super::super::super::pane_value_conversion::value_as_string;
use super::super::badge::projected_badge_value_text;
use super::super::dialog::projected_dialog_value_text;
use super::super::drag_overlay::ProjectedDragOverlayData;
use super::super::notification_center::projected_notification_center_value_text;

pub(super) fn projected_value_text(
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
    drag_overlay: &ProjectedDragOverlayData,
) -> String {
    projected_dialog_value_text(component_role, attributes)
        .or_else(|| drag_overlay.value_text.clone())
        .or_else(|| projected_notification_center_value_text(component_role, attributes))
        .or_else(|| projected_badge_value_text(component_role, attributes))
        .or_else(|| {
            (component_role == "search-field")
                .then(|| attributes.get("query").and_then(value_as_string))
                .flatten()
        })
        .or_else(|| attributes.get("value_text").and_then(value_as_string))
        .or_else(|| {
            attributes
                .get("value")
                .or_else(|| attributes.get("items"))
                .or_else(|| attributes.get("entries"))
                .map(UiValue::from_toml)
                .map(|value| value.display_text())
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use toml::Value;

    use super::projected_value_text;

    #[test]
    fn search_field_projects_its_query_as_the_rendered_value() {
        let attributes = BTreeMap::from([("query".to_string(), Value::String("strafe".into()))]);

        assert_eq!(
            projected_value_text("search-field", &attributes, &Default::default()),
            "strafe"
        );
    }
}
