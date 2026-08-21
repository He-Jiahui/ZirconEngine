use super::{builtin_component_descriptors_for_tests, primitive_root_prop_default};
use crate::ui::template::{EditorPropDefault, EditorPropLiteral};

#[test]
fn builtin_catalog_exposes_tooltip_content_and_icon_parameters() {
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

    for (property_name, value_type, expected_default) in [
        (
            "text",
            "text",
            EditorPropLiteral::Text("Tooltip".to_string()),
        ),
        (
            "label_text",
            "text",
            EditorPropLiteral::Text("This is a tooltip".to_string()),
        ),
        (
            "icon",
            "icon_ref",
            EditorPropLiteral::Text("info".to_string()),
        ),
        ("show_icon", "boolean", EditorPropLiteral::Boolean(false)),
        (
            "surface_variant",
            "enum",
            EditorPropLiteral::Text("workbench-tooltip".to_string()),
        ),
    ] {
        let (document_id, actual_value_type, actual_default) =
            default_for("WorkbenchTooltip", property_name);
        assert_eq!(actual_value_type, value_type);
        assert_eq!(
            actual_default,
            &EditorPropDefault::Literal(expected_default.clone()),
            "WorkbenchTooltip.{property_name} should preserve its authored tooltip default"
        );
        assert_eq!(
            primitive_root_prop_default(document_id, property_name),
            expected_default,
            "WorkbenchTooltip.{property_name} should match its primitive .zui root prop"
        );
    }
}
