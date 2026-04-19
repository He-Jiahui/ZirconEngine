use crate::template::UiTemplateLoader;
use crate::template::{
    UiTemplateError, UiTemplateInstance, UiTemplateSurfaceBuilder, UiTemplateTreeBuilder,
    UiTemplateValidator,
};
use crate::binding::UiEventKind;
use crate::event_ui::UiTreeId;
use crate::{
    AxisConstraint, StretchMode, UiAxis, UiContainerKind, UiFrame, UiInputPolicy,
    UiLinearBoxConfig, UiScrollState, UiScrollableBoxConfig, UiScrollbarVisibility, UiSize,
    UiVirtualListConfig,
};

const WORKBENCH_TEMPLATE_TOML: &str = r#"
version = 1

[root]
template = "WorkbenchShell"
slots = { menu_bar = [{ template = "MenuBar" }], activity_rail = [{ component = "ActivityRail", control_id = "ActivityRailRoot" }], document_host = [{ component = "ToolWindowStack", control_id = "DocumentHost" }] }

[components.WorkbenchShell]
slots = { menu_bar = { required = true }, activity_rail = { required = true }, document_host = { required = true } }
root = { component = "WorkbenchShell", children = [{ slot = "menu_bar" }, { slot = "activity_rail" }, { slot = "document_host" }] }

[components.MenuBar]
root = { component = "UiHostToolbar", children = [
    { component = "UiHostIconButton", control_id = "OpenProject", bindings = [{ id = "WorkbenchMenuBar/OpenProject", event = "Click", route = "MenuAction.OpenProject" }], attributes = { icon = "folder-open-outline", label = "Open" } },
    { component = "UiHostIconButton", control_id = "SaveProject", bindings = [{ id = "WorkbenchMenuBar/SaveProject", event = "Click", route = "MenuAction.SaveProject" }], attributes = { icon = "save-outline", label = "Save" } }
] }
"#;

const SHARED_CONTAINER_TEMPLATE_TOML: &str = r#"
version = 1

[root]
component = "ScrollableBox"
control_id = "ScrollRoot"
children = [
    { component = "HorizontalBox", control_id = "Row" },
    { component = "Space", control_id = "Gap" },
    { component = "UiHostIconButton", control_id = "InteractiveLeaf", bindings = [{ id = "Demo/Click", event = "Click", route = "Demo.Click" }], attributes = { label = "Demo" } }
]
"#;

const LAYOUT_CONTRACT_TEMPLATE_TOML: &str = r#"
version = 1

