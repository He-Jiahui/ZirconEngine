use super::*;

#[test]
fn material_demo_window_compiles_and_resolves_material_dark_states() {
    let document = UiV2AssetLoader::load_toml_str(include_str!(
        "../../../../../zircon_editor/assets/ui/editor/material_demo_window.zui"
    ))
    .unwrap();

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let resolved = UiV2StyleResolver::resolve(&document, &compiled.arena).unwrap();

    let components = compiled
        .arena
        .nodes
        .iter()
        .map(|node| node.component.as_str())
        .collect::<BTreeSet<_>>();
    for component in [
        "Button",
        "IconButton",
        "TextField",
        "Checkbox",
        "Switch",
        "Dropdown",
        "Slider",
        "Tabs",
        "Menu",
        "Tooltip",
        "Scrollbar",
        "Splitter",
        "Panel",
        "Modal",
    ] {
        assert!(components.contains(component), "missing {component}");
    }

    assert_eq!(
        resolved.nodes["window"].self_values["background"].as_str(),
        Some("#121212")
    );
    assert_eq!(
        resolved.nodes["primary_button"].self_values["background"].as_str(),
        Some("rgba(255,255,255,0.08)")
    );
    assert_eq!(
        resolved.nodes["icon_button"].self_values["background"].as_str(),
        Some("rgba(255,255,255,0.12)")
    );
    assert_eq!(
        resolved.nodes["primary_button"].self_values["outline"].as_str(),
        Some("#90caf9")
    );
    assert_eq!(
        resolved.nodes["text_field"].self_values["fg"].as_str(),
        Some("#ffa726")
    );
    assert_eq!(
        resolved.nodes["modal"].self_values["fg"].as_str(),
        Some("#f44336")
    );
    assert_eq!(
        resolved.nodes["switch"].self_values["fg"].as_str(),
        Some("rgba(255,255,255,0.30)")
    );
}

#[test]
fn editor_base_theme_tokens_drive_workbench_chrome_assets() {
    let activity_surface = editor_v2_theme_surface(
        "workbench_activity_rail.zui",
        "runtime.ui.v2.editor_base_activity_rail",
    );
    let activity_root = node_id_by_control_id(&activity_surface, "ActivityRailRoot");

    assert_eq!(
        runtime_color_attr(&activity_surface, activity_root, "background"),
        Some("#1b1f23")
    );
    assert_eq!(
        runtime_style_token(&activity_surface, activity_root, "background.color"),
        Some("token.panel_bg -> theme.palette.surface.2")
    );
    assert_eq!(
        runtime_color_attr(&activity_surface, activity_root, "border"),
        Some("#394147")
    );
    assert_eq!(
        runtime_style_token(&activity_surface, activity_root, "border.color"),
        Some("token.border -> theme.palette.separator")
    );

    let status_surface = editor_v2_theme_surface(
        "workbench_status_bar.zui",
        "runtime.ui.v2.editor_base_status_bar",
    );
    let status_root = node_id_by_control_id(&status_surface, "WorkbenchStatusBarRoot");

    assert_eq!(
        runtime_color_attr(&status_surface, status_root, "background"),
        Some("#252b31")
    );
    assert_eq!(
        runtime_style_token(&status_surface, status_root, "background.color"),
        Some("token.surface_hover -> theme.palette.surface.3")
    );
    assert_eq!(
        runtime_color_attr(&status_surface, status_root, "foreground"),
        Some("#e8ecee")
    );
    assert_eq!(
        runtime_style_token(&status_surface, status_root, "foreground.color"),
        Some("token.text -> theme.palette.text.primary")
    );
}

