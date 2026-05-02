use std::collections::BTreeMap;

use zircon_runtime::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::{
    component::UiSlotSchema,
    template::{UiChildMount, UiNodeDefinition, UiNodeDefinitionKind},
};

pub(crate) fn native_node_accepts_children(node: &UiNodeDefinition) -> bool {
    !available_native_slot_names(node).is_empty()
}

pub(crate) fn default_native_mount(node: &UiNodeDefinition) -> Option<String> {
    available_native_slot_names(node).into_iter().next()
}

pub(crate) fn available_native_slot_names(node: &UiNodeDefinition) -> Vec<String> {
    if !matches!(node.kind, UiNodeDefinitionKind::Native) {
        return Vec::new();
    }
    let Some(widget_type) = node.widget_type.as_deref() else {
        return Vec::new();
    };
    let registry = UiComponentDescriptorRegistry::editor_showcase();
    let Some(descriptor) = registry.descriptor(widget_type) else {
        return Vec::new();
    };
    available_slots(&descriptor.slot_schema, &node.children)
}

fn available_slots(slots: &[UiSlotSchema], children: &[UiChildMount]) -> Vec<String> {
    let mut counts = BTreeMap::<String, usize>::new();
    for child in children {
        let slot_name = child.mount.clone().unwrap_or_default();
        let entry = counts.entry(slot_name).or_insert(0);
        *entry += 1;
    }

    slots
        .iter()
        .filter_map(|slot| {
            let occupied = counts.get(&slot.name).copied().unwrap_or_default();
            (slot.multiple || occupied == 0).then(|| slot.name.clone())
        })
        .collect()
}
