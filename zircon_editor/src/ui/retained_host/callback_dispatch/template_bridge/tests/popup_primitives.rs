use super::*;

#[test]
fn popup_menu_item_state_parses_action_label_and_disabled_separator_state() {
    let item = template_popup_menu_item_state("Delete|danger,disabled,icon=trash")
        .expect("menu item should parse");
    assert_eq!(item.action_id, "menu.item.delete");
    assert_eq!(item.label, "Delete");
    assert!(item.disabled);
    assert!(!item.separator);

    let separator = template_popup_menu_item_state("---").expect("separator should parse as state");
    assert!(separator.disabled);
    assert!(separator.separator);
    assert!(template_popup_menu_item_state("").is_none());
}

#[test]
fn popup_menu_item_state_prefers_an_explicit_generation_action_id() {
    let item = template_popup_menu_item_state(
        "Create UI Layout|action=menu.item.asset_create.17.3,icon=plus",
    )
    .expect("compiled menu item should parse");

    assert_eq!(item.action_id, "menu.item.asset_create.17.3");
    assert_eq!(item.label, "Create UI Layout");
}

#[test]
fn popup_menu_transient_flag_cleanup_preserves_persistent_flags_and_shortcuts() {
    assert_eq!(
        menu_item_without_transient_flags("Open|hovered,icon=folder,pressed|Ctrl+O"),
        "Open|icon=folder|Ctrl+O"
    );
    assert_eq!(
        menu_item_without_transient_flags("Inspect|focused|Ctrl+I"),
        "Inspect||Ctrl+I"
    );
    assert_eq!(menu_item_without_transient_flags("---"), "---");
}

#[test]
fn popup_menu_checked_state_moves_between_choice_rows_without_losing_flags() {
    assert_eq!(
        menu_item_with_checked_state("Play In Editor|checked,icon=play", false),
        "Play In Editor|icon=play"
    );
    assert_eq!(
        menu_item_with_checked_state("Simulate|icon=play", true),
        "Simulate|icon=play,checked"
    );
    assert_eq!(
        menu_item_with_checked_state("Standalone|disabled,icon=grid", true),
        "Standalone|disabled,icon=grid,checked"
    );
}
