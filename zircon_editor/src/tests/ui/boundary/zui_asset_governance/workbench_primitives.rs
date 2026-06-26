use std::collections::BTreeSet;
use std::fs;

use toml::Value;
use zircon_runtime::ui::v2::UiZuiAssetLoader;
use zircon_runtime_interface::ui::v2::UiV2AssetKind;

use super::support::editor_asset_root;

struct WorkbenchPrimitiveContract {
    file_name: &'static str,
    component_name: &'static str,
    root_component: &'static str,
    interactive: bool,
    sampled_in_component_drawer: bool,
}

struct WorkbenchShellSurfaceContract {
    file_name: &'static str,
    component_name: &'static str,
    root_node: &'static str,
    root_component: &'static str,
    root_control_id: &'static str,
    root_classes: &'static [&'static str],
    required_widget_imports: &'static [&'static str],
    required_mounted_components: &'static [&'static str],
    required_control_ids: &'static [&'static str],
}

struct WorkbenchOverlayPrimitiveContract {
    file_name: &'static str,
    component_name: &'static str,
    placement: &'static str,
    required_props: &'static [&'static str],
}

const WORKBENCH_PRIMITIVE_CONTRACTS: &[WorkbenchPrimitiveContract] = &[
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/inputs/workbench_button.zui",
        component_name: "WorkbenchButton",
        root_component: "Button",
        interactive: true,
        sampled_in_component_drawer: true,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/inputs/workbench_icon_button.zui",
        component_name: "WorkbenchIconButton",
        root_component: "IconButton",
        interactive: true,
        sampled_in_component_drawer: true,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/chrome/workbench_rail_button.zui",
        component_name: "WorkbenchRailButton",
        root_component: "IconButton",
        interactive: true,
        sampled_in_component_drawer: false,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/inputs/workbench_field.zui",
        component_name: "WorkbenchField",
        root_component: "InputField",
        interactive: true,
        sampled_in_component_drawer: true,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/inputs/workbench_search_input.zui",
        component_name: "WorkbenchSearchInput",
        root_component: "SearchField",
        interactive: true,
        sampled_in_component_drawer: true,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/inputs/workbench_checkbox.zui",
        component_name: "WorkbenchCheckbox",
        root_component: "Checkbox",
        interactive: true,
        sampled_in_component_drawer: true,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/inputs/workbench_radio.zui",
        component_name: "WorkbenchRadio",
        root_component: "Radio",
        interactive: true,
        sampled_in_component_drawer: true,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/inputs/workbench_toggle.zui",
        component_name: "WorkbenchToggle",
        root_component: "Toggle",
        interactive: true,
        sampled_in_component_drawer: true,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/inputs/workbench_tab.zui",
        component_name: "WorkbenchTab",
        root_component: "ToggleButton",
        interactive: true,
        sampled_in_component_drawer: true,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/inputs/workbench_segmented_control.zui",
        component_name: "WorkbenchSegmentedControl",
        root_component: "SegmentedControl",
        interactive: true,
        sampled_in_component_drawer: true,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/inputs/workbench_dropdown.zui",
        component_name: "WorkbenchDropdown",
        root_component: "Dropdown",
        interactive: true,
        sampled_in_component_drawer: true,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/inputs/workbench_slider.zui",
        component_name: "WorkbenchSlider",
        root_component: "RangeField",
        interactive: true,
        sampled_in_component_drawer: true,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/inputs/workbench_range_slider.zui",
        component_name: "WorkbenchRangeSlider",
        root_component: "RangeSlider",
        interactive: true,
        sampled_in_component_drawer: true,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/inputs/workbench_number_field.zui",
        component_name: "WorkbenchNumberField",
        root_component: "NumberField",
        interactive: true,
        sampled_in_component_drawer: true,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/inputs/workbench_tab_strip.zui",
        component_name: "WorkbenchTabStrip",
        root_component: "Tabs",
        interactive: true,
        sampled_in_component_drawer: true,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/data/workbench_label.zui",
        component_name: "WorkbenchLabel",
        root_component: "Label",
        interactive: false,
        sampled_in_component_drawer: true,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/data/workbench_icon.zui",
        component_name: "WorkbenchIcon",
        root_component: "Icon",
        interactive: false,
        sampled_in_component_drawer: true,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/data/workbench_list_row.zui",
        component_name: "WorkbenchListRow",
        root_component: "ListRow",
        interactive: true,
        sampled_in_component_drawer: true,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/data/workbench_tree_row.zui",
        component_name: "WorkbenchTreeRow",
        root_component: "TreeRow",
        interactive: true,
        sampled_in_component_drawer: false,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/data/workbench_table_row.zui",
        component_name: "WorkbenchTableRow",
        root_component: "Table",
        interactive: true,
        sampled_in_component_drawer: true,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/data/workbench_divider.zui",
        component_name: "WorkbenchDivider",
        root_component: "Divider",
        interactive: false,
        sampled_in_component_drawer: true,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/feedback/workbench_popup_menu.zui",
        component_name: "WorkbenchPopupMenu",
        root_component: "ContextActionMenu",
        interactive: true,
        sampled_in_component_drawer: true,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/feedback/workbench_context_menu.zui",
        component_name: "WorkbenchContextMenu",
        root_component: "ContextMenu",
        interactive: true,
        sampled_in_component_drawer: false,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/feedback/workbench_dropdown_popup.zui",
        component_name: "WorkbenchDropdownPopup",
        root_component: "DropdownPopup",
        interactive: true,
        sampled_in_component_drawer: false,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/feedback/workbench_command_palette.zui",
        component_name: "WorkbenchCommandPalette",
        root_component: "CommandPalette",
        interactive: true,
        sampled_in_component_drawer: false,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/feedback/workbench_notification_center.zui",
        component_name: "WorkbenchNotificationCenter",
        root_component: "NotificationCenter",
        interactive: true,
        sampled_in_component_drawer: false,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/feedback/workbench_drag_overlay.zui",
        component_name: "WorkbenchDragOverlay",
        root_component: "DragOverlay",
        interactive: true,
        sampled_in_component_drawer: false,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/feedback/workbench_alert.zui",
        component_name: "WorkbenchAlert",
        root_component: "Alert",
        interactive: false,
        sampled_in_component_drawer: false,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/feedback/workbench_dialog.zui",
        component_name: "WorkbenchDialog",
        root_component: "Dialog",
        interactive: true,
        sampled_in_component_drawer: false,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/feedback/workbench_confirm_dialog.zui",
        component_name: "WorkbenchConfirmDialog",
        root_component: "ConfirmDialog",
        interactive: true,
        sampled_in_component_drawer: false,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/feedback/workbench_progress_bar.zui",
        component_name: "WorkbenchProgressBar",
        root_component: "Progress",
        interactive: false,
        sampled_in_component_drawer: true,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/feedback/workbench_skeleton.zui",
        component_name: "WorkbenchSkeleton",
        root_component: "Skeleton",
        interactive: false,
        sampled_in_component_drawer: true,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/feedback/workbench_tooltip.zui",
        component_name: "WorkbenchTooltip",
        root_component: "Tooltip",
        interactive: false,
        sampled_in_component_drawer: true,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/feedback/workbench_toast.zui",
        component_name: "WorkbenchToast",
        root_component: "Alert",
        interactive: false,
        sampled_in_component_drawer: true,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/data/workbench_property_row.zui",
        component_name: "WorkbenchPropertyRow",
        root_component: "PropertyRow",
        interactive: false,
        sampled_in_component_drawer: false,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/data/workbench_component_property_row.zui",
        component_name: "WorkbenchComponentPropertyRow",
        root_component: "InputField",
        interactive: true,
        sampled_in_component_drawer: false,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/chrome/workbench_chip.zui",
        component_name: "WorkbenchChip",
        root_component: "Label",
        interactive: false,
        sampled_in_component_drawer: false,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/chrome/workbench_axis_value_field.zui",
        component_name: "WorkbenchAxisValueField",
        root_component: "InputField",
        interactive: true,
        sampled_in_component_drawer: false,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/chrome/workbench_section_title.zui",
        component_name: "WorkbenchSectionTitle",
        root_component: "Label",
        interactive: false,
        sampled_in_component_drawer: false,
    },
    WorkbenchPrimitiveContract {
        file_name: "workbench/primitives/feedback/workbench_status_item.zui",
        component_name: "WorkbenchStatusItem",
        root_component: "Label",
        interactive: false,
        sampled_in_component_drawer: false,
    },
];

