#[test]
fn table_selection_prefers_scoped_shell_content_with_explicit_full_fallback() {
    let source = include_str!("../click.rs");
    let function = source
        .split("fn dispatch_template_table_row_selected")
        .nth(1)
        .and_then(|body| {
            body.split("fn dispatch_pane_surface_control_clicked")
                .next()
        })
        .expect("table selection dispatch implementation");

    assert!(function.contains("mark_presentation_dirty_for_pane(pane_id)"));
    assert!(function.contains("self.mark_presentation_dirty()"));
    assert!(
        function.find("mark_presentation_dirty_for_pane").unwrap()
            < function.find("mark_presentation_dirty()").unwrap()
    );
}
