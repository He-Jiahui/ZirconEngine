use zircon_ui::{UiNodeId, UiPointerDispatchEffect, UiPointerDispatcher, UiPointerEventKind};

pub(in crate::ui::slint_host::welcome_recent_pointer) fn register_handled_pointer_node(
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
