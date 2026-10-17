//! 将平台命令键与常见快捷键归类为剪贴板意图。
//! 本层不接触 Surface 或宿主队列，安全文本、权限与版本检查由调用方继续执行。

use zircon_runtime_interface::ui::dispatch::{UiKeyboardInputEvent, UiKeyboardInputState};

/// 可按值传递的请求类别，不持有文本、选区或异步宿主资源。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::surface::input) enum KeyboardClipboardAction {
    Copy,
    Cut,
    Paste,
}

/// 识别平台语义键与常见剪贴板快捷键，只产生请求意图；权限及异步版本由后续剪贴板入口校验。
///
/// # 参数与返回
/// 借用本次键盘事件，返回独立拥有的类别；未匹配时返回空值。
/// # 调用时机与约束
/// 可编辑路由在文字插入前询问；仅按下态可产生类别，重复与释放态均忽略。
/// # 参考
/// 同级剪贴板派发层消费结果并决定是否形成宿主请求。
pub(in crate::ui::surface::input) fn keyboard_clipboard_action(
    keyboard: &UiKeyboardInputEvent,
) -> Option<KeyboardClipboardAction> {
    if keyboard.state != UiKeyboardInputState::Pressed {
        return None;
    }
    if keyboard.metadata.modifiers.alt {
        return None;
    }

    let logical_key = keyboard.logical_key.as_str();
    if !keyboard.metadata.modifiers.control
        && !keyboard.metadata.modifiers.shift
        && !keyboard.metadata.modifiers.super_key
    {
        match logical_key {
            "Copy" | "copy" => return Some(KeyboardClipboardAction::Copy),
            "Cut" | "cut" => return Some(KeyboardClipboardAction::Cut),
            "Paste" | "paste" => return Some(KeyboardClipboardAction::Paste),
            _ => {}
        }
    }
    if keyboard.metadata.modifiers.shift
        && !keyboard.metadata.modifiers.control
        && !keyboard.metadata.modifiers.super_key
        && (logical_key == "Delete" || keyboard.key_code == 46)
    {
        return Some(KeyboardClipboardAction::Cut);
    }
    if !(keyboard.metadata.modifiers.control || keyboard.metadata.modifiers.super_key) {
        return None;
    }

    if matches!(logical_key, "c" | "C") || keyboard.key_code == 67 {
        return Some(KeyboardClipboardAction::Copy);
    }
    if matches!(logical_key, "x" | "X") || keyboard.key_code == 88 {
        return Some(KeyboardClipboardAction::Cut);
    }
    if matches!(logical_key, "v" | "V") || keyboard.key_code == 86 {
        return Some(KeyboardClipboardAction::Paste);
    }
    None
}
