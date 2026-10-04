#[test]
fn route_registration_does_not_clone_the_activity_projection() {
    let source = include_str!("../activity_routes.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    assert!(!implementation.contains("activity.clone()"));
    assert!(implementation.contains("std::mem::take(&mut activity.actions)"));
}
