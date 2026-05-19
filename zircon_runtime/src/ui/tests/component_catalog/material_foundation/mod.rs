use std::{collections::BTreeSet, fs, path::Path};

use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::{
    UiComponentDescriptorKind, UiComponentEventKind, UiComponentLayoutRole, UiHostCapability,
    UiHostCapabilitySet, UiRenderCapability, UiValue, UiValueKind,
};
use zircon_runtime_interface::ui::style::{
    ButtonColor, ButtonIconPlacement, ButtonSize, ButtonVariant,
};

use super::{assert_has_event, assert_has_prop};

mod inputs;
mod selection_inputs;

#[test]
fn material_editor_foundation_catalog_covers_planned_component_layers() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let expected = [
        "Accordion",
        "AgentChat",
        "Alert",
        "AppBar",
        "AssetGrid",
        "AssetList",
        "Autocomplete",
        "Avatar",
        "AvatarGroup",
        "Backdrop",
        "Badge",
        "BarChart",
        "BottomNavigation",
        "Box",
        "Breadcrumbs",
        "Button",
        "ButtonGroup",
        "Card",
        "CategorizedList",
        "Charts",
        "ChatComposer",
        "ChatConversationList",
        "ChatMessageList",
        "Checkbox",
        "Chip",
        "ClickAwayListener",
        "Collapse",
        "Composite",
        "Container",
        "ContextMenu",
        "CssBaseline",
        "DataGrid",
        "DateTimePickers",
        "Dialog",
        "Divider",
        "DockHost",
        "DocumentNode",
        "Drawer",
        "Dropdown",
        "Fade",
        "FieldEditor",
        "FilterBar",
        "FlexGroup",
        "FloatingActionButton",
        "FloatingWindow",
        "FolderTree",
        "Gauge",
        "GizmoControls",
        "GraphCanvas",
        "Grid",
        "GridGroup",
        "Grow",
        "HorizontalGroup",
        "Icon",
        "IconButton",
        "ImageList",
        "InitColorSchemeScript",
        "Input",
        "InspectorSection",
        "LineChart",
        "Link",
        "List",
        "ListView",
        "Masonry",
        "MaterialTreeView",
        "Menu",
        "Menubar",
        "MetadataPane",
        "Modal",
        "NoSsr",
        "NumberField",
        "Overlay",
        "Pagination",
        "Panel",
        "PaneToolbar",
        "Paper",
        "PieChart",
        "Popover",
        "Popper",
        "Portal",
        "PreviewPane",
        "Progress",
        "PropertyGrid",
        "Radio",
        "Rating",
        "ScrollView",
        "Scrollbar",
        "SearchField",
        "Select",
        "SeverityChips",
        "Skeleton",
        "Slider",
        "Slide",
        "Slot",
        "Snackbar",
        "SourceEditor",
        "SparkLineChart",
        "SpeedDial",
        "Splitter",
        "Stack",
        "StatusActionControls",
        "Stepper",
        "SvgIcon",
        "Switch",
        "TabStack",
        "Table",
        "Tabs",
        "TextField",
        "TextareaAutosize",
        "Timeline",
        "ToggleButton",
        "ToggleButtonGroup",
        "Tooltip",
        "TransferList",
        "TreeView",
        "Typography",
        "UseMediaQuery",
        "VerticalGroup",
        "View",
        "ViewTab",
        "ViewportHost",
        "VirtualList",
        "VisualDesigner",
        "Window",
        "WindowFrame",
        "WorkbenchShell",
        "Zoom",
    ];

    assert_eq!(registry.len(), expected.len());
    assert_eq!(
        registry.component_ids().collect::<BTreeSet<_>>(),
        expected.iter().copied().collect::<BTreeSet<_>>()
    );

    for descriptor in registry.descriptors() {
        for class in ["material_dark", "material", "material-dark"] {
            assert!(
                descriptor
                    .default_classes
                    .iter()
                    .any(|value| value == class),
                "{} missing default class {class}",
                descriptor.id
            );
        }
        for state_name in [
            "hovered", "pressed", "selected", "disabled", "focused", "error", "warning",
        ] {
            assert!(
                descriptor.state_prop(state_name).is_some(),
                "{} missing state {state_name}",
                descriptor.id
            );
        }
        assert!(descriptor.prop("density").is_some());
    }

    let tree_view = registry
        .descriptor("TreeView")
        .expect("TreeView descriptor");
    assert_has_prop(tree_view, "query");
    assert_has_prop(tree_view, "expanded");
    assert_has_event(tree_view, UiComponentEventKind::ToggleExpanded);
    assert_has_event(tree_view, UiComponentEventKind::Commit);

    let property_grid = registry
        .descriptor("PropertyGrid")
        .expect("PropertyGrid descriptor");
    assert_has_prop(property_grid, "selection_summary");

    let inspector_section = registry
        .descriptor("InspectorSection")
        .expect("InspectorSection descriptor");
    assert_has_prop(inspector_section, "text");
    assert_has_prop(inspector_section, "expanded");
    assert_has_event(inspector_section, UiComponentEventKind::ToggleExpanded);

    let drawer = registry.descriptor("Drawer").expect("Drawer descriptor");
    assert_has_prop(drawer, "slot");
    assert_has_prop(drawer, "mode");
    assert_has_prop(drawer, "active_view");
    assert_has_event(drawer, UiComponentEventKind::SelectOption);

    let view = registry.descriptor("View").expect("View descriptor");
    assert_has_prop(view, "view_id");
    assert_has_prop(view, "dirty");
    assert_has_event(view, UiComponentEventKind::Focus);

    let button = registry.descriptor("Button").expect("Button descriptor");
    assert_button_style_schema(button, "none");

    let button_group = registry
        .descriptor("ButtonGroup")
        .expect("ButtonGroup descriptor");
    assert_eq!(
        button_group.descriptor_kind,
        UiComponentDescriptorKind::Composite
    );
    assert_enum_options(
        button_group,
        "button_group_orientation",
        &["horizontal", "vertical"],
    );
    assert_enum_options(
        button_group,
        "button_group_attached_radius",
        &["first", "middle", "last", "only"],
    );
    assert_has_prop(button_group, "button_group_segment_count");
    assert_has_prop(button_group, "button_group_disabled_propagates");
    for (prop, expected) in [
        ("button_variant", "contained"),
        ("button_color", "primary"),
        ("button_size", "medium"),
        ("icon_placement", "none"),
    ] {
        assert_eq!(
            button_group
                .default_props
                .iter()
                .find(|(name, _)| name == prop)
                .map(|(_, value)| value),
            Some(&UiValue::Enum(expected.to_string())),
            "ButtonGroup should declare child button default `{prop}`"
        );
    }
    let button_group_slot = button_group
        .slot_schema("buttons")
        .expect("ButtonGroup buttons slot");
    assert!(button_group_slot.multiple);
    assert!(
        !button_group.supports_event(UiComponentEventKind::SelectOption),
        "ButtonGroup stays structural; child buttons own selection/click routes"
    );
    assert!(
        !button_group.supports_event(UiComponentEventKind::Commit),
        "ButtonGroup stays structural; child buttons own Commit/click routes"
    );

    let icon_button = registry
        .descriptor("IconButton")
        .expect("IconButton descriptor");
    assert_button_style_schema(icon_button, "icon_only");

    let fab = registry
        .descriptor("FloatingActionButton")
        .expect("FloatingActionButton descriptor");
    assert_button_style_schema_with_variant_default(fab, "icon_only", "default");
    assert_enum_options(fab, "button_shape", &["circular", "extended", "pill"]);
    for (prop, expected) in [
        ("button_variant", "contained"),
        ("button_color", "primary"),
        ("button_size", "medium"),
        ("icon_placement", "icon_only"),
        ("button_shape", "circular"),
        ("surface_variant", "elevated"),
    ] {
        assert_eq!(
            fab.default_props
                .iter()
                .find(|(name, _)| name == prop)
                .map(|(_, value)| value),
            Some(&UiValue::Enum(expected.to_string())),
            "FloatingActionButton should declare default `{prop}`"
        );
    }
    for (prop, expected) in [
        ("corner_radius", 999.0),
        ("border_width", 0.0),
        ("elevation", 2.0),
    ] {
        assert_eq!(
            fab.default_props
                .iter()
                .find(|(name, _)| name == prop)
                .map(|(_, value)| value),
            Some(&UiValue::Float(expected)),
            "FloatingActionButton should declare default `{prop}`"
        );
    }
    assert_has_event(fab, UiComponentEventKind::Commit);
    assert!(fab
        .required_render_capabilities
        .contains(&UiRenderCapability::Vector));

    let text_field = registry
        .descriptor("TextField")
        .expect("TextField descriptor");
    assert_enum_options(text_field, "variant", &["outlined", "filled", "standard"]);
    for prop in [
        "value_text",
        "label",
        "placeholder",
        "helper_text",
        "multiline",
        "select_mode",
    ] {
        assert_has_prop(text_field, prop);
    }
    assert_eq!(
        text_field
            .default_props
            .iter()
            .find(|(name, _)| name == "variant")
            .map(|(_, value)| value),
        Some(&UiValue::Enum("outlined".to_string())),
        "TextField should default to outlined Material field styling"
    );
    for event in [
        UiComponentEventKind::Focus,
        UiComponentEventKind::ValueChanged,
        UiComponentEventKind::Commit,
    ] {
        assert_has_event(text_field, event);
    }
    assert!(text_field
        .required_host_capabilities
        .contains(&UiHostCapability::TextInput));

    let textarea = registry
        .descriptor("TextareaAutosize")
        .expect("TextareaAutosize descriptor");
    assert_enum_options(textarea, "variant", &["outlined", "filled", "standard"]);
    for prop in [
        "value_text",
        "placeholder",
        "helper_text",
        "multiline",
        "autosize",
        "min_rows",
        "max_rows",
    ] {
        assert_has_prop(textarea, prop);
    }
    for (prop, expected) in [("multiline", true), ("autosize", true)] {
        assert_eq!(
            textarea
                .default_props
                .iter()
                .find(|(name, _)| name == prop)
                .map(|(_, value)| value),
            Some(&UiValue::Bool(expected)),
            "TextareaAutosize should default `{prop}` to `{expected}`"
        );
    }
    for (prop, expected) in [("min_rows", 2), ("max_rows", 8)] {
        assert_eq!(
            textarea
                .default_props
                .iter()
                .find(|(name, _)| name == prop)
                .map(|(_, value)| value),
            Some(&UiValue::Int(expected)),
            "TextareaAutosize should default `{prop}` to `{expected}`"
        );
    }
    for event in [
        UiComponentEventKind::Focus,
        UiComponentEventKind::ValueChanged,
        UiComponentEventKind::Commit,
    ] {
        assert_has_event(textarea, event);
    }
    assert!(textarea
        .required_host_capabilities
        .contains(&UiHostCapability::TextInput));

    inputs::assert_descriptors(&registry);
    selection_inputs::assert_descriptors(&registry);

    let window = registry.descriptor("Window").expect("Window descriptor");
    assert_has_prop(window, "window_id");
    assert_has_prop(window, "dock_policy");
    assert_has_prop(window, "floating");
    assert_has_event(window, UiComponentEventKind::BeginDrag);

    let workbench_shell = registry
        .descriptor("WorkbenchShell")
        .expect("WorkbenchShell descriptor");
    assert_has_prop(workbench_shell, "skin_id");
    assert_has_prop(workbench_shell, "panel_preset_id");
    assert_has_prop(workbench_shell, "shell_preset_id");
    assert_has_prop(workbench_shell, "window_model_preset_id");

    let dock_host = registry
        .descriptor("DockHost")
        .expect("DockHost descriptor");
    assert_eq!(dock_host.descriptor_kind, UiComponentDescriptorKind::Layout);
    assert_eq!(dock_host.layout_role, UiComponentLayoutRole::EditorDock);
    assert!(dock_host
        .required_host_capabilities
        .contains(&UiHostCapability::Editor));

    let virtual_list = registry
        .descriptor("VirtualList")
        .expect("VirtualList descriptor");
    assert_eq!(virtual_list.layout_role, UiComponentLayoutRole::VirtualList);
    assert_has_prop(virtual_list, "item_count");
    assert_has_prop(virtual_list, "item_extent");
    assert_has_prop(virtual_list, "overscan");
    assert_has_event(virtual_list, UiComponentEventKind::SetVisibleRange);
    assert!(virtual_list
        .required_host_capabilities
        .contains(&UiHostCapability::VirtualizedLayout));
    assert!(virtual_list
        .required_render_capabilities
        .contains(&UiRenderCapability::VirtualizedLayout));

    let tree_view = registry
        .descriptor("TreeView")
        .expect("TreeView descriptor");
    assert_has_prop(tree_view, "query");
    assert_has_event(tree_view, UiComponentEventKind::ToggleExpanded);
    assert_has_event(tree_view, UiComponentEventKind::OpenPopupAt);

    let property_grid = registry
        .descriptor("PropertyGrid")
        .expect("PropertyGrid descriptor");
    assert_has_event(property_grid, UiComponentEventKind::ValueChanged);

    let search_field = registry
        .descriptor("SearchField")
        .expect("SearchField descriptor");
    assert_has_prop(search_field, "query");
    assert_has_event(search_field, UiComponentEventKind::ValueChanged);
    assert!(search_field
        .required_host_capabilities
        .contains(&UiHostCapability::TextInput));

    let field_editor = registry
        .descriptor("FieldEditor")
        .expect("FieldEditor descriptor");
    assert_has_prop(field_editor, "text");
    assert_has_prop(field_editor, "value_text");
    assert!(field_editor.slot_schema("field").is_some());

    let asset_grid = registry
        .descriptor("AssetGrid")
        .expect("AssetGrid descriptor");
    assert_has_prop(asset_grid, "item_count");
    assert_has_event(asset_grid, UiComponentEventKind::OpenReference);
    assert_has_event(asset_grid, UiComponentEventKind::LocateReference);

    let viewport_host = registry
        .descriptor("ViewportHost")
        .expect("ViewportHost descriptor");
    assert_eq!(
        viewport_host.descriptor_kind,
        UiComponentDescriptorKind::Layout
    );
    assert_eq!(viewport_host.layout_role, UiComponentLayoutRole::Canvas);
    assert!(viewport_host
        .required_host_capabilities
        .contains(&UiHostCapability::CanvasRender));
    assert!(viewport_host
        .required_render_capabilities
        .contains(&UiRenderCapability::Canvas));
    assert_has_event(viewport_host, UiComponentEventKind::SetWorldSurface);

    let graph_canvas = registry
        .descriptor("GraphCanvas")
        .expect("GraphCanvas descriptor");
    assert_eq!(graph_canvas.layout_role, UiComponentLayoutRole::Canvas);
    assert!(graph_canvas.slot_schema("nodes").is_some());
    assert!(graph_canvas.slot_schema("edges").is_some());
    assert_has_event(graph_canvas, UiComponentEventKind::DropHover);

    let source_editor = registry
        .descriptor("SourceEditor")
        .expect("SourceEditor descriptor");
    assert_has_prop(source_editor, "text");
    assert!(source_editor
        .required_host_capabilities
        .contains(&UiHostCapability::TextInput));

    let timeline = registry
        .descriptor("Timeline")
        .expect("Timeline descriptor");
    assert_has_prop(timeline, "time");
    assert_has_prop(timeline, "duration");
    assert_has_event(timeline, UiComponentEventKind::DragDelta);

    let drawer = registry.descriptor("Drawer").expect("Drawer descriptor");
    assert_has_prop(drawer, "slot");
    assert_has_prop(drawer, "mode");
    assert_has_prop(drawer, "active_view");
    assert_has_event(drawer, UiComponentEventKind::SelectOption);

    let view = registry.descriptor("View").expect("View descriptor");
    assert_has_prop(view, "view_id");
    assert_has_prop(view, "dirty");
    assert_has_event(view, UiComponentEventKind::Focus);

    let window = registry.descriptor("Window").expect("Window descriptor");
    assert_has_prop(window, "window_id");
    assert_has_prop(window, "dock_policy");
    assert_has_prop(window, "floating");
    assert_has_event(window, UiComponentEventKind::BeginDrag);

    let document_node = registry
        .descriptor("DocumentNode")
        .expect("DocumentNode descriptor");
    assert_eq!(
        document_node.descriptor_kind,
        UiComponentDescriptorKind::Layout
    );
    assert_eq!(document_node.layout_role, UiComponentLayoutRole::EditorDock);
    assert_has_prop(document_node, "node_kind");

    let tab_stack = registry
        .descriptor("TabStack")
        .expect("TabStack descriptor");
    assert_has_prop(tab_stack, "active_tab");
    assert!(tab_stack.slot_schema("tabs").is_some());
    assert!(tab_stack.slot_schema("content").is_some());
    assert_has_event(tab_stack, UiComponentEventKind::SelectOption);

    let floating_window = registry
        .descriptor("FloatingWindow")
        .expect("FloatingWindow descriptor");
    assert_has_prop(floating_window, "window_id");
    assert_has_prop(floating_window, "focused_view");
    assert_has_event(floating_window, UiComponentEventKind::Focus);
    assert_has_event(floating_window, UiComponentEventKind::BeginDrag);

    let workbench_shell = registry
        .descriptor("WorkbenchShell")
        .expect("WorkbenchShell descriptor");
    assert_has_prop(workbench_shell, "skin_id");
    assert_has_prop(workbench_shell, "panel_preset_id");
    assert_has_prop(workbench_shell, "shell_preset_id");
    assert_has_prop(workbench_shell, "window_model_preset_id");

    for component_id in [
        "ButtonGroup",
        "FloatingActionButton",
        "Select",
        "Autocomplete",
        "ToggleButtonGroup",
        "Rating",
        "Chip",
        "List",
        "Table",
        "Alert",
        "Dialog",
        "Popover",
        "Snackbar",
        "Accordion",
        "AppBar",
        "Card",
        "Paper",
        "Breadcrumbs",
        "BottomNavigation",
        "Pagination",
        "Stepper",
        "TransferList",
        "Box",
        "Container",
        "Grid",
        "Stack",
    ] {
        let descriptor = registry
            .descriptor(component_id)
            .unwrap_or_else(|| panic!("missing Material descriptor `{component_id}`"));
        assert_has_prop(descriptor, "surface_variant");
        assert_has_prop(descriptor, "corner_radius");
        assert_has_prop(descriptor, "border_width");
    }

    let material_tree = registry
        .descriptor("MaterialTreeView")
        .expect("MUI X Tree View descriptor");
    assert_has_prop(material_tree, "editable");
    assert_has_event(material_tree, UiComponentEventKind::ToggleExpanded);

    let data_grid = registry
        .descriptor("DataGrid")
        .expect("DataGrid descriptor");
    assert_eq!(data_grid.layout_role, UiComponentLayoutRole::VirtualList);
    assert_has_prop(data_grid, "columns");
    assert_has_prop(data_grid, "rows");
    assert_has_event(data_grid, UiComponentEventKind::SetVisibleRange);

    let date_time_pickers = registry
        .descriptor("DateTimePickers")
        .expect("DateTimePickers descriptor");
    assert_has_prop(date_time_pickers, "date_value");
    assert_has_prop(date_time_pickers, "time_value");
    assert_has_prop(date_time_pickers, "picker_mode");
    assert_has_event(date_time_pickers, UiComponentEventKind::OpenPopup);
    assert_has_event(date_time_pickers, UiComponentEventKind::ClosePopup);
    assert_has_event(date_time_pickers, UiComponentEventKind::Commit);

    for component_id in [
        "Charts",
        "LineChart",
        "BarChart",
        "PieChart",
        "SparkLineChart",
        "Gauge",
    ] {
        let descriptor = registry
            .descriptor(component_id)
            .unwrap_or_else(|| panic!("missing chart descriptor `{component_id}`"));
        assert_has_prop(descriptor, "series");
        assert_has_event(descriptor, UiComponentEventKind::Hover);
    }

    let agent_chat = registry
        .descriptor("AgentChat")
        .expect("AgentChat descriptor");
    assert_has_prop(agent_chat, "messages");
    assert_has_prop(agent_chat, "composer_text");
    assert_has_prop(agent_chat, "streaming");
    assert_has_event(agent_chat, UiComponentEventKind::Commit);

    let editor_visible = registry.descriptors_for_host(&UiHostCapabilitySet::editor_authoring());
    assert_eq!(editor_visible.len(), expected.len());
    let runtime_visible_ids = registry
        .descriptors_for_host(&UiHostCapabilitySet::runtime_basic())
        .into_iter()
        .map(|descriptor| descriptor.id.as_str())
        .collect::<BTreeSet<_>>();
    assert!(!runtime_visible_ids.contains("DockHost"));
    assert!(!runtime_visible_ids.contains("WorkbenchShell"));
}

