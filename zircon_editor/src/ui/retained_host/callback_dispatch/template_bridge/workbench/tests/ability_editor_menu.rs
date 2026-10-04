use std::collections::HashSet;

use super::*;

#[test]
fn ability_editor_menu_actions_are_complete_and_unique() {
    assert_eq!(8, ABILITY_EDITOR_MENU_COMMANDS.len());
    assert_eq!(
        ABILITY_EDITOR_MENU_COMMANDS.len(),
        ABILITY_EDITOR_MENU_COMMANDS
            .iter()
            .map(|command| command.menu_action_id)
            .collect::<HashSet<_>>()
            .len()
    );
    assert!(ABILITY_EDITOR_MENU_COMMANDS
        .iter()
        .all(|command| command.menu_action_id.starts_with("menu.item.ability.")));
    assert!(ABILITY_EDITOR_MENU_COMMANDS.iter().all(|command| {
        command
            .extension_action_id
            .starts_with("workbench.extension.")
    }));
    assert!(ABILITY_EDITOR_MENU_COMMANDS
        .iter()
        .all(|command| command.extension_action_id.ends_with(".open")));
}
