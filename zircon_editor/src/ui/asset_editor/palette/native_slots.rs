use zircon_runtime::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::{
    component::UiSlotSchema,
    template::{UiChildMount, UiNodeDefinition, UiNodeDefinitionKind},
};

pub(crate) fn native_node_accepts_children(node: &UiNodeDefinition) -> bool {
    native_slot_schemas(node).is_some_and(|slots| {
        slots
            .iter()
            .any(|slot| native_slot_is_available(slot, &node.children))
    })
}

pub(crate) fn default_native_mount(node: &UiNodeDefinition) -> Option<String> {
    native_slot_schemas(node).and_then(|slots| {
        slots
            .iter()
            .find(|slot| native_slot_is_available(slot, &node.children))
            .map(|slot| slot.name.clone())
    })
}

fn native_slot_schemas(node: &UiNodeDefinition) -> Option<&'static [UiSlotSchema]> {
    if !matches!(node.kind, UiNodeDefinitionKind::Native) {
        return None;
    }
    let widget_type = node.widget_type.as_deref()?;
    let registry = UiComponentDescriptorRegistry::editor_showcase_shared();
    registry
        .descriptor(widget_type)
        .map(|descriptor| descriptor.slot_schema.as_slice())
}

fn native_slot_is_available(slot: &UiSlotSchema, children: &[UiChildMount]) -> bool {
    slot.multiple
        || !children
            .iter()
            .any(|child| child.mount.as_deref().unwrap_or_default() == slot.name.as_str())
}

#[cfg(test)]
#[path = "tests/native_slots.rs"]
mod tests;
