use super::*;
use zircon_runtime_interface::ui::component::{UiComponentCategory, UiPropSchema, UiValueKind};

#[test]
fn text_input_widget_contract_infers_canonical_descriptor_value_property() {
    for (id, role, properties, expected) in [
        ("SearchField", "search-field", vec!["query"], "query"),
        (
            "InputBase",
            "input-base",
            vec!["value", "value_text"],
            "value",
        ),
        (
            "FieldEditor",
            "field-editor",
            vec!["text", "value_text"],
            "value_text",
        ),
        ("SourceEditor", "source-editor", vec!["text"], "text"),
    ] {
        let descriptor = properties.into_iter().fold(
            UiComponentDescriptor::new(id, id, UiComponentCategory::Input, role),
            |descriptor, property| {
                descriptor.with_prop(UiPropSchema::new(property, UiValueKind::String))
            },
        );

        let widget =
            resolve_component_widget_contract(UiWidgetContract::default(), Some(&descriptor));

        assert_eq!(widget.behavior, UiWidgetBehavior::TextInput);
        assert_eq!(widget.value_property.as_deref(), Some(expected));
    }
}

#[test]
fn authored_text_input_value_property_is_not_replaced() {
    let descriptor = UiComponentDescriptor::new(
        "TextField",
        "TextField",
        UiComponentCategory::Input,
        "text-field",
    )
    .with_prop(UiPropSchema::new("value_text", UiValueKind::String));
    let widget = UiWidgetContract {
        value_property: Some("document_text".to_string()),
        ..UiWidgetContract::default()
    };

    let widget = resolve_component_widget_contract(widget, Some(&descriptor));

    assert_eq!(widget.value_property.as_deref(), Some("document_text"));
}
