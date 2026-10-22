use super::EditorStartupSessionDocument;

#[test]
fn first_run_explains_both_project_actions_and_preserves_real_errors() {
    let mut session = EditorStartupSessionDocument::default();
    let snapshot = session.welcome_pane_snapshot(false);
    assert!(snapshot.recent_projects.is_empty());
    assert!(snapshot.status_message.contains("parent folder"));
    assert!(snapshot.status_message.contains("create"));
    assert!(snapshot.status_message.contains("open"));
    session.status_message = "Project manifest could not be read".to_string();
    assert_eq!(
        session.welcome_pane_snapshot(false).status_message,
        session.status_message
    );
}

#[test]
fn a_valid_new_project_keeps_create_enabled_and_explains_disabled_open() {
    let session = EditorStartupSessionDocument {
        creation_validation: String::new(),
        can_open_existing: false,
        ..Default::default()
    };

    let form = session.welcome_pane_snapshot(false).form;

    assert!(form.can_create);
    assert!(!form.can_open_existing);
    assert!(form.validation_message.contains("No existing project"));
    assert!(form.validation_message.contains("parent folder and name"));
    assert!(session.creation_validation.is_empty());
}

#[test]
fn an_existing_project_keeps_open_enabled_when_creation_is_unavailable() {
    let session = EditorStartupSessionDocument {
        creation_validation: "The project directory already exists".to_string(),
        can_open_existing: true,
        ..Default::default()
    };

    let form = session.welcome_pane_snapshot(false).form;

    assert!(!form.can_create);
    assert!(form.can_open_existing);
    assert_eq!(form.validation_message, session.creation_validation);
}

#[test]
fn an_unfinished_probe_keeps_both_actions_disabled_and_shows_its_status() {
    let session = EditorStartupSessionDocument::default();

    let form = session.welcome_pane_snapshot(false).form;

    assert!(!form.can_create);
    assert!(!form.can_open_existing);
    assert_eq!(form.validation_message, session.creation_validation);
}
