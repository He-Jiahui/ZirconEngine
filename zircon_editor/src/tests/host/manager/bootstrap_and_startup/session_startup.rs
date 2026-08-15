use super::*;

#[test]
fn startup_session_defaults_to_component_showcase_without_recent_project() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_startup_welcome");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let session = manager.resolve_startup_session().unwrap();

    assert_eq!(session.mode, EditorSessionMode::Welcome);
    assert_eq!(
        session.open_builtin_view.as_deref(),
        Some("editor.ui_component_showcase")
    );
    assert!(session.project.is_none());
    assert!(session.recent_projects.is_empty());
    assert_eq!(session.draft.project_name, "ZirconProject");
    assert_eq!(session.draft.template, NewProjectTemplate::RenderableEmpty);
    assert_eq!(session.status_message, "Opened UI Component Showcase");

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn create_project_and_open_persists_recent_project_and_returns_project_session() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_startup_recent");
    let location = unique_temp_dir("zircon_editor_welcome_recent");
    fs::create_dir_all(&location).unwrap();
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let draft = NewProjectDraft {
        project_name: "RecentProject".to_string(),
        location: location.to_string_lossy().into_owned(),
        template: NewProjectTemplate::RenderableEmpty,
    };

    let opened = manager.create_project_and_open(draft).unwrap();
    let recent = manager.recent_projects_snapshot().unwrap();
    let default_startup = manager.resolve_startup_session().unwrap();

    assert_eq!(opened.mode, EditorSessionMode::Project);
    assert!(opened.project.is_some());
    assert!(opened.open_builtin_view.is_none());
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].summary.name, "RecentProject");
    assert_eq!(recent[0].validation, RecentProjectValidation::Valid);
    assert_eq!(default_startup.mode, EditorSessionMode::Project);
    assert!(default_startup.project.is_some());
    assert!(default_startup.open_builtin_view.is_none());
    assert_eq!(default_startup.recent_projects.len(), 1);
    assert_eq!(
        default_startup.recent_projects[0].validation,
        RecentProjectValidation::Valid
    );
    assert!(
        default_startup
            .status_message
            .contains("Restored recent project")
    );
    assert!(
        default_startup
            .status_message
            .contains("RecentProject (scene=")
    );
    assert!(
        opened
            .status_message
            .contains("scene=res://scenes/main.scene.toml")
    );
    assert!(
        default_startup
            .status_message
            .contains("scene=res://scenes/main.scene.toml")
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(location);
}

