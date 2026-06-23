use zircon_runtime_interface::ui::{component::UiValueKind, tree::UiDirtyFlags};

pub(super) fn render_dirty() -> UiDirtyFlags {
    UiDirtyFlags {
        render: true,
        ..UiDirtyFlags::default()
    }
}

fn virtualized_range_dirty() -> UiDirtyFlags {
    UiDirtyFlags {
        layout: true,
        hit_test: true,
        render: true,
        input: true,
        visible_range: true,
        ..UiDirtyFlags::default()
    }
}

pub(super) fn metadata_attribute_dirty(
    component: &str,
    property: &str,
    value_kind: UiValueKind,
) -> UiDirtyFlags {
    match property {
        "value" if is_render_only_numeric_value_component(component) => render_dirty(),
        "open" | "popup_open" if is_mui_overlay_component(component) => UiDirtyFlags {
            layout: true,
            hit_test: true,
            render: true,
            input: true,
            ..UiDirtyFlags::default()
        },
        _ if is_mui_overlay_component(component) && is_overlay_position_attribute(property) => {
            UiDirtyFlags {
                layout: true,
                hit_test: true,
                render: true,
                input: true,
                ..UiDirtyFlags::default()
            }
        }
        _ if is_mui_overlay_component(component) && is_overlay_interaction_attribute(property) => {
            UiDirtyFlags {
                layout: true,
                hit_test: true,
                render: true,
                input: true,
                ..UiDirtyFlags::default()
            }
        }
        _ if is_mui_feedback_component(component) && is_mui_feedback_attribute(property) => {
            UiDirtyFlags {
                layout: true,
                hit_test: true,
                render: true,
                text: true,
                input: true,
                ..UiDirtyFlags::default()
            }
        }
        _ if is_mui_transition_component(component) && is_transition_attribute(property) => {
            UiDirtyFlags {
                layout: matches!(property, "in" | "orientation" | "collapsed_size"),
                hit_test: true,
                render: true,
                input: true,
                ..UiDirtyFlags::default()
            }
        }
        "text" | "label" | "value" | "value_text" | "font_size" | "line_height" => UiDirtyFlags {
            layout: true,
            render: true,
            text: true,
            ..UiDirtyFlags::default()
        },
        "caret_offset"
        | "selection_anchor"
        | "selection_focus"
        | "composition_start"
        | "composition_end"
        | "composition_text"
        | "composition_restore_text" => render_dirty(),
        "read_only" | "readOnly" | "input_read_only" | "inputReadOnly" | "autofocus"
        | "auto_focus" | "autoFocus" => UiDirtyFlags {
            render: true,
            input: true,
            ..UiDirtyFlags::default()
        },
        _ if is_mui_customization_attribute(property) => UiDirtyFlags {
            layout: true,
            hit_test: true,
            render: true,
            text: true,
            input: true,
            ..UiDirtyFlags::default()
        },
        _ if is_virtualized_collection_component(component)
            && is_virtualized_range_attribute(property) =>
        {
            virtualized_range_dirty()
        }
        _ if is_layout_metadata_attribute(property) => UiDirtyFlags {
            layout: true,
            hit_test: true,
            render: true,
            ..UiDirtyFlags::default()
        },
        _ if matches!(value_kind, UiValueKind::Bool) && property.contains("visible") => {
            UiDirtyFlags {
                layout: true,
                hit_test: true,
                render: true,
                input: true,
                ..UiDirtyFlags::default()
            }
        }
        _ => render_dirty(),
    }
}

fn is_mui_customization_attribute(property: &str) -> bool {
    matches!(
        property,
        "mui_variant"
            | "mui_color"
            | "mui_size"
            | "mui_slots"
            | "mui_slot_props"
            | "mui_sx"
            | "mui_classes"
            | "variant"
            | "color"
            | "size"
            | "slots"
            | "slotProps"
            | "sx"
            | "classes"
            | "className"
    )
}