fn assert_button_style_schema(
    descriptor: &zircon_runtime_interface::ui::component::UiComponentDescriptor,
    expected_icon_placement: &str,
) {
    assert_button_style_schema_with_variant_default(descriptor, expected_icon_placement, "default");
}

pub(super) fn assert_button_style_schema_with_variant_default(
    descriptor: &zircon_runtime_interface::ui::component::UiComponentDescriptor,
    expected_icon_placement: &str,
    expected_variant_default: &str,
) {
    assert_enum_options(descriptor, "button_variant", &ButtonVariant::OPTIONS);
    assert_enum_options(descriptor, "button_color", &ButtonColor::OPTIONS);
    assert_enum_options(descriptor, "button_size", &ButtonSize::OPTIONS);
    assert_enum_options(descriptor, "icon_placement", &ButtonIconPlacement::OPTIONS);
    assert_eq!(
        descriptor.prop("button_variant").unwrap().default_value,
        Some(UiValue::Enum(expected_variant_default.to_string()))
    );
    assert_eq!(
        descriptor.prop("button_variant").unwrap().value_kind,
        UiValueKind::Enum
    );
    assert_eq!(
        descriptor.prop("icon_placement").unwrap().default_value,
        Some(UiValue::Enum(expected_icon_placement.to_string()))
    );
}

