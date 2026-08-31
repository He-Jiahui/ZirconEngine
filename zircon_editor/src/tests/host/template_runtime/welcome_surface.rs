use super::support::*;

#[test]
fn editor_ui_host_runtime_projects_builtin_welcome_surface_template_into_retained_projection() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    assert_eq!(
        runtime
            .component_descriptor("WelcomeSurfaceControls")
            .unwrap()
            .binding_namespace,
        "WelcomeSurface"
    );

    let projection = runtime
        .project_document("res://ui/editor/host/startup_welcome_controls.zui")
        .unwrap();

    assert_eq!(
        projection.document_id,
        "res://ui/editor/host/startup_welcome_controls.zui"
    );
    assert_eq!(projection.root.component, "VerticalGroup");
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
            "WelcomePrimaryActions",
            "WelcomeRecentActions",
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

    let recover_recent = projection
        .bindings
        .iter()
        .find(|binding| binding.binding_id == "WelcomeSurface/RecoverRecentProject")
        .unwrap();
    assert_eq!(recover_recent.binding.path().event_kind, UiEventKind::Click);
    assert_eq!(
        recover_recent.binding.path().control_id,
        "RecoverRecentProject"
    );

    let safe_recent = projection
        .bindings
        .iter()
        .find(|binding| binding.binding_id == "WelcomeSurface/SafeRecentProject")
        .unwrap();
    assert_eq!(safe_recent.binding.path().event_kind, UiEventKind::Click);
    assert_eq!(safe_recent.binding.path().control_id, "SafeRecentProject");
}

#[test]
fn welcome_surface_actions_fit_the_available_drawer_width() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/ui/editor/host/startup_welcome_controls.zui"),
    )
    .expect("welcome surface controls template should be readable");

    for control_id in ["CreateProject", "OpenExistingProject", "OpenRecentProject"] {
        let control_start = source
            .find(&format!("control_id = \"{control_id}\""))
            .expect("welcome action must remain in the template");
        let control = &source[control_start..];
        let layout_end = control
            .find("\nevents =")
            .expect("welcome action must declare an event after its layout");
        assert!(
            control[..layout_end].contains("width = { stretch = \"Stretch\" }"),
            "{control_id} must share the available action-row width"
        );
    }

    assert!(
        source.contains(
            "control_id = \"RemoveRecentProject\"\nprops = { label = \"Remove\", icon = \"editor_pages/workbench/tabs/close-tab.svg\", icon_placement = \"icon_only\""
        ),
        "remove-recent must remain an accessible icon action instead of consuming a second text-button width"
    );
    assert!(
        source.contains(
            "control_id = \"RecoverRecentProject\"\nprops = { label = \"Recover\", icon = \"editor_pages/workbench/menu/undo-history.svg\", icon_placement = \"icon_only\""
        ),
        "recover-recent must remain an accessible icon action instead of widening recent rows"
    );
    assert!(
        source.contains(
            "control_id = \"SafeRecentProject\"\nprops = { label = \"Safe Mode\", icon = \"editor_pages/hierarchy/row_state/warning-state.svg\", icon_placement = \"icon_only\""
        ),
        "safe-mode must remain an accessible icon action instead of widening recent rows"
    );
    assert!(
        source.contains("width = { min = \"$editor.control.height.dense\", preferred = \"$editor.control.height.dense\", max = \"$editor.control.height.dense\", stretch = \"Fixed\" }"),
        "the remove action must use the shared square dense-control size"
    );
    assert!(
        !source.contains("preferred = 116.0") && !source.contains("preferred = 92.0"),
        "welcome action rows must not retain fixed button widths"
    );
}
