use super::{builtin_component_descriptors_for_tests, primitive_root_prop_default};
use crate::ui::template::{EditorPropDefault, EditorPropLiteral};

#[test]
fn builtin_catalog_exposes_dialog_content_and_confirmation_parameters() {
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
            "WorkbenchDialog",
            "popup_open",
            "boolean",
            EditorPropLiteral::Boolean(true),
        ),
        (
            "WorkbenchDialog",
            "title",
            "text",
            EditorPropLiteral::Text("Scene Settings".to_string()),
        ),
        (
            "WorkbenchDialog",
            "message",
            "text",
            EditorPropLiteral::Text(
                "Review scene-level settings before applying changes.".to_string(),
            ),
        ),
        (
            "WorkbenchConfirmDialog",
            "popup_open",
            "boolean",
            EditorPropLiteral::Boolean(true),
        ),
        (
            "WorkbenchConfirmDialog",
            "title",
            "text",
            EditorPropLiteral::Text("Delete selected node?".to_string()),
        ),
        (
            "WorkbenchConfirmDialog",
            "message",
            "text",
            EditorPropLiteral::Text("This removes the node from the scene hierarchy.".to_string()),
        ),
        (
            "WorkbenchConfirmDialog",
            "confirm_text",
            "text",
            EditorPropLiteral::Text("Delete".to_string()),
        ),
        (
            "WorkbenchConfirmDialog",
            "cancel_text",
            "text",
            EditorPropLiteral::Text("Cancel".to_string()),
        ),
        (
            "WorkbenchConfirmDialog",
            "severity",
            "enum",
            EditorPropLiteral::Text("warning".to_string()),
        ),
        (
            "WorkbenchConfirmDialog",
            "destructive",
            "boolean",
            EditorPropLiteral::Boolean(true),
        ),
    ] {
        let (document_id, actual_value_type, actual_default) =
            default_for(component_id, property_name);
        assert_eq!(actual_value_type, value_type);
        assert_eq!(
            actual_default,
            &EditorPropDefault::Literal(expected_default.clone()),
            "{component_id}.{property_name} should preserve its authored dialog default"
        );
        assert_eq!(
            primitive_root_prop_default(document_id, property_name),
            expected_default,
            "{component_id}.{property_name} should match its primitive .zui root prop"
        );
    }
}
