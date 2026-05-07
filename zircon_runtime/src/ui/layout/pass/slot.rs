use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{UiContainerKind, UiMargin, UiSlot, UiSlotKind},
    tree::UiTree,
};

pub(super) fn slot_for_container_child<'a>(
    tree: &'a UiTree,
    parent_id: UiNodeId,
    child_id: UiNodeId,
    container: UiContainerKind,
) -> Option<&'a UiSlot> {
    let slot_kind = slot_kind_for_container(container)?;
    tree.slots.iter().find(|slot| {
        slot.parent_id == parent_id && slot.child_id == child_id && slot.kind == slot_kind
    })
}

pub(super) fn ordered_children_for_container(
    tree: &UiTree,
    parent_id: UiNodeId,
    children: &[UiNodeId],
    container: UiContainerKind,
) -> Vec<UiNodeId> {
    let mut indexed: Vec<_> = children
        .iter()
        .copied()
        .enumerate()
        .map(|(index, child_id)| {
            let order = slot_for_container_child(tree, parent_id, child_id, container)
                .map(|slot| slot.order)
                .unwrap_or_default();
            (order, index, child_id)
        })
        .collect();
    indexed.sort_by_key(|(order, index, _)| (*order, *index));
    indexed
        .into_iter()
        .map(|(_, _, child_id)| child_id)
        .collect()
}

pub(super) fn has_slot_frame_policy(slot: Option<&UiSlot>) -> bool {
    slot.is_some_and(|slot| {
        slot.padding != UiMargin::default() || slot.alignment != Default::default()
    })
}

pub(super) fn slot_padding(slot: Option<&UiSlot>) -> UiMargin {
    slot.filter(|slot| slot.padding != UiMargin::default())
        .map(|slot| slot.padding)
        .unwrap_or_default()
}

fn slot_kind_for_container(container: UiContainerKind) -> Option<UiSlotKind> {
    match container {
        UiContainerKind::Free => Some(UiSlotKind::Free),
        UiContainerKind::Container => Some(UiSlotKind::Container),
        UiContainerKind::Overlay => Some(UiSlotKind::Overlay),
        UiContainerKind::Space => None,
        UiContainerKind::HorizontalBox(_) | UiContainerKind::VerticalBox(_) => {
            Some(UiSlotKind::Linear)
        }
        UiContainerKind::WrapBox(_) => Some(UiSlotKind::Flow),
        UiContainerKind::GridBox(_) => Some(UiSlotKind::Grid),
        UiContainerKind::ScrollableBox(_) => Some(UiSlotKind::Scrollable),
    }
}
