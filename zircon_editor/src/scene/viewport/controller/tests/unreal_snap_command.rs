use super::*;
use crate::core::settings::{SettingsMutationCoordinator, SettingsMutationError};
use std::path::PathBuf;
use std::sync::Arc;

// Reuse the production coordinator and Project binding. The native validation
// runner owns TEMP/TMP under the approved drive-root cargo-targets directory.
struct BoundSnapProject {
    root: PathBuf,
    coordinator: Arc<SettingsMutationCoordinator>,
}

impl BoundSnapProject {
    fn new() -> Self {
        static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zircon-unreal-snap-project-{}-{time}-{id}",
            std::process::id()
        ));
        std::fs::create_dir_all(root.join("assets/scenes")).unwrap();
        std::fs::write(
            root.join("assets/scenes/main.scene.toml"),
            "entities = []\n",
        )
        .unwrap();
        let coordinator = Arc::new(SettingsMutationCoordinator::in_memory_with_defaults());
        coordinator.bind_project(&root).unwrap();
        Self { root, coordinator }
    }
}

impl Drop for BoundSnapProject {
    fn drop(&mut self) {
        // Drain/cancel the owning persistence lane before deleting its test root.
        self.coordinator.shutdown().wait();
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
use zircon_runtime_interface::math::UVec2;

#[test]
fn unreal_snap_toggle_is_global_editor_pref_and_grid_visibility_keeps_snap_and_step() {
    let project = BoundSnapProject::new();
    let prefs = Arc::clone(&project.coordinator);
    let mut first = SceneViewportController::with_settings_coordinator(
        UVec2::new(1280, 800),
        Arc::clone(&prefs),
    );
    let second = SceneViewportController::with_settings_coordinator(UVec2::new(900, 620), prefs);
    assert!(!second.chrome_settings().scale_snap_enabled);
    first
        .apply_command(None, &ViewportCommand::ToggleSnap(TransformHandleKind::Scale))
        .unwrap();
    first
        .apply_command(None, &ViewportCommand::SetTranslateSnap(2.0))
        .unwrap();
    first
        .apply_command(None, &ViewportCommand::SetRotateSnapDegrees(30.0))
        .unwrap();
    first
        .apply_command(None, &ViewportCommand::SetScaleSnap(0.25))
        .unwrap();
    assert!(
        first
            .apply_command(
                None,
                &ViewportCommand::ToggleSnap(TransformHandleKind::Move)
            )
            .unwrap()
            .settings_changed
    );
    let second_state = second.chrome_settings();
    assert!(!second_state.translate_snap_enabled);
    assert!(second_state.rotate_snap_enabled && second_state.scale_snap_enabled);
    first
        .apply_command(None, &ViewportCommand::SetGridVisibility(false))
        .unwrap();
    assert_eq!(first.settings().grid_mode, GridMode::Hidden);
    let hidden = first.chrome_settings();
    assert!(
        !hidden.translate_snap_enabled && hidden.rotate_snap_enabled && hidden.scale_snap_enabled
    );
    assert_eq!(
        (
            hidden.translate_step,
            hidden.rotate_step_deg,
            hidden.scale_step
        ),
        (2.0, 30.0, 0.25)
    );
    first
        .apply_command(None, &ViewportCommand::SetGridVisibility(true))
        .unwrap();
    assert_eq!(first.settings().grid_mode, GridMode::VisibleNoSnap);
    first
        .apply_command(
            None,
            &ViewportCommand::ToggleSnap(TransformHandleKind::Rotate),
        )
        .unwrap();
    first
        .apply_command(
            None,
            &ViewportCommand::ToggleSnap(TransformHandleKind::Scale),
        )
        .unwrap();
    let state = first.chrome_settings();
    assert!(
        !state.translate_snap_enabled && !state.rotate_snap_enabled && !state.scale_snap_enabled
    );
    assert_eq!(
        (
            state.translate_step,
            state.rotate_step_deg,
            state.scale_step
        ),
        (2.0, 30.0, 0.25)
    );
}

#[test]
fn unreal_legacy_combined_grid_command_keeps_its_all_snap_meaning() {
    let project = BoundSnapProject::new();
    let mut controller = SceneViewportController::with_settings_coordinator(
        UVec2::new(1280, 800),
        Arc::clone(&project.coordinator),
    );
    for (mode, enabled) in [
        (GridMode::VisibleNoSnap, false),
        (GridMode::VisibleAndSnap, true),
        (GridMode::Hidden, false),
    ] {
        controller
            .apply_command(None, &ViewportCommand::SetGridMode(mode))
            .unwrap();
        let snap = controller.chrome_settings();
        assert_eq!(controller.settings().grid_mode, mode);
        assert_eq!(
            (
                snap.translate_snap_enabled,
                snap.rotate_snap_enabled,
                snap.scale_snap_enabled
            ),
            (enabled, enabled, enabled)
        );
    }
}

#[test]
fn unreal_snap_toggle_rejects_unbound_project_without_mutating_shared_state() {
    let coordinator = Arc::new(SettingsMutationCoordinator::in_memory_with_defaults());
    let mut controller = SceneViewportController::with_settings_coordinator(
        UVec2::new(1280, 800),
        Arc::clone(&coordinator),
    );
    let before = controller.chrome_settings();
    let generation = coordinator.authority().snapshot().generation();
    let error = controller
        .apply_command(
            None,
            &ViewportCommand::ToggleSnap(TransformHandleKind::Move),
        )
        .unwrap_err();
    assert_eq!(
        error,
        SceneViewportControllerError::SettingsMutation(SettingsMutationError::ProjectNotBound)
    );
    assert_eq!(controller.chrome_settings(), before);
    assert_eq!(coordinator.authority().snapshot().generation(), generation);
    coordinator.shutdown().wait();
}
