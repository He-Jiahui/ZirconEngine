#[test]
fn close_refuses_to_clear_a_session_while_recovery_work_is_active() {
    let source = include_str!("../editor_manager_project.rs");
    let close = source
        .find("pub(crate) fn begin_project_close")
        .expect("project close owner should exist");
    let recovery_gate = source[close..]
        .find("self.ensure_project_recovery_is_settled()?;")
        .map(|offset| close + offset)
        .expect("project close should check recovery state");
    let begin_close = source[close..]
        .find("self.begin_project_close_operation()")
        .map(|offset| close + offset)
        .expect("project close should begin the durable close phase");

    assert!(recovery_gate < begin_close);
}
