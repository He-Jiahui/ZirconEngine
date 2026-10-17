use winit::{
    dpi::PhysicalPosition,
    event::{
        ButtonSource, ElementState, FingerId, MouseButton, MouseScrollDelta, PointerKind,
        PointerSource,
    },
};
use zircon_runtime_interface::ui::{
    dispatch::UiPointerId,
    layout::UiPoint,
    surface::{UiPointerButton, UiPointerEventKind},
    window::{
        UiWindowEventKind, UiWindowInputContext, UiWindowInputPumpEvent, UiWindowPlatformInputEvent,
    },
};

use super::window::{input_event, window_event};

const PIXEL_SCROLL_LINE_DELTA_SCALE: f32 = 0.1;

// 触摸以 finger_id 维护独立会话；鼠标/笔移动先走窗口光标事件，由共享窗口泵补全指针生命周期。
pub(super) fn translate_pointer_moved(
    context: UiWindowInputContext,
    position: PhysicalPosition<f64>,
    source: &PointerSource,
) -> Option<UiWindowInputPumpEvent> {
    let point = point_from_physical_position(position);
    match source {
        PointerSource::Touch { finger_id, .. } => Some(input_event(
            UiWindowPlatformInputEvent::touch_moved(context, pointer_id(*finger_id), point),
        )),
        PointerSource::Mouse | PointerSource::Unknown | PointerSource::TabletTool { .. } => {
            Some(window_event(
                &context,
                UiWindowEventKind::CursorMoved {
                    position: point,
                    delta: None,
                },
            ))
        }
    }
}

// 进入通知不发起触摸接触会话，传入位置因此不用；触摸启动依赖按钮通知。
pub(super) fn translate_pointer_entered(
    context: &UiWindowInputContext,
    _position: PhysicalPosition<f64>,
    kind: &PointerKind,
) -> Option<UiWindowInputPumpEvent> {
    match kind {
        PointerKind::Mouse | PointerKind::Unknown | PointerKind::TabletTool(_) => {
            Some(window_event(context, UiWindowEventKind::CursorEntered))
        }
        PointerKind::Touch(_) => None,
    }
}

// 触摸离开生成取消；缺少离开位置时用零点编码，不读取宿主光标缓存。
pub(super) fn translate_pointer_left(
    context: UiWindowInputContext,
    position: Option<PhysicalPosition<f64>>,
    kind: &PointerKind,
) -> Option<UiWindowInputPumpEvent> {
    let point = position
        .map(point_from_physical_position)
        .unwrap_or_default();
    match kind {
        PointerKind::Touch(finger_id) => Some(input_event(
            UiWindowPlatformInputEvent::touch_canceled(context, pointer_id(*finger_id), point),
        )),
        PointerKind::Mouse | PointerKind::Unknown | PointerKind::TabletTool(_) => {
            Some(window_event(&context, UiWindowEventKind::CursorLeft))
        }
    }
}

// precise_scroll 保留平台横纵轴及单位；标量仅为旧的纵向滚动消费者提供兼容尺度，不能据此恢复像素值。
pub(super) fn translate_mouse_wheel_event(
    context: UiWindowInputContext,
    point: UiPoint,
    delta: MouseScrollDelta,
) -> UiWindowPlatformInputEvent {
    let (scroll_delta, precise_scroll) = match delta {
        MouseScrollDelta::LineDelta(x, y) => (
            y,
            zircon_runtime_interface::ui::dispatch::UiPreciseScrollDelta::lines(x, y),
        ),
        MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => {
            let x = x as f32;
            let y = y as f32;
            (
                y * PIXEL_SCROLL_LINE_DELTA_SCALE,
                zircon_runtime_interface::ui::dispatch::UiPreciseScrollDelta::pixels(x, y),
            )
        }
    };
    UiWindowPlatformInputEvent::pointer(
        context,
        zircon_runtime_interface::ui::dispatch::UiPointerEvent::new(
            UiPointerEventKind::Scroll,
            point,
        )
        .with_scroll_delta(scroll_delta),
        Some(precise_scroll),
    )
}

// 先识别触摸接触，避免降为鼠标主键；其它来源仅保留可映射的三种鼠标键。
pub(super) fn translate_pointer_button(
    context: UiWindowInputContext,
    state: ElementState,
    button: ButtonSource,
    position: PhysicalPosition<f64>,
) -> Option<UiWindowInputPumpEvent> {
    if let Some(finger_id) = touch_button_finger_id(&button) {
        let point = point_from_physical_position(position);
        let input = match state {
            ElementState::Pressed => {
                UiWindowPlatformInputEvent::touch_started(context, finger_id, point)
            }
            ElementState::Released => {
                UiWindowPlatformInputEvent::touch_ended(context, finger_id, point)
            }
        };
        return Some(input_event(input));
    }

    let button = pointer_button(button)?;
    let point = point_from_physical_position(position);
    let input = match state {
        ElementState::Pressed => {
            UiWindowPlatformInputEvent::mouse_button_down(context, button, point)
        }
        ElementState::Released => {
            UiWindowPlatformInputEvent::mouse_button_up(context, button, point)
        }
    };
    Some(input_event(input))
}

fn touch_button_finger_id(button: &ButtonSource) -> Option<UiPointerId> {
    match button {
        ButtonSource::Touch { finger_id, .. } => Some(pointer_id(*finger_id)),
        ButtonSource::Mouse(_) | ButtonSource::TabletTool { .. } | ButtonSource::Unknown(_) => None,
    }
}

// 辅助键或未知按钮无法表示为共享三键枚举，交给上层的空结果路径处理。
fn pointer_button(button: ButtonSource) -> Option<UiPointerButton> {
    match button.mouse_button() {
        Some(MouseButton::Left) => Some(UiPointerButton::Primary),
        Some(MouseButton::Right) => Some(UiPointerButton::Secondary),
        Some(MouseButton::Middle) => Some(UiPointerButton::Middle),
        _ => None,
    }
}

// 复制平台接触标识，不分配全局身份，也不维护接触与窗口之间的映射表。
fn pointer_id(finger_id: FingerId) -> UiPointerId {
    UiPointerId::new(finger_id.into_raw() as u64)
}

// 只窄化数值表示，不按 DPI 缩放或保证有限性；命中坐标约定由宿主与消费方保持一致。
fn point_from_physical_position<T>(position: PhysicalPosition<T>) -> UiPoint
where
    T: Into<f64>,
{
    UiPoint::new(position.x.into() as f32, position.y.into() as f32)
}
