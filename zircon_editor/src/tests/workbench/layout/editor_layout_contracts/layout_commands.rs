use super::*;

#[test]
fn jetbrains_docking_state_commands_drive_drawer_split_and_active_contracts(
) -> Result<(), LayoutCommandError> {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();
    let project = ViewInstanceId::new("editor.project#jetbrains-contract");
    let scene = ViewInstanceId::new("editor.scene#jetbrains-contract");
    let material = ViewInstanceId::new("editor.material#jetbrains-contract");

    manager.apply(
        &mut layout,
        LayoutCommand::AttachView {
            instance_id: project.clone(),
            target: ViewHost::Drawer(ActivityDrawerSlot::LeftBottom),
            anchor: None,
        },
    )?;
    manager.apply(
        &mut layout,
        LayoutCommand::SetDrawerMode {
            slot: ActivityDrawerSlot::LeftBottom,
            mode: ActivityDrawerMode::Collapsed,
        },
    )?;

    let Some(collapsed_drawer) = layout.drawers.get(&ActivityDrawerSlot::LeftBottom) else {
        panic!("expected left-bottom drawer");
    };
    assert_eq!(collapsed_drawer.mode, ActivityDrawerMode::Collapsed);
    assert_eq!(collapsed_drawer.tab_stack.tabs, vec![project.clone()]);
    assert_eq!(collapsed_drawer.active_view, None);

    manager.apply(
        &mut layout,
        LayoutCommand::ActivateDrawerTab {
            slot: ActivityDrawerSlot::LeftBottom,
            instance_id: project.clone(),
        },
    )?;
    let Some(active_drawer) = layout.drawers.get(&ActivityDrawerSlot::LeftBottom) else {
        panic!("expected active left-bottom drawer");
    };
    assert_eq!(active_drawer.mode, ActivityDrawerMode::Pinned);
    assert_eq!(active_drawer.tab_stack.active_tab.as_ref(), Some(&project));
    assert_eq!(active_drawer.active_view.as_ref(), Some(&project));

    manager.apply(
        &mut layout,
        LayoutCommand::OpenView {
            instance_id: scene.clone(),
            target: ViewHost::Document(MainPageId::workbench(), vec![]),
        },
    )?;
    manager.apply(
        &mut layout,
        LayoutCommand::CreateSplit {
            workspace: WorkspaceTarget::MainPage(MainPageId::workbench()),
            path: vec![],
            axis: SplitAxis::Horizontal,
            placement: SplitPlacement::After,
            new_instance: material.clone(),
        },
    )?;

    let MainHostPageLayout::WorkbenchPage {
        document_workspace, ..
    } = &layout.main_pages[0]
    else {
        panic!("expected workbench page");
    };
    let DocumentNode::SplitNode {
        axis,
        ratio,
        first,
        second,
    } = document_workspace
    else {
        panic!("expected split document root");
    };
    assert_eq!(*axis, SplitAxis::Horizontal);
    assert_eq!(*ratio, 0.5);

    let DocumentNode::Tabs(first_tabs) = first.as_ref() else {
        panic!("expected first split tab stack");
    };
    let DocumentNode::Tabs(second_tabs) = second.as_ref() else {
        panic!("expected second split tab stack");
    };
    assert_eq!(first_tabs.tabs, vec![scene.clone()]);
    assert_eq!(second_tabs.tabs, vec![material.clone()]);

    manager.apply(
        &mut layout,
        LayoutCommand::FocusView {
            instance_id: scene.clone(),
        },
    )?;
    let MainHostPageLayout::WorkbenchPage {
        document_workspace, ..
    } = &layout.main_pages[0]
    else {
        panic!("expected workbench page");
    };
    let DocumentNode::SplitNode { first, .. } = document_workspace else {
        panic!("expected split document root");
    };
    let DocumentNode::Tabs(first_tabs) = first.as_ref() else {
        panic!("expected first split tab stack");
    };
    assert_eq!(first_tabs.active_tab.as_ref(), Some(&scene));

    Ok(())
}

#[test]
fn layout_command_failures_are_typed_for_docking_contract_errors() {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();
    let missing = ViewInstanceId::new("editor.missing#typed-error");

    let missing_tab = match manager.apply(
        &mut layout,
        LayoutCommand::ActivateDrawerTab {
            slot: ActivityDrawerSlot::LeftBottom,
            instance_id: missing.clone(),
        },
    ) {
        Ok(_) => panic!("activating a drawer tab outside the drawer must fail"),
        Err(error) => error,
    };
    assert_eq!(
        missing_tab,
        LayoutCommandError::DrawerMissingTab {
            slot: ActivityDrawerSlot::LeftBottom,
            instance_id: missing
        }
    );

    let non_split = match manager.apply(
        &mut layout,
        LayoutCommand::ResizeSplit {
            workspace: WorkspaceTarget::MainPage(MainPageId::workbench()),
            path: vec![],
            ratio: 0.7,
        },
    ) {
        Ok(_) => panic!("resizing a tab node must fail"),
        Err(error) => error,
    };
    assert_eq!(
        non_split,
        LayoutCommandError::TargetPathIsNotSplitNode {
            workspace: WorkspaceTarget::MainPage(MainPageId::workbench()),
            path: Vec::new()
        }
    );
}

#[test]
fn built_in_layout_presets_match_authoring_review_focus_debug_contracts() {
    let presets = LayoutPreset::builtin_presets();

    assert_eq!(
        presets.iter().map(|preset| preset.name).collect::<Vec<_>>(),
        vec![
            LayoutPresetName::Authoring,
            LayoutPresetName::Review,
            LayoutPresetName::Focus,
            LayoutPresetName::Debug,
        ]
    );
    assert!(presets
        .iter()
        .find(|preset| preset.name == LayoutPresetName::Focus)
        .unwrap()
        .drawer_states
        .iter()
        .all(|state| state.mode == ActivityDrawerMode::Collapsed));
    assert!(presets
        .iter()
        .find(|preset| preset.name == LayoutPresetName::Debug)
        .unwrap()
        .size_overrides
        .iter()
        .any(|override_value| override_value.token.as_str() == "--bottom-output-height"));
}

#[test]
fn page_templates_bind_core_pages_to_the_shared_skeleton_regions() {
    let scene = PageLayoutTemplate::scene();
    let material = PageLayoutTemplate::material();
    let inspector = PageLayoutTemplate::inspector();

    assert_eq!(scene.default_preset, LayoutPresetName::Authoring);
    assert!(scene.has_region_role(EditorRegion::RightTop, EditorRegionRole::HierarchyStructure));
    assert!(scene.has_region_role(EditorRegion::RightBottom, EditorRegionRole::DetailInspector));
    assert!(material.has_region_role(EditorRegion::Center, EditorRegionRole::CenterDocument));
    assert!(inspector.has_region_role(EditorRegion::Center, EditorRegionRole::CenterDocument));
}

#[test]
fn floating_window_declarations_preserve_modal_and_layer_contracts() {
    let command_palette = FloatingWindow::command_palette();
    let preferences = FloatingWindow::preferences();

    assert_eq!(command_palette.kind, FloatingWindowKind::CommandPalette);
    assert_eq!(command_palette.layer, FloatingLayer::TopOverlay);
    assert!(!command_palette.modal);
    assert_eq!(preferences.kind, FloatingWindowKind::Preferences);
    assert!(preferences.modal);
    assert!(preferences
        .content_asset
        .ends_with("workbench_preferences.zui"));
}
