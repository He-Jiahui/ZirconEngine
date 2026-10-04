use std::num::NonZeroU64;

use super::*;

#[test]
fn default_runtime_entry_app_config_uses_visible_game_window() {
    let config = RuntimeEntryAppConfig::default();

    assert!(config.window_descriptor.primary_window.is_some());
    assert!(config.window_descriptor.visible);
    assert_eq!(config.event_loop_policy, EventLoopPolicy::Game);
    assert!(config.window_lifecycle_policy.should_close_on_request());
    assert!(config
        .window_lifecycle_policy
        .should_exit_after_primary_close());
    assert!(!config.exit_after_first_presented_frame());
    assert!(!config.reference_cpu_presenter());
}

#[test]
fn runtime_entry_app_config_can_select_absent_primary_window_policy() {
    let config = RuntimeEntryAppConfig::default()
        .with_window_descriptor(WindowDescriptor::default().without_primary_window())
        .with_event_loop_policy(EventLoopPolicy::Headless);

    assert_eq!(config.window_descriptor.primary_window, None);
    assert!(!config.window_descriptor.visible);
    assert_eq!(config.event_loop_policy, EventLoopPolicy::Headless);
}

#[test]
fn runtime_entry_app_config_can_disable_close_when_requested_policy() {
    let config = RuntimeEntryAppConfig::default().with_close_when_requested(false);

    assert!(!config.window_lifecycle_policy.close_when_requested);
    assert!(!config
        .window_lifecycle_policy()
        .should_exit_after_primary_close());
}

#[test]
fn runtime_entry_app_config_can_override_window_lifecycle_policy() {
    let policy = WindowLifecyclePolicy::default().with_close_when_requested(false);
    let config = RuntimeEntryAppConfig::default().with_window_lifecycle_policy(policy);

    assert_eq!(config.window_lifecycle_policy(), policy);
}

#[test]
fn runtime_entry_app_config_can_exit_after_first_presented_frame() {
    let config = RuntimeEntryAppConfig::default().with_exit_after_first_presented_frame(true);

    assert!(config.exit_after_first_presented_frame());
}

#[test]
fn runtime_entry_app_config_can_exit_after_a_requested_presented_frame_count() {
    let limit = NonZeroU64::new(120).unwrap();
    let config = RuntimeEntryAppConfig::default().with_exit_after_presented_frames(limit);

    assert_eq!(config.exit_after_presented_frames(), Some(limit));
    assert!(!config.exit_after_first_presented_frame());
}

#[test]
fn runtime_entry_app_config_can_request_a_first_frame_capture() {
    let path = std::path::PathBuf::from("E:/evidence/runtime-first-frame.png");
    let resolved_path = zircon_runtime::asset::project::ProjectPaths::resolve_path(&path)
        .expect("capture path should resolve");
    let config =
        RuntimeEntryAppConfig::default().with_first_frame_capture_path(Some(resolved_path.clone()));

    assert_eq!(config.first_frame_capture_path(), Some(&resolved_path));
}

#[test]
fn runtime_entry_app_config_requires_persisted_scene_diagnostics_only_when_enabled() {
    assert!(
        !RuntimeEntryAppConfig::default().require_persisted_scene_diagnostics(),
        "the F0 no-project startup path must not require F2 scene diagnostics"
    );

    let config = RuntimeEntryAppConfig::default().with_persisted_scene_diagnostics(true);

    assert!(config.require_persisted_scene_diagnostics());
}

#[test]
fn runtime_entry_app_config_requires_explicit_reference_cpu_presenter_opt_in() {
    let config = RuntimeEntryAppConfig::default().with_reference_cpu_presenter(true);

    assert!(config.reference_cpu_presenter());
}

#[test]
fn runtime_entry_app_config_keeps_native_v2_opt_in_off_by_default() {
    assert!(!RuntimeEntryAppConfig::default().native_ime_composition_requested());
    assert!(RuntimeEntryAppConfig::default()
        .with_native_ime_composition_requested(true)
        .native_ime_composition_requested());
}
