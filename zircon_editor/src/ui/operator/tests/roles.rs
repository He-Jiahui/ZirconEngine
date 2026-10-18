use super::*;

fn project_bindings() -> Vec<OperatorTemplateBinding> {
    serde_json::from_value(serde_json::json!([
        {"role":"project.browser.root", "control_id":"node-a"},
        {"role":"project.browser.projects", "control_id":"node-b", "slot":"recent.projects"},
        {"role":"project.browser.open", "control_id":"node-c", "command":"file.project.open"}
    ])).unwrap()
}

#[test]
fn shipped_roles_only_reference_registered_commands() {
    OperatorRoleContract::builtin().unwrap().validate_commands(&EditorCommandRegistry::default_workbench()).unwrap();
}

#[test]
fn unknown_command_in_a_reviewed_contract_is_rejected() {
    let source = include_str!("../../../../assets/ui/operator/contract/roles.contract.toml")
        .replace("file.project.open", "missing.command");
    let contract = OperatorRoleContract::from_toml(&source).unwrap();
    assert!(contract.validate_commands(&EditorCommandRegistry::default_workbench()).is_err());
}

#[test]
fn designer_node_renaming_preserves_role_semantics() {
    let mut bindings = project_bindings();
    bindings[2].control_id = "arbitrary-new-design-id".into();
    let index = OperatorRoleBindings::validate(&OperatorRoleContract::builtin().unwrap(),
        &EditorCommandRegistry::default_workbench(), "project", bindings).unwrap();
    let open = index.binding("project.browser.open").unwrap();
    assert_eq!(open.control_id, "arbitrary-new-design-id");
    assert_eq!(open.command.as_deref(), Some("file.project.open"));
}

#[test]
fn missing_duplicate_unknown_and_mismatched_roles_are_errors() {
    let contract = OperatorRoleContract::builtin().unwrap();
    let commands = EditorCommandRegistry::default_workbench();
    let mut missing = project_bindings();
    missing.pop();
    assert_eq!(contract.missing_required_roles("project", &missing), ["project.browser.open"]);
    assert!(OperatorRoleBindings::validate(&contract, &commands, "project", missing).is_err());
    let mut duplicate = project_bindings();
    duplicate.push(duplicate[2].clone());
    assert!(OperatorRoleBindings::validate(&contract, &commands, "project", duplicate).is_err());
    let mut unknown = project_bindings();
    unknown.push(OperatorTemplateBinding { role: OperatorRole("project.unknown.action".into()),
        control_id: "unknown".into(), command: Some("fake.command".into()), slot: None });
    assert!(OperatorRoleBindings::validate(&contract, &commands, "project", unknown).is_err());
    let mut mismatch = project_bindings();
    mismatch[2].command = Some("runtime.play_mode.enter".into());
    assert!(OperatorRoleBindings::validate(&contract, &commands, "project", mismatch).is_err());
}
