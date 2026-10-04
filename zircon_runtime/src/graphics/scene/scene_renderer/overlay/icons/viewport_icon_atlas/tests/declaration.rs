const SOURCE: &str = include_str!("../declaration.rs");

fn production_source() -> &'static str {
    SOURCE
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("viewport icon atlas source should retain a test-module boundary")
}

#[test]
fn viewport_icon_upload_debt_is_replayed_then_committed_from_fixed_slots() {
    let source = production_source();

    assert!(source.contains("entries: vec![IconEntry::Unloaded; 2]"));
    assert!(source.contains("for entry in &self.entries"));
    assert!(source.contains("texture_uploads.push(upload.clone())"));
    assert!(source.contains("IconEntry::Pending { sprite, .. }"));
    assert!(source.contains("*entry = IconEntry::Ready(sprite)"));
}
