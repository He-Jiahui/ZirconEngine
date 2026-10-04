use super::RuntimePopupMenuItem;

#[test]
fn raw_item_keeps_explicit_action_id_separate_from_display_label() {
    let item =
        RuntimePopupMenuItem::from_raw("Open Workspace|action=menu.item.open_project,icon=folder");

    assert_eq!(item.id, "menu.item.open_project");
    assert_eq!(item.label, "Open Workspace");
}

#[test]
fn raw_item_uses_label_as_legacy_id_fallback() {
    let item = RuntimePopupMenuItem::from_raw("Open Project|icon=folder");

    assert_eq!(item.id, "Open Project");
    assert_eq!(item.label, "Open Project");
}