#[test]
fn editor_material_theme_runtime_pseudo_states_drive_imported_v2_surface() {
    let mut surface = welcome_material_surface("runtime.ui.v2.editor_material_state");
    let button_id = node_id_by_control_id(&surface, "WelcomeStartupDemoButton");

    assert_eq!(
        runtime_color_attr(&surface, button_id, "background"),
        Some("#1b1f23")
    );
    assert_eq!(
        runtime_style_token(&surface, button_id, "background.color"),
        Some("token.material_surface -> theme.palette.surface.2")
    );

    assert!(surface.component_states.set_hovered(button_id, true));
    surface
        .mark_component_state_render_dirty(button_id)
        .unwrap();
    assert_eq!(
        runtime_color_attr(&surface, button_id, "background"),
        Some("#252b31")
    );
    assert_eq!(
        runtime_color_attr(&surface, button_id, "foreground"),
        Some("#e8ecee")
    );
    assert_eq!(
        runtime_style_token(&surface, button_id, "background.color"),
        Some("token.material_surface_hover -> theme.palette.surface.3")
    );

    assert!(surface.component_states.set_pressed(button_id, true));
    surface
        .mark_component_state_render_dirty(button_id)
        .unwrap();
    assert_eq!(
        runtime_color_attr(&surface, button_id, "background"),
        Some("#103c4a")
    );

    assert!(surface.component_states.set_focused(button_id, true));
    surface
        .mark_component_state_render_dirty(button_id)
        .unwrap();
    assert_eq!(
        runtime_color_attr(&surface, button_id, "border"),
        Some("#3cc7d6")
    );
    assert_eq!(
        runtime_style_token(&surface, button_id, "border.color"),
        Some("token.material_focus_ring -> theme.palette.accent")
    );
}

#[test]
fn editor_material_runtime_pseudo_states_rebuild_render_extract_variants() {
    let mut surface = welcome_material_surface("runtime.ui.v2.editor_material_render_state");
    let root_size = UiSize::new(960.0, 640.0);
    surface.compute_layout(root_size).unwrap();
    let button_id = node_id_by_control_id(&surface, "WelcomeStartupDemoButton");
    let button_frame = surface.arranged_tree.get(button_id).unwrap().frame;
    let pointer = UiPoint::new(button_frame.x + 2.0, button_frame.y + 2.0);

    assert_eq!(
        render_command_background(&surface, button_id),
        Some("#1b1f23")
    );

    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Move, pointer),
        )
        .unwrap();
    let hover_report = surface.rebuild_dirty(root_size).unwrap();
    assert!(hover_report.render_rebuilt);
    assert!(!hover_report.layout_recomputed);
    assert!(!hover_report.arranged_rebuilt);
    assert_eq!(
        render_command_background(&surface, button_id),
        Some("#252b31")
    );

    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, pointer)
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    let press_report = surface.rebuild_dirty(root_size).unwrap();
    assert!(press_report.render_rebuilt);
    assert!(!press_report.layout_recomputed);
    assert_eq!(
        render_command_background(&surface, button_id),
        Some("#103c4a")
    );
    assert_eq!(render_command_border(&surface, button_id), Some("#3cc7d6"));
}

#[test]
fn layout_demo_window_compiles_with_window_drawer_and_data_view_components() {
    let document = UiV2AssetLoader::load_toml_str(include_str!(
        "../../../../../zircon_editor/assets/ui/editor/layout_demo_window.zui"
    ))
    .unwrap();

    let source_components = document
        .nodes
        .values()
        .map(|node| node.component.as_str())
        .collect::<BTreeSet<_>>();
    assert!(source_components.contains("Slot"), "missing source Slot");

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let root = compiled
        .arena
        .node(compiled.arena.root.expect("layout demo root"))
        .unwrap();
    assert_eq!(root.component, "DockHost");

    let components = compiled
        .arena
        .nodes
        .iter()
        .map(|node| node.component.as_str())
        .collect::<BTreeSet<_>>();
    for component in [
        "Window",
        "WorkbenchShell",
        "Drawer",
        "View",
        "WindowFrame",
        "DocumentNode",
        "TabStack",
        "FloatingWindow",
        "FlexGroup",
        "HorizontalGroup",
        "GridGroup",
        "Overlay",
        "ListView",
        "VirtualList",
        "TreeView",
        "PropertyGrid",
        "InspectorSection",
        "Composite",
    ] {
        assert!(components.contains(component), "missing {component}");
    }
    assert!(compiled.arena.node_count() >= 25);
}

