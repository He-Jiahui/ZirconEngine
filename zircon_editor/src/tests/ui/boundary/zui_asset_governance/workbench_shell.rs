use super::support::{collect_zui_files, editor_asset_root, resource_locator_for_path};
use toml::Value;
use zircon_runtime::ui::v2::UiV2PrototypeStoreFileCache;
use zircon_runtime_interface::ui::v2::UiV2AssetDocument;
use zircon_runtime_interface::ui::v2::UiV2AssetKind;

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
    component_name: Option<&'static str>,
    document_root: Option<&'static str>,
    root_node: &'static str,
    root_component: &'static str,
    root_control_id: &'static str,
    node_count: usize,
    root_children: &'static [&'static str],
}

const L4_SHELL_TOPOLOGY_SNAPSHOTS: &[L4ShellTopologySnapshot] = &[
    L4ShellTopologySnapshot {
        file_name: "workbench_activity_rail.zui",
        component_name: None,
        document_root: Some("activity_rail_root"),
        root_node: "activity_rail_root",
        root_component: "Overlay",
        root_control_id: "ActivityRailRoot",
        node_count: 6,
        root_children: &[
            "activity_rail_panel",
            "activity_rail_button_0",
            "activity_rail_button_icon_0",
            "activity_rail_button_1",
            "activity_rail_button_icon_1",
        ],
    },
    L4ShellTopologySnapshot {
        file_name: "component_showcase.zui",
        component_name: None,
        document_root: Some("component_showcase_root"),
        root_node: "component_showcase_root",
        root_component: "VerticalBox",
        root_control_id: "UiComponentShowcaseRoot",
        node_count: 12,
        root_children: &[
            "component_showcase_header",
            "component_showcase_toolbar",
            "component_showcase_body",
            "component_showcase_bottom_log",
        ],
    },
    L4ShellTopologySnapshot {
        file_name: "unreal/components/details.zui",
        component_name: Some("UnrealDetailsPanel"),
        document_root: None,
        root_node: "panel",
        root_component: "VerticalGroup",
        root_control_id: "WorkbenchInspectorPanel",
        node_count: 33,
        root_children: &["tab", "summary", "add", "search", "content"],
    },
    L4ShellTopologySnapshot {
        file_name: "unreal/level_editor_window.zui",
        component_name: None,
        document_root: Some("root"),
        root_node: "main_band",
        root_component: "HorizontalGroup",
        root_control_id: "WorkbenchWindowMainBandRegion",
        node_count: 12,
        root_children: &["viewport", "right_dock"],
    },
    L4ShellTopologySnapshot {
        file_name: "unreal/components/outliner.zui",
        component_name: Some("UnrealOutlinerPanel"),
        document_root: None,
        root_node: "panel",
        root_component: "VerticalGroup",
        root_control_id: "WorkbenchSceneTreePanel",
        node_count: 22,
        root_children: &["tab", "content"],
    },
    L4ShellTopologySnapshot {
        file_name: "unreal/components/status_bar.zui",
        component_name: Some("UnrealEditorStatusBar"),
        document_root: None,
        root_node: "root",
        root_component: "HorizontalGroup",
        root_control_id: "UnrealEditorStatusBar",
        node_count: 16,
        root_children: &[
            "drawer_toggle",
            "ready",
            "errors",
            "warnings",
            "messages",
            "fill",
            "grid",
            "snap",
            "snap_toggle",
            "world",
            "target",
            "zoom",
            "task",
        ],
    },
    L4ShellTopologySnapshot {
        file_name: "unreal/components/toolbar.zui",
        component_name: Some("UnrealLevelEditorToolbar"),
        document_root: None,
        root_node: "toolbar",
        root_component: "HorizontalBox",
        root_control_id: "WorkbenchWindowTopToolbar",
        node_count: 13,
        root_children: &["command_scroll", "settings"],
    },
    L4ShellTopologySnapshot {
        file_name: "unreal/components/viewport.zui",
        component_name: Some("UnrealLevelViewport"),
        document_root: None,
        root_node: "root",
        root_component: "VerticalGroup",
        root_control_id: "UnrealLevelViewport",
        node_count: 16,
        root_children: &["viewport"],
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

    // Surviving module leaves retain the original Workbench composition gate.
    // Current Unreal/native surfaces compile via the existing cache owner.
    let mut cache = UiV2PrototypeStoreFileCache::new();
    for snapshot in L4_SHELL_TOPOLOGY_SNAPSHOTS {
        let path = editor_root.join("ui/editor").join(snapshot.file_name);
        let outcome = cache
            .load_store(std::iter::once(path))
            .unwrap_or_else(|error| {
                panic!(
                    "current L4 surface {} must compile: {error}",
                    snapshot.file_name
                )
            });
        assert!(
            outcome.unresolved_imports.is_empty(),
            "{} must have a complete native import closure",
            snapshot.file_name
        );
        assert_eq!(
            outcome.root_document.asset.kind,
            if snapshot.component_name.is_some() {
                UiV2AssetKind::Component
            } else {
                UiV2AssetKind::View
            }
        );
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
    let shell_root = editor_root.join("ui/editor");
    let mut checked_assets = 0usize;
    let mut offenders = Vec::new();

    for snapshot in L4_SHELL_TOPOLOGY_SNAPSHOTS {
        checked_assets += 1;
        let path = shell_root.join(snapshot.file_name);
        let locator = resource_locator_for_path(&editor_root, &path);
        let document = super::support::load_zui_document(&path);

        if let Some(component_name) = snapshot.component_name {
            assert_eq!(document.asset.kind, UiV2AssetKind::Component);
            let component = document.components.get(component_name).unwrap_or_else(|| {
                panic!("{locator} should declare shell component {component_name}")
            });
            assert_eq!(
                document.components.len(),
                1,
                "{locator} must remain one component prototype"
            );
            assert_eq!(
                component.root, snapshot.root_node,
                "{locator} component root changed"
            );
        } else {
            assert_eq!(document.asset.kind, UiV2AssetKind::View);
            assert!(
                document.components.is_empty(),
                "{locator} must remain a real View"
            );
            assert_eq!(
                document.root_node_id(),
                snapshot.document_root,
                "{locator} document root changed"
            );
        }

        let Some(root_node) = document.nodes.get(snapshot.root_node) else {
            offenders.push(format!(
                "{locator} component `{:?}` references missing root `{}`",
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
    let editor_root = editor_asset_root().join("ui/editor");
    let window =
        super::support::load_zui_document(&editor_root.join("unreal/level_editor_window.zui"));
    let toolbar =
        super::support::load_zui_document(&editor_root.join("unreal/components/toolbar.zui"));
    let drawer = super::support::load_zui_document(
        &editor_root.join("unreal/components/content_drawer.zui"),
    );
    let status =
        super::support::load_zui_document(&editor_root.join("unreal/components/status_bar.zui"));
    let outliner =
        super::support::load_zui_document(&editor_root.join("unreal/components/outliner.zui"));
    let details =
        super::support::load_zui_document(&editor_root.join("unreal/components/details.zui"));
    let viewport =
        super::support::load_zui_document(&editor_root.join("unreal/components/viewport.zui"));

    assert_eq!(window.asset.kind, UiV2AssetKind::View);
    assert_eq!(window.root_node_id(), Some("root"));
    assert!(window.components.is_empty(), "main_band is a window node");
    assert_eq!(window.nodes["main_band"].component, "HorizontalGroup");
    assert_eq!(
        window.nodes["main_band"].control_id.as_deref(),
        Some("WorkbenchWindowMainBandRegion")
    );
    assert_eq!(
        window.nodes["editor_content"]
            .children
            .iter()
            .map(|child| child.node.as_str())
            .collect::<Vec<_>>(),
        ["menu_bar", "toolbar", "main_band", "status_bar"]
    );
    assert_eq!(
        window.nodes["root"]
            .children
            .iter()
            .map(|child| child.node.as_str())
            .collect::<Vec<_>>(),
        ["editor_content", "content_drawer", "menu_popups"]
    );
    for node in ["menu_bar", "toolbar", "status_bar"] {
        assert_auto_slot(child_linear_slot(window, "editor_content", node));
    }
    assert_linear_slot(
        child_linear_slot(window, "editor_content", "main_band"),
        "Stretch",
        1.0,
    );
    assert_eq!(
        axis_string(node_axis(window, "main_band", "height"), "stretch"),
        "Stretch"
    );
    assert_eq!(
        axis_string(node_axis(window, "viewport", "width"), "stretch"),
        "Stretch"
    );
    assert_eq!(axis_f64(node_axis(window, "viewport", "width"), "min"), 0.0);
    assert_eq!(
        axis_string(node_axis(window, "right_dock", "width"), "stretch"),
        "Stretch"
    );
    assert_eq!(
        axis_f64(node_axis(window, "right_dock", "width"), "min"),
        240.0
    );
    assert_eq!(
        window.nodes["main_band"]
            .children
            .iter()
            .map(|child| child.node.as_str())
            .collect::<Vec<_>>(),
        ["viewport", "right_dock"]
    );
    assert_linear_slot(
        child_linear_slot(window, "main_band", "viewport"),
        "Stretch",
        1.0,
    );
    assert_linear_slot(
        child_linear_slot(window, "main_band", "right_dock"),
        "Stretch",
        0.25,
    );
    assert!(
        axis_f64(child_linear_slot(window, "main_band", "viewport"), "value")
            > axis_f64(
                child_linear_slot(window, "main_band", "right_dock"),
                "value"
            ),
        "viewport must retain more layout budget than the auxiliary dock"
    );
    assert_linear_slot(
        child_linear_slot(window, "right_dock", "outliner"),
        "Stretch",
        0.4,
    );
    assert_linear_slot(
        child_linear_slot(window, "right_dock", "details"),
        "Stretch",
        1.0,
    );
    for node in ["outliner", "details"] {
        assert_eq!(axis_f64(node_axis(window, node, "height"), "min"), 96.0);
        assert_eq!(
            axis_string(node_axis(window, node, "height"), "stretch"),
            "Stretch"
        );
    }
    assert_fixed_numeric_axis(node_axis(window, "menu_bar", "height"), 26.0);
    assert_fixed_numeric_axis(node_axis(toolbar, "toolbar", "height"), 48.0);
    assert_fixed_numeric_axis(node_axis(window, "status_bar", "height"), 32.0);
    assert_fixed_numeric_axis(node_axis(status, "root", "height"), 32.0);
    for node in [
        "ready",
        "errors",
        "warnings",
        "messages",
        "grid",
        "snap",
        "snap_toggle",
        "world",
        "target",
        "zoom",
    ] {
        assert_eq!(
            axis_string(node_axis(status, node, "height"), "stretch"),
            "Stretch",
            "status controls must share their current region height"
        );
    }
    for (document, root) in [(outliner, "panel"), (details, "panel"), (viewport, "root")] {
        assert_eq!(
            axis_string(node_axis(document, root, "width"), "stretch"),
            "Stretch"
        );
        assert_eq!(
            axis_string(node_axis(document, root, "height"), "stretch"),
            "Stretch"
        );
    }
    assert_fixed_token_axis(
        node_axis(outliner, "tab", "height"),
        "$editor.chrome.panel_header.height",
    );
    assert_fixed_token_axis(
        node_axis(details, "tab", "height"),
        "$editor.chrome.panel_header.height",
    );
    assert_fixed_token_axis(
        node_axis(viewport, "toolbar", "height"),
        "$editor.chrome.viewport_toolbar.height",
    );
    assert_fixed_token_axis(
        node_axis(drawer, "drawer", "height"),
        "$editor.density.bottom_output_height",
    );
    assert_eq!(
        drawer.nodes["drawer"]
            .props
            .get("visible")
            .and_then(Value::as_bool),
        Some(false)
    );
    assert_eq!(
        drawer.nodes["drawer"]
            .props
            .get("visibility")
            .and_then(Value::as_str),
        Some("collapsed")
    );
    assert_eq!(
        window.nodes["content_drawer"].component,
        "UnrealContentDrawer"
    );
    assert_eq!(
        window.nodes["menu_popups"].component,
        "UnrealLevelEditorMenuPopups"
    );
    assert_eq!(
        window.nodes["content_drawer"]
            .layout
            .as_ref()
            .unwrap()
            .get("z_index")
            .and_then(Value::as_integer),
        Some(160)
    );
    assert_eq!(
        window.nodes["menu_popups"]
            .layout
            .as_ref()
            .unwrap()
            .get("z_index")
            .and_then(Value::as_integer),
        Some(180)
    );
    for node in [
        "root",
        "main_band",
        "viewport",
        "right_dock",
        "outliner",
        "details",
    ] {
        assert_eq!(
            window.nodes[node]
                .layout
                .as_ref()
                .unwrap()
                .get("clip")
                .and_then(Value::as_bool),
            Some(true),
            "current region {node} must retain its authored clip owner"
        );
    }
}

#[test]
fn workbench_toolbar_groups_preserve_content_inside_horizontal_scroll() {
    let toolbar = super::support::load_zui_document(
        &editor_asset_root().join("ui/editor/unreal/components/toolbar.zui"),
    );
    let row = &toolbar.nodes["command_scroll"];
    assert_eq!(row.component, "ScrollableBox");
    assert_eq!(
        row.control_id.as_deref(),
        Some("WorkbenchToolbarCommandRow")
    );
    let layout = row.layout.as_ref().expect("command scroll layout");
    assert_eq!(layout.get("clip").and_then(Value::as_bool), Some(true));
    let container = layout
        .get("container")
        .and_then(Value::as_table)
        .expect("command scroll container");
    assert_eq!(axis_string(container, "kind"), "ScrollableBox");
    assert_eq!(axis_string(container, "axis"), "Horizontal");
    assert_eq!(axis_string(container, "scrollbar_visibility"), "Never");
    assert_eq!(
        axis_string(node_axis(toolbar, "command_scroll", "width"), "stretch"),
        "Stretch"
    );
    assert_eq!(
        axis_f64(node_axis(toolbar, "command_scroll", "width"), "min"),
        0.0
    );
    assert_eq!(
        row.children
            .iter()
            .map(|child| child.node.as_str())
            .collect::<Vec<_>>(),
        [
            "save",
            "find_content",
            "modes",
            "add",
            "blueprints",
            "cinematics",
            "play",
            "pause",
            "stop",
            "play_options"
        ]
    );
    for child in &row.children {
        let width = node_axis(toolbar, &child.node, "width");
        let min = axis_f64(width, "min");
        let preferred = axis_f64(width, "preferred");
        let max = axis_f64(width, "max");
        assert_eq!(axis_string(width, "stretch"), "Fixed");
        assert!(
            min > 0.0 && min <= preferred && preferred <= max,
            "scrollable command {} must retain its authored content width",
            child.node
        );
        assert_fixed_numeric_axis(node_axis(toolbar, &child.node, "height"), 32.0);
    }
    assert_eq!(
        toolbar.nodes["toolbar"]
            .children
            .iter()
            .map(|child| child.node.as_str())
            .collect::<Vec<_>>(),
        ["command_scroll", "settings"],
        "Settings remains outside the scrollable command row"
    );
    assert_fixed_numeric_axis(node_axis(toolbar, "settings", "height"), 32.0);
    use crate::ui::retained_host::callback_dispatch::template_bridge::BuiltinWorkbenchWindowTemplateSurfaceBridge;
    use zircon_runtime_interface::ui::layout::UiSize;
    for width in [480.0, 900.0, 1672.0] {
        let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(width, 941.0))
            .expect("current Unreal window must load through the production bridge");
        let save = bridge
            .control_frame("WorkbenchToolbarSave")
            .expect("current Save control must be visible");
        assert_eq!(
            save.width, 32.0,
            "horizontal scrolling must keep the fixed Save width at {width}"
        );
        assert_eq!(save.height, 32.0);
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

fn assert_fixed_numeric_axis(axis: &toml::Table, value: f64) {
    for key in ["min", "preferred", "max"] {
        assert_eq!(axis_f64(axis, key), value);
    }
    assert_eq!(axis_string(axis, "stretch"), "Fixed");
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
