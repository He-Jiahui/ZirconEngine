use std::collections::HashSet;

use super::*;

#[test]
fn asset_editor_menu_actions_are_complete_and_unique() {
    assert_eq!(22, ASSET_EDITOR_MENU_COMMANDS.len());
    assert_eq!(
        ASSET_EDITOR_MENU_COMMANDS.len(),
        ASSET_EDITOR_MENU_COMMANDS
            .iter()
            .map(|command| (command.menu_control_id, command.menu_action_id))
            .collect::<HashSet<_>>()
            .len()
    );
    assert!(ASSET_EDITOR_MENU_COMMANDS
        .iter()
        .all(|command| command.menu_action_id.starts_with("menu.item.assets.")));
    assert!(ASSET_EDITOR_MENU_COMMANDS.iter().all(|command| {
        command
            .extension_action_id
            .starts_with("workbench.extension.")
    }));
    assert!(ASSET_EDITOR_MENU_COMMANDS
        .iter()
        .all(|command| command.extension_action_id.ends_with(".open")));
}
