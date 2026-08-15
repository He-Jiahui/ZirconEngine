use zircon_runtime::ui::dispatch::UiPointerDispatcher;
use zircon_runtime_interface::ui::{
    dispatch::UiPointerDispatchEffect, event_ui::UiNodeId, surface::UiPointerEventKind,
};

pub(in crate::ui::retained_host::welcome_recent_pointer) fn register_handled_pointer_node(
    dispatcher: &mut UiPointerDispatcher,
    node_id: UiNodeId,
) {
    for kind in [
        UiPointerEventKind::Move,
        UiPointerEventKind::Down,
        UiPointerEventKind::Scroll,
    ] {
        dispatcher.register(node_id, kind, |_context| UiPointerDispatchEffect::handled());
    }
}
