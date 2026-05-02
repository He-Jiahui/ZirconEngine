use zircon_runtime_interface::ui::{
    binding::UiEventKind, event_ui::UiActionDescriptor, event_ui::UiParameterDescriptor,
    event_ui::UiValueType,
};

pub(super) fn viewport_actions() -> Vec<UiActionDescriptor> {
    vec![
        UiActionDescriptor::new(
            "pointer_move",
            UiEventKind::Hover,
            "ViewportCommand.PointerMoved",
        )
        .with_parameter(UiParameterDescriptor::new("x", UiValueType::Float))
        .with_parameter(UiParameterDescriptor::new("y", UiValueType::Float)),
        UiActionDescriptor::new(
            "left_press",
            UiEventKind::Press,
            "ViewportCommand.LeftPressed",
        )
        .with_parameter(UiParameterDescriptor::new("x", UiValueType::Float))
        .with_parameter(UiParameterDescriptor::new("y", UiValueType::Float)),
        UiActionDescriptor::new(
            "left_release",
            UiEventKind::Release,
            "ViewportCommand.LeftReleased",
        ),
        UiActionDescriptor::new(
            "right_press",
            UiEventKind::Press,
            "ViewportCommand.RightPressed",
        )
        .with_parameter(UiParameterDescriptor::new("x", UiValueType::Float))
        .with_parameter(UiParameterDescriptor::new("y", UiValueType::Float)),
        UiActionDescriptor::new(
            "right_release",
            UiEventKind::Release,
            "ViewportCommand.RightReleased",
        ),
        UiActionDescriptor::new(
            "middle_press",
            UiEventKind::Press,
            "ViewportCommand.MiddlePressed",
        )
        .with_parameter(UiParameterDescriptor::new("x", UiValueType::Float))
        .with_parameter(UiParameterDescriptor::new("y", UiValueType::Float)),
        UiActionDescriptor::new(
            "middle_release",
            UiEventKind::Release,
            "ViewportCommand.MiddleReleased",
        ),
        UiActionDescriptor::new("scroll", UiEventKind::Scroll, "ViewportCommand.Scrolled")
            .with_parameter(UiParameterDescriptor::new("delta", UiValueType::Float)),
        UiActionDescriptor::new("resize", UiEventKind::Resize, "ViewportCommand.Resized")
            .with_parameter(UiParameterDescriptor::new("width", UiValueType::Unsigned))
            .with_parameter(UiParameterDescriptor::new("height", UiValueType::Unsigned)),
    ]
}
