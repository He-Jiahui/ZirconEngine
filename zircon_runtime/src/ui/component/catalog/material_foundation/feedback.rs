use super::shared::*;

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        primitive("Alert", "Alert", UiComponentCategory::Feedback, "alert")
            .with_prop(text_prop())
            .with_prop(enum_prop_with_options(
                "severity",
                "success",
                ["success", "info", "warning", "error"]
                    .into_iter()
                    .map(enum_option_descriptor),
            ))
            .with_prop(enum_prop_with_options(
                "variant",
                "standard",
                ["standard", "filled", "outlined"]
                    .into_iter()
                    .map(enum_option_descriptor),
            ))
            .with_prop(default_string_prop("color", ""))
            .with_prop(icon_prop())
            .with_prop(bool_prop("show_icon", true))
            .with_prop(map_prop("iconMapping"))
            .with_prop(default_string_prop("closeText", "Close"))
            .slot(UiSlotSchema::new("icon"))
            .slot(UiSlotSchema::new("message").multiple(true))
            .slot(UiSlotSchema::new("action"))
            .slot(UiSlotSchema::new("closeButton"))
            .slot(UiSlotSchema::new("closeIcon"))
            .event(UiComponentEventKind::ClosePopup),
        primitive(
            "AlertTitle",
            "Alert Title",
            UiComponentCategory::Feedback,
            "alert-title",
        )
        .with_prop(text_prop()),
        overlay_layer_props(
            primitive(
                "Backdrop",
                "Backdrop",
                UiComponentCategory::Feedback,
                "backdrop",
            )
            .with_prop(bool_prop("open", false))
            .event(UiComponentEventKind::ClosePopup),
        ),
        overlay_layer_props(modal_interaction_props(
            composite("Dialog", "Dialog", UiComponentCategory::Feedback, "dialog")
                .with_prop(bool_prop("open", false))
                .with_prop(text_prop()),
        ))
        .slot(UiSlotSchema::new("title"))
        .slot(UiSlotSchema::new("content").multiple(true))
        .slot(UiSlotSchema::new("actions").multiple(true))
        .events([
            UiComponentEventKind::OpenPopup,
            UiComponentEventKind::ClosePopup,
            UiComponentEventKind::Commit,
        ]),
        overlay_layer_props(modal_interaction_props(
            composite("Modal", "Modal", UiComponentCategory::Container, "modal")
                .with_prop(bool_prop("open", false)),
        ))
        .slot(UiSlotSchema::new("content").multiple(true))
        .event(UiComponentEventKind::ClosePopup),
        overlay_layer_props(modal_interaction_props(popup_position_props(
            composite(
                "Popover",
                "Popover",
                UiComponentCategory::Feedback,
                "popover",
            )
            .with_prop(bool_prop("open", false)),
            "bottom",
        )))
        .slot(UiSlotSchema::new("anchor").required(true))
        .slot(UiSlotSchema::new("content").multiple(true))
        .events([
            UiComponentEventKind::OpenPopupAt,
            UiComponentEventKind::ClosePopup,
        ]),
        overlay_layer_props(popup_position_props(
            composite("Popper", "Popper", UiComponentCategory::Feedback, "popper")
                .with_prop(bool_prop("open", false)),
            "bottom-start",
        ))
        .with_prop(bool_prop("needs_support", true))
        .slot(UiSlotSchema::new("anchor").required(true))
        .slot(UiSlotSchema::new("content").multiple(true))
        .events([
            UiComponentEventKind::OpenPopupAt,
            UiComponentEventKind::ClosePopup,
        ]),
        primitive(
            "Progress",
            "Progress",
            UiComponentCategory::Feedback,
            "progress",
        )
        .with_prop(number_value_prop())
        .with_prop(enum_prop("variant", "linear")),
        primitive(
            "Skeleton",
            "Skeleton",
            UiComponentCategory::Feedback,
            "skeleton",
        )
        .with_prop(default_string_prop("component", "span"))
        .with_prop(enum_prop_with_options(
            "variant",
            "text",
            ["text", "rectangular", "rounded", "circular"]
                .into_iter()
                .map(enum_option_descriptor),
        ))
        .with_prop(default_string_prop("animation", "pulse"))
        .with_prop(optional_float_prop("width"))
        .with_prop(optional_float_prop("height"))
        .slot(UiSlotSchema::new("children").multiple(true)),
        overlay_layer_props(composite(
            "Snackbar",
            "Snackbar",
            UiComponentCategory::Feedback,
            "snackbar",
        ))
        .with_prop(bool_prop("open", false))
        .with_prop(text_prop())
        .with_prop(default_string_prop("message", ""))
        .with_prop(int_prop("auto_hide_duration_ms", 0))
        .with_prop(int_prop("autoHideDuration", 0))
        .with_prop(int_prop("resume_hide_duration_ms", 0))
        .with_prop(int_prop("resumeHideDuration", 0))
        .with_prop(bool_prop("disable_window_blur_listener", false))
        .with_prop(bool_prop("disableWindowBlurListener", false))
        .with_prop(enum_prop_with_options(
            "anchor_origin_vertical",
            "bottom",
            ["top", "bottom"].into_iter().map(enum_option_descriptor),
        ))
        .with_prop(enum_prop_with_options(
            "anchor_origin_horizontal",
            "left",
            ["left", "center", "right"]
                .into_iter()
                .map(enum_option_descriptor),
        ))
        .with_prop(map_prop("anchorOrigin"))
        .slot(UiSlotSchema::new("action"))
        .events([
            UiComponentEventKind::OpenPopup,
            UiComponentEventKind::ClosePopup,
        ]),
        composite(
            "SnackbarContent",
            "Snackbar Content",
            UiComponentCategory::Feedback,
            "snackbar-content",
        )
        .with_prop(default_string_prop("message", ""))
        .with_prop(default_string_prop("role", "alert"))
        .slot(UiSlotSchema::new("message").multiple(true))
        .slot(UiSlotSchema::new("action")),
        overlay_layer_props(popup_position_props(
            composite(
                "Tooltip",
                "Tooltip",
                UiComponentCategory::Feedback,
                "tooltip",
            )
            .with_prop(bool_prop("open", false)),
            "top",
        ))
        .slot(UiSlotSchema::new("anchor").required(true))
        .slot(UiSlotSchema::new("content").required(true))
        .events([UiComponentEventKind::Hover, UiComponentEventKind::Focus]),
        overlay_layer_props(composite(
            "SpeedDial",
            "Speed Dial",
            UiComponentCategory::Input,
            "speed-dial",
        ))
        .with_prop(bool_prop("open", false))
        .slot(UiSlotSchema::new("actions").multiple(true))
        .events([
            UiComponentEventKind::OpenPopup,
            UiComponentEventKind::SelectOption,
            UiComponentEventKind::ClosePopup,
        ]),
        primitive(
            "SpeedDialAction",
            "Speed Dial Action",
            UiComponentCategory::Input,
            "speed-dial-action",
        )
        .with_prop(icon_prop())
        .with_prop(default_string_prop("tooltipTitle", ""))
        .with_prop(bool_prop("tooltipOpen", false))
        .with_prop(enum_prop_with_options(
            "tooltipPlacement",
            "left",
            ["left", "right"].into_iter().map(enum_option_descriptor),
        ))
        .with_prop(bool_prop("open", false))
        .with_prop(int_prop("delay", 0))
        .slot(UiSlotSchema::new("fab"))
        .slot(UiSlotSchema::new("icon"))
        .slot(UiSlotSchema::new("staticTooltipLabel"))
        .events([
            UiComponentEventKind::SelectOption,
            UiComponentEventKind::Commit,
        ]),
        primitive(
            "SpeedDialIcon",
            "Speed Dial Icon",
            UiComponentCategory::Visual,
            "speed-dial-icon",
        )
        .with_prop(icon_prop())
        .with_prop(bool_prop("open", false))
        .with_prop(default_string_prop("openIcon", ""))
        .slot(UiSlotSchema::new("icon"))
        .slot(UiSlotSchema::new("openIcon"))
        .requires_render_capability(UiRenderCapability::Vector),
        editor_panel_component(
            "StatusActionControls",
            "Status Action Controls",
            UiComponentCategory::Feedback,
            "status-action-controls",
        )
        .with_prop(string_prop("status"))
        .slot(UiSlotSchema::new("actions").multiple(true))
        .events([
            UiComponentEventKind::Commit,
            UiComponentEventKind::ValueChanged,
        ]),
    ]
}