const OVERLAY_REQUIRED_PROPS: &[&str] = &[
    "open",
    "popup_open",
    "options",
    "focused_index",
    "keyboard_navigation",
    "typeahead_buffer",
    "typeahead_buffer_expired",
    "typeahead_timeout_ms",
    "hovered_option_id",
    "submenu_pending_option_id",
    "submenu_open_option_id",
    "submenu_hover_ready",
    "submenu_hover_delay_ms",
    "submenu_focus_scope",
    "submenu_focus_loop",
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
    "disable_auto_focus",
    "disable_enforce_focus",
    "disable_restore_focus",
    "disable_escape_key_down",
    "close_on_backdrop_click",
    "keep_mounted",
    "aria_modal",
    "aria_labelledby",
    "aria_describedby",
    "z_index",
    "disable_portal",
    "portal_layer",
];

const WORKBENCH_OVERLAY_PRIMITIVE_CONTRACTS: &[WorkbenchOverlayPrimitiveContract] = &[
    WorkbenchOverlayPrimitiveContract {
        file_name: "workbench/primitives/feedback/workbench_command_palette.zui",
        component_name: "WorkbenchCommandPalette",
        placement: "top",
        required_props: COMMAND_PALETTE_REQUIRED_PROPS,
    },
    WorkbenchOverlayPrimitiveContract {
        file_name: "workbench/primitives/feedback/workbench_notification_center.zui",
        component_name: "WorkbenchNotificationCenter",
        placement: "bottom-end",
        required_props: NOTIFICATION_CENTER_REQUIRED_PROPS,
    },
    WorkbenchOverlayPrimitiveContract {
        file_name: "workbench/primitives/feedback/workbench_dialog.zui",
        component_name: "WorkbenchDialog",
        placement: "center",
        required_props: DIALOG_REQUIRED_PROPS,
    },
    WorkbenchOverlayPrimitiveContract {
        file_name: "workbench/primitives/feedback/workbench_confirm_dialog.zui",
        component_name: "WorkbenchConfirmDialog",
        placement: "center",
        required_props: CONFIRM_DIALOG_REQUIRED_PROPS,
    },
    WorkbenchOverlayPrimitiveContract {
        file_name: "workbench/primitives/feedback/workbench_context_menu.zui",
        component_name: "WorkbenchContextMenu",
        placement: "right-start",
        required_props: OVERLAY_REQUIRED_PROPS,
    },
    WorkbenchOverlayPrimitiveContract {
        file_name: "workbench/primitives/feedback/workbench_dropdown_popup.zui",
        component_name: "WorkbenchDropdownPopup",
        placement: "bottom-start",
        required_props: OVERLAY_REQUIRED_PROPS,
    },
];

