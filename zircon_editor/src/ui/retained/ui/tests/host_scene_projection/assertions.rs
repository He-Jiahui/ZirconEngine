pub(super) fn assert_host_contract_scene(
    projected: &crate::ui::retained_host::host_contract::HostWindowSceneData,
) {
    let floating_window = projected
        .floating_layer
        .floating_windows
        .row_data(0)
        .expect("floating window should project");

    assert_eq!(floating_window.active_pane.id, "floating-pane");
    assert_eq!(floating_window.active_pane.kind, "FloatingKind");
    assert_eq!(projected.left_dock.pane.id, "left-pane");
    assert_eq!(projected.left_dock.pane.title, "Left");
    assert_eq!(
        projected.left_dock.pane.ui_asset.header.asset_id,
        "asset://ui/test.zui"
    );
    assert_eq!(projected.left_dock.pane.ui_asset.header.mode, "split");
    assert_eq!(projected.left_dock.pane.ui_asset.header.selection, "Root");
    let projected_ui_asset_nodes = (0..projected.left_dock.pane.ui_asset.nodes.row_count())
        .filter_map(|row| projected.left_dock.pane.ui_asset.nodes.row_data(row))
        .collect::<Vec<_>>();
    let projected_ui_asset_node = |control_id: &str| {
        projected_ui_asset_nodes
            .iter()
            .find(|node| node.control_id == control_id)
            .unwrap_or_else(|| panic!("projected ui asset node `{control_id}` should exist"))
    };
    assert_eq!(projected_ui_asset_node("HeaderPanel").frame.x, 11.0);
    assert_eq!(projected_ui_asset_node("HeaderPanel").frame.width, 640.0);
    assert_eq!(projected_ui_asset_node("HeaderAssetRow").frame.x, 21.0);
    assert_eq!(projected_ui_asset_node("HeaderStatusRow").frame.y, 28.0);
    assert_eq!(
        projected_ui_asset_node("HeaderActionRow").frame.height,
        20.0
    );
    assert_eq!(projected_ui_asset_node("PalettePanel").frame.height, 240.0);
    assert_eq!(
        projected
            .left_dock
            .pane
            .ui_asset
            .center_column_node
            .control_id,
        "CenterColumn"
    );
    assert_eq!(
        projected.left_dock.pane.ui_asset.center_column_node.frame.x,
        260.0
    );
    assert_eq!(
        projected
            .left_dock
            .pane
            .ui_asset
            .designer_panel_node
            .frame
            .y,
        80.0
    );
    assert_eq!(
        projected
            .left_dock
            .pane
            .ui_asset
            .designer_canvas_panel_node
            .frame
            .height,
        214.0
    );
    assert_eq!(projected_ui_asset_node("RenderStackPanel").frame.y, 328.0);
    assert_eq!(projected_ui_asset_node("ActionBarPanel").frame.height, 88.0);
    assert_eq!(projected_ui_asset_node("ActionInsertRow").frame.x, 280.0);
    assert_eq!(projected_ui_asset_node("ActionReparentRow").frame.y, 450.0);
    assert_eq!(
        projected_ui_asset_node("ActionStructureRow").frame.width,
        380.0
    );
    assert_eq!(
        projected_ui_asset_node("SourceInfoPanel").frame.height,
        58.0
    );
    assert_eq!(
        projected_ui_asset_node("MockWorkspacePanel").frame.width,
        400.0
    );
    assert_eq!(
        projected_ui_asset_node("MockSubjectsPanel").frame.height,
        72.0
    );
    assert_eq!(projected_ui_asset_node("MockEditorPanel").frame.y, 606.0);
    assert_eq!(
        projected_ui_asset_node("MockStateGraphPanel").frame.y,
        782.0
    );
    assert_eq!(projected_ui_asset_node("SourceTextPanel").frame.y, 860.0);
    assert_eq!(
        projected
            .left_dock
            .pane
            .ui_asset
            .inspector_panel_node
            .frame
            .height,
        240.0
    );
    assert_eq!(
        projected_ui_asset_node("InspectorContentPanel").frame.y,
        106.0
    );
    assert_eq!(
        projected
            .left_dock
            .pane
            .ui_asset
            .stylesheet_panel_node
            .frame
            .width,
        260.0
    );
    assert_eq!(
        projected_ui_asset_node("StylesheetActionRow").frame.y,
        356.0
    );
    assert_eq!(
        projected_ui_asset_node("StylesheetStatePrimaryRow")
            .frame
            .height,
        24.0
    );
    assert_eq!(
        projected_ui_asset_node("StylesheetStateSecondaryRow")
            .frame
            .x,
        710.0
    );
    assert_eq!(
        projected_ui_asset_node("StylesheetContentPanel")
            .frame
            .height,
        49.0
    );
    assert_eq!(
        projected
            .left_dock
            .pane
            .ui_asset
            .collections
            .palette
            .items
            .row_data(0)
            .expect("palette item should project"),
        "Button"
    );
    let projected_hierarchy_nodes = (0..projected.left_dock.pane.hierarchy.nodes.row_count())
        .filter_map(|row| projected.left_dock.pane.hierarchy.nodes.row_data(row))
        .collect::<Vec<_>>();
    assert_eq!(
        projected_hierarchy_nodes
            .iter()
            .find(|node| node.control_id == "HierarchyListPanel")
            .expect("hierarchy list panel node should project")
            .frame
            .x,
        8.0
    );
    assert_eq!(
        projected
            .left_dock
            .pane
            .hierarchy
            .hierarchy_nodes
            .row_count(),
        2
    );
    assert_eq!(
        projected
            .left_dock
            .pane
            .hierarchy
            .hierarchy_nodes
            .row_data(0)
            .expect("hierarchy node should project")
            .name,
        "Root"
    );
    assert_eq!(projected.document_dock.pane.id, "document-pane");
    assert_eq!(projected.document_dock.pane.title, "Document");
    assert_eq!(projected.right_dock.pane.id, "right-pane");
    assert_eq!(projected.right_dock.pane.title, "Right");
    assert_eq!(projected.right_dock.pane.inspector.info, "Node 42");
    let projected_inspector_nodes = (0..projected.right_dock.pane.inspector.nodes.row_count())
        .filter_map(|row| projected.right_dock.pane.inspector.nodes.row_data(row))
        .collect::<Vec<_>>();
    assert_eq!(
        projected_inspector_nodes
            .iter()
            .find(|node| node.control_id == "InspectorPositionRow")
            .expect("inspector position row should project")
            .frame
            .y,
        88.0
    );
    assert!(projected.right_dock.pane.inspector.delete_enabled);
    assert_eq!(projected.right_dock.pane.animation.mode, "sequence");
    assert_eq!(
        projected.right_dock.pane.animation.asset_path,
        "asset://animation/walk.anim"
    );
    let animation_nodes = &projected.right_dock.pane.animation.nodes;
    let animation_node = |control_id: &str| {
        (0..animation_nodes.row_count())
            .filter_map(|row| animation_nodes.row_data(row))
            .find(|node| node.control_id == control_id)
            .unwrap_or_else(|| panic!("animation node `{control_id}` should project"))
    };
    assert_eq!(animation_node("AnimationEditorHeaderPanel").frame.x, 14.0);
    assert_eq!(
        animation_node("AnimationEditorHeaderStatusRow").frame.y,
        62.0
    );
    assert_eq!(
        animation_node("AnimationSequenceTracksPanel").frame.height,
        250.0
    );
    assert_eq!(projected.document_dock.pane.kind, "Project");
    assert_eq!(
        projected
            .document_dock
            .pane
            .project_overview
            .nodes
            .row_count(),
        3
    );
    assert_eq!(
        projected
            .document_dock
            .pane
            .project_overview
            .nodes
            .row_data(0)
            .expect("project overview node should project")
            .control_id,
        "ProjectOverviewOuterPanel"
    );
    assert_eq!(
        projected
            .document_dock
            .pane
            .project_overview
            .nodes
            .row_data(1)
            .expect("project overview path node should project")
            .text,
        "res://project"
    );
    assert_eq!(animation_node("AnimationGraphNodesPanel").frame.y, 234.0);
    assert_eq!(
        animation_node("AnimationStateMachineTransitionsPanel")
            .frame
            .height,
        146.0
    );
    assert_eq!(
        projected
            .right_dock
            .pane
            .animation
            .track_items
            .row_data(0)
            .expect("track item should project"),
        "Root Translation"
    );
    assert_eq!(projected.bottom_dock.pane.id, "bottom-pane");
    assert_eq!(projected.bottom_dock.pane.title, "Bottom");
    assert_eq!(projected.bottom_dock.pane.kind, "Assets");
    let projected_assets_nodes = (0..projected.bottom_dock.pane.assets_activity.nodes.row_count())
        .filter_map(|row| {
            projected
                .bottom_dock
                .pane
                .assets_activity
                .nodes
                .row_data(row)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        projected_assets_nodes
            .iter()
            .find(|node| node.control_id == "AssetsActivityToolbarPanel")
            .expect("toolbar node should project")
            .frame
            .x,
        18.0
    );
    assert_eq!(
        projected_assets_nodes
            .iter()
            .find(|node| node.control_id == "AssetsActivityTreePanel")
            .expect("tree node should project")
            .frame
            .width,
        248.0
    );
    assert_eq!(
        projected_assets_nodes
            .iter()
            .find(|node| node.control_id == "AssetsActivityUtilityTabsRow")
            .expect("utility tabs node should project")
            .frame
            .height,
        32.0
    );
    assert_eq!(
        projected_assets_nodes
            .iter()
            .find(|node| node.control_id == "AssetsActivityReferenceRightPanel")
            .expect("reference node should project")
            .frame
            .x,
        364.0
    );
    assert_eq!(
        projected.bottom_dock.pane.console.output.as_ref(),
        "Build finished"
    );
    let projected_console_nodes = (0..projected.bottom_dock.pane.console.nodes.row_count())
        .filter_map(|row| projected.bottom_dock.pane.console.nodes.row_data(row))
        .collect::<Vec<_>>();
    assert_eq!(
        projected_console_nodes
            .iter()
            .find(|node| node.control_id == "ConsoleBodySection")
            .expect("console body section node should project")
            .frame
            .height,
        152.0
    );
    assert_eq!(projected.menu_chrome.active_preset_name, "Default");
    assert_eq!(projected.menu_chrome.resolved_preset_name, "Default");
    assert_eq!(projected.menu_chrome.preset_names.row_count(), 1);
    assert_eq!(
        projected
            .menu_chrome
            .preset_names
            .row_data(0)
            .expect("preset name should project"),
        "Default"
    );
}
