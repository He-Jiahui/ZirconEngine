use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEventKind, UiHostCapability, UiValue,
};

use super::super::{assert_has_event, assert_has_prop};
use super::assert_enum_options;

pub(super) fn assert_descriptors(registry: &UiComponentDescriptorRegistry) {
    assert_command_palette(registry);
    assert_dialog(registry);
    assert_confirm_dialog(registry);
    assert_snackbar(registry);
    assert_notification_center(registry);
    assert_drag_overlay(registry);
}

fn assert_command_palette(registry: &UiComponentDescriptorRegistry) {
    let command_palette = registry
        .descriptor("CommandPalette")
        .expect("CommandPalette descriptor");
    for prop in [
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
        "disable_auto_focus",
        "close_on_backdrop_click",
        "z_index",
        "portal_layer",
    ] {
        assert_has_prop(command_palette, prop);
    }
    for slot in ["input", "list", "emptyState", "footer"] {
        assert_has_slot(command_palette, slot);
    }
    for event in [
        UiComponentEventKind::OpenPopup,
        UiComponentEventKind::KeyboardText,
        UiComponentEventKind::KeyboardAction,
        UiComponentEventKind::ValueChanged,
        UiComponentEventKind::SelectOption,
        UiComponentEventKind::Commit,
        UiComponentEventKind::ClosePopup,
    ] {
        assert_has_event(command_palette, event);
    }
    assert_default_value(
        command_palette,
        "placement",
        UiValue::Enum("top".to_string()),
    );
    assert_default_value(
        command_palette,
        "placeholder",
        UiValue::String("Search commands".to_string()),
    );
    assert_default_value(
        command_palette,
        "close_on_backdrop_click",
        UiValue::Bool(true),
    );
    assert!(command_palette
        .required_host_capabilities
        .contains(&UiHostCapability::Editor));
    assert!(command_palette
        .required_host_capabilities
        .contains(&UiHostCapability::TextInput));
}

fn assert_dialog(registry: &UiComponentDescriptorRegistry) {
    let dialog = registry.descriptor("Dialog").expect("Dialog descriptor");
    for prop in [
        "open",
        "popup_open",
        "text",
        "title",
        "message",
        "dialog_action_id",
        "disable_escape_key_down",
        "close_on_backdrop_click",
        "z_index",
        "portal_layer",
    ] {
        assert_has_prop(dialog, prop);
    }
    for slot in ["title", "content", "actions"] {
        assert_has_slot(dialog, slot);
    }
    for event in [
        UiComponentEventKind::OpenPopup,
        UiComponentEventKind::ClosePopup,
        UiComponentEventKind::Commit,
        UiComponentEventKind::KeyboardAction,
    ] {
        assert_has_event(dialog, event);
    }
}

fn assert_confirm_dialog(registry: &UiComponentDescriptorRegistry) {
    let confirm_dialog = registry
        .descriptor("ConfirmDialog")
        .expect("ConfirmDialog descriptor");
    for prop in [
        "open",
        "popup_open",
        "title",
        "message",
        "confirm_text",
        "cancel_text",
        "confirm_action_id",
        "cancel_action_id",
        "dialog_action_id",
        "confirmed",
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
        "disable_auto_focus",
        "close_on_backdrop_click",
        "z_index",
        "portal_layer",
    ] {
        assert_has_prop(confirm_dialog, prop);
    }
    assert_enum_options(confirm_dialog, "severity", &["info", "warning", "error"]);
    assert_enum_options(confirm_dialog, "default_action", &["cancel", "confirm"]);
    for slot in [
        "title",
        "content",
        "actions",
        "confirmButton",
        "cancelButton",
    ] {
        assert_has_slot(confirm_dialog, slot);
    }
    for event in [
        UiComponentEventKind::OpenPopup,
        UiComponentEventKind::ClosePopup,
        UiComponentEventKind::Commit,
        UiComponentEventKind::ValueChanged,
        UiComponentEventKind::KeyboardAction,
    ] {
        assert_has_event(confirm_dialog, event);
    }
    assert_default_value(
        confirm_dialog,
        "placement",
        UiValue::Enum("center".to_string()),
    );
    assert_default_value(
        confirm_dialog,
        "default_action",
        UiValue::Enum("cancel".to_string()),
    );
    assert_default_value(
        confirm_dialog,
        "requires_explicit_action",
        UiValue::Bool(true),
    );
    assert!(confirm_dialog
        .required_host_capabilities
        .contains(&UiHostCapability::Editor));
}

