#[test]
fn selected_camera_history_key_borrows_the_existing_descriptor() {
    let source = include_str!("../camera_history_key.rs");

    assert!(source.contains("if let Some(descriptor)"));
    assert!(!source.contains(concat!("selected_camera_descriptor()", ".", "cloned()")));
}
