use super::welcome_surface_binding_control_id;

#[test]
fn welcome_surface_bridge_maps_each_recent_project_action_to_its_typed_control() {
    for (action, control_id) in [
        ("welcome.project.open_recent", "OpenRecentProject"),
        ("welcome.project.safe_recent", "SafeRecentProject"),
        ("welcome.project.recover_recent", "RecoverRecentProject"),
        ("welcome.project.remove_recent", "RemoveRecentProject"),
    ] {
        assert_eq!(welcome_surface_binding_control_id(action), Some(control_id));
    }
}