fn assert_notification_center(registry: &UiComponentDescriptorRegistry) {
    let notification_center = registry
        .descriptor("NotificationCenter")
        .expect("NotificationCenter descriptor");
    for prop in [
        "open",
        "popup_open",
        "notifications",
        "unread_count",
        "focused_index",
        "selected_notification_id",
        "visible_limit",
        "keyboard_navigation",
        "title",
        "empty_text",
        "placement",
        "popup_anchor_x",
        "popup_anchor_y",
        "popup_anchor_width",
        "popup_anchor_height",
        "z_index",
        "portal_layer",
    ] {
        assert_has_prop(notification_center, prop);
    }
    for slot in ["header", "item", "emptyState", "actions"] {
        assert_has_slot(notification_center, slot);
    }
    for event in [
        UiComponentEventKind::OpenPopup,
        UiComponentEventKind::ClosePopup,
        UiComponentEventKind::SelectOption,
        UiComponentEventKind::ValueChanged,
        UiComponentEventKind::KeyboardAction,
        UiComponentEventKind::OpenReference,
        UiComponentEventKind::ClearReference,
    ] {
        assert_has_event(notification_center, event);
    }
    assert_default_value(notification_center, "open", UiValue::Bool(false));
    assert_default_value(
        notification_center,
        "placement",
        UiValue::Enum("bottom-end".to_string()),
    );
    assert_default_value(
        notification_center,
        "title",
        UiValue::String("Notifications".to_string()),
    );
    assert_default_value(
        notification_center,
        "portal_layer",
        UiValue::String("overlay".to_string()),
    );
    assert!(notification_center
        .required_host_capabilities
        .contains(&UiHostCapability::Editor));
}

fn assert_snackbar(registry: &UiComponentDescriptorRegistry) {
    let snackbar = registry
        .descriptor("Snackbar")
        .expect("Snackbar descriptor");
    for prop in [
        "open",
        "text",
        "message",
        "toast_queue",
        "current_toast_id",
        "expired_toast_id",
        "queue_length",
        "action_label",
        "auto_hide_duration_ms",
        "autoHideDuration",
        "resume_hide_duration_ms",
        "resumeHideDuration",
        "anchor_origin_vertical",
        "anchor_origin_horizontal",
        "anchorOrigin",
        "z_index",
        "portal_layer",
    ] {
        assert_has_prop(snackbar, prop);
    }
    for event in [
        UiComponentEventKind::OpenPopup,
        UiComponentEventKind::ClosePopup,
        UiComponentEventKind::ValueChanged,
        UiComponentEventKind::Commit,
    ] {
        assert_has_event(snackbar, event);
    }
    assert_default_value(snackbar, "open", UiValue::Bool(false));
    assert_default_value(snackbar, "queue_length", UiValue::Int(0));
    assert_default_value(snackbar, "current_toast_id", UiValue::String(String::new()));
    assert_default_value(snackbar, "expired_toast_id", UiValue::String(String::new()));
}

fn assert_drag_overlay(registry: &UiComponentDescriptorRegistry) {
    let drag_overlay = registry
        .descriptor("DragOverlay")
        .expect("DragOverlay descriptor");
    for prop in [
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
        "portal_layer",
    ] {
        assert_has_prop(drag_overlay, prop);
    }
    assert_enum_options(
        drag_overlay,
        "payload_kind",
        &["asset", "scene-instance", "object", "unknown"],
    );
    assert_enum_options(
        drag_overlay,
        "drop_indicator_edge",
        &["none", "inside", "top", "bottom", "left", "right"],
    );
    for slot in ["preview", "indicator"] {
        assert_has_slot(drag_overlay, slot);
    }
    for event in [
        UiComponentEventKind::BeginDrag,
        UiComponentEventKind::DragDelta,
        UiComponentEventKind::EndDrag,
        UiComponentEventKind::DropHover,
        UiComponentEventKind::ActiveDragTarget,
        UiComponentEventKind::DropReference,
    ] {
        assert_has_event(drag_overlay, event);
    }
    assert_default_value(drag_overlay, "open", UiValue::Bool(false));
    assert_default_value(drag_overlay, "drop_allowed", UiValue::Bool(true));
    assert_default_value(
        drag_overlay,
        "portal_layer",
        UiValue::String("overlay".to_string()),
    );
    assert!(drag_overlay
        .required_host_capabilities
        .contains(&UiHostCapability::Editor));
}

fn assert_has_slot(descriptor: &UiComponentDescriptor, slot_name: &str) {
    assert!(
        descriptor
            .slot_schema
            .iter()
            .any(|slot| slot.name == slot_name),
        "{} missing slot `{slot_name}`",
        descriptor.id
    );
}

fn assert_default_value(descriptor: &UiComponentDescriptor, prop_name: &str, expected: UiValue) {
    assert_eq!(
        descriptor
            .prop(prop_name)
            .and_then(|prop| prop.default_value.as_ref()),
        Some(&expected),
        "{} should default `{prop_name}` to {expected:?}",
        descriptor.id
    );
}
