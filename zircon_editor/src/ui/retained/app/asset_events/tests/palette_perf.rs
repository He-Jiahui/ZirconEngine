#[test]
fn unchanged_palette_drag_hover_does_not_dirty_presentation() {
    let source = include_str!("../palette.rs");
    let production = source.split("#[cfg(test)]").next().unwrap_or(source);

    assert!(production.contains("Ok(true) => self.mark_presentation_dirty_for_view(&instance_id)"));
    assert!(production.contains("Ok(false) => {}"));
}
