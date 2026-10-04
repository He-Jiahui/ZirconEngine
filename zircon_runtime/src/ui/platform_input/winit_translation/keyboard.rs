use winit::event::KeyEvent;
use winit::keyboard::ModifiersState;
use zircon_runtime_interface::ui::{
    dispatch::UiInputModifiers,
    window::{UiWindowInputContext, UiWindowPlatformInputEvent},
};

use super::super::keyboard_map::{
    dom_key_code, keyboard_state, logical_key_name, native_scan_code, physical_key_name,
};

/// 宿主在修饰键通知后更新环境状态，再把结果放入后续事件上下文。
/// winit 的此载荷不含锁定键状态，caps_lock/num_lock 的 false 不能视作对系统锁定状态的查询。
pub fn translate_winit_modifiers(state: ModifiersState) -> UiInputModifiers {
    UiInputModifiers {
        shift: state.shift_key(),
        control: state.control_key(),
        alt: state.alt_key(),
        super_key: state.meta_key(),
        caps_lock: false,
        num_lock: false,
    }
}

// 同时保留物理键、逻辑键和文本：快捷键匹配与文字输入的身份来源不同，不能从 key_code 反推文本。
pub(super) fn translate_keyboard_event(
    context: UiWindowInputContext,
    event: &KeyEvent,
    synthetic: bool,
) -> UiWindowPlatformInputEvent {
    let mut context = context;
    context.metadata.synthetic = synthetic;

    UiWindowPlatformInputEvent::keyboard(
        context,
        keyboard_state(event.state, event.repeat),
        dom_key_code(&event.logical_key),
        native_scan_code(event.physical_key),
        physical_key_name(event.physical_key),
        logical_key_name(&event.logical_key),
        event.text.as_ref().map(ToString::to_string),
    )
}
