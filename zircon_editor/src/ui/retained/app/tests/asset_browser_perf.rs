#[test]
fn asset_details_scroll_reuses_the_committed_browser_projection() {
    let source = include_str!("../detail_scroll_pointer/asset_browser.rs");
    let production = source.split("#[cfg(test)]").next().unwrap_or(source);

    assert!(!production.contains("self.runtime.editor_snapshot()"));
}
