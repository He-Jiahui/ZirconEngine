use std::collections::HashSet;

use super::*;

#[test]
fn hud_editor_menu_actions_are_complete_and_unique() {
    assert_eq!(10, HUD_EDITOR_MENU_COMMANDS.len());
    assert_eq!(
        HUD_EDITOR_MENU_COMMANDS.len(),
        HUD_EDITOR_MENU_COMMANDS
            .iter()
            .map(|command| command.menu_action_id)
            .collect::<HashSet<_>>()
            .len()
    );
    assert!(HUD_EDITOR_MENU_COMMANDS
        .iter()
        .all(|command| command.menu_action_id.starts_with("menu.item.hud.")));
    assert!(HUD_EDITOR_MENU_COMMANDS.iter().all(|command| {
        command
            .extension_action_id
            .starts_with("workbench.extension.")
    }));
    assert!(HUD_EDITOR_MENU_COMMANDS
        .iter()
        .all(|command| command.extension_action_id.ends_with(".open")));
}
