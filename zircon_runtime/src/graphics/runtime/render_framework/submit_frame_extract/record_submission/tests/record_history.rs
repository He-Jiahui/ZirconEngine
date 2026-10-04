#[test]
fn missing_history_rotation_fails_with_a_typed_framework_error() {
    let production = include_str!("../record_history.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("history record production boundary");

    assert!(production.contains("RenderFrameworkError::InvalidSubmissionState"));
    assert!(!production.contains("unreachable!"));
    assert!(!production.contains("panic!"));
}
