use super::{ApplicationLifecycleMachine, SurfaceReleaseAction};

#[test]
fn surface_admission_is_coalesced_after_one_successful_creation() {
    let mut lifecycle = ApplicationLifecycleMachine::default();

    assert!(lifecycle.surface_creation_requested());
    lifecycle.confirm_surface_created();

    assert!(!lifecycle.surface_creation_requested());
    assert!(lifecycle.allows_frame_pump());
}

#[test]
fn surface_destruction_allows_one_recreation_without_a_second_resume_transition() {
    let mut lifecycle = ApplicationLifecycleMachine::default();
    lifecycle.confirm_surface_created();

    assert_eq!(lifecycle.destroy_surfaces(), SurfaceReleaseAction::Release);
    assert!(lifecycle.surface_creation_requested());
    assert_eq!(lifecycle.destroy_surfaces(), SurfaceReleaseAction::Noop);
}

#[test]
fn suspension_releases_active_surfaces_and_blocks_frame_pumps_until_resumed() {
    let mut lifecycle = ApplicationLifecycleMachine::default();
    lifecycle.confirm_surface_created();

    assert_eq!(lifecycle.suspend(), Some(SurfaceReleaseAction::Release));
    assert!(!lifecycle.allows_frame_pump());
    assert_eq!(lifecycle.suspend(), None);
    assert!(lifecycle.resume());
    assert!(lifecycle.surface_creation_requested());
}
