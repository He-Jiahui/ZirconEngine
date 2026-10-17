//! 识别键盘历史方向，实际历史移动由活动文档会话执行。
//! 返回值不借用事件或文档，分类过程不持有跨调用状态。

use zircon_runtime_interface::ui::dispatch::{UiKeyboardInputEvent, UiKeyboardInputState};

use crate::ui::dispatch::UiTextHistoryDirection;

/// 识别文档历史命令，交给活动文档会话提交历史移动；这里只分类，不直接改变 surface 文本。
///
/// # 参数与返回
/// 借用本次键盘事件，返回撤销或重做方向；非命令输入返回空值。
/// # 调用时机与约束
/// 可编辑路由先于普通文本编辑检查；仅按下态、Control 或 Super 且无 Alt 时可匹配。
/// 物理 Z/Y 只在逻辑键为空时回退，避免已知逻辑键被另一布局的键码覆盖。
/// # 参考
/// 同级可编辑路由把方向交给文档历史派发层，不在此读取或修改历史栈。
pub(in crate::ui::surface::input) fn keyboard_text_history_direction(
    keyboard: &UiKeyboardInputEvent,
) -> Option<UiTextHistoryDirection> {
    if keyboard.state != UiKeyboardInputState::Pressed {
        return None;
    }
    let modifiers = &keyboard.metadata.modifiers;
    if (!modifiers.control && !modifiers.super_key) || modifiers.alt {
        return None;
    }
    let logical_key = keyboard.logical_key.as_str();
    let is_z = matches!(logical_key, "z" | "Z")
        || (logical_key.is_empty() && matches!(keyboard.key_code, 90 | 122));
    let is_y = matches!(logical_key, "y" | "Y")
        || (logical_key.is_empty() && matches!(keyboard.key_code, 89 | 121));
    if is_z {
        return Some(if modifiers.shift {
            UiTextHistoryDirection::Redo
        } else {
            UiTextHistoryDirection::Undo
        });
    }
    (is_y && !modifiers.shift).then_some(UiTextHistoryDirection::Redo)
}

#[cfg(test)]
#[path = "tests/history.rs"]
mod tests;
