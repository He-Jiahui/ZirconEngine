//! 键盘文本输入的私有判定与动作构造层。
//!
//! 可编辑路由依次处理历史、剪贴板、新行、文字负载，再请求编辑动作。
//! 本层借用事件与状态；结果由上层提交到文本状态机、文档历史或剪贴板派发层。
//! 参考同级 `editable_text` 与 `keyboard` 的调用入口；专用文本和 IME 事件使用各自入口。

#[path = "text_keyboard/clipboard.rs"]
mod clipboard;
#[path = "text_keyboard/edit_actions.rs"]
mod edit_actions;
#[path = "text_keyboard/history.rs"]
mod history;
#[path = "text_keyboard/payload.rs"]
mod payload;

pub(super) use clipboard::{keyboard_clipboard_action, KeyboardClipboardAction};
pub(super) use edit_actions::keyboard_text_edit_actions;
pub(super) use history::keyboard_text_history_direction;
pub(super) use payload::{keyboard_requests_newline, keyboard_text_payload};
