use std::error::Error as StdError;

use crate::ui::dispatch::UiInputMethodSurroundingTextError;

fn assert_std_error<T: StdError + Send + Sync + 'static>() {}

#[test]
fn ui_input_method_surrounding_text_error_is_std_error() {
    assert_std_error::<UiInputMethodSurroundingTextError>();

    let error = UiInputMethodSurroundingTextError::CursorBadPosition;
    assert_eq!(error.to_string(), "cursor byte is not a UTF-8 boundary");
    assert!(error.source().is_none());
}
