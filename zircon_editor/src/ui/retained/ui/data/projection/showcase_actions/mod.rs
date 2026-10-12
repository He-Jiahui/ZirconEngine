#[path = "../../showcase_actions/action_buttons.rs"]
mod action_buttons;
#[path = "binding_ids.rs"]
mod binding_ids;
#[path = "../../showcase_actions/commit_action.rs"]
mod commit_action;
#[path = "../../showcase_actions/drag_actions.rs"]
mod drag_actions;
#[path = "edit_action.rs"]
mod edit_action;
#[path = "../../showcase_actions/primary_action.rs"]
mod primary_action;

pub(super) use self::action_buttons::preferred_showcase_action_buttons;
pub(super) use self::binding_ids::showcase_action_id_for_suffix;
pub(super) use self::commit_action::preferred_showcase_commit_action_id;
pub(super) use self::drag_actions::{
    preferred_showcase_drag_action_id, preferred_showcase_pointer_drag_action_id,
};
pub(super) use self::edit_action::preferred_showcase_edit_action_id;
pub(super) use self::primary_action::preferred_showcase_action_id;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
