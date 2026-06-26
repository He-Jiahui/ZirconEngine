use std::fs;

use zircon_runtime::ui::v2::UiZuiAssetLoader;

use super::support::{collect_zui_files, editor_asset_root, resource_locator_for_path};

const L4_SHELL_STRUCTURAL_COMPONENTS: &[&str] = &[
    "Container",
    "DocumentTabs",
    "HorizontalGroup",
    "Overlay",
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
        node_count: 15,
        root_children: &[
            "status_ready",
            "status_errors",
            "status_warnings",
            "status_messages",
            "status_fill",
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
        node_count: 39,
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
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
        let document = UiZuiAssetLoader::load_zui_str(&source)
            .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

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
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
        let document = UiZuiAssetLoader::load_zui_str(&source)
            .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));

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
