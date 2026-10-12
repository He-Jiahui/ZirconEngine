#[test]
fn autosave_admission_is_fenced_while_project_recovery_remains_active() {
    let source = include_str!("../autosave.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("autosave source should contain its production section");
    let recovery_gate = production
        .find("self.editor_manager.project_recovery_is_active()")
        .expect("autosave must read the manager recovery lifecycle");
    let poll = production
        .find(".autosave()\n            .poll_project")
        .expect("autosave service polling should remain after the recovery gate");

    assert!(recovery_gate < poll);
}