const COMMAND_PALETTE_REQUIRED_PROPS: &[&str] = &[
    "open",
    "popup_open",
    "query",
    "placeholder",
    "commands",
    "filtered_commands",
    "recent_commands",
    "disabled_commands",
    "selected_command_id",
    "focused_index",
    "keyboard_navigation",
    "empty_text",
    "command_source",
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
    "disable_auto_focus",
    "disable_enforce_focus",
    "disable_restore_focus",
    "disable_escape_key_down",
    "close_on_backdrop_click",
    "keep_mounted",
    "aria_modal",
    "aria_labelledby",
    "aria_describedby",
    "z_index",
    "disable_portal",
    "portal_layer",
];

const NOTIFICATION_CENTER_REQUIRED_PROPS: &[&str] = &[
    "open",
    "popup_open",
    "title",
    "unread_count",
    "notifications",
    "selected_notification_id",
    "focused_index",
    "visible_limit",
    "empty_text",
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
    "disable_auto_focus",
    "disable_enforce_focus",
    "disable_restore_focus",
    "disable_escape_key_down",
    "close_on_backdrop_click",
    "keep_mounted",
    "aria_modal",
    "aria_labelledby",
    "aria_describedby",
    "z_index",
    "disable_portal",
    "portal_layer",
];

const DRAG_OVERLAY_REQUIRED_PROPS: &[&str] = &[
    "open",
    "dragging",
    "drop_hovered",
    "active_drag_target",
    "payload_kind",
    "payload_label",
    "payload_reference",
    "source_control_id",
    "target_control_id",
    "cursor_x",
    "cursor_y",
    "offset_x",
    "offset_y",
    "preview_width",
    "preview_height",
    "drop_allowed",
    "drop_target_x",
    "drop_target_y",
    "drop_target_width",
    "drop_target_height",
    "drop_indicator_edge",
    "drop_indicator_text",
    "z_index",
    "disable_portal",
    "portal_layer",
];

