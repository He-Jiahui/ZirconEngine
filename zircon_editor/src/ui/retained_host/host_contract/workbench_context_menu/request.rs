use super::super::data::WorkbenchContextMenuRequestData;
use super::super::surface_hit_test::TemplateNodePointerHit;
use super::classifier::context_menu_provider_for_hit;
use super::path::target_value_text;

pub(in crate::ui::retained_host::host_contract) fn workbench_context_menu_request_for_hit(
    hit: &TemplateNodePointerHit,
    x: f32,
    y: f32,
) -> Option<WorkbenchContextMenuRequestData> {
    if hit.control_id.is_empty() {
        return None;
    }
    if matches!(
        hit.dispatch_kind.as_str(),
        "workbench_menu_item" | "workbench_option"
    ) {
        return None;
    }

    let provider = context_menu_provider_for_hit(hit)?;
    let target_value = target_value_text(hit);
    let target_path = provider.target_path(hit, target_value.as_str());
    Some(WorkbenchContextMenuRequestData {
        target_control_id: hit.control_id.clone(),
        target_action_id: hit.action_id.clone(),
        target_dispatch_kind: hit.dispatch_kind.clone(),
        target_role: hit.component_role.clone(),
        target_value_text: target_value,
        target_path,
        popup_anchor_x: x,
        popup_anchor_y: y,
        menu_items: provider.menu_items(),
    })
}
