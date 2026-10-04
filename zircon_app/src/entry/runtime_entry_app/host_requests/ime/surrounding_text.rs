//! Runtime 周围文本和光标锚点进入 Winit IME 更新前的有效性边界。

use winit::window::ImeSurroundingText;
use zircon_runtime::diagnostic_log::write_warn;
use zircon_runtime_interface::ZrRuntimeImeSurroundingTextV1;

pub(super) fn default_ime_surrounding_text() -> Option<ImeSurroundingText> {
    ImeSurroundingText::new(String::new(), 0, 0).ok()
}

pub(super) fn runtime_ime_surrounding_text(
    text: ZrRuntimeImeSurroundingTextV1,
) -> Option<ImeSurroundingText> {
    match ImeSurroundingText::new(text.value, text.cursor, text.anchor) {
        Ok(text) => Some(text),
        Err(_) => {
            write_warn("runtime_ime", "runtime_ime_surrounding_text_invalid");
            None
        }
    }
}

#[cfg(test)]
#[path = "tests/surrounding_text.rs"]
mod tests;
