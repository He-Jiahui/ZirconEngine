use crate::core::editor_event::{
    EditorEvent, EditorEventEffect, EditorEventTransient, EditorOperationEvent,
};
use crate::ui::host::EditorHostEventController;

use super::execution_outcome::ExecutionOutcome;
use super::{
    animation_event::execute_animation_event, asset_event::execute_asset_event,
    draft_event::execute_draft_event, inspector_event::execute_inspector_event,
    layout_command::execute_layout_command, menu_action::execute_menu_action,
    selection_event::execute_selection, viewport_event::execute_viewport_event,
};
pub(crate) fn execute_event(
    controller: &EditorHostEventController,
    event: &EditorEvent,
) -> Result<ExecutionOutcome, String> {
    let mut shell = controller.shell().lock();
    match event {
        EditorEvent::WorkbenchMenu(action) => execute_menu_action(controller, &mut shell, action),
        EditorEvent::Layout(command) => execute_layout_command(&mut shell, command),
        EditorEvent::Selection(event) => execute_selection(&mut shell, event),
        EditorEvent::Asset(event) => execute_asset_event(controller, &mut shell, event),
        EditorEvent::Draft(event) => execute_draft_event(&mut shell, event),
        EditorEvent::Animation(event) => execute_animation_event(&mut shell, event),
        EditorEvent::Inspector(event) => execute_inspector_event(&mut shell, event),
        EditorEvent::Viewport(event) => execute_viewport_event(controller, &mut shell, event),
        EditorEvent::Operation(event) => match event {
            EditorOperationEvent::ControlFailure { error, .. } => Err(error.clone()),
            EditorOperationEvent::CommandExecuted { .. } => Ok(ExecutionOutcome {
                changed: true,
                effects: vec![EditorEventEffect::PresentationChanged],
            }),
        },
        EditorEvent::Transient(update) => {
            shell.transient.apply(update);
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
