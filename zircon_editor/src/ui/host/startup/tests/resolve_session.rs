use super::*;

#[test]
fn project_chooser_preserves_first_run_guidance_without_opening_a_project() {
    let session = project_chooser_startup_session(Vec::new());

    assert_eq!(session.mode, EditorSessionMode::Welcome);
    assert!(session.project.is_none());
    assert!(session.open_builtin_view.is_none());
    assert!(session.recent_projects.is_empty());
    assert_eq!(
        session.status_message,
        EditorStartupSessionDocument::default().status_message
    );
    assert!(session.status_message.contains("parent folder"));
    assert!(session.status_message.contains("create"));
    assert!(session.status_message.contains("open"));
    assert!(!session.can_open_existing);
    assert!(!session.creation_validation.is_empty());
}
