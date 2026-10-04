use super::default_ime_surrounding_text;

#[test]
fn default_surrounding_text_is_available_without_a_panic_contract() {
    assert!(default_ime_surrounding_text().is_some());
}
