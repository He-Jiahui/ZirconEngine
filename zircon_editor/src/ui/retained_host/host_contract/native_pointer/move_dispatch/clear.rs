use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::template_hover_damage::template_hover_damage;

pub(super) fn clear_hovered_template_move(ui: &UiHostWindow) -> NativePointerDispatchResult {
    let before = ui.get_pane_interaction_state();
    ui.clear_hovered_template_node_for_pointer_move();
    if let Some(damage) = template_hover_damage(&before, &ui.get_pane_interaction_state()) {
        return NativePointerDispatchResult::region(damage);
    }
    NativePointerDispatchResult::idle()
}
