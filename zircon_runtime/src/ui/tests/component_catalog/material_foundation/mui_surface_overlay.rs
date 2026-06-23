use super::*;

#[test]
fn material_editor_foundation_catalog_covers_mui_surface_overlay_contracts() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
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
}
