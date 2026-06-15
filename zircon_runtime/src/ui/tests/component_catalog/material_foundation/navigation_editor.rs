use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEventKind, UiHostCapability, UiValue,
};

use super::super::{assert_has_event, assert_has_prop};

pub(super) fn assert_descriptors(registry: &UiComponentDescriptorRegistry) {
    assert_transfer_list(registry);
    assert_context_menu(registry);
}

fn assert_transfer_list(registry: &UiComponentDescriptorRegistry) {
    let transfer_list = registry
        .descriptor("TransferList")
        .expect("TransferList descriptor");
    for prop in [
        "source_items",
        "sourceItems",
        "target_items",
        "targetItems",
        "selected_items",
        "selectedItems",
        "source_selected_items",
        "sourceSelectedItems",
        "target_selected_items",
        "targetSelectedItems",
        "disabled_items",
        "disabledItems",
        "disabled_actions",
        "disabledActions",
    ] {
        assert_has_prop(transfer_list, prop);
    }
    for slot in ["source", "target", "actions"] {
        assert_has_slot(transfer_list, slot);
    }
    assert_has_event(transfer_list, UiComponentEventKind::SelectOption);
    assert_has_event(transfer_list, UiComponentEventKind::MoveElement);
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

fn assert_context_menu(registry: &UiComponentDescriptorRegistry) {
    let context_menu = registry
        .descriptor("ContextMenu")
        .expect("ContextMenu descriptor");
    for prop in [
        "open",
        "popup_open",
        "options",
        "context_target",
        "context_target_path",
        "focused_index",
        "disabled_options",
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
        "z_index",
        "disable_portal",
        "portal_layer",
    ] {
        assert_has_prop(context_menu, prop);
    }
    for slot in ["paper", "list", "transition", "items"] {
        assert_has_slot(context_menu, slot);
    }
    for event in [
        UiComponentEventKind::KeyboardAction,
        UiComponentEventKind::KeyboardText,
        UiComponentEventKind::TypeaheadExpired,
        UiComponentEventKind::ValueChanged,
        UiComponentEventKind::Focus,
        UiComponentEventKind::OpenPopupAt,
        UiComponentEventKind::SelectOption,
        UiComponentEventKind::ClosePopup,
        UiComponentEventKind::Commit,
    ] {
        assert_has_event(context_menu, event);
    }
    assert_default_value(
        context_menu,
        "placement",
        UiValue::Enum("right-start".to_string()),
    );
    assert_default_value(context_menu, "close_on_backdrop_click", UiValue::Bool(true));
    assert!(context_menu
        .required_host_capabilities
        .contains(&UiHostCapability::Editor));
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
