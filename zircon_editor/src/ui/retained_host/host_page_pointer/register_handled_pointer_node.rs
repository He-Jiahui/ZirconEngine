use zircon_runtime::ui::dispatch::UiPointerDispatcher;
use zircon_runtime_interface::ui::{
    dispatch::UiPointerDispatchEffect, event_ui::UiNodeId, surface::UiPointerEventKind,
};

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
