use super::data::WorkbenchContextMenuRequestData;
use super::surface_hit_test::TemplateNodePointerHit;
use crate::ui::retained_host::primitives::SharedString;

pub(super) fn workbench_context_menu_request_for_hit(
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
    Some(WorkbenchContextMenuRequestData {
        target_control_id: hit.control_id.clone(),
        target_action_id: hit.action_id.clone(),
        target_dispatch_kind: hit.dispatch_kind.clone(),
        target_role: hit.component_role.clone(),
        target_value_text: target_value_text(hit),
        target_path: provider.target_path(hit),
        popup_anchor_x: x,
        popup_anchor_y: y,
        menu_items: provider.menu_items(),
    })
}

enum WorkbenchContextMenuProvider {
    SceneNode,
    ModuleNode,
    GenericWorkbench,
}

impl WorkbenchContextMenuProvider {
    fn target_path(&self, hit: &TemplateNodePointerHit) -> SharedString {
        let target = target_value_text(hit);
        match self {
            Self::SceneNode => format!("workbench://scene/{}", path_segment(target.as_str())),
            Self::ModuleNode => format!("workbench://module/{}", path_segment(target.as_str())),
            Self::GenericWorkbench => {
                format!(
                    "workbench://control/{}",
                    path_segment(hit.control_id.as_str())
                )
            }
        }
        .into()
    }

    fn menu_items(&self) -> Vec<SharedString> {
        match self {
            Self::SceneNode => vec![
                "Open|icon=folder",
                "Rename|icon=edit",
                "Duplicate|icon=copy",
                "---",
                "Delete|danger,icon=trash",
            ],
            Self::ModuleNode => vec![
                "Open Module|icon=folder",
                "Pin Module|icon=pin",
                "Reset Module|icon=rotate-ccw",
            ],
            Self::GenericWorkbench => vec![
                "Inspect|icon=search",
                "Copy Id|icon=copy",
                "Reveal In Workbench|icon=target",
            ],
        }
        .into_iter()
        .map(Into::into)
        .collect()
    }
}

fn context_menu_provider_for_hit(
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

fn target_value_text(hit: &TemplateNodePointerHit) -> SharedString {
    if !hit.value_text.is_empty() {
        return hit.value_text.clone();
    }
    hit.control_id.clone()
}

fn path_segment(value: &str) -> String {
    value
        .chars()
        .filter_map(|ch| {
            if ch.is_ascii_alphanumeric() {
                Some(ch.to_ascii_lowercase())
            } else if matches!(ch, '-' | '_' | '.') {
                Some(ch)
            } else if ch.is_whitespace() {
                Some('-')
            } else {
                None
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::super::data::FrameRect;
    use super::*;

    #[test]
    fn scene_tree_hit_projects_scene_node_context_menu() {
        let mut hit = hit("WorkbenchScenePropsItem");
        hit.value_text = "Props".into();
        hit.action_id = "workbench.hierarchy.select_props".into();

        let request = workbench_context_menu_request_for_hit(&hit, 144.0, 256.0)
            .expect("scene tree row should provide a context menu");

        assert_eq!(
            request.target_control_id.as_str(),
            "WorkbenchScenePropsItem"
        );
        assert_eq!(request.target_value_text.as_str(), "Props");
        assert_eq!(request.target_path.as_str(), "workbench://scene/props");
        assert_eq!(request.popup_anchor_x, 144.0);
        assert_eq!(request.popup_anchor_y, 256.0);
        assert!(request
            .menu_items
            .iter()
            .any(|item| item.as_str() == "Rename|icon=edit"));
        assert!(request
            .menu_items
            .iter()
            .any(|item| item.as_str() == "Delete|danger,icon=trash"));
    }

    #[test]
    fn popup_rows_do_not_spawn_nested_context_menus() {
        let mut hit = hit("WorkbenchPopupMenu");
        hit.dispatch_kind = "workbench_menu_item".into();
        hit.action_id = "menu.item.delete".into();

        assert!(workbench_context_menu_request_for_hit(&hit, 24.0, 48.0).is_none());
    }

    fn hit(control_id: &str) -> TemplateNodePointerHit {
        TemplateNodePointerHit {
            control_id: control_id.into(),
            action_id: SharedString::new(),
            binding_id: SharedString::new(),
            dispatch_kind: SharedString::new(),
            component_role: "tree-row".into(),
            component_family: None,
            value_text: SharedString::new(),
            edit_action_id: SharedString::new(),
            commit_action_id: SharedString::new(),
            frame: FrameRect::default(),
        }
    }
}
