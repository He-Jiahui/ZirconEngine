use crate::ui::retained_host::host_contract::surface_hit_test::TemplateNodePointerHit;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

pub(super) fn set_hovered_workbench_template_hit(ui: &UiHostWindow, hit: &TemplateNodePointerHit) {
    if matches!(
        hit.dispatch_kind.as_str(),
        "workbench_option" | "workbench_menu_item"
    ) {
        ui.set_hovered_template_row_for_pointer_move(
            hit.control_id.clone(),
            hit.dispatch_kind.clone(),
            hit.action_id.clone(),
            hit.value_text.clone(),
            hit.frame.clone(),
        );
    } else {
        ui.set_hovered_template_node_for_pointer_move(hit.control_id.clone(), hit.frame.clone());
    }
}
