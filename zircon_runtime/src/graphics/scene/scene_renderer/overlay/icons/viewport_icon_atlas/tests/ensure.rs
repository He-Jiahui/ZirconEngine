const SOURCE: &str = include_str!("../ensure.rs");

fn production_source() -> &'static str {
    SOURCE
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("viewport icon ensure source should retain a test-module boundary")
}

#[test]
fn viewport_icon_cache_publishes_pending_before_returning_the_candidate_binding() {
    let source = production_source();
    let prepare = source
        .find("let prepared = prepare_sprite(")
        .expect("viewport icon prepare stage");
    let pending = source
        .find("self.entries[slot] = IconEntry::Pending")
        .expect("viewport icon pending publication");
    let return_binding = source
        .find("Ok(Some(bind_group))")
        .expect("viewport icon candidate binding return");

    assert!(prepare < pending);
    assert!(pending < return_binding);
    assert!(!source.contains("wgpu::Queue"));
    assert!(!source.contains("IconEntry::Ready(sprite.clone())"));
}
