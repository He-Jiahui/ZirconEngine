use zircon_runtime_interface::ui::{
    binding::UiEventKind, event_ui::UiActionDescriptor, event_ui::UiParameterDescriptor,
    event_ui::UiValueType,
};

pub(super) fn inspector_actions() -> Vec<UiActionDescriptor> {
    vec![
        UiActionDescriptor::new(
            "inspector.apply_batch.invoke",
            UiEventKind::Click,
            "InspectorFieldBatch",
        )
        .with_parameter(UiParameterDescriptor::new(
            "subject_path",
            UiValueType::String,
        ))
        .with_parameter(UiParameterDescriptor::new("changes", UiValueType::Array)),
        UiActionDescriptor::new(
            "inspector.field.edit",
            UiEventKind::Change,
            "DraftCommand.SetInspectorField",
        )
        .with_parameter(UiParameterDescriptor::new(
            "subject_path",
            UiValueType::String,
        ))
        .with_parameter(UiParameterDescriptor::new("field_id", UiValueType::String))
        .with_parameter(UiParameterDescriptor::new("value", UiValueType::String)),
        UiActionDescriptor::new(
            "animation.track.create",
            UiEventKind::Click,
            "AnimationCommand.CreateTrack",
        )
        .with_parameter(UiParameterDescriptor::new(
            "track_path",
            UiValueType::String,
        )),
    ]
}
