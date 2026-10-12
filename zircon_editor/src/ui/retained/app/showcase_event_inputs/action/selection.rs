use crate::ui::template_runtime::UiComponentShowcaseDemoEventInput;
use zircon_runtime_interface::ui::component::UiValue;

use super::super::{action_matches, select_option};

pub(super) fn demo_selection_input(action_id: &str) -> Option<UiComponentShowcaseDemoEventInput> {
    match action_id {
        action if action_matches(action, "segmented_control_changed") => {
            Some(select_option("rotate", true))
        }
        action if action_matches(action, "tab_changed") => Some(select_option("scene", true)),
        action if action_matches(action, "tab_strip_changed") => {
            Some(select_option("assets", true))
        }
        action if action_matches(action, "dropdown_changed") => Some(select_option("editor", true)),
        action if action_matches(action, "combo_box_changed") => {
            Some(select_option("native", true))
        }
        action if action_matches(action, "enum_field_changed") => {
            Some(select_option("UnityInspector", true))
        }
        action if action_matches(action, "flags_field_changed") => {
            Some(select_option("Disabled", true))
        }
        action if action_matches(action, "search_select_changed") => {
            Some(select_option("runtime.ui.RangeField", true))
        }
        action if action_matches(action, "search_select_query_changed") => Some(
            UiComponentShowcaseDemoEventInput::Value(UiValue::String("vector".to_string())),
        ),
        action if action_matches(action, "context_action_menu_open_at") => {
            Some(UiComponentShowcaseDemoEventInput::OpenPopupAt { x: 184.0, y: 88.0 })
        }
        action if action_matches(action, "context_action_menu_changed") => {
            Some(select_option("Open Source", true))
        }
        _ => None,
    }
}
