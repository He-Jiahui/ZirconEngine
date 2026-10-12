use super::structured_menu_item;

#[test]
fn explicit_action_id_is_independent_from_display_label() {
    let item =
        structured_menu_item("Open Workspace|action=menu.item.open_project,icon=folder|Ctrl+O");

    assert_eq!(item.action_id, "menu.item.open_project");
    assert_eq!(item.label, "Open Workspace");
    assert_eq!(item.shortcut, "Ctrl+O");
}

#[test]
fn label_derived_action_id_remains_as_legacy_fallback() {
    let item = structured_menu_item("Open Project|icon=folder");

    assert_eq!(item.action_id, "menu.item.open_project");
}
