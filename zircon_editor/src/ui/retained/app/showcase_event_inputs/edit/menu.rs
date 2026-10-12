use crate::ui::template_runtime::UiComponentShowcaseDemoEventInput;

use super::super::action_matches;

pub(super) fn demo_menu_edit_input(
    action_id: &str,
    value: &str,
) -> Option<UiComponentShowcaseDemoEventInput> {
    if action_matches(action_id, "context_action_menu_open_at") {
        if let Some((x, y)) = parse_popup_anchor(value) {
            return Some(UiComponentShowcaseDemoEventInput::OpenPopupAt { x, y });
        }
    }
    None
}

fn parse_popup_anchor(value: &str) -> Option<(f64, f64)> {
    let (x, y) = value.split_once(',')?;
    Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
}