const DIALOG_REQUIRED_PROPS: &[&str] = &[
    "open",
    "popup_open",
    "text",
    "title",
    "message",
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
    "disable_auto_focus",
    "disable_enforce_focus",
    "disable_restore_focus",
    "disable_escape_key_down",
    "close_on_backdrop_click",
    "keep_mounted",
    "aria_modal",
    "aria_labelledby",
    "aria_describedby",
    "z_index",
    "disable_portal",
    "portal_layer",
];

const CONFIRM_DIALOG_REQUIRED_PROPS: &[&str] = &[
    "open",
    "popup_open",
    "title",
    "message",
    "confirm_text",
    "cancel_text",
    "confirm_action_id",
    "cancel_action_id",
    "severity",
    "default_action",
    "destructive",
    "confirm_enabled",
    "requires_explicit_action",
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
    "disable_auto_focus",
    "disable_enforce_focus",
    "disable_restore_focus",
    "disable_escape_key_down",
    "close_on_backdrop_click",
    "keep_mounted",
    "aria_modal",
    "aria_labelledby",
    "aria_describedby",
    "z_index",
    "disable_portal",
    "portal_layer",
];

const WORKBENCH_SHELL_SURFACE_CONTRACTS: &[WorkbenchShellSurfaceContract] = &[
    WorkbenchShellSurfaceContract {
        file_name: "workbench/shell/workbench_activity_rail.zui",
        component_name: "WorkbenchActivityRail",
        root_node: "activity_rail",
        root_component: "VerticalGroup",
        root_control_id: "WorkbenchWindowActivityRail",
        root_classes: &["workbench-rail"],
        required_widget_imports: &[
            "res://ui/editor/components/workbench/primitives/chrome/workbench_rail_button.zui#WorkbenchRailButton",
        ],
        required_mounted_components: &["WorkbenchRailButton"],
        required_control_ids: &[
            "WorkbenchRailScene",
            "WorkbenchRailCube",
            "WorkbenchRailGraph",
            "WorkbenchRailImage",
            "WorkbenchRailAudio",
            "WorkbenchRailCode",
        ],
    },
    WorkbenchShellSurfaceContract {
        file_name: "workbench/shell/workbench_top_toolbar.zui",
        component_name: "WorkbenchTopToolbar",
        root_node: "top_toolbar",
        root_component: "VerticalGroup",
        root_control_id: "WorkbenchWindowTopToolbar",
        root_classes: &["workbench-topbar"],
        required_widget_imports: &[
            "res://ui/editor/components/workbench/primitives/inputs/workbench_button.zui#WorkbenchButton",
            "res://ui/editor/components/workbench/primitives/inputs/workbench_icon_button.zui#WorkbenchIconButton",
            "res://ui/editor/components/workbench/primitives/inputs/workbench_tab.zui#WorkbenchTab",
        ],
        required_mounted_components: &["WorkbenchButton", "WorkbenchIconButton", "WorkbenchTab"],
        required_control_ids: &[
            "WorkbenchToolbarCommandRow",
            "WorkbenchToolbarFileGroup",
            "WorkbenchModuleTabs",
            "WorkbenchModuleCommands",
            "WorkbenchToolbarToolGroup",
            "WorkbenchToolbarRunGroup",
        ],
    },
    WorkbenchShellSurfaceContract {
        file_name: "workbench/shell/workbench_scene_tree_panel.zui",
        component_name: "WorkbenchSceneTreePanel",
        root_node: "scene_tree_panel",
        root_component: "VerticalGroup",
        root_control_id: "WorkbenchSceneTreePanel",
        root_classes: &["workbench-panel", "workbench-left-panel"],
        required_widget_imports: &[
            "res://ui/editor/components/workbench/primitives/inputs/workbench_search_input.zui#WorkbenchSearchInput",
            "res://ui/editor/components/workbench/primitives/inputs/workbench_icon_button.zui#WorkbenchIconButton",
            "res://ui/editor/components/workbench/primitives/inputs/workbench_tab.zui#WorkbenchTab",
            "res://ui/editor/components/workbench/primitives/data/workbench_tree_row.zui#WorkbenchTreeRow",
        ],
        required_mounted_components: &[
            "WorkbenchSearchInput",
            "WorkbenchIconButton",
            "WorkbenchTab",
            "WorkbenchTreeRow",
        ],
        required_control_ids: &[
            "LeftDrawerHeaderRoot",
            "WorkbenchSceneSearchField",
            "WorkbenchSceneFilter",
            "WorkbenchSceneTree",
            "WorkbenchSceneRootItem",
        ],
    },
    WorkbenchShellSurfaceContract {
        file_name: "workbench/shell/workbench_viewport_panel.zui",
        component_name: "WorkbenchViewportPanel",
        root_node: "viewport_panel",
        root_component: "VerticalGroup",
        root_control_id: "WorkbenchViewportPanel",
        root_classes: &["workbench-viewport-panel"],
        required_widget_imports: &[
            "res://ui/editor/components/workbench/primitives/chrome/workbench_chip.zui#WorkbenchChip",
        ],
        required_mounted_components: &["WorkbenchChip"],
        required_control_ids: &[
            "WorkbenchViewportToolbar",
            "WorkbenchViewportSurface",
            "WorkbenchViewportMode",
            "WorkbenchViewportGizmoPanel",
        ],
    },
    WorkbenchShellSurfaceContract {
        file_name: "workbench/shell/workbench_inspector_panel.zui",
        component_name: "WorkbenchInspectorPanel",
        root_node: "inspector_panel",
        root_component: "VerticalGroup",
        root_control_id: "WorkbenchInspectorPanel",
        root_classes: &["workbench-panel", "workbench-right-panel"],
        required_widget_imports: &[
            "res://ui/editor/components/workbench/primitives/chrome/workbench_axis_value_field.zui#WorkbenchAxisValueField",
            "res://ui/editor/components/workbench/primitives/inputs/workbench_button.zui#WorkbenchButton",
            "res://ui/editor/components/workbench/primitives/data/workbench_component_property_row.zui#WorkbenchComponentPropertyRow",
            "res://ui/editor/components/workbench/primitives/inputs/workbench_dropdown.zui#WorkbenchDropdown",
            "res://ui/editor/components/workbench/primitives/chrome/workbench_section_title.zui#WorkbenchSectionTitle",
            "res://ui/editor/components/workbench/primitives/inputs/workbench_tab.zui#WorkbenchTab",
        ],
        required_mounted_components: &[
            "WorkbenchAxisValueField",
            "WorkbenchButton",
            "WorkbenchComponentPropertyRow",
            "WorkbenchDropdown",
            "WorkbenchSectionTitle",
            "WorkbenchTab",
        ],
        required_control_ids: &[
            "RightDrawerHeaderRoot",
            "WorkbenchInspectorTransform",
            "WorkbenchInspectorMesh",
            "WorkbenchAddComponent",
        ],
    },
    WorkbenchShellSurfaceContract {
        file_name: "workbench/shell/workbench_status_bar.zui",
        component_name: "WorkbenchStatusBar",
        root_node: "status_bar",
        root_component: "HorizontalGroup",
        root_control_id: "WorkbenchWindowStatusBar",
        root_classes: &["workbench-status"],
        required_widget_imports: &[
            "res://ui/editor/components/workbench/primitives/chrome/workbench_chip.zui#WorkbenchChip",
            "res://ui/editor/components/workbench/primitives/inputs/workbench_icon_button.zui#WorkbenchIconButton",
            "res://ui/editor/components/workbench/primitives/feedback/workbench_status_item.zui#WorkbenchStatusItem",
        ],
        required_mounted_components: &[
            "WorkbenchChip",
            "WorkbenchIconButton",
            "WorkbenchStatusItem",
        ],
        required_control_ids: &[
            "WorkbenchStatusReady",
            "WorkbenchStatusErrors",
            "WorkbenchStatusWarnings",
            "WorkbenchStatusGrid",
            "WorkbenchStatusZoom",
        ],
    },
    WorkbenchShellSurfaceContract {
        file_name: "workbench/shell/workbench_main_band.zui",
        component_name: "WorkbenchMainBand",
        root_node: "main_band",
        root_component: "Overlay",
        root_control_id: "WorkbenchMainBand",
        root_classes: &["workbench-main-band"],
        required_widget_imports: &[
            "res://ui/editor/components/workbench/shell/workbench_activity_rail.zui#WorkbenchActivityRail",
            "res://ui/editor/components/workbench/shell/workbench_inspector_panel.zui#WorkbenchInspectorPanel",
            "res://ui/editor/components/workbench/modules/core/index/workbench_module_workspace.zui#WorkbenchModuleWorkspace",
            "res://ui/editor/components/workbench/shell/workbench_scene_tree_panel.zui#WorkbenchSceneTreePanel",
            "res://ui/editor/components/workbench/shell/workbench_viewport_panel.zui#WorkbenchViewportPanel",
        ],
        required_mounted_components: &[
            "WorkbenchActivityRail",
            "WorkbenchInspectorPanel",
            "WorkbenchModuleWorkspace",
            "WorkbenchSceneTreePanel",
            "WorkbenchViewportPanel",
        ],
        required_control_ids: &[
            "WorkbenchSceneWorkspace",
            "WorkbenchMainBandActivityRail",
            "WorkbenchMainBandSceneTreePanel",
            "WorkbenchMainBandViewportPanel",
            "WorkbenchMainBandInspectorPanel",
            "WorkbenchMainBandModuleWorkspace",
        ],
    },
];

