use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::{UiComponentDescriptor, UiComponentEventKind};

use super::super::{assert_has_event, assert_has_prop};

pub(super) fn assert_descriptors(registry: &UiComponentDescriptorRegistry) {
    assert_transfer_list(registry);
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
        "{} missing TransferList slot `{slot_name}`",
        descriptor.id
    );
}
