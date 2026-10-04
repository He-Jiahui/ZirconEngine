//! 键盘事件文本的非拥有 ABI 视图。

use winit::event::KeyEvent;
use zircon_runtime_interface::ZrByteSlice;

/// 视图借用 event 的文本，只适用于 event 存活期间的同步 Runtime 调用。
pub(super) fn keyboard_text_payload(event: &KeyEvent) -> ZrByteSlice {
    event
        .text
        .as_ref()
        .map(|text| ZrByteSlice {
            data: text.as_bytes().as_ptr(),
            len: text.len(),
        })
        .unwrap_or_else(ZrByteSlice::empty)
}
