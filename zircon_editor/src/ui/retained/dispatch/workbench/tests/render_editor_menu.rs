use std::collections::HashSet;

use super::*;

#[test]
fn render_editor_menu_actions_are_complete_and_unique() {
    assert_eq!(3, RENDER_EDITOR_MENU_COMMANDS.len());
    assert_eq!(
        RENDER_EDITOR_MENU_COMMANDS.len(),
        RENDER_EDITOR_MENU_COMMANDS
            .iter()
            .map(|command| command.menu_action_id)
            .collect::<HashSet<_>>()
            .len()
    );
    assert!(RENDER_EDITOR_MENU_COMMANDS
        .iter()
        .all(|command| command.menu_action_id.starts_with("menu.item.render.")));
    assert!(RENDER_EDITOR_MENU_COMMANDS.iter().all(|command| {
        command
            .extension_action_id
            .starts_with("workbench.extension.")
    }));
    assert!(RENDER_EDITOR_MENU_COMMANDS
        .iter()
        .all(|command| command.extension_action_id.ends_with(".open")));
}
