use winit::window::ImeSurroundingText;
use zircon_runtime::diagnostic_log::write_warn;
use zircon_runtime_interface::ZrRuntimeImeSurroundingTextV1;

pub(super) fn default_ime_surrounding_text() -> ImeSurroundingText {
    ImeSurroundingText::new(String::new(), 0, 0).expect("empty IME surrounding text is valid")
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
