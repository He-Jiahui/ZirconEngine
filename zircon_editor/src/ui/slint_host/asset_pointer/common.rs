use zircon_runtime::ui::dispatch::UiPointerDispatcher;
use zircon_runtime_interface::ui::{
    dispatch::UiPointerDispatchEffect,
    event_ui::{UiNodeId, UiStateFlags},
    surface::UiPointerEventKind,
};

pub(super) const ROOT_NODE_ID: UiNodeId = UiNodeId::new(1);
pub(super) const VIEWPORT_NODE_ID: UiNodeId = UiNodeId::new(2);
const ITEM_NODE_ID_BASE: u64 = 100;

pub(super) fn item_node_id(index: usize) -> UiNodeId {
    UiNodeId::new(ITEM_NODE_ID_BASE + index as u64)
}

pub(super) fn register_handled_pointer_node(
    dispatcher: &mut UiPointerDispatcher,
    node_id: UiNodeId,
) {
    dispatcher.register(node_id, UiPointerEventKind::Move, |_context| {
        UiPointerDispatchEffect::handled()
    });
    dispatcher.register(node_id, UiPointerEventKind::Down, |_context| {
        UiPointerDispatchEffect::handled()
    });
}

pub(super) fn base_state(interactive: bool) -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: interactive,
        clickable: interactive,
        hoverable: interactive,
        focusable: false,
        pressed: false,
        checked: false,
        dirty: false,
    }
}