[root]
component = "WorkspaceShell"
control_id = "WorkspaceShellRoot"
attributes = { layout = { container = { kind = "VerticalBox", gap = 12.0 }, width = { stretch = "Stretch" }, height = { stretch = "Stretch" }, clip = true } }
children = [
    { component = "UiHostToolbar", control_id = "Toolbar", attributes = { layout = { container = { kind = "HorizontalBox", gap = 8.0 }, width = { stretch = "Stretch" }, height = { min = 48.0, preferred = 48.0, max = 48.0, stretch = "Fixed" } } }, children = [
        { component = "UiHostIconButton", control_id = "ToolbarAction", bindings = [{ id = "Toolbar/Action", event = "Click", route = "Toolbar.Action" }], attributes = { label = "Action", layout = { width = { min = 120.0, preferred = 120.0, max = 120.0, stretch = "Fixed" }, height = { min = 32.0, preferred = 32.0, max = 32.0, stretch = "Fixed" } } } }
    ] },
    { component = "ViewportHost", control_id = "ViewportHost", attributes = { layout = { container = { kind = "Overlay" }, width = { stretch = "Stretch" }, height = { stretch = "Stretch" } } }, children = [
        { component = "OverlayBadge", control_id = "OverlayBadge", attributes = { layout = { width = { min = 60.0, preferred = 60.0, max = 60.0, stretch = "Fixed" }, height = { min = 24.0, preferred = 24.0, max = 24.0, stretch = "Fixed" }, anchor = { x = 1.0, y = 0.0 }, pivot = { x = 1.0, y = 0.0 }, position = { x = -16.0, y = 12.0 }, z_index = 4 } } }
    ] },
    { component = "AssetList", control_id = "AssetList", attributes = { layout = { container = { kind = "ScrollableBox", axis = "Vertical", gap = 6.0, scrollbar_visibility = "Always", virtualization = { item_extent = 28.0, overscan = 2 } }, width = { stretch = "Stretch" }, height = { min = 120.0, preferred = 120.0, max = 120.0, stretch = "Fixed" }, clip = true } }, children = [
        { component = "AssetRow", control_id = "AssetRow0", attributes = { layout = { width = { stretch = "Stretch" }, height = { min = 28.0, preferred = 28.0, max = 28.0, stretch = "Fixed" } } } },
        { component = "AssetRow", control_id = "AssetRow1", attributes = { layout = { width = { stretch = "Stretch" }, height = { min = 28.0, preferred = 28.0, max = 28.0, stretch = "Fixed" } } } },
        { component = "AssetRow", control_id = "AssetRow2", attributes = { layout = { width = { stretch = "Stretch" }, height = { min = 28.0, preferred = 28.0, max = 28.0, stretch = "Fixed" } } } },
        { component = "AssetRow", control_id = "AssetRow3", attributes = { layout = { width = { stretch = "Stretch" }, height = { min = 28.0, preferred = 28.0, max = 28.0, stretch = "Fixed" } } } },
        { component = "AssetRow", control_id = "AssetRow4", attributes = { layout = { width = { stretch = "Stretch" }, height = { min = 28.0, preferred = 28.0, max = 28.0, stretch = "Fixed" } } } }
    ] }
]
"#;

#[test]
fn template_loader_parses_component_slots_and_binding_refs_from_toml() {
    let document = UiTemplateLoader::load_toml_str(WORKBENCH_TEMPLATE_TOML).unwrap();

    assert_eq!(document.version, 1);
    assert_eq!(document.root.template.as_deref(), Some("WorkbenchShell"));

    let shell = document.components.get("WorkbenchShell").unwrap();
    assert!(shell.slots["menu_bar"].required);
    assert!(!shell.slots["menu_bar"].multiple);

    let menu_bar = document.components.get("MenuBar").unwrap();
    let toolbar_root = &menu_bar.root;
    assert_eq!(toolbar_root.component.as_deref(), Some("UiHostToolbar"));
    assert_eq!(toolbar_root.children.len(), 2);
    assert_eq!(
        toolbar_root.children[0].bindings[0].id,
        "WorkbenchMenuBar/OpenProject"
    );
    assert_eq!(
        toolbar_root.children[0].bindings[0].event,
        UiEventKind::Click
    );
    assert_eq!(
        toolbar_root.children[0].bindings[0].route.as_deref(),
        Some("MenuAction.OpenProject")
    );
}

#[test]
fn template_instance_expands_composite_slots_and_preserves_stable_bindings() {
    let document = UiTemplateLoader::load_toml_str(WORKBENCH_TEMPLATE_TOML).unwrap();
    UiTemplateValidator::validate_document(&document).unwrap();

    let instance = UiTemplateInstance::from_document(&document).unwrap();

    assert_eq!(instance.root.component.as_deref(), Some("WorkbenchShell"));
    assert_eq!(instance.root.children.len(), 3);
    assert_eq!(
        instance.root.children[0].component.as_deref(),
        Some("UiHostToolbar")
    );
    assert_eq!(
        instance.root.children[1].component.as_deref(),
        Some("ActivityRail")
    );
    assert_eq!(
        instance.root.children[2].control_id.as_deref(),
        Some("DocumentHost")
    );

    let bindings = instance.binding_refs();
    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0].id, "WorkbenchMenuBar/OpenProject");
    assert_eq!(bindings[0].route.as_deref(), Some("MenuAction.OpenProject"));
    assert_eq!(bindings[1].id, "WorkbenchMenuBar/SaveProject");
    assert_eq!(bindings[1].route.as_deref(), Some("MenuAction.SaveProject"));
}