#[test]
fn workbench_primitive_component_assets_keep_native_component_contract() {
    let mut offenders = Vec::new();

    for contract in WORKBENCH_PRIMITIVE_CONTRACTS {
        let path = editor_asset_root()
            .join("ui/editor/components")
            .join(contract.file_name);
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
        let document = UiZuiAssetLoader::load_zui_str(&source)
            .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));
        let Some(component) = document.components.get(contract.component_name) else {
            offenders.push(format!(
                "{} should declare component `{}`",
                path.display(),
                contract.component_name
            ));
            continue;
        };
        let Some(root_node) = document.nodes.get(&component.root) else {
            offenders.push(format!(
                "{} component `{}` references missing root `{}`",
                path.display(),
                contract.component_name,
                component.root
            ));
            continue;
        };

        if document.asset.kind != UiV2AssetKind::Component {
            offenders.push(format!(
                "{} should remain a component asset",
                path.display()
            ));
        }
        if component.root != "root" {
            offenders.push(format!(
                "{} component `{}` should use the stable `root` node",
                path.display(),
                contract.component_name
            ));
        }
        if root_node.component != contract.root_component {
            offenders.push(format!(
                "{} component `{}` root node should render `{}` but renders `{}`",
                path.display(),
                contract.component_name,
                contract.root_component,
                root_node.component
            ));
        }
        let expected_control_id = format!("{}Root", contract.component_name);
        if root_node.control_id.as_deref() != Some(expected_control_id.as_str()) {
            offenders.push(format!(
                "{} component `{}` root node should expose control id `{}Root`",
                path.display(),
                contract.component_name,
                contract.component_name
            ));
        }
        if !component
            .default_classes
            .iter()
            .any(|class| class == "workbench-primitive")
        {
            offenders.push(format!(
                "{} component `{}` should expose the shared `workbench-primitive` class",
                path.display(),
                contract.component_name
            ));
        }
        if root_node
            .layout
            .as_ref()
            .is_none_or(|layout| !layout.contains_key("width") || !layout.contains_key("height"))
        {
            offenders.push(format!(
                "{} component `{}` root node should declare width and height layout contracts",
                path.display(),
                contract.component_name
            ));
        }
        if contract.interactive {
            for prop in [
                "input_interactive",
                "input_clickable",
                "input_hoverable",
                "input_focusable",
            ] {
                if root_node.props.get(prop).and_then(Value::as_bool) != Some(true) {
                    offenders.push(format!(
                        "{} component `{}` root node should set `{prop} = true`",
                        path.display(),
                        contract.component_name
                    ));
                }
            }
        }
    }

    assert!(
        WORKBENCH_PRIMITIVE_CONTRACTS.len() >= 39,
        "workbench primitive contract should cover the low-level atom/collection/property/shell-leaf set"
    );
    assert!(
        offenders.is_empty(),
        "workbench primitive .zui component assets must stay componentized, layout-explicit, and input-ready before module assembly: {offenders:#?}"
    );
}

