use super::support::*;

#[test]
fn editor_ui_host_runtime_projects_builtin_workbench_template_into_slint_projection() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_workbench_shell().unwrap();

    assert_eq!(
        runtime
            .component_descriptor("UiHostWindow")
            .unwrap()
            .binding_namespace,
        "UiHostWindow"
    );

    let projection = runtime
        .project_document(UI_HOST_WINDOW_DOCUMENT_ID)
        .unwrap();

    assert_eq!(projection.document_id, UI_HOST_WINDOW_DOCUMENT_ID);
    assert_eq!(projection.root.component, "UiHostWindow");
    assert_eq!(
        projection
            .root
            .children
            .iter()
            .map(|node| node.component.as_str())
            .collect::<Vec<_>>(),
        vec!["VerticalBox", "Container", "Overlay", "Overlay", "Overlay"]
    );

    let open_project = projection
        .bindings
        .iter()
        .find(|binding| binding.binding_id == "WorkbenchMenuBar/OpenProject")
        .unwrap();
    assert_eq!(open_project.binding.path().event_kind, UiEventKind::Click);
    assert_eq!(open_project.binding.path().view_id, "WorkbenchMenuBar");
    assert_eq!(open_project.binding.path().control_id, "OpenProject");
}

#[test]
fn editor_ui_host_runtime_keeps_legacy_workbench_shell_document_alias() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    let projection = runtime
        .project_document(WORKBENCH_SHELL_DOCUMENT_ID)
        .unwrap();
    assert_eq!(projection.document_id, WORKBENCH_SHELL_DOCUMENT_ID);
    assert_eq!(projection.root.component, "UiHostWindow");

    let surface = runtime
        .build_shared_surface(WORKBENCH_SHELL_DOCUMENT_ID)
        .unwrap();
    assert_eq!(
        surface.tree.tree_id.0,
        format!("template.{WORKBENCH_SHELL_DOCUMENT_ID}")
    );
}

#[test]
fn editor_ui_host_runtime_projects_asset_document_source_into_slint_projection() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime
        .register_document_source("workbench.shell.asset", ASSET_WORKBENCH_DOCUMENT_TOML)
        .unwrap();
    runtime
        .register_binding(
            "WorkbenchMenuBar/OpenProject",
            crate::ui::binding::EditorUiBinding::new(
                "WorkbenchMenuBar",
                "OpenProject",
                crate::ui::binding::EditorUiEventKind::Click,
                crate::ui::binding::EditorUiBindingPayload::menu_action("OpenProject"),
            ),
        )
        .unwrap();
    runtime
        .register_binding(
            "WorkbenchMenuBar/SaveProject",
            crate::ui::binding::EditorUiBinding::new(
                "WorkbenchMenuBar",
                "SaveProject",
                crate::ui::binding::EditorUiEventKind::Click,
                crate::ui::binding::EditorUiBindingPayload::menu_action("SaveProject"),
            ),
        )
        .unwrap();

    let projection = runtime.project_document("workbench.shell.asset").unwrap();

    assert_eq!(projection.document_id, "workbench.shell.asset");
    assert_eq!(projection.root.component, "UiHostWindow");
    assert_eq!(
        projection
            .root
            .children
            .iter()
            .map(|node| node.component.as_str())
            .collect::<Vec<_>>(),
        vec!["UiHostToolbar", "StatusBar"]
    );
}
