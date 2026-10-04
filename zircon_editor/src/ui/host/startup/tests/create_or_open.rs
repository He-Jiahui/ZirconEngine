use super::project_activation_action;

#[test]
fn opened_project_is_not_reopened_just_to_update_recents() {
    let source = include_str!("../create_or_open.rs");
    let reopening_call = ["self", ".update_recent_project(&document.root_path)"].concat();
    assert!(!source.contains(&reopening_call));
}

#[test]
fn degraded_project_inputs_are_visible_in_the_startup_status_action() {
    assert_eq!(
        project_activation_action("Project opened", 4, 4, 0, "persisted-v1"),
        "Project opened"
    );
    assert_eq!(
        project_activation_action("Project opened", 4, 3, 0, "persisted-v1"),
        "Project opened (degraded)"
    );
    assert_eq!(
        project_activation_action("Restored recent project", 4, 4, 0, "degraded-missing"),
        "Restored recent project (degraded)"
    );
}
