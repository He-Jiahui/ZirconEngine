use super::support::{collect_zui_files, editor_asset_root, resource_locator_for_path};
use toml::Value;
use zircon_runtime_interface::ui::v2::UiV2AssetDocument;

const L4_SHELL_STRUCTURAL_COMPONENTS: &[&str] = &[
    "Container",
    "DocumentTabs",
    "HorizontalGroup",
    "Overlay",
    "Slot",
    "Space",
    "VerticalGroup",
];

const RAW_L1_L3_COMPONENTS: &[&str] = &[
    "Alert",
    "Autocomplete",
    "Badge",
    "Button",
    "ButtonBase",
    "Checkbox",
    "Chip",
    "CommandPalette",
    "ConfirmDialog",
    "ContextActionMenu",
    "ContextMenu",
    "Dialog",
    "Divider",
    "Dropdown",
    "DropdownPopup",
    "Icon",
    "IconButton",
    "Input",
    "InputBase",
    "InputField",
    "Label",
    "ListRow",
    "Menu",
    "MenuList",
    "NotificationCenter",
    "NumberField",
    "Progress",
    "ProgressBar",
    "Radio",
    "RangeField",
    "RangeSlider",
    "SearchField",
    "SegmentedControl",
    "Skeleton",
    "Slider",
    "Snackbar",
    "Switch",
    "Tab",
    "Table",
    "Tabs",
    "TextareaAutosize",
    "TextField",
    "Toast",
    "Toggle",
    "ToggleButton",
    "Tooltip",
    "TreeRow",
];

struct L4ShellTopologySnapshot {
    file_name: &'static str,
    component_name: &'static str,
    root_node: &'static str,
    root_component: &'static str,
    root_control_id: &'static str,
    node_count: usize,
    root_children: &'static [&'static str],
}

const L4_SHELL_TOPOLOGY_SNAPSHOTS: &[L4ShellTopologySnapshot] = &[
    L4ShellTopologySnapshot {
        file_name: "workbench_activity_rail.zui",
        component_name: "WorkbenchActivityRail",
        root_node: "activity_rail",
        root_component: "VerticalGroup",
        root_control_id: "WorkbenchWindowActivityRail",
        node_count: 7,
        root_children: &[
            "rail_scene",
            "rail_cube",
            "rail_graph",
            "rail_image",
            "rail_audio",
            "rail_code",
        ],
    },
    L4ShellTopologySnapshot {
        file_name: "workbench_component_drawer.zui",
        component_name: "WorkbenchComponentDrawer",
        root_node: "component_drawer",
        root_component: "VerticalGroup",
        root_control_id: "WorkbenchComponentDrawer",
        node_count: 99,
        root_children: &["drawer_tabs", "component_drawer_content"],
    },
    L4ShellTopologySnapshot {
        file_name: "workbench_inspector_panel.zui",
        component_name: "WorkbenchInspectorPanel",
        root_node: "inspector_panel",
        root_component: "VerticalGroup",
        root_control_id: "WorkbenchInspectorPanel",
        node_count: 43,
        root_children: &["inspector_tabs", "inspector_content"],
    },
    L4ShellTopologySnapshot {
        file_name: "workbench_main_band.zui",
        component_name: "WorkbenchMainBand",
        root_node: "main_band",
        root_component: "Overlay",
        root_control_id: "WorkbenchMainBand",
        node_count: 9,
        root_children: &["scene_workspace", "module_workspace"],
    },
    L4ShellTopologySnapshot {
        file_name: "workbench_scene_tree_panel.zui",
        component_name: "WorkbenchSceneTreePanel",
        root_node: "scene_tree_panel",
        root_component: "VerticalGroup",
        root_control_id: "WorkbenchSceneTreePanel",
        node_count: 20,
        root_children: &["scene_tabs", "scene_content"],
    },
    L4ShellTopologySnapshot {
        file_name: "workbench_status_bar.zui",
        component_name: "WorkbenchStatusBar",
        root_node: "status_bar",
        root_component: "HorizontalGroup",
        root_control_id: "WorkbenchWindowStatusBar",
        node_count: 14,
        root_children: &[
            "status_ready",
            "status_errors",
            "status_warnings",
            "status_messages",
            "status_task",
            "status_grid",
            "status_snap",
            "status_snap_icon",
            "status_world_icon",
            "status_target_icon",
            "status_zoom",
        ],
    },
    L4ShellTopologySnapshot {
        file_name: "workbench_top_toolbar.zui",
        component_name: "WorkbenchTopToolbar",
        root_node: "top_toolbar",
        root_component: "VerticalGroup",
        root_control_id: "WorkbenchWindowTopToolbar",
        node_count: 45,
        root_children: &["toolbar_command_row", "toolbar_module_tabs"],
    },
    L4ShellTopologySnapshot {
        file_name: "workbench_viewport_panel.zui",
        component_name: "WorkbenchViewportPanel",
        root_node: "viewport_panel",
        root_component: "VerticalGroup",
        root_control_id: "WorkbenchViewportPanel",
        node_count: 90,
        root_children: &["document_tabs", "viewport_toolbar", "viewport_surface"],
    },
];

