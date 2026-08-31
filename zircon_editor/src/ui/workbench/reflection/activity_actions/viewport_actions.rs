use zircon_runtime_interface::ui::{
    binding::UiEventKind, event_ui::UiActionDescriptor, event_ui::UiParameterDescriptor,
    event_ui::UiValueType,
};

pub(super) const VIEWPORT_ACTION_COUNT: usize = 9;

pub(super) fn viewport_actions() -> [UiActionDescriptor; VIEWPORT_ACTION_COUNT] {
    [
        UiActionDescriptor::new(
            "workbench.viewport.pointer.move",
            UiEventKind::Hover,
            "ViewportCommand.PointerMoved",
        )
        .with_parameter(UiParameterDescriptor::new("x", UiValueType::Float))
        .with_parameter(UiParameterDescriptor::new("y", UiValueType::Float)),
        UiActionDescriptor::new(
            "workbench.viewport.pointer.left.press",
            UiEventKind::Press,
            "ViewportCommand.LeftPressed",
        )
        .with_parameter(UiParameterDescriptor::new("x", UiValueType::Float))
        .with_parameter(UiParameterDescriptor::new("y", UiValueType::Float)),
        UiActionDescriptor::new(
            "workbench.viewport.pointer.left.release",
            UiEventKind::Release,
            "ViewportCommand.LeftReleased",
        ),
        UiActionDescriptor::new(
            "workbench.viewport.pointer.right.press",
            UiEventKind::Press,
            "ViewportCommand.RightPressed",
        )
        .with_parameter(UiParameterDescriptor::new("x", UiValueType::Float))
        .with_parameter(UiParameterDescriptor::new("y", UiValueType::Float)),
        UiActionDescriptor::new(
            "workbench.viewport.pointer.right.release",
            UiEventKind::Release,
            "ViewportCommand.RightReleased",
        ),
        UiActionDescriptor::new(
            "workbench.viewport.pointer.middle.press",
            UiEventKind::Press,
            "ViewportCommand.MiddlePressed",
        )
        .with_parameter(UiParameterDescriptor::new("x", UiValueType::Float))
        .with_parameter(UiParameterDescriptor::new("y", UiValueType::Float)),
        UiActionDescriptor::new(
            "workbench.viewport.pointer.middle.release",
            UiEventKind::Release,
            "ViewportCommand.MiddleReleased",
        ),
        UiActionDescriptor::new(
            "workbench.viewport.scroll",
            UiEventKind::Scroll,
            "ViewportCommand.Scrolled",
        )
        .with_parameter(UiParameterDescriptor::new("delta", UiValueType::Float)),
        UiActionDescriptor::new(
            "workbench.viewport.resize",
            UiEventKind::Resize,
            "ViewportCommand.Resized",
        )
        .with_parameter(UiParameterDescriptor::new("width", UiValueType::Unsigned))
        .with_parameter(UiParameterDescriptor::new("height", UiValueType::Unsigned)),
    ]
}