#[test]
fn restored_legacy_manifest_recent_path_is_migrated_to_the_project_root() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_startup_manifest_recent_migration");
    let project_root = unique_temp_dir("zircon_editor_startup_manifest_recent_project");
    create_project_with_default_world(&project_root);
    let manifest_path = project_root.join("zircon-project.toml");
    let manifest_path_string = manifest_path.to_string_lossy().into_owned();
    let summary = ProjectAuthority::default()
        .probe_project(&manifest_path)
        .unwrap()
        .summary()
        .clone();
    let runtime = editor_runtime_with_config_path(&path);
    let resolver = ManagerResolver::new(runtime.handle());
    let config = resolver.resolve(resolver.config_handle().unwrap()).unwrap();
    let legacy_session = StoredStartupSession {
        last_project_path: Some(manifest_path_string.clone()),
        recent_projects: vec![StoredRecentProjectEntry {
            summary,
            path: manifest_path_string,
            last_opened_unix_ms: 42,
        }],
    };
    config
        .set_value(
            "editor.startup.session",
            serde_json::to_value(&legacy_session).unwrap(),
        )
        .unwrap();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let restored = manager.resolve_startup_session().unwrap();
    let saved = ProjectAuthority::default()
        .decode_startup_session(serde_json::Value::from(
            config
                .get_value("editor.startup.session")
                .expect("restored session must be saved"),
        ))
        .unwrap();
    let project_root = project_root.to_string_lossy().into_owned();

    assert_eq!(restored.mode, EditorSessionMode::Project);
    assert_eq!(restored.recent_projects.len(), 1);
    assert_eq!(restored.recent_projects[0].path, project_root);
    assert_eq!(
        saved.last_project_path.as_deref(),
        Some(project_root.as_str())
    );
    assert_eq!(saved.recent_projects.len(), 1);
    assert_eq!(saved.recent_projects[0].path, project_root);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn explicit_project_open_session_bypasses_component_showcase_builtin_view() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_startup_project_open");
    let project_root = unique_temp_dir("zircon_editor_explicit_project_open");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    create_project_with_default_world(&project_root);

    let opened = manager.open_project_and_remember(&project_root).unwrap();

    assert_eq!(opened.mode, EditorSessionMode::Project);
    assert!(opened.project.is_some());
    assert!(opened.open_builtin_view.is_none());
    assert!(opened.status_message.starts_with("Project opened:"));
    assert!(opened.status_message.contains("assets="));
    assert_eq!(opened.recent_projects.len(), 1);
    assert_eq!(
        opened.recent_projects[0].validation,
        RecentProjectValidation::Valid
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn project_open_applies_completed_plugin_manifest_to_the_manager() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_project_plugin_manifest");
    let project_root = unique_temp_dir("zircon_editor_project_plugin_manifest");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    create_project_with_default_world(&project_root);

    let opened = manager.open_project_and_remember(&project_root).unwrap();
    let plugin_source = manager.plugin_panel_source();
    let plugin_rows = plugin_source.rows().collect::<Vec<_>>();

    assert!(opened.project.is_some());
    assert!(!plugin_rows.is_empty());
    assert!(
        plugin_rows
            .iter()
            .all(|row| row.state() == EditorPluginState::Disabled),
        "the default project manifest must disable every completed editor package"
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn startup_session_falls_back_to_welcome_when_last_project_is_missing() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_startup_missing_recent");
    let project_root = unique_temp_dir("zircon_editor_missing_recent_project");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    create_project_with_default_world(&project_root);
    manager.open_project_and_remember(&project_root).unwrap();
    fs::remove_dir_all(&project_root).unwrap();

    let session = manager.resolve_startup_session().unwrap();

    assert_eq!(session.mode, EditorSessionMode::Welcome);
    assert_eq!(
        session.open_builtin_view.as_deref(),
        Some("editor.ui_component_showcase")
    );
    assert!(session.project.is_none());
    assert_eq!(session.recent_projects.len(), 1);
    assert_eq!(
        session.recent_projects[0].validation,
        RecentProjectValidation::Missing
    );
    assert!(
        session
            .status_message
            .contains("Could not restore last project")
    );
    assert!(
        session
            .status_message
            .contains("Opened UI Component Showcase")
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn project_open_with_corrupt_workspace_falls_back_to_global_layout_with_diagnostic() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_corrupt_workspace_fallback");
    let project_root = unique_temp_dir("zircon_editor_corrupt_workspace_project");
    let runtime = editor_runtime_with_config_path(&path);
    let resolver = ManagerResolver::new(runtime.handle());
    let config = resolver.resolve(resolver.config_handle().unwrap()).unwrap();
    let custom_layout = empty_layout_with_page("global-layout");
    config
        .set_value(
            "editor.workbench.default_layout",
            serde_json::to_value(&custom_layout).unwrap(),
        )
        .unwrap();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    create_project_with_default_world(&project_root);
    let workspace_path = project_root.join(".zircon").join("editor-workspace.json");
    fs::create_dir_all(workspace_path.parent().unwrap()).unwrap();
    fs::write(&workspace_path, "{ this is not valid workspace json").unwrap();

    let opened = manager.open_project_and_remember(&project_root).unwrap();
    let project = opened.project.as_ref().expect("project should still open");

    assert!(project.editor_workspace.is_none());
    assert_eq!(project.workspace_restore_diagnostics.len(), 1);
    assert!(
        opened
            .status_message
            .contains("Project opened with default layout")
    );
    assert!(opened.status_message.contains("editor-workspace.json"));

    manager
        .apply_project_workspace(project.editor_workspace.clone())
        .unwrap();
    assert_eq!(
        manager.current_layout().active_main_page,
        MainPageId::new("global-layout")
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}
