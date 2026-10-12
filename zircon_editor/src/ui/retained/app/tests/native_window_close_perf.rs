#[test]
fn main_window_close_reuses_the_view_instance_snapshot() {
    let source = include_str!("../native_window_close.rs");
    let production = source.split("#[cfg(test)]").next().expect("implementation");
    let snapshot_call = ["current_view_", "instances()"].concat();

    assert_eq!(production.matches(&snapshot_call).count(), 1);
    assert!(production.contains("all_dirty_close_views(&dirty_documents)"));
}
