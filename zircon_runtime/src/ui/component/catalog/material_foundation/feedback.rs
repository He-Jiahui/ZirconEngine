use super::shared::*;

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        primitive("Alert", "Alert", UiComponentCategory::Feedback, "alert")
            .with_prop(text_prop())
            .with_prop(enum_prop("severity", "info"))
            .slot(UiSlotSchema::new("action"))
            .event(UiComponentEventKind::ClosePopup),
        primitive(
            "Backdrop",
            "Backdrop",
            UiComponentCategory::Feedback,
            "backdrop",
        )
        .with_prop(bool_prop("open", false))
        .event(UiComponentEventKind::ClosePopup),
        composite("Dialog", "Dialog", UiComponentCategory::Feedback, "dialog")
            .with_prop(bool_prop("open", false))
            .with_prop(text_prop())
            .slot(UiSlotSchema::new("title"))
            .slot(UiSlotSchema::new("content").multiple(true))
            .slot(UiSlotSchema::new("actions").multiple(true))
            .events([
                UiComponentEventKind::OpenPopup,
                UiComponentEventKind::ClosePopup,
                UiComponentEventKind::Commit,
            ]),
        composite("Modal", "Modal", UiComponentCategory::Container, "modal")
            .with_prop(bool_prop("open", false))
            .slot(UiSlotSchema::new("content").multiple(true))
            .event(UiComponentEventKind::ClosePopup),
        composite(
            "Popover",
            "Popover",
            UiComponentCategory::Feedback,
            "popover",
        )
        .with_prop(bool_prop("open", false))
        .with_prop(enum_prop("placement", "bottom"))
        .slot(UiSlotSchema::new("anchor").required(true))
        .slot(UiSlotSchema::new("content").multiple(true))
        .events([
            UiComponentEventKind::OpenPopupAt,
            UiComponentEventKind::ClosePopup,
        ]),
        composite("Popper", "Popper", UiComponentCategory::Feedback, "popper")
            .with_prop(bool_prop("open", false))
            .with_prop(enum_prop("placement", "bottom-start"))
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
        .with_prop(enum_prop("variant", "rounded")),
        composite(
            "Snackbar",
            "Snackbar",
            UiComponentCategory::Feedback,
            "snackbar",
        )
        .with_prop(bool_prop("open", false))
        .with_prop(text_prop())
        .slot(UiSlotSchema::new("action"))
        .events([
            UiComponentEventKind::OpenPopup,
            UiComponentEventKind::ClosePopup,
        ]),
        composite(
            "Tooltip",
            "Tooltip",
            UiComponentCategory::Feedback,
            "tooltip",
        )
        .with_prop(bool_prop("open", false))
        .with_prop(enum_prop("placement", "top"))
        .slot(UiSlotSchema::new("anchor").required(true))
        .slot(UiSlotSchema::new("content").required(true))
        .events([UiComponentEventKind::Hover, UiComponentEventKind::Focus]),
        composite(
            "SpeedDial",
            "Speed Dial",
            UiComponentCategory::Input,
            "speed-dial",
        )
        .with_prop(bool_prop("open", false))
        .slot(UiSlotSchema::new("actions").multiple(true))
        .events([
            UiComponentEventKind::OpenPopup,
            UiComponentEventKind::SelectOption,
            UiComponentEventKind::ClosePopup,
        ]),
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
