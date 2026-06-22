use crate::ui::retained_host::host_contract::data::HostTextInputFocusData;
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::surface_hit_test::TemplateNodePointerHit;
use crate::ui::retained_host::host_contract::template_input_semantics::text_input_edit_target_id;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

pub(in crate::ui::retained_host::host_contract) fn focus_template_node_text_input(
    ui: &UiHostWindow,
    hit: &TemplateNodePointerHit,
) -> bool {
    let target_id = text_input_edit_target_id(hit);
    if target_id.is_empty() {
        return false;
    }
    ui.global::<UiHostContext>()
        .set_text_input_focus(HostTextInputFocusData {
            control_id: hit.control_id.clone(),
            dispatch_kind: hit.dispatch_kind.clone(),
            action_id: hit.action_id.clone(),
            edit_action_id: target_id,
            commit_action_id: hit.commit_action_id.clone(),
            value_text: hit.value_text.clone(),
            edit_frame: hit.frame.clone(),
        });
    true
}
