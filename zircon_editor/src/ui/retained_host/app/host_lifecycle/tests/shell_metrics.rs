#[test]
fn shell_size_sync_reads_committed_event_metrics_without_a_trailing_gate() {
    let source = include_str!("../shell_metrics.rs");
    let function = source
        .split("pub(in crate::ui::retained_host::app) fn sync_shell_size")
        .nth(1)
        .and_then(|body| {
            body.split("pub(in crate::ui::retained_host::app) fn publish")
                .next()
        })
        .expect("sync_shell_size implementation");
    let bootstrap = function
        .find("self.ui.get_host_window_bootstrap()")
        .expect("shell-size sync should read committed window metrics");

    let invalidate = function
        .find("self.invalidate_host(HostInvalidationMask::WINDOW_METRICS)")
        .expect("changed metrics should invalidate the geometry transaction");
    assert!(bootstrap < invalidate);
    assert!(!function.contains("native_resize_reflow_pending"));
}

#[test]
fn shell_metric_sync_projects_the_effective_root_scale_into_the_paint_snapshot() {
    let source = include_str!("../shell_metrics.rs");
    let function = source
        .split("pub(in crate::ui::retained_host::app) fn sync_shell_size")
        .nth(1)
        .and_then(|body| {
            body.split("pub(in crate::ui::retained_host::app) fn publish")
                .next()
        })
        .expect("sync_shell_size implementation");

    assert!(function.contains("ResolutionContext::from_physical_size_with_scale_mode"));
    assert!(function.contains("next_effective_scale"));
    assert!(function.contains("apply_host_paint_scale_factor(next_effective_scale)"));
    assert!(!function.contains("apply_host_paint_scale_factor(next_scale_factor)"));
}

#[test]
fn shell_metric_sync_keeps_the_transaction_resize_specific() {
    let source = include_str!("../shell_metrics.rs");
    let function = source
        .split("pub(in crate::ui::retained_host::app) fn sync_shell_size")
        .nth(1)
        .and_then(|body| {
            body.split("pub(in crate::ui::retained_host::app) fn publish")
                .next()
        })
        .expect("sync_shell_size implementation");

    assert!(function.contains("invalidate_host(HostInvalidationMask::WINDOW_METRICS)"));
    assert!(!function.contains("union(HostInvalidationMask::PRESENTATION_DATA)"));
}