#[test]
fn l4_surfaces_contain_no_inline_primitive_structures() {
    let editor_root = editor_asset_root();
    let shell_root = editor_root.join("ui/editor/components/workbench/shell");
    let mut checked_assets = 0usize;
    let mut checked_nodes = 0usize;
    let mut mounted_workbench_components = 0usize;
    let mut offenders = Vec::new();

    for path in collect_zui_files(&shell_root) {
        checked_assets += 1;
        let locator = resource_locator_for_path(&editor_root, &path);
        let document = super::support::load_zui_document(&path);

        for (node_id, node) in &document.nodes {
            checked_nodes += 1;
            let component = node.component.as_str();
            if component.starts_with("Workbench") {
                mounted_workbench_components += 1;
                continue;
            }
            if L4_SHELL_STRUCTURAL_COMPONENTS.contains(&component) {
                continue;
            }
            if RAW_L1_L3_COMPONENTS.contains(&component) {
                offenders.push(format!(
                    "{locator} node `{node_id}` inlines raw primitive `{component}`; use a Workbench primitive or shell component instead"
                ));
                continue;
            }
            offenders.push(format!(
                "{locator} node `{node_id}` uses `{component}` outside the Workbench primitive/shell/structural allowlist"
            ));
        }
    }

    assert!(
        checked_assets >= 8,
        "L4 Workbench shell should cover the committed shell component assets"
    );
    assert!(
        checked_nodes > 0,
        "L4 Workbench shell assets should declare nodes"
    );
    assert!(
        mounted_workbench_components > 0,
        "L4 Workbench shell assets should compose Workbench primitive or shell components"
    );
    assert!(
        offenders.is_empty(),
        "L4 Workbench shell assets must compose Workbench primitives/shell components and structural containers instead of inlining L1-L3 raw primitives: {offenders:#?}"
    );
}

