pub(crate) fn assert_not_available_error(error: impl ToString) {
    assert!(error.to_string().contains("not available"));
}