#[test]
fn template_validator_rejects_missing_required_slots() {
    let document = UiTemplateLoader::load_toml_str(
        r#"
version = 1

[root]
template = "WorkbenchShell"

[components.WorkbenchShell]
slots = { menu_bar = { required = true } }
root = { component = "WorkbenchShell", children = [{ slot = "menu_bar" }] }
"#,
    )
    .unwrap();

    let error = UiTemplateValidator::validate_document(&document).unwrap_err();
    assert_eq!(
        error,
        UiTemplateError::MissingRequiredSlot {
            template_id: "WorkbenchShell".to_string(),
            slot_name: "menu_bar".to_string(),
        }
    );
}

#[test]
fn template_tree_builder_projects_template_instance_into_shared_ui_tree_with_metadata() {
    let document = UiTemplateLoader::load_toml_str(WORKBENCH_TEMPLATE_TOML).unwrap();
    let instance = UiTemplateInstance::from_document(&document).unwrap();

    let tree =
        UiTemplateTreeBuilder::build_tree(UiTreeId::new("workbench.template"), &instance).unwrap();

    assert_eq!(tree.roots.len(), 1);
    assert_eq!(tree.nodes.len(), 6);

    let root = tree.node(tree.roots[0]).unwrap();
    let root_template = root.template_metadata.as_ref().expect("root metadata");
    assert_eq!(root_template.component, "WorkbenchShell");
    assert_eq!(root_template.control_id, None);

    let open_project = tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("OpenProject")
        })
        .unwrap();
    let open_project_template = open_project.template_metadata.as_ref().unwrap();
    assert_eq!(open_project_template.component, "UiHostIconButton");
    assert_eq!(
        open_project_template
            .attributes
            .get("icon")
            .unwrap()
            .as_str(),
        Some("folder-open-outline")
    );
    assert_eq!(
        open_project_template
            .attributes
            .get("label")
            .unwrap()
            .as_str(),
        Some("Open")
    );
    assert_eq!(open_project_template.bindings.len(), 1);
    assert_eq!(
        open_project_template.bindings[0].id,
        "WorkbenchMenuBar/OpenProject"
    );
    assert_eq!(
        open_project_template.bindings[0].route.as_deref(),
        Some("MenuAction.OpenProject")
    );
    assert!(open_project.node_path.0.contains("OpenProject"));
    assert!(open_project.state_flags.clickable);
    assert!(open_project.state_flags.hoverable);
    assert!(open_project.state_flags.focusable);
    assert_eq!(open_project.input_policy, UiInputPolicy::Receive);
}

#[test]
fn template_surface_builder_maps_known_container_components_into_shared_runtime_nodes() {
    let document = UiTemplateLoader::load_toml_str(SHARED_CONTAINER_TEMPLATE_TOML).unwrap();
    let instance = UiTemplateInstance::from_document(&document).unwrap();

    let surface = UiTemplateSurfaceBuilder::build_surface(
        UiTreeId::new("shared.container.template"),
        &instance,
    )
    .unwrap();

    assert_eq!(surface.tree.nodes.len(), 4);
    assert_eq!(
        surface.render_extract.tree_id.0,
        "shared.container.template"
    );

    let root = surface.tree.node(surface.tree.roots[0]).unwrap();
    assert_eq!(
        root.template_metadata
            .as_ref()
            .unwrap()
            .control_id
            .as_deref(),
        Some("ScrollRoot")
    );
    assert_eq!(
        root.container,
        UiContainerKind::ScrollableBox(Default::default())
    );
    assert_eq!(root.scroll_state, Some(UiScrollState::default()));
    assert!(root.clip_to_bounds);

    let row = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("Row")
        })
        .unwrap();
    assert_eq!(
        row.container,
        UiContainerKind::HorizontalBox(Default::default())
    );

    let gap = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("Gap")
        })
        .unwrap();
    assert_eq!(gap.container, UiContainerKind::Space);

    let interactive_leaf = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("InteractiveLeaf")
        })
        .unwrap();
    assert_eq!(interactive_leaf.input_policy, UiInputPolicy::Receive);
    assert!(interactive_leaf.state_flags.clickable);
    assert!(interactive_leaf.state_flags.hoverable);
    assert!(interactive_leaf.state_flags.focusable);
}

