#[test]
fn production_focus_path_is_fail_closed_without_legacy_window_synthesis() {
    let source = include_str!("../focus.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("production focus source");

    assert!(!production.contains(".expect("));
    assert!(!production.contains("activity_windows.is_empty()"));
    assert!(!production.contains("default_activity_window_mut()"));
}
