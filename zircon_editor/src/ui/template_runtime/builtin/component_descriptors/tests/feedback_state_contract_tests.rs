use super::{builtin_component_descriptors_for_tests, primitive_root_prop_default};
use crate::ui::template::{EditorPropDefault, EditorPropLiteral};

#[test]
fn builtin_catalog_exposes_drag_and_loading_feedback_state_parameters() {
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
            "WorkbenchDragOverlay",
            "open",
            "boolean",
            EditorPropLiteral::Boolean(true),
        ),
        (
            "WorkbenchDragOverlay",
            "dragging",
            "boolean",
            EditorPropLiteral::Boolean(true),
        ),
        (
            "WorkbenchDragOverlay",
            "drop_hovered",
            "boolean",
            EditorPropLiteral::Boolean(true),
        ),
        (
            "WorkbenchDragOverlay",
            "active_drag_target",
            "boolean",
            EditorPropLiteral::Boolean(true),
        ),
        (
            "WorkbenchDragOverlay",
            "payload_kind",
            "enum",
            EditorPropLiteral::Text("asset".to_string()),
        ),
        (
            "WorkbenchDragOverlay",
            "payload_label",
            "text",
            EditorPropLiteral::Text("StoneWall.mesh".to_string()),
        ),
        (
            "WorkbenchDragOverlay",
            "payload_reference",
            "text",
            EditorPropLiteral::Text("assets/stone_wall.mesh".to_string()),
        ),
        (
            "WorkbenchDragOverlay",
            "drop_allowed",
            "boolean",
            EditorPropLiteral::Boolean(true),
        ),
        (
            "WorkbenchDragOverlay",
            "drop_indicator_text",
            "text",
            EditorPropLiteral::Text("Drop into scene".to_string()),
        ),
        (
            "WorkbenchSkeleton",
            "variant",
            "enum",
            EditorPropLiteral::Text("rounded".to_string()),
        ),
        (
            "WorkbenchSkeleton",
            "animation",
            "enum",
            EditorPropLiteral::Text("pulse".to_string()),
        ),
        (
            "WorkbenchSkeleton",
            "loading",
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
            "{component_id}.{property_name} should preserve its authored feedback state default"
        );
        assert_eq!(
            primitive_root_prop_default(document_id, property_name),
            expected_default,
            "{component_id}.{property_name} should match its primitive .zui root prop"
        );
    }
}
