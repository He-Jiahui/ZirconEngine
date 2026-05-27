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

mod button_inputs;
mod data_display;
mod data_display_subcomponents;
mod form_controls;
mod inputs;
mod lab_subcomponents;
mod layout;
mod mui_web_inventory;
mod mui_x;
mod navigation;
mod navigation_editor;
mod navigation_secondary;
mod selection_inputs;
mod surface_subcomponents;
mod surfaces;
mod virtualization;

#[test]
fn material_editor_foundation_catalog_covers_planned_component_layers() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let expected = [
        "Accordion",
        "AccordionActions",
        "AccordionDetails",
        "AccordionSummary",
        "AgentChat",
        "Alert",
        "AlertTitle",
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
        "BottomNavigationAction",
        "Box",
        "Breadcrumbs",
        "Button",
        "ButtonBase",
        "ButtonGroup",
        "Card",
        "CardActionArea",
        "CardActions",
        "CardContent",
        "CardHeader",
        "CardMedia",
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
        "DialogActions",
        "DialogContent",
        "DialogContentText",
        "DialogTitle",
        "Divider",
        "DockHost",
        "DocumentNode",
        "Drawer",
        "Dropdown",
        "Fade",
        "FieldEditor",
        "FilterBar",
        "FilledInput",
        "FlexGroup",
        "FloatingActionButton",
        "FloatingWindow",
        "FolderTree",
        "FormControl",
        "FormControlLabel",
        "FormGroup",
        "FormHelperText",
        "FormLabel",
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
        "ImageListItem",
        "ImageListItemBar",
        "InitColorSchemeScript",
        "Input",
        "InputAdornment",
        "InputBase",
        "InputLabel",
        "InspectorSection",
        "LineChart",
        "Link",
        "List",
        "ListItem",
        "ListItemAvatar",
        "ListItemButton",
        "ListItemIcon",
        "ListItemSecondaryAction",
        "ListItemText",
        "ListSubheader",
        "ListView",
        "Masonry",
        "MaterialTreeView",
        "Menu",
        "MenuItem",
        "MenuList",
        "Menubar",
        "MetadataPane",
        "Modal",
        "MobileStepper",
        "NativeSelect",
        "NoSsr",
        "NumberField",
        "Overlay",
        "OutlinedInput",
        "Pagination",
        "PaginationItem",
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
        "RadioGroup",
        "Rating",
        "ScrollView",
        "Scrollbar",
        "SearchField",
        "Select",
        "SeverityChips",
        "ScopedCssBaseline",
        "Skeleton",
        "Slider",
        "Slide",
        "Slot",
        "Snackbar",
        "SnackbarContent",
        "SourceEditor",
        "SparkLineChart",
        "SpeedDial",
        "SpeedDialAction",
        "SpeedDialIcon",
        "Splitter",
        "Stack",
        "Step",
        "StepButton",
        "StepConnector",
        "StepContent",
        "StepIcon",
        "StepLabel",
        "StatusActionControls",
        "Stepper",
        "SvgIcon",
        "SwipeableDrawer",
        "Switch",
        "Tab",
        "TabContext",
        "TabList",
        "TabPanel",
        "TabScrollButton",
        "TabStack",
        "Table",
        "TableBody",
        "TableCell",
        "TableContainer",
        "TableFooter",
        "TableHead",
        "TablePagination",
        "TablePaginationActions",
        "TableRow",
        "TableSortLabel",
        "Tabs",
        "TextField",
        "TextareaAutosize",
        "Timeline",
        "TimelineConnector",
        "TimelineContent",
        "TimelineDot",
        "TimelineItem",
        "TimelineOppositeContent",
        "TimelineSeparator",
        "Toolbar",
        "ToggleButton",
        "ToggleButtonGroup",
        "Tooltip",
        "TransferList",
        "TreeItem",
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
        assert_mui_web_customization_schema(descriptor);
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

    button_inputs::assert_descriptors(&registry);
    data_display::assert_descriptors(&registry);
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
    form_controls::assert_descriptors(&registry);
    data_display_subcomponents::assert_descriptors(&registry);
    surface_subcomponents::assert_descriptors(&registry);
    navigation_editor::assert_descriptors(&registry);
    navigation_secondary::assert_descriptors(&registry);
    lab_subcomponents::assert_descriptors(&registry);

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
    assert_has_prop(timeline, "position");
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
        "DialogActions",
        "DialogContent",
        "DialogContentText",
        "DialogTitle",
        "Popover",
        "Snackbar",
        "Accordion",
        "AccordionActions",
        "AccordionDetails",
        "AccordionSummary",
        "AppBar",
        "Card",
        "CardActionArea",
        "CardActions",
        "CardContent",
        "CardHeader",
        "CardMedia",
        "Paper",
        "Toolbar",
        "SwipeableDrawer",
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

    for component_id in ["Popover", "Popper", "Tooltip", "Menu"] {
        let descriptor = registry
            .descriptor(component_id)
            .unwrap_or_else(|| panic!("missing popup descriptor `{component_id}`"));
        for prop in [
            "placement",
            "popup_anchor_x",
            "popup_anchor_y",
            "popup_anchor_width",
            "popup_anchor_height",
            "anchor_origin_vertical",
            "anchor_origin_horizontal",
            "transform_origin_vertical",
            "transform_origin_horizontal",
            "popup_offset_x",
            "popup_offset_y",
        ] {
            assert_has_prop(descriptor, prop);
        }
    }

    for component_id in ["Dialog", "Modal", "Popover", "Menu"] {
        let descriptor = registry
            .descriptor(component_id)
            .unwrap_or_else(|| panic!("missing modal interaction descriptor `{component_id}`"));
        for prop in [
            "disable_auto_focus",
            "disable_enforce_focus",
            "disable_restore_focus",
            "disable_escape_key_down",
            "close_on_backdrop_click",
            "keep_mounted",
            "aria_modal",
            "aria_labelledby",
            "aria_describedby",
        ] {
            assert_has_prop(descriptor, prop);
        }
    }

    for component_id in [
        "Backdrop",
        "Dialog",
        "Modal",
        "Popover",
        "Popper",
        "Tooltip",
        "Snackbar",
        "SpeedDial",
        "Drawer",
        "Menu",
        "SwipeableDrawer",
    ] {
        let descriptor = registry
            .descriptor(component_id)
            .unwrap_or_else(|| panic!("missing overlay layer descriptor `{component_id}`"));
        for prop in ["z_index", "disable_portal", "portal_layer"] {
            assert_has_prop(descriptor, prop);
        }
    }

    let alert = registry.descriptor("Alert").expect("Alert descriptor");
    assert_enum_options(alert, "severity", &["success", "info", "warning", "error"]);
    assert_eq!(
        alert
            .prop("severity")
            .and_then(|prop| prop.default_value.clone()),
        Some(UiValue::Enum("success".to_string())),
        "Alert should default severity to local MUI Alert.js"
    );
    assert_enum_options(alert, "variant", &["standard", "filled", "outlined"]);
    for prop in ["color", "icon", "show_icon", "iconMapping", "closeText"] {
        assert_has_prop(alert, prop);
    }
    for slot_name in ["icon", "message", "action", "closeButton", "closeIcon"] {
        assert!(
            alert.slot_schema.iter().any(|slot| slot.name == slot_name),
            "Alert missing MUI slot `{slot_name}`"
        );
    }
    let alert_title = registry
        .descriptor("AlertTitle")
        .expect("AlertTitle descriptor");
    assert_has_prop(alert_title, "text");

    let snackbar = registry
        .descriptor("Snackbar")
        .expect("Snackbar descriptor");
    for prop in [
        "message",
        "auto_hide_duration_ms",
        "autoHideDuration",
        "resume_hide_duration_ms",
        "resumeHideDuration",
        "disable_window_blur_listener",
        "disableWindowBlurListener",
        "anchor_origin_vertical",
        "anchor_origin_horizontal",
        "anchorOrigin",
    ] {
        assert_has_prop(snackbar, prop);
    }
    assert_enum_options(snackbar, "anchor_origin_vertical", &["top", "bottom"]);
    assert_enum_options(
        snackbar,
        "anchor_origin_horizontal",
        &["left", "center", "right"],
    );
    assert_eq!(
        snackbar
            .prop("anchor_origin_horizontal")
            .and_then(|prop| prop.default_value.clone()),
        Some(UiValue::Enum("left".to_string())),
        "Snackbar should default horizontal anchor to local MUI Snackbar.js"
    );
    let snackbar_content = registry
        .descriptor("SnackbarContent")
        .expect("SnackbarContent descriptor");
    for prop in ["message", "role"] {
        assert_has_prop(snackbar_content, prop);
    }
    for slot_name in ["message", "action"] {
        assert!(
            snackbar_content
                .slot_schema
                .iter()
                .any(|slot| slot.name == slot_name),
            "SnackbarContent missing MUI slot `{slot_name}`"
        );
    }

    surfaces::assert_descriptors(&registry);
    navigation::assert_descriptors(&registry);
    layout::assert_descriptors(&registry);
    mui_x::assert_descriptors(&registry);

    for component_id in ["Collapse", "Fade", "Grow", "Slide", "Zoom"] {
        let descriptor = registry
            .descriptor(component_id)
            .unwrap_or_else(|| panic!("missing transition descriptor `{component_id}`"));
        for prop in [
            "transition_kind",
            "in",
            "transition_status",
            "transition_progress",
            "timeout_ms",
            "transition_duration_ms",
            "easing",
            "transition_easing",
            "mount_on_enter",
            "unmount_on_exit",
        ] {
            assert_has_prop(descriptor, prop);
        }
    }
    assert_has_prop(
        registry
            .descriptor("Collapse")
            .expect("Collapse descriptor"),
        "orientation",
    );
    assert_has_prop(
        registry
            .descriptor("Collapse")
            .expect("Collapse descriptor"),
        "collapsed_size",
    );
    assert_has_prop(
        registry.descriptor("Slide").expect("Slide descriptor"),
        "direction",
    );

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

fn assert_mui_web_customization_schema(
    descriptor: &zircon_runtime_interface::ui::component::UiComponentDescriptor,
) {
    for (name, expected_default) in [
        ("mui_variant", ""),
        ("mui_color", "primary"),
        ("mui_size", "medium"),
    ] {
        let schema = descriptor
            .prop(name)
            .unwrap_or_else(|| panic!("{} missing prop `{name}`", descriptor.id));
        assert_eq!(schema.value_kind, UiValueKind::String);
        assert_eq!(
            schema.default_value,
            Some(UiValue::String(expected_default.to_string())),
            "{} should default `{name}` to `{expected_default}`",
            descriptor.id
        );
    }

    for name in [
        "mui_slots",
        "mui_slot_props",
        "mui_sx",
        "slots",
        "slotProps",
        "sx",
        "classes",
    ] {
        let schema = descriptor
            .prop(name)
            .unwrap_or_else(|| panic!("{} missing prop `{name}`", descriptor.id));
        assert_eq!(schema.value_kind, UiValueKind::Map);
        assert_eq!(
            schema.default_value,
            Some(UiValue::Map(Default::default())),
            "{} should default `{name}` to an empty map",
            descriptor.id
        );
    }

    let classes = descriptor
        .prop("mui_classes")
        .unwrap_or_else(|| panic!("{} missing prop `mui_classes`", descriptor.id));
    assert_eq!(classes.value_kind, UiValueKind::Array);
    assert_eq!(
        classes.default_value,
        Some(UiValue::Array(Vec::new())),
        "{} should default `mui_classes` to an empty array",
        descriptor.id
    );

    let class_name = descriptor
        .prop("className")
        .unwrap_or_else(|| panic!("{} missing prop `className`", descriptor.id));
    assert_eq!(class_name.value_kind, UiValueKind::String);
    assert_eq!(
        class_name.default_value,
        Some(UiValue::String(String::new())),
        "{} should default `className` to an empty string",
        descriptor.id
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
        "button_inputs.rs",
        "inputs.rs",
        "selection_inputs.rs",
        "text_inputs.rs",
        "form_controls.rs",
        "lab_subcomponents.rs",
        "data_display.rs",
        "data_display_editor.rs",
        "data_display_subcomponents.rs",
        "data_display_table.rs",
        "feedback.rs",
        "surfaces.rs",
        "navigation.rs",
        "navigation_subcomponents.rs",
        "navigation_secondary.rs",
        "navigation_editor.rs",
        "layout_mui.rs",
        "layout.rs",
        "layout_utilities.rs",
        "layout_transitions.rs",
        "layout_editor.rs",
        "mui_x.rs",
        "surface_subcomponents.rs",
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
        if stem != "shared" {
            assert!(
                source.lines().count() <= 300,
                "{module} should stay below the split-module size budget"
            );
            assert!(
                mod_source.contains(&format!("descriptors.extend({stem}::descriptors());")),
                "Material foundation registry should include `{stem}` descriptors"
            );
        }
    }
}
