use super::shared::*;

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        command_palette_descriptor(),
        confirm_dialog_descriptor(),
        notification_center_descriptor(),
        drag_overlay_descriptor(),
    ]
}

fn command_palette_descriptor() -> UiComponentDescriptor {
    overlay_layer_props(modal_interaction_props(popup_position_props(
        editor_panel_component(
            "CommandPalette",
            "Command Palette",
            UiComponentCategory::Input,
            "command-palette",
        )
        .layout_role(UiComponentLayoutRole::Popup)
        .with_prop(bool_prop("open", false))
        .with_prop(bool_prop("popup_open", false))
        .with_prop(default_string_prop("query", ""))
        .with_prop(default_string_prop("placeholder", "Search commands"))
        .with_prop(array_prop("commands"))
        .with_prop(array_prop("filtered_commands"))
        .with_prop(array_prop("recent_commands"))
        .with_prop(array_prop("disabled_commands"))
        .with_prop(default_string_prop("selected_command_id", ""))
        .with_prop(int_prop("focused_index", 0))
        .with_prop(bool_prop("keyboard_navigation", true))
        .with_prop(default_string_prop("empty_text", "No commands found"))
        .with_prop(default_string_prop("command_source", "")),
        "top",
    )))
    .slot(UiSlotSchema::new("input"))
    .slot(UiSlotSchema::new("list").multiple(true))
    .slot(UiSlotSchema::new("emptyState"))
    .slot(UiSlotSchema::new("footer"))
    .events([
        UiComponentEventKind::OpenPopup,
        UiComponentEventKind::KeyboardText,
        UiComponentEventKind::KeyboardAction,
        UiComponentEventKind::ValueChanged,
        UiComponentEventKind::SelectOption,
        UiComponentEventKind::Commit,
        UiComponentEventKind::ClosePopup,
    ])
    .requires_host_capability(UiHostCapability::TextInput)
}

fn confirm_dialog_descriptor() -> UiComponentDescriptor {
    overlay_layer_props(modal_interaction_props(popup_position_props(
        editor_panel_component(
            "ConfirmDialog",
            "Confirm Dialog",
            UiComponentCategory::Feedback,
            "confirm-dialog",
        )
        .with_prop(bool_prop("open", false))
        .with_prop(bool_prop("popup_open", false))
        .with_prop(default_string_prop("title", "Confirm action"))
        .with_prop(default_string_prop(
            "message",
            "This action cannot be undone.",
        ))
        .with_prop(default_string_prop("confirm_text", "Confirm"))
        .with_prop(default_string_prop("cancel_text", "Cancel"))
        .with_prop(default_string_prop("confirm_action_id", "confirm"))
        .with_prop(default_string_prop("cancel_action_id", "cancel"))
        .with_prop(enum_prop_with_options(
            "severity",
            "warning",
            ["info", "warning", "error"]
                .into_iter()
                .map(enum_option_descriptor),
        ))
        .with_prop(enum_prop_with_options(
            "default_action",
            "cancel",
            ["cancel", "confirm"]
                .into_iter()
                .map(enum_option_descriptor),
        ))
        .with_prop(bool_prop("destructive", false))
        .with_prop(bool_prop("confirm_enabled", true))
        .with_prop(bool_prop("requires_explicit_action", true))
        .with_prop(default_string_prop("dialog_action_id", ""))
        .with_prop(bool_prop("confirmed", false)),
        "center",
    )))
    .slot(UiSlotSchema::new("title"))
    .slot(UiSlotSchema::new("content").multiple(true))
    .slot(UiSlotSchema::new("actions").multiple(true))
    .slot(UiSlotSchema::new("confirmButton"))
    .slot(UiSlotSchema::new("cancelButton"))
    .events([
        UiComponentEventKind::OpenPopup,
        UiComponentEventKind::ClosePopup,
        UiComponentEventKind::Commit,
        UiComponentEventKind::ValueChanged,
        UiComponentEventKind::KeyboardAction,
    ])
}

