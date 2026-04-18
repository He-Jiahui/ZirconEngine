use crate::core::editor_event::{EditorEvent, EditorEventEffect};

use super::super::execution_outcome::ExecutionOutcome;
use super::super::runtime_inner::EditorEventRuntimeInner;
use super::{
    asset_event::execute_asset_event, draft_event::execute_draft_event,
    inspector_event::execute_inspector_event, layout_command::execute_layout_command,
    menu_action::execute_menu_action, selection_event::execute_selection,
    viewport_event::execute_viewport_event,
};

pub(in crate::core::editor_event::runtime) fn execute_event(
    inner: &mut EditorEventRuntimeInner,
    event: &EditorEvent,
) -> Result<ExecutionOutcome, String> {
    match event {
        EditorEvent::WorkbenchMenu(action) => execute_menu_action(inner, action),
        EditorEvent::Layout(command) => execute_layout_command(inner, command),
        EditorEvent::Selection(event) => execute_selection(inner, event),
        EditorEvent::Asset(event) => execute_asset_event(inner, event),
        EditorEvent::Draft(event) => execute_draft_event(inner, event),
        EditorEvent::Inspector(event) => execute_inspector_event(inner, event),
        EditorEvent::Viewport(event) => execute_viewport_event(inner, event),
        EditorEvent::Transient(update) => {
            inner.transient.apply(update);
            Ok(ExecutionOutcome {
                changed: true,
                effects: vec![
                    EditorEventEffect::PresentationChanged,
                    EditorEventEffect::ReflectionChanged,
                ],
            })
        }
    }
}