#[test]
fn l4_surfaces_keep_runtime_region_topology_snapshot() {
    let editor_root = editor_asset_root();
    let shell_root = editor_root.join("ui/editor/components/workbench/shell");
    let mut checked_assets = 0usize;
    let mut offenders = Vec::new();

    for snapshot in L4_SHELL_TOPOLOGY_SNAPSHOTS {
        checked_assets += 1;
        let path = shell_root.join(snapshot.file_name);
        let locator = resource_locator_for_path(&editor_root, &path);
        let document = super::support::load_zui_document(&path);

        let Some(component) = document.components.get(snapshot.component_name) else {
            offenders.push(format!(
                "{locator} should declare shell component `{}`",
                snapshot.component_name
            ));
            continue;
        };
        if component.root != snapshot.root_node {
            offenders.push(format!(
                "{locator} component `{}` root changed from `{}` to `{}`",
                snapshot.component_name, snapshot.root_node, component.root
            ));
            continue;
        }

        let Some(root_node) = document.nodes.get(snapshot.root_node) else {
            offenders.push(format!(
                "{locator} component `{}` references missing root `{}`",
                snapshot.component_name, snapshot.root_node
            ));
            continue;
        };
        if root_node.component != snapshot.root_component {
            offenders.push(format!(
                "{locator} root `{}` should render `{}` but renders `{}`",
                snapshot.root_node, snapshot.root_component, root_node.component
            ));
        }
        if root_node.control_id.as_deref() != Some(snapshot.root_control_id) {
            offenders.push(format!(
                "{locator} root `{}` should keep control id `{}` but has `{:?}`",
                snapshot.root_node, snapshot.root_control_id, root_node.control_id
            ));
        }
        if document.nodes.len() != snapshot.node_count {
            offenders.push(format!(
                "{locator} node count changed from {} to {}",
                snapshot.node_count,
                document.nodes.len()
            ));
        }

        let root_children = root_node
            .children
            .iter()
            .map(|child| child.node.as_str())
            .collect::<Vec<_>>();
        if root_children != snapshot.root_children {
            offenders.push(format!(
                "{locator} root `{}` children changed from {:?} to {:?}",
                snapshot.root_node, snapshot.root_children, root_children
            ));
        }
    }

    assert_eq!(
        checked_assets,
        L4_SHELL_TOPOLOGY_SNAPSHOTS.len(),
        "L4 Workbench shell topology snapshots should be evaluated"
    );
    assert!(
        offenders.is_empty(),
        "L4 Workbench shell topology must stay stable across primitive-composition refactors: {offenders:#?}"
    );
}

