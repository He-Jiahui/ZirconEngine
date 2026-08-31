use zircon_runtime_interface::ui::{
    binding::UiEventKind, event_ui::UiActionDescriptor, event_ui::UiParameterDescriptor,
    event_ui::UiValueType,
};

pub(super) const COMMON_TAB_ACTION_COUNT: usize = 2;

pub(super) fn common_tab_actions() -> [UiActionDescriptor; COMMON_TAB_ACTION_COUNT] {
    [
        UiActionDescriptor::new(
            "workbench.view.focus",
            UiEventKind::Click,
            "DockCommand.FocusView",
        )
        .with_parameter(UiParameterDescriptor::new(
            "instance_id",
            UiValueType::String,
        )),
        UiActionDescriptor::new(
            "workbench.view.detach_to_window",
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
