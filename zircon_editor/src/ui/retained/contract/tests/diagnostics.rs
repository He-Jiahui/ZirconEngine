use super::*;

#[test]
fn diagnostics_overlay_text_changes_after_two_recorded_presents() {
    let mut diagnostics = HostRefreshDiagnostics::default();
    diagnostics.record_present(120, true, false);
    let first = diagnostics.overlay_text();

    diagnostics.record_present(48, false, true);
    let second = diagnostics.overlay_text();

    assert_ne!(first, second);
    assert!(second.contains("present 2"));
    assert!(second.contains("pixels 168"));
}

#[test]
fn diagnostics_region_present_increments_region_not_full_count() {
    let mut diagnostics = HostRefreshDiagnostics::default();

    diagnostics.record_present(42, false, true);

    assert_eq!(diagnostics.region_paint_count, 1);
    assert_eq!(diagnostics.full_paint_count, 0);
}

#[test]
fn diagnostics_full_present_increments_full_count() {
    let mut diagnostics = HostRefreshDiagnostics::default();

    diagnostics.record_present(42, true, false);

    assert_eq!(diagnostics.full_paint_count, 1);
    assert_eq!(diagnostics.region_paint_count, 0);
}

#[test]
fn diagnostics_overlay_includes_invalidation_paint_only_and_render_counts() {
    let mut diagnostics = HostRefreshDiagnostics::default();
    diagnostics.record_present(64, true, false);
    let text = diagnostics.with_invalidation_counts(2, 3, 4).overlay_text();

    assert!(text.contains("slow 2"));
    assert!(text.contains("render 3"));
    assert!(text.contains("paint-only 4"));
}

#[test]
fn startup_overlay_matches_live_overlay_field_shape() {
    assert_eq!(
        HostRefreshDiagnostics::default().overlay_text(),
        STARTUP_REFRESH_DIAGNOSTICS_OVERLAY
    );
    assert!(STARTUP_REFRESH_DIAGNOSTICS_OVERLAY.contains("pixels 0"));
    assert!(STARTUP_REFRESH_DIAGNOSTICS_OVERLAY.contains("slow 0"));
    assert!(STARTUP_REFRESH_DIAGNOSTICS_OVERLAY.contains("render 0"));
    assert!(STARTUP_REFRESH_DIAGNOSTICS_OVERLAY.contains("paint-only 0"));
}
