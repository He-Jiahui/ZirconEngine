use zircon_ui::{
    dispatch::{UiPointerDispatchEffect, UiPointerDispatcher},
    event_ui::UiNodeId,
    UiPointerEventKind,
};

pub(in crate::ui::slint_host::document_tab_pointer) fn register_handled_pointer_node(
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
