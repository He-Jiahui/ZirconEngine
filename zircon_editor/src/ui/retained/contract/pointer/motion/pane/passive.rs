use crate::ui::retained_host::host_contract::window::UiHostWindow;

pub(super) fn clear_passive_pane_move_hover(ui: &UiHostWindow) {
    ui.clear_hovered_template_node_for_pointer_move();
}
