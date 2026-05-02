use zircon_runtime_interface::ui::{
    binding::UiEventKind, event_ui::UiActionDescriptor, event_ui::UiParameterDescriptor,
    event_ui::UiValueType,
};

pub(super) fn common_tab_actions() -> Vec<UiActionDescriptor> {
    vec![
        UiActionDescriptor::new("focus_view", UiEventKind::Click, "DockCommand.FocusView")
            .with_parameter(UiParameterDescriptor::new(
                "instance_id",
                UiValueType::String,
            )),
        UiActionDescriptor::new(
            "detach_to_window",
            UiEventKind::Click,
            "DockCommand.DetachViewToWindow",
        )
        .with_parameter(UiParameterDescriptor::new(
            "instance_id",
            UiValueType::String,
        ))
        .with_parameter(UiParameterDescriptor::new("window_id", UiValueType::String)),
    ]
}
