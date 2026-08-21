use super::{builtin_component_descriptors_for_tests, primitive_root_prop_default};
use crate::ui::template::{EditorPropDefault, EditorPropLiteral};

#[test]
fn builtin_catalog_exposes_feedback_container_state_and_progress_parameters() {
    let descriptors = builtin_component_descriptors_for_tests();
    let default_for = |component_id: &str, property_name: &str| {
        descriptors
            .iter()
            .find(|descriptor| descriptor.component_id == component_id)
            .and_then(|descriptor| {
                descriptor
                    .props
                    .iter()
                    .find(|property| property.name == property_name)
                    .map(|property| {
                        (
                            descriptor.document_id.as_str(),
                            &property.value_type,
                            &property.default,
                        )
                    })
            })
            .unwrap_or_else(|| {
                panic!("builtin catalog should expose `{property_name}` on `{component_id}`")
            })
    };

    for (component_id, property_name, value_type, expected_default) in [
        (
            "WorkbenchNotificationCenter",
            "popup_open",
            "boolean",
            EditorPropLiteral::Boolean(true),
        ),
        (
            "WorkbenchNotificationCenter",
            "title",
            "text",
            EditorPropLiteral::Text("Notifications".to_string()),
        ),
        (
            "WorkbenchNotificationCenter",
            "unread_count",
            "number",
            EditorPropLiteral::Integer(2),
        ),
        (
            "WorkbenchNotificationCenter",
            "notifications",
            "text_list",
            EditorPropLiteral::TextList(vec![
                "build|title=Build failed|message=Shader compile error|severity=error|unread=true"
                    .to_string(),
                "asset|title=Asset import complete|message=StoneWall.mesh ready|severity=success|unread=true"
                    .to_string(),
                "source|title=Source control synced|message=No local conflicts|severity=info|unread=false"
                    .to_string(),
            ]),
        ),
        (
            "WorkbenchNotificationCenter",
            "selected_notification_id",
            "text",
            EditorPropLiteral::Text("build".to_string()),
        ),
        (
            "WorkbenchNotificationCenter",
            "empty_text",
            "text",
            EditorPropLiteral::Text("No notifications".to_string()),
        ),
        (
            "WorkbenchProgressBar",
            "value",
            "number",
            EditorPropLiteral::Float(64.0),
        ),
        (
            "WorkbenchProgressBar",
            "min",
            "number",
            EditorPropLiteral::Float(0.0),
        ),
        (
            "WorkbenchProgressBar",
            "max",
            "number",
            EditorPropLiteral::Float(100.0),
        ),
        (
            "WorkbenchProgressBar",
            "variant",
            "enum",
            EditorPropLiteral::Text("linear".to_string()),
        ),
        (
            "WorkbenchProgressBar",
            "label_text",
            "text",
            EditorPropLiteral::Text("Progress".to_string()),
        ),
        (
            "WorkbenchProgressBar",
            "show_label",
            "boolean",
            EditorPropLiteral::Boolean(false),
        ),
    ] {
        let (document_id, actual_value_type, actual_default) =
            default_for(component_id, property_name);
        assert_eq!(actual_value_type, value_type);
        assert_eq!(
            actual_default,
            &EditorPropDefault::Literal(expected_default.clone()),
            "{component_id}.{property_name} should preserve its authored feedback default"
        );
        assert_eq!(
            primitive_root_prop_default(document_id, property_name),
            expected_default,
            "{component_id}.{property_name} should match its primitive .zui root prop"
        );
    }
}