fn is_overlay_position_attribute(property: &str) -> bool {
    matches!(
        property,
        "placement"
            | "popup_anchor_x"
            | "popup_anchor_y"
            | "popup_anchor_width"
            | "popup_anchor_height"
            | "anchor_origin_vertical"
            | "anchor_origin_horizontal"
            | "transform_origin_vertical"
            | "transform_origin_horizontal"
            | "popup_offset_x"
            | "popup_offset_y"
            | "offset_x"
            | "offset_y"
    )
}

fn is_overlay_interaction_attribute(property: &str) -> bool {
    matches!(
        property,
        "disable_auto_focus"
            | "disableAutoFocus"
            | "disable_enforce_focus"
            | "disableEnforceFocus"
            | "disable_restore_focus"
            | "disableRestoreFocus"
            | "disable_escape_key_down"
            | "disableEscapeKeyDown"
            | "close_on_backdrop_click"
            | "closeOnBackdropClick"
            | "keep_mounted"
            | "keepMounted"
            | "aria_modal"
            | "ariaModal"
            | "aria_labelledby"
            | "ariaLabelledby"
            | "aria_describedby"
            | "ariaDescribedby"
            | "z_index"
            | "mui_z_index"
            | "zIndex"
            | "disable_portal"
            | "portal_layer"
    )
}

fn is_mui_feedback_attribute(property: &str) -> bool {
    matches!(
        property,
        "severity"
            | "message"
            | "icon"
            | "show_icon"
            | "iconMapping"
            | "closeText"
            | "auto_hide_duration_ms"
            | "autoHideDuration"
            | "resume_hide_duration_ms"
            | "resumeHideDuration"
            | "disable_window_blur_listener"
            | "disableWindowBlurListener"
            | "anchorOrigin"
    )
}

fn is_transition_attribute(property: &str) -> bool {
    matches!(
        property,
        "transition_kind"
            | "in"
            | "transition_in"
            | "transition_status"
            | "transition_progress"
            | "animation_progress"
            | "timeout_ms"
            | "transition_duration_ms"
            | "easing"
            | "transition_easing"
            | "direction"
            | "orientation"
            | "collapsed_size"
            | "mount_on_enter"
            | "unmount_on_exit"
    )
}

fn is_mui_overlay_component(component: &str) -> bool {
    matches!(
        component,
        "Modal"
            | "Dialog"
            | "AlertDialog"
            | "Popover"
            | "Popper"
            | "Tooltip"
            | "Snackbar"
            | "Menu"
            | "Drawer"
            | "Backdrop"
    )
}

fn is_mui_feedback_component(component: &str) -> bool {
    matches!(component, "Alert" | "Snackbar")
}

fn is_mui_transition_component(component: &str) -> bool {
    matches!(component, "Collapse" | "Fade" | "Grow" | "Slide" | "Zoom")
}

fn is_virtualized_collection_component(component: &str) -> bool {
    matches!(
        component,
        "VirtualList"
            | "List"
            | "ListView"
            | "MenuList"
            | "Table"
            | "TableContainer"
            | "DataGrid"
            | "Autocomplete"
            | "ImageList"
            | "TransferList"
            | "ScrollView"
    )
}

fn is_virtualized_range_attribute(property: &str) -> bool {
    matches!(
        property,
        "total_count"
            | "item_count"
            | "itemCount"
            | "row_count"
            | "rowCount"
            | "rows"
            | "items"
            | "viewport_start"
            | "viewport_count"
            | "visible_end"
            | "visibleEnd"
            | "requested_start"
            | "requestedStart"
            | "requested_count"
            | "requestedCount"
            | "item_extent"
            | "itemSize"
            | "row_height"
            | "rowHeight"
            | "overscan"
            | "overscan_count"
            | "overscanCount"
            | "scroll_offset"
            | "scrollTop"
            | "disable_virtualization"
            | "disableVirtualization"
    )
}

fn is_render_only_numeric_value_component(component: &str) -> bool {
    matches!(
        component,
        "RangeField"
            | "Slider"
            | "ProgressBar"
            | "Progress"
            | "LinearProgress"
            | "CircularProgress"
    )
}

fn is_layout_metadata_attribute(property: &str) -> bool {
    matches!(
        property,
        "layout" | "width" | "height" | "min_width" | "min_height" | "padding" | "gap"
    ) || property.starts_with("layout_")
}
