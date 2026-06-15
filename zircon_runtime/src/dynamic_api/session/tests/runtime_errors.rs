use super::super::runtime_session_error;

#[test]
fn runtime_session_error_preserves_step_when_inner_error_is_empty() {
    assert_eq!(
        runtime_session_error("load default level", ""),
        "load default level failed without additional diagnostics"
    );
    assert_eq!(
        runtime_session_error("load default level", "scene asset missing"),
        "load default level: scene asset missing"
    );
}
