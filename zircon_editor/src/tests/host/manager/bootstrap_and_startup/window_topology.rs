use super::*;

#[test]
fn opening_functional_editor_window_creates_instance_scoped_floating_window() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_functional_window_open");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let instance_id = manager
        .open_view(ViewDescriptorId::new("editor.material_editor_window"), None)
        .unwrap();

    assert_eq!(
        instance_id,
        ViewInstanceId::new("editor.material_editor_window#1")
    );
    let window_id = MainPageId::new("window:editor.material_editor_window#1");
    let layout = manager.current_layout();
    let floating = layout
        .floating_windows
        .iter()
        .find(|window| window.window_id == window_id)
        .expect("material editor should open in a floating window");
    assert_eq!(floating.focused_view, Some(instance_id.clone()));
    assert!(floating.workspace.contains(&instance_id));
    let native_host = manager
        .native_window_hosts()
        .into_iter()
        .find(|host| host.window_id == window_id)
        .expect("floating editor window should own a native host state");
    assert_eq!(
        native_host.surface_tree_id.0,
        "zircon.editor.native_window.window:editor.material_editor_window#1"
    );
    assert_eq!(
        manager
            .current_view_instances()
            .into_iter()
            .find(|instance| instance.instance_id == instance_id)
            .map(|instance| instance.host),
        Some(ViewHost::FloatingWindow(window_id, vec![]))
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn opening_drawer_backed_windows_creates_distinct_exclusive_pages() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_exclusive_window_open");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let asset_browser = manager
        .open_view(ViewDescriptorId::new("editor.asset_browser_window"), None)
        .unwrap();
    let diagnostics = manager
        .open_view(ViewDescriptorId::new("editor.diagnostics_window"), None)
        .unwrap();

    let asset_page = MainPageId::new("page:editor.asset_browser_window#1");
    let diagnostics_page = MainPageId::new("page:editor.diagnostics_window#1");
    let layout = manager.current_layout();
    assert!(
        layout
            .main_pages
            .iter()
            .any(|page| page.id() == &asset_page)
    );
    assert!(
        layout
            .main_pages
            .iter()
            .any(|page| page.id() == &diagnostics_page)
    );
    assert_eq!(layout.active_main_page, diagnostics_page);
    let instances = manager.current_view_instances();
    assert_eq!(
        instances
            .iter()
            .find(|instance| instance.instance_id == asset_browser)
            .map(|instance| instance.host.clone()),
        Some(ViewHost::ExclusivePage(asset_page))
    );
    assert_eq!(
        instances
            .iter()
            .find(|instance| instance.instance_id == diagnostics)
            .map(|instance| instance.host.clone()),
        Some(ViewHost::ExclusivePage(diagnostics_page))
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn scene_and_game_tabs_are_not_closeable() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_non_closeable_docs");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    assert!(
        !manager
            .close_view(&ViewInstanceId::new("editor.scene#1"))
            .unwrap()
    );
    assert!(
        !manager
            .close_view(&ViewInstanceId::new("editor.game#1"))
            .unwrap()
    );
    assert!(
        manager
            .current_view_instances()
            .iter()
            .any(|instance| instance.instance_id.0 == "editor.scene#1")
    );
    assert!(
        manager
            .current_view_instances()
            .iter()
            .any(|instance| instance.instance_id.0 == "editor.game#1")
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn editor_manager_registers_animation_document_view_descriptors() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_animation_view_descriptors");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    let descriptor_ids = manager
        .descriptors()
        .into_iter()
        .map(|descriptor| descriptor.descriptor_id)
        .collect::<Vec<_>>();

    assert!(descriptor_ids.contains(&ViewDescriptorId::new("editor.animation_sequence")));
    assert!(descriptor_ids.contains(&ViewDescriptorId::new("editor.animation_graph")));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
}
