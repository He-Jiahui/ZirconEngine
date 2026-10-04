use winit::dpi::PhysicalSize;
use zircon_runtime_interface::ui::{
    dispatch::UiWindowId,
    layout::UiSize,
    window::{
        UiWindowEvent, UiWindowEventKind, UiWindowEventMetadata, UiWindowInputContext,
        UiWindowInputPumpEvent, UiWindowMetrics, UiWindowPixelSize, UiWindowPlatformInputEvent,
    },
};

pub(super) fn input_event(event: UiWindowPlatformInputEvent) -> UiWindowInputPumpEvent {
    UiWindowInputPumpEvent::Input(event.normalize())
}

pub(super) fn window_event(
    context: &UiWindowInputContext,
    kind: UiWindowEventKind,
) -> UiWindowInputPumpEvent {
    UiWindowInputPumpEvent::Window(UiWindowEvent::new(window_metadata(context), kind))
}

pub(super) fn window_metadata(context: &UiWindowInputContext) -> UiWindowEventMetadata {
    UiWindowEventMetadata::for_window(
        context
            .metadata
            .window_id
            .clone()
            .unwrap_or_else(UiWindowId::default),
        context.metadata.timestamp,
        context.metadata.sequence,
    )
    .synthetic(context.metadata.synthetic)
}

// resize 只有物理尺寸；沿用上下文的有效 DPI 才能与表面布局的逻辑尺寸保持一致。
// 宿主应随缩放事件更新上下文，否则只能采用未提供 DPI 时的 1:1 回退。
pub(super) fn window_metrics_from_physical_size(
    size: PhysicalSize<u32>,
    prior_metrics: Option<UiWindowMetrics>,
) -> UiWindowMetrics {
    let scale_factor = prior_metrics
        .map(|metrics| metrics.scale_factor)
        .filter(|scale_factor| scale_factor.is_finite() && *scale_factor > 0.0)
        .unwrap_or(1.0);
    UiWindowMetrics::new(
        UiSize::new(
            size.width as f32 / scale_factor as f32,
            size.height as f32 / scale_factor as f32,
        ),
        UiWindowPixelSize::new(size.width, size.height),
        scale_factor,
    )
}
