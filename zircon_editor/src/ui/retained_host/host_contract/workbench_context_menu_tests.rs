use super::request::workbench_context_menu_request_for_hit;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::surface_hit_test::TemplateNodePointerHit;
use crate::ui::retained_host::primitives::SharedString;

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
