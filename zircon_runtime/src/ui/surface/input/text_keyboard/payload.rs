//! 区分 Enter 意图与普通键盘携带的文字借用。
//! 多行约束、提交和文本状态变化由同级可编辑路由执行。

use zircon_runtime_interface::ui::dispatch::{UiKeyboardInputEvent, UiKeyboardInputState};

/// 判定应由上层处理换行或单行提交的 Enter 输入。
///
/// # 参数与返回
/// 借用本次事件；按下或重复态且无修饰键的 Enter 返回真。
/// # 调用时机与约束
/// 在文字负载与普通编辑动作之前检查；是否允许多行由调用方的文本约束决定。
pub(in crate::ui::surface::input) fn keyboard_requests_newline(
    keyboard: &UiKeyboardInputEvent,
) -> bool {
    if !matches!(
        keyboard.state,
        UiKeyboardInputState::Pressed | UiKeyboardInputState::Repeated
    ) {
        return false;
    }
    if keyboard.metadata.modifiers.alt
        || keyboard.metadata.modifiers.control
        || keyboard.metadata.modifiers.shift
        || keyboard.metadata.modifiers.super_key
    {
        return false;
    }

    keyboard.logical_key == "Enter" || keyboard.key_code == 13
}

/// 从普通键盘事件借用可提交文字，排除导航 Tab、命令修饰与控制字符。
/// 专用文本/IME事件另走自己的入口，宿主应选择一致的输入源，避免同一文字重复提交。
///
/// # 参数与返回
/// 借用本次事件；成功返回事件自带文本的切片，不分配文本或延长该借用的有效期。
/// # 约束
/// 仅接受按下与重复态；Shift 可保留，Alt、Control、Super 与 Tab 均被排除。
/// 空负载或含任意控制字符时拒绝整个负载，内容提交仍由调用方完成。
pub(in crate::ui::surface::input) fn keyboard_text_payload(
    keyboard: &UiKeyboardInputEvent,
) -> Option<&str> {
    if !matches!(
        keyboard.state,
        UiKeyboardInputState::Pressed | UiKeyboardInputState::Repeated
    ) {
        return None;
    }
    if keyboard.metadata.modifiers.alt
        || keyboard.metadata.modifiers.control
        || keyboard.metadata.modifiers.super_key
    {
        return None;
    }
    if keyboard.logical_key == "Tab" || keyboard.key_code == 9 {
        return None;
    }

    let text = keyboard.text.as_deref()?;
    if text.is_empty() || keyboard_text_contains_control(text) {
        return None;
    }
    Some(text)
}

// ASCII 字节快路与 Unicode 标量回退使用一致的控制字符语义，混合文本不能只扫描字节。
fn keyboard_text_contains_control(text: &str) -> bool {
    if text.is_ascii() {
        return text.as_bytes().iter().any(u8::is_ascii_control);
    }
    text.chars().any(char::is_control)
}

#[cfg(test)]
#[path = "payload/tests/byte_control.rs"]
mod byte_control_tests;
