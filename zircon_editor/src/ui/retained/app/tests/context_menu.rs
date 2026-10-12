use super::*;

#[test]
fn keep_changes_item_is_projected_only_for_playing_scene_rows() {
    let mut playing_scene = request("workbench://scene/cube");
    project_keep_play_changes_context_item(&mut playing_scene, true);
    assert!(playing_scene
        .menu_items
        .iter()
        .any(|item| item.as_str().contains("menu.item.keep_play_changes")));

    let mut edit_scene = request("workbench://scene/cube");
    project_keep_play_changes_context_item(&mut edit_scene, false);
    assert!(!edit_scene
        .menu_items
        .iter()
        .any(|item| item.as_str().contains("menu.item.keep_play_changes")));

    let mut playing_module = request("workbench://module/navigation");
    project_keep_play_changes_context_item(&mut playing_module, true);
    assert!(!playing_module
        .menu_items
        .iter()
        .any(|item| item.as_str().contains("menu.item.keep_play_changes")));
}

fn request(target_path: &str) -> WorkbenchContextMenuRequestData {
    WorkbenchContextMenuRequestData {
        target_path: target_path.into(),
        menu_items: vec!["Open|icon=folder".into(), "---".into()],
        ..Default::default()
    }
}
