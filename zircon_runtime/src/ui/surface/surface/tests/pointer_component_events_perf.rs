#[test]
fn pointer_component_events_reserve_route_lower_bound() {
    let source = include_str!("../pointer_component_events.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("pointer component event implementation before tests");

    assert!(implementation.contains("let event_capacity = route"));
    assert!(implementation.contains(".saturating_add(route.left.len())"));
    assert!(implementation.contains("let mut events = Vec::with_capacity(event_capacity);"));
}