#[test]
fn fyrox_panel_demo_window_compiles_with_all_panel_role_components() {
    let document = UiV2AssetLoader::load_toml_str(include_str!(
        "../../../../../zircon_editor/assets/ui/editor/fyrox_panel_demo_window.zui"
    ))
    .unwrap();

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let root = compiled
        .arena
        .node(compiled.arena.root.expect("fyrox panel demo root"))
        .unwrap();
    assert_eq!(root.component, "WorkbenchShell");

    let components = compiled
        .arena
        .nodes
        .iter()
        .map(|node| node.component.as_str())
        .collect::<BTreeSet<_>>();
    for component in [
        "AssetGrid",
        "AssetList",
        "CategorizedList",
        "ContextMenu",
        "FieldEditor",
        "FilterBar",
        "FolderTree",
        "GizmoControls",
        "GraphCanvas",
        "InspectorSection",
        "MetadataPane",
        "PaneToolbar",
        "PreviewPane",
        "PropertyGrid",
        "SearchField",
        "SeverityChips",
        "SourceEditor",
        "StatusActionControls",
        "Timeline",
        "TreeView",
        "VirtualList",
        "ViewportHost",
        "VisualDesigner",
    ] {
        assert!(components.contains(component), "missing {component}");
    }
    assert_eq!(compiled.arena.node_count(), document.nodes.len());
}

#[test]
fn ui_v2_builds_deep_surface_without_recursive_template_tree() {
    const NODE_COUNT: usize = 10_000;
    let mut document = v2_document("asset://ui/tests/deep.v2.ui", "n0");
    for index in 0..NODE_COUNT {
        let child = (index + 1 < NODE_COUNT).then(|| UiV2ChildMount {
            node: format!("n{}", index + 1),
            slot: BTreeMap::new(),
        });
        document.nodes.insert(
            format!("n{index}"),
            UiV2NodeDefinition {
                component: "Container".to_string(),
                control_id: (index + 1 == NODE_COUNT).then(|| "DeepLeaf".to_string()),
                children: child.into_iter().collect(),
                ..Default::default()
            },
        );
    }

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.deep"),
        &document,
        &compiled,
    )
    .unwrap();

    assert_eq!(surface.tree.nodes.len(), NODE_COUNT);
    assert_eq!(surface.tree.roots.len(), 1);
}

#[test]
fn ui_v2_surface_builder_preserves_direct_runtime_tree_contracts() {
    let document = UiV2AssetLoader::load_toml_str(
        r#"
[asset]
kind = "view"
id = "asset://ui/tests/surface_tree_contract.v2.ui"
version = 2

[root]
node = "root"

[nodes.root]
component = "VerticalBox"
control_id = "Root"
layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" }, container = { kind = "VerticalBox", gap = 4.0 } }
children = [{ node = "action", slot = { layout = { padding = { left = 3.0, top = 5.0, right = 7.0, bottom = 11.0 }, linear_size = { rule = "Auto" } } } }]

[nodes.action]
component = "Button"
control_id = "ActionButton"
props = { text = "Run", input_clickable = false, input_focusable = true }
layout = { width = { min = 80.0, preferred = 120.0, max = 160.0, stretch = "Fixed" }, height = { min = 24.0, preferred = 32.0, max = 48.0, stretch = "Fixed" } }
events = [{ id = "Run", event = "Click", route = "Runtime.Run" }]
"#,
    )
    .unwrap();

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.surface_tree_contract"),
        &document,
        &compiled,
    )
    .unwrap();
    let root = surface.tree.nodes.get(&surface.tree.roots[0]).unwrap();
    let action = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("ActionButton")
        })
        .expect("action node should be projected");

    assert_eq!(root.children, vec![action.node_id]);
    assert!(root.layout_stretch_width);
    assert!(root.layout_stretch_height);
    assert!(!action.state_flags.clickable);
    assert!(action.state_flags.hoverable);
    assert!(action.state_flags.focusable);
    let metadata = action.template_metadata.as_ref().unwrap();
    assert_eq!(metadata.component, "Button");
    assert_eq!(
        metadata.attributes.get("text").and_then(Value::as_str),
        Some("Run")
    );
    assert_eq!(metadata.bindings.len(), 1);
    let slot = surface
        .tree
        .slots
        .iter()
        .find(|slot| slot.child_id == action.node_id)
        .expect("action slot should be projected");
    assert_eq!(slot.padding.left, 3.0);
    assert_eq!(
        slot.linear_sizing.as_ref().map(|sizing| sizing.rule),
        Some(zircon_runtime_interface::ui::layout::UiLinearSlotSizeRule::Auto)
    );
}

