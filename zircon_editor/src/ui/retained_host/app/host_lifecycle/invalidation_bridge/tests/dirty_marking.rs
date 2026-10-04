#[test]
fn pane_presentation_uses_shell_content_invalidation_with_no_global_mask() {
    let source = include_str!("../dirty_marking.rs");
    let function = source
        .split("fn mark_presentation_dirty_for_pane")
        .nth(1)
        .and_then(|body| body.split("fn mark_render_and_presentation_dirty").next())
        .expect("pane presentation invalidation implementation");

    assert!(function.contains("shell_content_scope_for_pane(pane_id)"));
    assert!(function.contains("HostInvalidationMask::SHELL_CONTENT"));
    assert!(!function.contains("HostInvalidationMask::PRESENTATION_DATA"));
    assert!(!function.contains("self.invalidate_host("));
}