#[test]
fn template_tree_builder_maps_layout_contract_attributes_into_shared_runtime_nodes() {
    let document = UiTemplateLoader::load_toml_str(LAYOUT_CONTRACT_TEMPLATE_TOML).unwrap();
    let instance = UiTemplateInstance::from_document(&document).unwrap();

    let tree =
        UiTemplateTreeBuilder::build_tree(UiTreeId::new("layout.contract"), &instance).unwrap();

    let root = tree.node(tree.roots[0]).unwrap();
    assert_eq!(
        root.container,
        UiContainerKind::VerticalBox(UiLinearBoxConfig { gap: 12.0 })
    );
    assert!(root.clip_to_bounds);

    let toolbar = tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("Toolbar")
        })
        .unwrap();
    assert_eq!(
        toolbar.container,
        UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 8.0 })
    );
    assert_eq!(
        toolbar.constraints.height,
        AxisConstraint {
            min: 48.0,
            max: 48.0,
            preferred: 48.0,
            priority: 0,
            weight: 1.0,
            stretch_mode: StretchMode::Fixed,
        }
    );

    let overlay_badge = tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("OverlayBadge")
        })
        .unwrap();
    assert_eq!(overlay_badge.anchor.x, 1.0);
    assert_eq!(overlay_badge.anchor.y, 0.0);
    assert_eq!(overlay_badge.pivot.x, 1.0);
    assert_eq!(overlay_badge.pivot.y, 0.0);
    assert_eq!(overlay_badge.position.x, -16.0);
    assert_eq!(overlay_badge.position.y, 12.0);
    assert_eq!(overlay_badge.z_index, 4);

    let asset_list = tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("AssetList")
        })
        .unwrap();
    assert_eq!(
        asset_list.container,
        UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
            axis: UiAxis::Vertical,
            gap: 6.0,
            scrollbar_visibility: UiScrollbarVisibility::Always,
            virtualization: Some(UiVirtualListConfig {
                item_extent: 28.0,
                overscan: 2,
            }),
        })
    );
    assert!(asset_list.clip_to_bounds);
}

#[test]
fn template_surface_builder_computes_layout_from_template_contract_attributes() {
    let document = UiTemplateLoader::load_toml_str(LAYOUT_CONTRACT_TEMPLATE_TOML).unwrap();
    let instance = UiTemplateInstance::from_document(&document).unwrap();

    let mut surface =
        UiTemplateSurfaceBuilder::build_surface(UiTreeId::new("layout.surface"), &instance)
            .unwrap();
    surface.compute_layout(UiSize::new(800.0, 600.0)).unwrap();

    let root = surface.tree.node(surface.tree.roots[0]).unwrap();
    assert_eq!(
        root.layout_cache.frame,
        UiFrame::new(0.0, 0.0, 800.0, 600.0)
    );

    let toolbar = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("Toolbar")
        })
        .unwrap();
    assert_eq!(
        toolbar.layout_cache.frame,
        UiFrame::new(0.0, 0.0, 800.0, 48.0)
    );

    let viewport_host = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("ViewportHost")
        })
        .unwrap();
    assert_eq!(
        viewport_host.layout_cache.frame,
        UiFrame::new(0.0, 60.0, 800.0, 408.0)
    );

    let overlay_badge = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("OverlayBadge")
        })
        .unwrap();
    assert_eq!(
        overlay_badge.layout_cache.frame,
        UiFrame::new(724.0, 72.0, 60.0, 24.0)
    );

    let asset_list = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("AssetList")
        })
        .unwrap();
    assert_eq!(
        asset_list.layout_cache.frame,
        UiFrame::new(0.0, 480.0, 800.0, 120.0)
    );
    assert_eq!(
        asset_list.scroll_state,
        Some(UiScrollState {
            offset: 0.0,
            viewport_extent: 120.0,
            content_extent: 164.0,
        })
    );
    assert_eq!(
        asset_list.layout_cache.virtual_window,
        Some(crate::UiVirtualListWindow {
            first_visible: 0,
            last_visible_exclusive: 5,
        })
    );
}
