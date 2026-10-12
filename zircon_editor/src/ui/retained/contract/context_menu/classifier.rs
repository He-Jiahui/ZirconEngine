use super::super::surface_hit_test::TemplateNodePointerHit;
use super::provider::WorkbenchContextMenuProvider;

pub(in crate::ui::retained_host::host_contract) fn context_menu_provider_for_hit(
    hit: &TemplateNodePointerHit,
) -> Option<WorkbenchContextMenuProvider> {
    if is_scene_node_hit(hit) {
        return Some(WorkbenchContextMenuProvider::SceneNode);
    }
    if is_module_node_hit(hit) {
        return Some(WorkbenchContextMenuProvider::ModuleNode);
    }
    is_actionable_workbench_hit(hit).then_some(WorkbenchContextMenuProvider::GenericWorkbench)
}

fn is_scene_node_hit(hit: &TemplateNodePointerHit) -> bool {
    let control_id = hit.control_id.as_str();
    let action_id = hit.action_id.as_str();
    control_id.starts_with("WorkbenchSceneVirtualItem")
        || (control_id.starts_with("WorkbenchScene") && control_id.ends_with("Item"))
        || action_id.starts_with("workbench.hierarchy.")
        || action_id.starts_with("scene_tree.")
}

fn is_module_node_hit(hit: &TemplateNodePointerHit) -> bool {
    let control_id = hit.control_id.as_str();
    let action_id = hit.action_id.as_str();
    control_id.starts_with("WorkbenchModule")
        || control_id.starts_with("WorkbenchAbility")
        || control_id.starts_with("WorkbenchEffect")
        || action_id.starts_with("workbench.module.")
}

fn is_actionable_workbench_hit(hit: &TemplateNodePointerHit) -> bool {
    !hit.action_id.is_empty()
        || !hit.binding_id.is_empty()
        || !hit.edit_action_id.is_empty()
        || !hit.commit_action_id.is_empty()
        || hit.control_id.as_str().starts_with("Workbench")
}
