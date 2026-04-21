use zircon_runtime::ui::{
    binding::UiEventKind, event_ui::UiActionDescriptor, event_ui::UiParameterDescriptor,
    event_ui::UiValueType,
};

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
        UiActionDescriptor::new(
            "create_animation_track",
            UiEventKind::Click,
            "AnimationCommand.CreateTrack",
        )
        .with_parameter(UiParameterDescriptor::new(
            "track_path",
            UiValueType::String,
        )),
    ]
}