#[test]
fn workbench_shell_uses_tokenized_viewport_first_region_constraints() {
    let editor_root = editor_asset_root();
    let window = super::support::load_zui_document(
        &editor_root.join("ui/editor/windows/workbench_window.zui"),
    );
    let main_band = super::support::load_zui_document(
        &editor_root.join("ui/editor/components/workbench/shell/workbench_main_band.zui"),
    );
    let component_drawer = super::support::load_zui_document(
        &editor_root.join("ui/editor/components/workbench/shell/workbench_component_drawer.zui"),
    );
    let status_bar = super::support::load_zui_document(
        &editor_root.join("ui/editor/components/workbench/shell/workbench_status_bar.zui"),
    );
    let scene_tree = super::support::load_zui_document(
        &editor_root.join("ui/editor/components/workbench/shell/workbench_scene_tree_panel.zui"),
    );
    let viewport = super::support::load_zui_document(
        &editor_root.join("ui/editor/components/workbench/shell/workbench_viewport_panel.zui"),
    );

    assert_fixed_token_axis(
        node_axis(window, "status_bar", "height"),
        "$editor.chrome.status_bar.height",
    );
    assert_stretch_axis(node_axis(window, "main_band", "height"), 100, 4.0);
    assert_auto_slot(child_linear_slot(window, "window_content", "top_toolbar"));
    assert_linear_slot(
        child_linear_slot(window, "window_content", "main_band"),
        "Stretch",
        4.0,
    );
    assert_auto_slot(child_linear_slot(window, "window_content", "status_bar"));
    let bottom_drawer = node_axis(window, "component_drawer_shell", "height");
    assert_eq!(
        token_axis_value(bottom_drawer, "min"),
        "$editor.chrome.panel_header.height"
    );
    assert_eq!(
        token_axis_value(bottom_drawer, "preferred"),
        "$editor.density.bottom_output_height"
    );
    assert_eq!(
        token_axis_value(bottom_drawer, "max"),
        "$editor.density.bottom_output_height"
    );
    assert_eq!(axis_string(bottom_drawer, "stretch"), "Fixed");
    assert_eq!(axis_i64(bottom_drawer, "priority"), 20);
    assert_linear_slot(
        child_linear_slot(window, "window_content", "component_drawer_shell"),
        "StretchContent",
        1.0,
    );

    assert_eq!(
        axis_string(
            node_axis(component_drawer, "component_drawer", "height"),
            "stretch"
        ),
        "Stretch"
    );
    assert_fixed_token_axis(
        node_axis(component_drawer, "drawer_tabs", "height"),
        "$editor.chrome.panel_header.height",
    );
    for tab in ["drawer_tab_components", "drawer_tab_console"] {
        assert_fixed_token_axis(
            node_axis(component_drawer, tab, "height"),
            "$editor.chrome.panel_header.height",
        );
    }
    assert_fixed_token_axis(
        node_axis(status_bar, "status_bar", "height"),
        "$editor.chrome.status_bar.height",
    );
    for control in [
        "status_task_label",
        "status_grid",
        "status_snap",
        "status_snap_icon",
        "status_world_icon",
        "status_target_icon",
        "status_zoom",
    ] {
        assert_fixed_token_axis(
            node_axis(status_bar, control, "height"),
            "$editor.chrome.status_bar.height",
        );
    }
    assert_eq!(
        axis_string(
            node_axis(scene_tree, "scene_tree_panel", "width"),
            "stretch"
        ),
        "Stretch"
    );
    for control in ["scene_tabs", "scene_tab_scene", "scene_tab_layers"] {
        assert_fixed_token_axis(
            node_axis(scene_tree, control, "height"),
            "$editor.chrome.panel_header.height",
        );
    }
    assert_fixed_token_axis(
        node_axis(viewport, "document_tabs", "height"),
        "$editor.chrome.document_header.height",
    );
    for control in [
        "viewport_toolbar",
        "viewport_mode",
        "viewport_lit",
        "viewport_angle",
        "viewport_speed",
    ] {
        assert_fixed_token_axis(
            node_axis(viewport, control, "height"),
            "$editor.chrome.viewport_toolbar.height",
        );
    }

    let left_drawer = node_axis(main_band, "left_drawer_shell", "width");
    assert_fixed_token_axis(
        node_axis(main_band, "activity_rail", "width"),
        "$editor.chrome.activity_rail.width",
    );
    assert_eq!(
        axis_i64(node_axis(main_band, "activity_rail", "width"), "priority"),
        80
    );
    assert_auto_slot(child_linear_slot(
        main_band,
        "scene_workspace",
        "activity_rail",
    ));
    assert_eq!(
        token_axis_value(left_drawer, "min"),
        "$editor.density.compact_side_min_width"
    );
    assert_eq!(
        token_axis_value(left_drawer, "preferred"),
        "$editor.density.left_drawer_width"
    );
    assert_eq!(
        token_axis_value(left_drawer, "max"),
        "$editor.density.compact_left_drawer_max_width"
    );
    assert_eq!(axis_string(left_drawer, "stretch"), "Fixed");
    assert_eq!(axis_i64(left_drawer, "priority"), 50);
    assert_linear_slot(
        child_linear_slot(main_band, "scene_workspace", "left_drawer_shell"),
        "StretchContent",
        1.0,
    );

    let viewport = node_axis(main_band, "viewport_panel", "width");
    assert_stretch_axis(viewport, 100, 4.0);
    assert_linear_slot(
        child_linear_slot(main_band, "scene_workspace", "viewport_panel"),
        "Stretch",
        4.0,
    );

    let right_drawer = node_axis(main_band, "right_drawer_shell", "width");
    assert_eq!(
        token_axis_value(right_drawer, "min"),
        "$editor.density.compact_side_min_width"
    );
    assert_eq!(
        token_axis_value(right_drawer, "preferred"),
        "$editor.density.right_drawer_width"
    );
    assert_eq!(
        token_axis_value(right_drawer, "max"),
        "$editor.density.right_drawer_width"
    );
    assert_eq!(axis_string(right_drawer, "stretch"), "Fixed");
    assert_eq!(axis_i64(right_drawer, "priority"), 40);
    assert_linear_slot(
        child_linear_slot(main_band, "scene_workspace", "right_drawer_shell"),
        "StretchContent",
        1.0,
    );

    assert!(
        axis_i64(viewport, "priority") > axis_i64(left_drawer, "priority")
            && axis_i64(left_drawer, "priority") > axis_i64(right_drawer, "priority"),
        "viewport must retain layout budget before primary and auxiliary drawers"
    );
}

