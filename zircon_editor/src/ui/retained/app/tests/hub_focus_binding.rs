#[test]
fn retained_host_focus_binding_uses_the_committed_session_identity() {
    let source = include_str!("../hub_focus_binding.rs");

    assert!(source.contains("active_project_session_focus_target()"));
    assert!(source.contains("HubFocusBindingTarget::new"));
    assert!(source.contains("hub_focus_binding\n            .sync"));
}
