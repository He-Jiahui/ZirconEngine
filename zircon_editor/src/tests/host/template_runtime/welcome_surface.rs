use super::support::*;

#[test]
fn editor_ui_host_runtime_projects_builtin_welcome_surface_template_into_slint_projection() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_workbench_shell().unwrap();

    assert_eq!(
        runtime
            .component_descriptor("WelcomeSurfaceControls")
            .unwrap()
            .binding_namespace,
        "WelcomeSurface"
    );

    let projection = runtime
        .project_document("startup.welcome_controls")
        .unwrap();

    assert_eq!(projection.document_id, "startup.welcome_controls");
    assert_eq!(projection.root.component, "WelcomeSurfaceControls");
    assert_eq!(
        projection
            .root
            .children
            .iter()
            .map(|node| node.control_id.as_deref().unwrap_or_default())
            .collect::<Vec<_>>(),
        vec![
            "ProjectNameEdited",
            "LocationEdited",
            "CreateProject",
            "OpenExistingProject",
            "OpenRecentProject",
            "RemoveRecentProject",
        ]
    );

    let project_name = projection
        .bindings
        .iter()
        .find(|binding| binding.binding_id == "WelcomeSurface/ProjectNameEdited")
        .unwrap();
    assert_eq!(project_name.binding.path().event_kind, UiEventKind::Change);
    assert_eq!(project_name.binding.path().view_id, "WelcomeSurface");
    assert_eq!(project_name.binding.path().control_id, "ProjectNameEdited");

    let remove_recent = projection
        .bindings
        .iter()
        .find(|binding| binding.binding_id == "WelcomeSurface/RemoveRecentProject")
        .unwrap();
    assert_eq!(remove_recent.binding.path().event_kind, UiEventKind::Click);
    assert_eq!(remove_recent.binding.path().view_id, "WelcomeSurface");
    assert_eq!(
        remove_recent.binding.path().control_id,
        "RemoveRecentProject"
    );
}