#[test]
fn workbench_overlay_primitives_expose_popup_shell_contract() {
    let mut offenders = Vec::new();

    for contract in WORKBENCH_OVERLAY_PRIMITIVE_CONTRACTS {
        let path = editor_asset_root()
            .join("ui/editor/components")
            .join(contract.file_name);
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
        let document = UiZuiAssetLoader::load_zui_str(&source)
            .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));
        let Some(component) = document.components.get(contract.component_name) else {
            offenders.push(format!(
                "{} should declare component `{}`",
                path.display(),
                contract.component_name
            ));
            continue;
        };
        let Some(root_node) = document.nodes.get(&component.root) else {
            offenders.push(format!(
                "{} component `{}` references missing root `{}`",
                path.display(),
                contract.component_name,
                component.root
            ));
            continue;
        };

        for prop in contract.required_props {
            if !root_node.props.contains_key(*prop) {
                offenders.push(format!(
                    "{} component `{}` should expose popup-shell prop `{prop}`",
                    path.display(),
                    contract.component_name
                ));
            }
        }
        if root_node.props.get("placement").and_then(Value::as_str) != Some(contract.placement) {
            offenders.push(format!(
                "{} component `{}` should set placement `{}`",
                path.display(),
                contract.component_name,
                contract.placement
            ));
        }
        if root_node
            .props
            .get("close_on_backdrop_click")
            .and_then(Value::as_bool)
            != Some(true)
        {
            offenders.push(format!(
                "{} component `{}` should keep outside-click dismissal enabled",
                path.display(),
                contract.component_name
            ));
        }
        if root_node
            .props
            .get("disable_portal")
            .and_then(Value::as_bool)
            != Some(false)
        {
            offenders.push(format!(
                "{} component `{}` should stay attached to the overlay portal layer",
                path.display(),
                contract.component_name
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "workbench overlay primitives must expose the retained popup shell contract before higher-level surfaces compose them: {offenders:#?}"
    );
}

#[test]
fn workbench_drag_overlay_exposes_drag_visual_contract() {
    let path = editor_asset_root()
        .join("ui/editor/components")
        .join("workbench/primitives/feedback/workbench_drag_overlay.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));
    let component = document
        .components
        .get("WorkbenchDragOverlay")
        .expect("WorkbenchDragOverlay component should be declared");
    let root = document
        .nodes
        .get(&component.root)
        .expect("WorkbenchDragOverlay root node should exist");

    let missing_props = DRAG_OVERLAY_REQUIRED_PROPS
        .iter()
        .filter(|prop| !root.props.contains_key(**prop))
        .copied()
        .collect::<Vec<_>>();
    assert!(
        missing_props.is_empty(),
        "WorkbenchDragOverlay should expose DragOverlay descriptor props for retained/native projection: {missing_props:#?}"
    );
    assert_eq!(
        root.props.get("payload_kind").and_then(Value::as_str),
        Some("asset")
    );
    assert_eq!(
        root.props
            .get("drop_indicator_edge")
            .and_then(Value::as_str),
        Some("bottom")
    );
    assert_eq!(
        root.props.get("disable_portal").and_then(Value::as_bool),
        Some(false)
    );
}

#[test]
fn workbench_shell_surface_component_assets_keep_bottom_up_composition_contract() {
    let mut offenders = Vec::new();

    for contract in WORKBENCH_SHELL_SURFACE_CONTRACTS {
        let path = editor_asset_root()
            .join("ui/editor/components")
            .join(contract.file_name);
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
        let document = UiZuiAssetLoader::load_zui_str(&source)
            .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));
        let Some(component) = document.components.get(contract.component_name) else {
            offenders.push(format!(
                "{} should declare component `{}`",
                path.display(),
                contract.component_name
            ));
            continue;
        };
        let Some(root_node) = document.nodes.get(&component.root) else {
            offenders.push(format!(
                "{} component `{}` references missing root `{}`",
                path.display(),
                contract.component_name,
                component.root
            ));
            continue;
        };

        if document.asset.kind != UiV2AssetKind::Component {
            offenders.push(format!(
                "{} should remain a component asset",
                path.display()
            ));
        }
        if component.root != contract.root_node {
            offenders.push(format!(
                "{} component `{}` should use stable root node `{}`",
                path.display(),
                contract.component_name,
                contract.root_node
            ));
        }
        if root_node.component != contract.root_component {
            offenders.push(format!(
                "{} component `{}` root node should render `{}` but renders `{}`",
                path.display(),
                contract.component_name,
                contract.root_component,
                root_node.component
            ));
        }
        if root_node.control_id.as_deref() != Some(contract.root_control_id) {
            offenders.push(format!(
                "{} component `{}` root node should expose control id `{}`",
                path.display(),
                contract.component_name,
                contract.root_control_id
            ));
        }
        for required_class in contract.root_classes {
            if !component
                .default_classes
                .iter()
                .any(|candidate| candidate.as_str() == *required_class)
            {
                offenders.push(format!(
                    "{} component `{}` should expose default class `{}`",
                    path.display(),
                    contract.component_name,
                    required_class
                ));
            }
            if !root_node
                .classes
                .iter()
                .any(|candidate| candidate.as_str() == *required_class)
            {
                offenders.push(format!(
                    "{} component `{}` root node should expose class `{}`",
                    path.display(),
                    contract.component_name,
                    required_class
                ));
            }
        }
        if root_node
            .layout
            .as_ref()
            .is_none_or(|layout| !layout.contains_key("width") || !layout.contains_key("height"))
        {
            offenders.push(format!(
                "{} component `{}` root node should declare width and height layout contracts",
                path.display(),
                contract.component_name
            ));
        }

        let widget_imports = document
            .imports
            .widgets
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        for required_import in contract.required_widget_imports {
            if !widget_imports.contains(*required_import) {
                offenders.push(format!(
                    "{} component `{}` should import `{}`",
                    path.display(),
                    contract.component_name,
                    required_import
                ));
            }
        }

        let node_components = document
            .nodes
            .values()
            .map(|node| node.component.as_str())
            .collect::<BTreeSet<_>>();
        for required_component in contract.required_mounted_components {
            if !node_components.contains(*required_component) {
                offenders.push(format!(
                    "{} component `{}` should mount `{}` instead of duplicating that lower-level component inline",
                    path.display(),
                    contract.component_name,
                    required_component
                ));
            }
        }

        let control_ids = document
            .nodes
            .values()
            .filter_map(|node| node.control_id.as_deref())
            .collect::<BTreeSet<_>>();
        for required_control_id in contract.required_control_ids {
            if !control_ids.contains(*required_control_id) {
                offenders.push(format!(
                    "{} component `{}` should keep control id `{}`",
                    path.display(),
                    contract.component_name,
                    required_control_id
                ));
            }
        }
    }

    assert!(
        WORKBENCH_SHELL_SURFACE_CONTRACTS.len() >= 7,
        "workbench shell surface contract should cover the activity rail, toolbar, panels, status bar, and main band"
    );
    assert!(
        offenders.is_empty(),
        "workbench shell surface .zui assets must stay componentized and compose lower-level Workbench primitives before module assembly: {offenders:#?}"
    );
}

