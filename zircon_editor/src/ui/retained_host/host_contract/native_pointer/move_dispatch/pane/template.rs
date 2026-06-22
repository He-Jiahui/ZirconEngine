use crate::ui::retained_host::host_contract::surface_hit_test::TemplateNodePointerHit;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

pub(super) fn dispatch_template_pane_move(ui: &UiHostWindow, hit: &TemplateNodePointerHit) {
    ui.set_hovered_template_node_for_pointer_move(hit.control_id.clone(), hit.frame.clone());
}
