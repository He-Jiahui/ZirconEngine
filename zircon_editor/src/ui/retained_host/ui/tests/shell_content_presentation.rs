use super::{
    changed_dock_from_pane_ids, same_shell, shell_mismatch, HostDockSurfaceId,
    ShellContentPatchFallback,
};
use crate::ui::retained_host::host_contract::HostWindowShellData;

#[test]
fn one_changed_pane_selects_one_dock() {
    assert_eq!(
        changed_dock_from_pane_ids(
            "hierarchy",
            "assets",
            "inspector",
            "inspector",
            "console",
            "console"
        ),
        Some(HostDockSurfaceId::Left)
    );
}

#[test]
fn no_change_or_multiple_changes_force_the_full_presentation_path() {
    assert_eq!(
        changed_dock_from_pane_ids("a", "a", "b", "b", "c", "c"),
        None
    );
    assert_eq!(
        changed_dock_from_pane_ids("a", "x", "b", "y", "c", "c"),
        None
    );
}

#[test]
fn shell_content_patch_records_specific_fallback_reasons() {
    let source = include_str!("../shell_content_presentation.rs");
    for counter in [
        "presentation_patch_fallback_shell_count",
        "presentation_patch_fallback_shell_identity_count",
        "presentation_patch_fallback_shell_status_count",
        "presentation_patch_fallback_shell_commands_count",
        "presentation_patch_fallback_shell_presets_count",
        "presentation_patch_fallback_shell_theme_count",
        "presentation_patch_fallback_shell_native_count",
        "presentation_patch_fallback_layout_count",
        "presentation_patch_fallback_status_count",
        "presentation_patch_fallback_document_count",
        "presentation_patch_fallback_dock_cardinality_count",
        "presentation_patch_fallback_hit_index_count",
    ] {
        assert!(
            source.contains(counter),
            "missing fallback counter {counter}"
        );
    }
}

#[test]
fn shell_mismatch_reports_the_changed_ownership_group() {
    let current = HostWindowShellData::default();

    let mut next = current.clone();
    next.project_path = "project".into();
    assert_eq!(
        shell_mismatch(&current, &next),
        Some(ShellContentPatchFallback::ShellIdentity)
    );

    let mut next = current.clone();
    next.status_secondary = "Selected Node".into();
    assert_eq!(
        shell_mismatch(&current, &next),
        Some(ShellContentPatchFallback::ShellStatus)
    );

    let mut next = current.clone();
    next.save_project_enabled = true;
    assert_eq!(
        shell_mismatch(&current, &next),
        Some(ShellContentPatchFallback::ShellCommands)
    );

    let mut next = current.clone();
    next.active_preset_name = "Editing".into();
    assert_eq!(
        shell_mismatch(&current, &next),
        Some(ShellContentPatchFallback::ShellPresets)
    );

    let mut next = current.clone();
    next.skin_id = "dark".into();
    assert_eq!(
        shell_mismatch(&current, &next),
        Some(ShellContentPatchFallback::ShellTheme)
    );

    let mut next = current.clone();
    next.native_window_title = "Floating".into();
    assert_eq!(
        shell_mismatch(&current, &next),
        Some(ShellContentPatchFallback::ShellNative)
    );
}

#[test]
fn native_window_minimums_or_drawer_state_do_not_invalidate_mounted_shell_content() {
    let current = HostWindowShellData::default();
    let mut next = current.clone();
    next.shell_min_width_px = 840.0;
    next.shell_min_height_px = 520.0;

    assert!(same_shell(&current, &next));

    next.left_expanded = true;
    next.drawers_visible = true;
    assert!(same_shell(&current, &next));

    next.debug_refresh_rate = "present 42 | pixels 4096 | slow 0 | render 0 | paint-only 42".into();
    assert!(same_shell(&current, &next));

    next.viewport_label = "Scene".into();
    assert!(!same_shell(&current, &next));
}