#[test]
fn workbench_component_drawer_composes_workbench_primitive_assets() {
    let path = editor_asset_root()
        .join("ui/editor/components")
        .join("workbench/shell/workbench_component_drawer.zui");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read `{}`: {error}", path.display()));
    let document = UiZuiAssetLoader::load_zui_str(&source)
        .unwrap_or_else(|error| panic!("parse `{}`: {error}", path.display()));
    let widget_imports = document
        .imports
        .widgets
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let node_components = document
        .nodes
        .values()
        .map(|node| node.component.as_str())
        .collect::<BTreeSet<_>>();

    let mut offenders = Vec::new();
    for contract in WORKBENCH_PRIMITIVE_CONTRACTS
        .iter()
        .filter(|contract| contract.sampled_in_component_drawer)
    {
        let import = format!(
            "res://ui/editor/components/{}#{}",
            contract.file_name, contract.component_name
        );
        if !widget_imports.contains(import.as_str()) {
            offenders.push(format!(
                "{} should import `{}` for componentized low-level samples",
                path.display(),
                import
            ));
        }
        if !node_components.contains(contract.component_name) {
            offenders.push(format!(
                "{} should mount `{}` instead of duplicating its structure inline",
                path.display(),
                contract.component_name
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "workbench component drawer must keep composing low-level Workbench primitives through .zui imports: {offenders:#?}"
    );
}