fn drag_overlay_descriptor() -> UiComponentDescriptor {
    overlay_layer_props(
        editor_panel_component(
            "DragOverlay",
            "Drag Overlay",
            UiComponentCategory::Feedback,
            "drag-overlay",
        )
        .layout_role(UiComponentLayoutRole::Popup)
        .with_prop(bool_prop("open", false))
        .with_prop(bool_prop("dragging", false))
        .with_prop(bool_prop("drop_hovered", false))
        .with_prop(bool_prop("active_drag_target", false))
        .with_prop(enum_prop_with_options(
            "payload_kind",
            "unknown",
            ["asset", "scene-instance", "object", "unknown"]
                .into_iter()
                .map(enum_option_descriptor),
        ))
        .with_prop(default_string_prop("payload_label", ""))
        .with_prop(default_string_prop("payload_reference", ""))
        .with_prop(default_string_prop("source_control_id", ""))
        .with_prop(default_string_prop("target_control_id", ""))
        .with_prop(optional_float_prop("cursor_x"))
        .with_prop(optional_float_prop("cursor_y"))
        .with_prop(float_prop("offset_x", 12.0))
        .with_prop(float_prop("offset_y", 12.0))
        .with_prop(float_prop("preview_width", 160.0))
        .with_prop(float_prop("preview_height", 32.0))
        .with_prop(bool_prop("drop_allowed", true))
        .with_prop(optional_float_prop("drop_target_x"))
        .with_prop(optional_float_prop("drop_target_y"))
        .with_prop(float_prop("drop_target_width", 0.0))
        .with_prop(float_prop("drop_target_height", 0.0))
        .with_prop(enum_prop_with_options(
            "drop_indicator_edge",
            "none",
            ["none", "inside", "top", "bottom", "left", "right"]
                .into_iter()
                .map(enum_option_descriptor),
        ))
        .with_prop(default_string_prop("drop_indicator_text", "")),
    )
    .slot(UiSlotSchema::new("preview"))
    .slot(UiSlotSchema::new("indicator"))
    .events([
        UiComponentEventKind::BeginDrag,
        UiComponentEventKind::DragDelta,
        UiComponentEventKind::EndDrag,
        UiComponentEventKind::DropHover,
        UiComponentEventKind::ActiveDragTarget,
        UiComponentEventKind::DropReference,
    ])
}

fn notification_center_descriptor() -> UiComponentDescriptor {
    overlay_layer_props(popup_position_props(
        editor_panel_component(
            "NotificationCenter",
            "Notification Center",
            UiComponentCategory::Feedback,
            "notification-center",
        )
        .layout_role(UiComponentLayoutRole::Popup)
        .with_prop(bool_prop("open", false))
        .with_prop(bool_prop("popup_open", false))
        .with_prop(array_prop("notifications"))
        .with_prop(int_prop("unread_count", 0))
        .with_prop(int_prop("focused_index", -1))
        .with_prop(default_string_prop("selected_notification_id", ""))
        .with_prop(int_prop("visible_limit", 5))
        .with_prop(bool_prop("keyboard_navigation", true))
        .with_prop(default_string_prop("title", "Notifications"))
        .with_prop(default_string_prop("empty_text", "No notifications"))
        .slot(UiSlotSchema::new("header"))
        .slot(UiSlotSchema::new("item").multiple(true))
        .slot(UiSlotSchema::new("emptyState"))
        .slot(UiSlotSchema::new("actions"))
        .events([
            UiComponentEventKind::OpenPopup,
            UiComponentEventKind::ClosePopup,
            UiComponentEventKind::SelectOption,
            UiComponentEventKind::ValueChanged,
            UiComponentEventKind::KeyboardAction,
            UiComponentEventKind::OpenReference,
            UiComponentEventKind::ClearReference,
        ]),
        "bottom-end",
    ))
}