#[test]
fn ui_v2_surface_builder_infers_interaction_from_component_catalog() {
    let document = UiV2AssetLoader::load_toml_str(
        r#"
[asset]
kind = "view"
id = "asset://ui/tests/catalog_interaction.v2.ui"
version = 2

[root]
node = "root"

[nodes.root]
component = "VerticalGroup"
children = [
    { node = "toggle" },
    { node = "range" },
    { node = "combo" },
    { node = "tree" },
    { node = "table" },
    { node = "message" },
    { node = "progress" },
]

[nodes.toggle]
component = "Toggle"
control_id = "DefaultToggle"
props = { checked = true }

[nodes.range]
component = "RangeField"
control_id = "DefaultRange"
props = { value = 42.0, min = 0.0, max = 100.0 }

[nodes.combo]
component = "ComboBox"
control_id = "DefaultCombo"
props = { value = "scene", options = ["scene", "asset"] }

[nodes.tree]
component = "TreeView"
control_id = "DefaultTree"
props = { selected_index = 0, expanded = true, items = ["Root"] }

[nodes.table]
component = "EditableTable"
control_id = "DefaultTable"
props = { selected_row = 0, selected_column = 0, rows = [], columns = [] }

[nodes.message]
component = "MessageBox"
control_id = "DefaultMessage"
props = { severity = "info", text = "Ready", rich_text = "<b>Ready</b>", open = true, actions = ["Dismiss"] }

[nodes.progress]
component = "ProgressBar"
control_id = "DefaultProgress"
props = { value = 0.5 }
"#,
    )
    .unwrap();

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.catalog_interaction"),
        &document,
        &compiled,
    )
    .unwrap();

    let node_by_control_id = |control_id: &str| {
        surface
            .tree
            .nodes
            .values()
            .find(|node| {
                node.template_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.control_id.as_deref())
                    == Some(control_id)
            })
            .unwrap_or_else(|| panic!("{control_id} should be projected"))
    };

    for control_id in [
        "DefaultToggle",
        "DefaultRange",
        "DefaultCombo",
        "DefaultTree",
        "DefaultTable",
        "DefaultMessage",
    ] {
        let node = node_by_control_id(control_id);
        assert_eq!(node.input_policy, UiInputPolicy::Receive, "{control_id}");
        assert!(node.state_flags.clickable, "{control_id} clickable");
        assert!(node.state_flags.hoverable, "{control_id} hoverable");
        assert!(node.state_flags.focusable, "{control_id} focusable");
    }

    let progress = node_by_control_id("DefaultProgress");
    assert_eq!(progress.input_policy, UiInputPolicy::Inherit);
    assert!(!progress.state_flags.clickable);
    assert!(!progress.state_flags.hoverable);
    assert!(!progress.state_flags.focusable);
}

#[test]
fn ui_v2_virtual_list_window_uses_visible_range_and_overscan() {
    let window = compute_virtual_list_window(48.0, 96.0, 24.0, 100, 2);

    assert_eq!(window.first_visible, 0);
    assert_eq!(window.last_visible_exclusive, 8);
}
