use super::*;

#[test]
fn editor_manager_bootstrap_prefers_global_default_layout() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_workbench_global");
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

    assert_eq!(
        manager.current_layout().active_main_page,
        custom_layout.active_main_page
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
}

#[test]
fn editor_manager_bootstrap_repairs_empty_global_default_layout() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_workbench_global_empty");
    let runtime = editor_runtime_with_config_path(&path);
    let resolver = ManagerResolver::new(runtime.handle());
    let config = resolver.resolve(resolver.config_handle().unwrap()).unwrap();
    let empty_layout = empty_layout_with_page("global-layout");
    config
        .set_value(
            "editor.workbench.default_layout",
            serde_json::to_value(&empty_layout).unwrap(),
        )
        .unwrap();

    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let layout = manager.current_layout();

    assert_eq!(layout.active_main_page, MainPageId::new("global-layout"));

    let left_top = layout
        .drawers
        .get(&ActivityDrawerSlot::LeftTop)
        .expect("left top drawer");
    assert_eq!(
        left_top.tab_stack.tabs,
        vec![
            ViewInstanceId::new("editor.hierarchy#1"),
            ViewInstanceId::new("editor.assets#1"),
        ]
    );
    assert_eq!(
        left_top.active_view,
        Some(ViewInstanceId::new("editor.hierarchy#1"))
    );

    let right_top = layout
        .drawers
        .get(&ActivityDrawerSlot::RightTop)
        .expect("right top drawer");
    assert_eq!(
        right_top.tab_stack.tabs,
        vec![ViewInstanceId::new("editor.inspector#1")]
    );

    let bottom = layout
        .drawers
        .get(&ActivityDrawerSlot::Bottom)
        .expect("bottom drawer");
    assert_eq!(
        bottom.tab_stack.tabs,
        vec![
            ViewInstanceId::new("editor.console#1"),
            ViewInstanceId::new("editor.runtime_diagnostics#1"),
            ViewInstanceId::new("editor.build_export_desktop#1"),
        ]
    );

    let workbench_page = layout
        .main_pages
        .iter()
        .find_map(|page| match page {
            MainHostPageLayout::WorkbenchPage {
                document_workspace, ..
            } => Some(document_workspace),
            MainHostPageLayout::ExclusiveActivityWindowPage { .. } => None,
        })
        .expect("workbench page");
    let DocumentNode::Tabs(document_tabs) = workbench_page else {
        panic!("expected root document tabs");
    };
    assert_eq!(
        document_tabs.tabs,
        vec![
            ViewInstanceId::new("editor.scene#1"),
            ViewInstanceId::new("editor.game#1"),
        ]
    );
    assert_eq!(
        document_tabs.active_tab,
        Some(ViewInstanceId::new("editor.scene#1"))
    );

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(path);
}
