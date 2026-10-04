#[test]
fn transient_drawer_resize_reuses_the_committed_shell_metrics_stage() {
    let source = include_str!("../movement.rs");
    let update = source
        .split("fn update_drawer_resize_capture")
        .nth(1)
        .and_then(|tail| tail.split("fn finish_drawer_resize_capture").next())
        .expect("drawer resize movement implementation");

    assert!(update.contains("HostInvalidationMask::WINDOW_METRICS"));
    assert!(!update.contains("mark_layout_dirty"));
    assert!(update.contains("if previous_preferred == preferred"));
    assert!(update.contains("return;"));
}
