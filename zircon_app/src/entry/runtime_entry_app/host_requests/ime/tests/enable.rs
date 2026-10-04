use super::{apply_ime_enable_with_request_data, default_ime_request_data};

#[test]
fn default_request_data_is_available_without_a_panic_contract() {
    assert!(default_ime_request_data().is_some());
}

#[test]
fn invalid_default_request_data_skips_window_enable_submission() {
    let mut submitted = false;

    let result = apply_ime_enable_with_request_data(None, |_| {
        submitted = true;
        Ok(())
    });

    assert!(result.is_ok());
    assert!(!submitted);
}
