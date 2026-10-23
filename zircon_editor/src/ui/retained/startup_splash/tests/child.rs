use super::*;

#[test]
fn progress_does_not_regress_and_only_ready_or_close_dismisses_the_card() {
    let mut state = SplashState::default();
    state.receive("gpu");
    state.receive("runtime");
    state.receive("unknown");
    assert_eq!(state.stage, EditorStartupStage::GpuInit);
    assert!(!state.closed);
    state.receive("project");
    assert!(!state.closed);
    state.receive("ready");
    assert!(state.closed);
    assert_eq!(state.stage, EditorStartupStage::Ready);
    state.receive("runtime");
    assert_eq!(state.stage, EditorStartupStage::Ready);
}

#[test]
fn failed_startup_can_close_without_reporting_ready() {
    let mut state = SplashState::default();
    state.receive("close");
    assert!(state.closed);
    assert_eq!(state.stage, EditorStartupStage::RuntimeLoad);
}
