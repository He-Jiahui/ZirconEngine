use super::{
    legacy_dirty_allows_shell_content_patch, shell_content_reuses_committed_layout,
    window_metrics_reuses_committed_shell, workbench_projection_reuses_host,
};
use crate::ui::retained_host::HostInvalidationMask;

#[test]
fn shell_content_accepts_the_legacy_presentation_compatibility_bit() {
    assert!(shell_content_reuses_committed_layout(
        HostInvalidationMask::SHELL_CONTENT
    ));
    assert!(shell_content_reuses_committed_layout(
        HostInvalidationMask::SHELL_CONTENT.union(HostInvalidationMask::PRESENTATION_DATA)
    ));
}

#[test]
fn workbench_projection_accepts_render_and_paint_without_full_shell_recompute() {
    assert!(workbench_projection_reuses_host(
        HostInvalidationMask::WORKBENCH_PROJECTION
            .union(HostInvalidationMask::PAINT_ONLY)
            .union(HostInvalidationMask::RENDER),
        HostInvalidationMask::RENDER,
    ));
}

#[test]
fn workbench_projection_rejects_global_presentation() {
    assert!(!workbench_projection_reuses_host(
        HostInvalidationMask::WORKBENCH_PROJECTION.union(HostInvalidationMask::PRESENTATION_DATA),
        HostInvalidationMask::PRESENTATION_DATA,
    ));
}

#[test]
fn workbench_projection_rejects_layout_hit_test_and_window_metrics() {
    for incompatible in [
        HostInvalidationMask::LAYOUT,
        HostInvalidationMask::HIT_TEST,
        HostInvalidationMask::WINDOW_METRICS,
    ] {
        assert!(!workbench_projection_reuses_host(
            HostInvalidationMask::WORKBENCH_PROJECTION.union(incompatible),
            incompatible,
        ));
    }
}

#[test]
fn shell_content_rejects_coalesced_layout_or_render_work() {
    assert!(!shell_content_reuses_committed_layout(
        HostInvalidationMask::SHELL_CONTENT.union(HostInvalidationMask::LAYOUT)
    ));
    assert!(!shell_content_reuses_committed_layout(
        HostInvalidationMask::SHELL_CONTENT.union(HostInvalidationMask::RENDER)
    ));
}

#[test]
fn shell_content_accepts_only_the_legacy_presentation_mirror() {
    assert!(legacy_dirty_allows_shell_content_patch(
        HostInvalidationMask::NONE
    ));
    assert!(legacy_dirty_allows_shell_content_patch(
        HostInvalidationMask::PRESENTATION_DATA
    ));
    assert!(!legacy_dirty_allows_shell_content_patch(
        HostInvalidationMask::PRESENTATION_DATA.union(HostInvalidationMask::RENDER)
    ));
}

#[test]
fn pure_window_metrics_reuses_the_committed_shell_stage() {
    assert!(window_metrics_reuses_committed_shell(
        HostInvalidationMask::WINDOW_METRICS,
        HostInvalidationMask::WINDOW_METRICS.union(HostInvalidationMask::PRESENTATION_DATA),
    ));
}

#[test]
fn window_metrics_rejects_coalesced_business_presentation() {
    assert!(!window_metrics_reuses_committed_shell(
        HostInvalidationMask::WINDOW_METRICS.union(HostInvalidationMask::PRESENTATION_DATA),
        HostInvalidationMask::WINDOW_METRICS.union(HostInvalidationMask::PRESENTATION_DATA),
    ));
    assert!(!window_metrics_reuses_committed_shell(
        HostInvalidationMask::WINDOW_METRICS,
        HostInvalidationMask::WINDOW_METRICS.union(HostInvalidationMask::LAYOUT),
    ));
}