#[test]
fn workbench_toolbar_groups_use_content_sized_priority_constraints() {
    let editor_root = editor_asset_root();
    let toolbar = super::support::load_zui_document(
        &editor_root.join("ui/editor/components/workbench/shell/workbench_top_toolbar.zui"),
    );

    for (node_id, priority) in [
        ("toolbar_file_group", 90),
        ("toolbar_module_commands", 60),
        ("toolbar_tool_group", 40),
        ("toolbar_run_group", 100),
        ("toolbar_layout_group", 20),
    ] {
        let width = node_axis(toolbar, node_id, "width");
        let min = axis_f64(width, "min");
        let preferred = axis_f64(width, "preferred");
        let max = axis_f64(width, "max");
        assert_eq!(axis_string(width, "stretch"), "Stretch");
        assert_eq!(axis_i64(width, "priority"), priority);
        assert!(
            min <= preferred && preferred <= max,
            "toolbar group `{node_id}` should declare ordered min/preferred/max constraints"
        );
        assert!(
            min < preferred || preferred < max,
            "toolbar group `{node_id}` must not pin its complete content width"
        );

        let slot = child_linear_slot(toolbar, "toolbar_command_row", node_id);
        assert_eq!(axis_string(slot, "rule"), "StretchContent");
        assert_eq!(axis_f64(slot, "value"), 1.0);
        assert_eq!(axis_f64(slot, "shrink_value"), 1.0);
    }
}

fn node_axis<'a>(document: &'a UiV2AssetDocument, node_id: &str, axis: &str) -> &'a toml::Table {
    document.nodes[node_id]
        .layout
        .as_ref()
        .and_then(|layout| layout.get(axis))
        .and_then(Value::as_table)
        .unwrap_or_else(|| panic!("node `{node_id}` should declare layout.{axis}"))
}

fn child_linear_slot<'a>(
    document: &'a UiV2AssetDocument,
    parent_id: &str,
    child_id: &str,
) -> &'a toml::Table {
    document.nodes[parent_id]
        .children
        .iter()
        .find(|child| child.node == child_id)
        .unwrap_or_else(|| panic!("node `{parent_id}` should mount child `{child_id}`"))
        .slot
        .get("layout")
        .and_then(Value::as_table)
        .and_then(|layout| layout.get("linear_size"))
        .and_then(Value::as_table)
        .unwrap_or_else(|| {
            panic!("node `{parent_id}` child `{child_id}` should declare slot.layout.linear_size")
        })
}

fn assert_linear_slot(linear_size: &toml::Table, rule: &str, value: f64) {
    assert_eq!(axis_string(linear_size, "rule"), rule);
    assert_eq!(axis_f64(linear_size, "value"), value);
    assert!(!linear_size.contains_key("min") && !linear_size.contains_key("max"));
}

fn assert_auto_slot(linear_size: &toml::Table) {
    assert_eq!(axis_string(linear_size, "rule"), "Auto");
    assert_eq!(axis_f64(linear_size, "shrink_value"), 0.0);
    assert!(!linear_size.contains_key("min") && !linear_size.contains_key("max"));
}

fn assert_fixed_token_axis(axis: &toml::Table, token: &str) {
    for key in ["min", "preferred", "max"] {
        assert_eq!(token_axis_value(axis, key), token);
    }
    assert_eq!(axis_string(axis, "stretch"), "Fixed");
}

fn assert_stretch_axis(axis: &toml::Table, priority: i64, weight: f64) {
    assert_eq!(axis_string(axis, "stretch"), "Stretch");
    assert_eq!(axis_i64(axis, "priority"), priority);
    assert_eq!(axis_f64(axis, "weight"), weight);
}

fn token_axis_value<'a>(axis: &'a toml::Table, key: &str) -> &'a str {
    axis.get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("axis should declare token-valued `{key}`"))
}

fn axis_string<'a>(axis: &'a toml::Table, key: &str) -> &'a str {
    axis.get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("axis should declare string `{key}`"))
}

fn axis_i64(axis: &toml::Table, key: &str) -> i64 {
    axis.get(key)
        .and_then(Value::as_integer)
        .unwrap_or_else(|| panic!("axis should declare integer `{key}`"))
}

fn axis_f64(axis: &toml::Table, key: &str) -> f64 {
    axis.get(key)
        .and_then(Value::as_float)
        .unwrap_or_else(|| panic!("axis should declare float `{key}`"))
}
