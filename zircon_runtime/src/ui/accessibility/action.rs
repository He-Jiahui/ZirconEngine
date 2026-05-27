use zircon_runtime_interface::ui::{
    accessibility::UiAccessibilityAction,
    dispatch::{UiAccessibilityInputEvent, UiDispatchReply, UiInputDispatchResult, UiInputEvent},
};

use crate::ui::surface::UiSurface;

use self::activate::dispatch_activate;
use self::expanded::dispatch_expanded_state;
use self::focus::dispatch_focus;
use self::popup::dispatch_dismiss;
use self::range::dispatch_adjust_value;
use self::scroll::dispatch_scroll_to;
use self::target::{reject_missing_target, validate_included_target};
use self::text::{dispatch_replace_selected_text, dispatch_set_text_selection};
use self::value::dispatch_set_value;

mod activate;
mod expanded;
mod focus;
mod popup;
mod range;
mod result;
mod scroll;
mod target;
mod text;
mod text_state;
mod value;
mod value_target;

pub(crate) fn dispatch_accessibility_action(
    surface: &mut UiSurface,
    event: UiAccessibilityInputEvent,
) -> UiInputDispatchResult {
    let request = event.request.clone();
    let mut result = UiInputDispatchResult::new(
        UiInputEvent::Accessibility(event),
        UiDispatchReply::unhandled(),
    );
    let target = request.target;
    let snapshot = surface.accessibility_snapshot();
    let Some(snapshot_node) = snapshot.node(target).cloned() else {
        return reject_missing_target(surface, &snapshot, target, result);
    };

    let result =
        match validate_included_target(&snapshot, target, request.action, &snapshot_node, result) {
            Ok(result) => result,
            Err(result) => return result,
        };

    match request.action {
        UiAccessibilityAction::Focus => dispatch_focus(surface, target, &snapshot_node, result),
        UiAccessibilityAction::Activate => {
            dispatch_activate(surface, target, &snapshot_node, result)
        }
        UiAccessibilityAction::SetValue => {
            dispatch_set_value(surface, &request, &snapshot_node, result)
        }
        UiAccessibilityAction::ReplaceSelectedText => {
            dispatch_replace_selected_text(surface, &request, &snapshot_node, result)
        }
        UiAccessibilityAction::SetTextSelection => {
            dispatch_set_text_selection(surface, &request, &snapshot_node, result)
        }
        UiAccessibilityAction::Increment | UiAccessibilityAction::Decrement => {
            dispatch_adjust_value(surface, &request, &snapshot_node, result)
        }
        UiAccessibilityAction::Expand => {
            dispatch_expanded_state(surface, &request, &snapshot_node, result, true)
        }
        UiAccessibilityAction::Collapse => {
            dispatch_expanded_state(surface, &request, &snapshot_node, result, false)
        }
        UiAccessibilityAction::ScrollTo => {
            dispatch_scroll_to(surface, &request, &snapshot_node, result)
        }
        UiAccessibilityAction::Dismiss => dispatch_dismiss(surface, target, &snapshot_node, result),
    }
}
