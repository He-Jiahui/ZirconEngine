use crate::core::editor_event::{
    EditorEvent, EditorEventEffect, EditorEventTransient, EditorOperationEvent,
};

use super::execution_outcome::ExecutionOutcome;
use super::{
    animation_event::execute_animation_event, asset_event::execute_asset_event,
    draft_event::execute_draft_event, inspector_event::execute_inspector_event,
    layout_command::execute_layout_command, menu_action::execute_menu_action,
    selection_event::execute_selection, viewport_event::execute_viewport_event,
};
use crate::core::editor_event::runtime::editor_event_runtime_state::EditorEventRuntimeState;

pub(crate) fn execute_event(
    inner: &mut EditorEventRuntimeState,
    event: &EditorEvent,
) -> Result<ExecutionOutcome, String> {
    match event {
        EditorEvent::WorkbenchMenu(action) => execute_menu_action(inner, action),
        EditorEvent::Layout(command) => execute_layout_command(inner, command),
        EditorEvent::Selection(event) => execute_selection(inner, event),
        EditorEvent::Asset(event) => execute_asset_event(inner, event),
        EditorEvent::Draft(event) => execute_draft_event(inner, event),
        EditorEvent::Animation(event) => execute_animation_event(inner, event),
        EditorEvent::Inspector(event) => execute_inspector_event(inner, event),
        EditorEvent::Viewport(event) => execute_viewport_event(inner, event),
        EditorEvent::Operation(event) => match event {
            EditorOperationEvent::ControlFailure { error, .. } => Err(error.clone()),
        },
        EditorEvent::Transient(update) => {
            inner.transient.apply(update);
            let effects = match update {
                EditorEventTransient::OpenCommandPalette => {
                    vec![EditorEventEffect::CommandPaletteOpenRequested]
                }
                _ => vec![
                    EditorEventEffect::PresentationChanged,
                    EditorEventEffect::ReflectionChanged,
                ],
            };
            Ok(ExecutionOutcome {
                changed: true,
                effects,
            })
        }
    }
}
