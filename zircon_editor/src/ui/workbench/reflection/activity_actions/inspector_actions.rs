use zircon_ui::{UiActionDescriptor, UiEventKind, UiParameterDescriptor, UiValueType};

pub(super) fn inspector_actions() -> Vec<UiActionDescriptor> {
    vec![
        UiActionDescriptor::new("apply_batch", UiEventKind::Click, "InspectorFieldBatch")
            .with_parameter(UiParameterDescriptor::new(
                "subject_path",
                UiValueType::String,
            ))
            .with_parameter(UiParameterDescriptor::new("changes", UiValueType::Array)),
        UiActionDescriptor::new(
            "edit_field",
            UiEventKind::Change,
            "DraftCommand.SetInspectorField",
        )
        .with_parameter(UiParameterDescriptor::new(
            "subject_path",
            UiValueType::String,
        ))
        .with_parameter(UiParameterDescriptor::new("field_id", UiValueType::String))
        .with_parameter(UiParameterDescriptor::new("value", UiValueType::String)),
    ]
}