pub(super) fn assert_enum_options(
    descriptor: &zircon_runtime_interface::ui::component::UiComponentDescriptor,
    name: &str,
    expected: &[&str],
) {
    let schema = descriptor
        .prop(name)
        .unwrap_or_else(|| panic!("{} missing prop `{name}`", descriptor.id));
    assert_eq!(schema.value_kind, UiValueKind::Enum);
    assert_eq!(
        schema
            .options
            .iter()
            .map(|option| option.id.as_str())
            .collect::<Vec<_>>(),
        expected
    );
}

#[test]
fn material_editor_foundation_catalog_stays_folder_backed_by_family() {
    let catalog_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/component/catalog");
    assert!(
        !catalog_root.join("material_foundation.rs").exists(),
        "Material foundation catalog should stay split by component family"
    );

    let foundation_root = catalog_root.join("material_foundation");
    let expected_modules = [
        "mod.rs",
        "shared.rs",
        "inputs.rs",
        "selection_inputs.rs",
        "text_inputs.rs",
        "data_display.rs",
        "feedback.rs",
        "surfaces.rs",
        "navigation.rs",
        "layout.rs",
        "mui_x.rs",
    ];
    let actual_modules = fs::read_dir(&foundation_root)
        .unwrap_or_else(|error| {
            panic!(
                "Material foundation catalog directory is readable at {}: {error}",
                foundation_root.display()
            )
        })
        .map(|entry| {
            entry
                .expect("Material foundation module entry is readable")
                .file_name()
                .to_string_lossy()
                .to_string()
        })
        .collect::<BTreeSet<_>>();

    assert_eq!(
        actual_modules,
        expected_modules
            .iter()
            .copied()
            .map(str::to_string)
            .collect::<BTreeSet<_>>(),
        "Material foundation modules should remain grouped by the planned MUI families"
    );

    let mod_source = fs::read_to_string(foundation_root.join("mod.rs"))
        .expect("Material foundation mod.rs is readable");
    for module in expected_modules
        .iter()
        .copied()
        .filter(|module| *module != "mod.rs")
    {
        let stem = module
            .strip_suffix(".rs")
            .expect("expected Rust module file");
        assert!(
            mod_source.contains(&format!("mod {stem};")),
            "Material foundation mod.rs should declare `{stem}`"
        );
        let source = fs::read_to_string(foundation_root.join(module))
            .unwrap_or_else(|error| panic!("{module} is readable: {error}"));
        assert!(
            source.lines().count() <= 300,
            "{module} should stay below the split-module size budget"
        );
        if stem != "shared" {
            assert!(
                mod_source.contains(&format!("descriptors.extend({stem}::descriptors());")),
                "Material foundation registry should include `{stem}` descriptors"
            );
        }
    }
}
