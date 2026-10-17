#[path = "winit_translation/ime.rs"]
mod ime;
#[path = "winit_translation/keyboard.rs"]
mod keyboard;
#[path = "winit_translation/pointer.rs"]
mod pointer;
#[path = "winit_translation/window.rs"]
mod window;

use winit::event::WindowEvent;
use zircon_runtime_interface::ui::{
    layout::UiPoint,
    window::{
        UiWindowEventKind, UiWindowInputContext, UiWindowInputPumpEvent, UiWindowPixelPosition,
        UiWindowRedrawReason,
    },
};

use ime::translate_ime_event;
use keyboard::translate_keyboard_event;
pub use keyboard::translate_winit_modifiers;
use pointer::{
    translate_mouse_wheel_event, translate_pointer_button, translate_pointer_entered,
    translate_pointer_left, translate_pointer_moved,
};
use window::{input_event, window_event, window_metrics_from_physical_size};

/// 将宿主事件交给共享窗口输入泵；宿主须提供当前窗口指标、修饰键及有序元数据。
/// 此处只解释平台载荷，不更新表面状态；返回 None 表示该事件仍由宿主处理。
///
/// # 调用时机与参数
/// 由宿主收到窗口通知时调用；`context` 是本次身份、修饰键及缓存的拥有快照。
/// `event` 仅在调用期间借用；需要跨越此调用的文本会复制到返回载荷。
///
/// # 返回与生命周期
/// 返回拥有载荷的窗口泵事件，不借用平台通知或适配器状态。
/// 空值也包含无对应共享输入的能力通知、触摸进入通知或无法表示的按钮。
///
/// # 约束与参考
/// 不保存缓存或推进序号；这些状态由宿主为后续调用维护。
/// 滚轮无位置时使用光标缓存，缺省为零点；指针位置不在此按 DPI 缩放。
/// 缩放通知不会就地更新环境，后续尺寸通知须由宿主提供新的度量快照。
/// 参见 [`UiWindowInputContext`] 与 [`UiWindowInputPumpEvent`]。
pub fn translate_winit_window_event(
    context: UiWindowInputContext,
    event: &WindowEvent,
) -> Option<UiWindowInputPumpEvent> {
    match event {
        WindowEvent::CloseRequested => {
            Some(window_event(&context, UiWindowEventKind::CloseRequested))
        }
        WindowEvent::SurfaceResized(size) => Some(window_event(
            &context,
            UiWindowEventKind::Resized {
                metrics: window_metrics_from_physical_size(*size, context.window_metrics),
            },
        )),
        WindowEvent::ScaleFactorChanged { scale_factor, .. } => Some(window_event(
            &context,
            UiWindowEventKind::ScaleFactorChanged {
                scale_factor: *scale_factor,
            },
        )),
        WindowEvent::Moved(position) => Some(window_event(
            &context,
            UiWindowEventKind::Moved {
                position: UiWindowPixelPosition::new(position.x, position.y),
            },
        )),
        WindowEvent::PointerMoved {
            position, source, ..
        } => translate_pointer_moved(context, *position, source),
        WindowEvent::PointerEntered { position, kind, .. } => {
            translate_pointer_entered(&context, *position, kind)
        }
        WindowEvent::PointerLeft { position, kind, .. } => {
            translate_pointer_left(context, *position, kind)
        }
        WindowEvent::PointerButton {
            state,
            button,
            position,
            ..
        } => translate_pointer_button(context, *state, button.clone(), *position),
        WindowEvent::KeyboardInput {
            event,
            is_synthetic,
            ..
        } => Some(input_event(translate_keyboard_event(
            context,
            event,
            *is_synthetic,
        ))),
        WindowEvent::Ime(event) => translate_ime_event(context, event),
        WindowEvent::MouseWheel { delta, .. } => {
            // Wheel events carry no cursor position. Use the last known position so the
            // event routes to the widget currently under the pointer instead of always
            // hitting the origin.
            let point = context.last_cursor_position.unwrap_or_default();
            Some(input_event(translate_mouse_wheel_event(
                context, point, *delta,
            )))
        }
        WindowEvent::RedrawRequested => Some(window_event(
            &context,
            UiWindowEventKind::RequestRedraw {
                reason: UiWindowRedrawReason::Host,
            },
        )),
        WindowEvent::Focused(focused) => Some(window_event(
            &context,
            UiWindowEventKind::Focused { focused: *focused },
        )),
        WindowEvent::Occluded(occluded) => Some(window_event(
            &context,
            UiWindowEventKind::Occluded {
                occluded: *occluded,
            },
        )),
        _ => None,
    }
}

#[cfg(test)]
#[path = "tests/winit_translation.rs"]
mod tests;
