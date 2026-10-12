use crate::ui::retained_host::host_contract::surface_hit_test::TemplateNodePointerMoveHit;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

pub(super) fn set_hovered_workbench_template_hit(
    ui: &UiHostWindow,
    hit: &TemplateNodePointerMoveHit<'_>,
) {
    if hit.kind.is_popup() {
        ui.set_hovered_template_row_for_pointer_move(
            hit.control_id,
            hit.kind.dispatch_kind(),
            hit.action_id,
            hit.value_text,
            &hit.frame,
        );
    } else {
        ui.set_hovered_template_node_for_pointer_move(hit.control_id, &hit.frame);
    }
}
