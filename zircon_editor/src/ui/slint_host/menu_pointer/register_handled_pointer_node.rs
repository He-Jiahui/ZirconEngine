use zircon_runtime::ui::{
    dispatch::{UiPointerDispatchEffect, UiPointerDispatcher},
    event_ui::UiNodeId,
    surface::UiPointerEventKind,
};

pub(in crate::ui::slint_host::menu_pointer) fn register_handled_pointer_node(
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
