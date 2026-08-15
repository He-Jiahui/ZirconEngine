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
mod tests {
    use super::default_ime_surrounding_text;

    #[test]
    fn default_surrounding_text_is_available_without_a_panic_contract() {
        assert!(default_ime_surrounding_text().is_some());
    }
}
