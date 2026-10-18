use crate::core::editor_event::{
    EditorEvent, EditorEventEffect, EditorEventTransient, EditorOperationEvent,
};
use crate::ui::host::EditorHostEventController;

use super::error::EditorEventExecutionError;
use super::execution_outcome::ExecutionOutcome;
use super::{
    animation_event::execute_animation_event, asset_event::execute_asset_event,
    draft_event::execute_draft_event, hierarchy_event::execute_hierarchy_event,
    inspector_event::execute_inspector_event, layout_command::execute_layout_command,
    menu_action::execute_menu_action, selection_event::execute_selection,
    viewport_event::execute_viewport_event,
};
pub(crate) fn execute_event(
    controller: &EditorHostEventController,
    event: &EditorEvent,
) -> Result<ExecutionOutcome, EditorEventExecutionError> {
    match event {
        EditorEvent::WorkbenchMenu(action) => {
            execute_menu_action(controller, action).map_err(EditorEventExecutionError::from)
        }
        EditorEvent::Layout(command) => {
            let manager = controller
                .enter_editor_manager()
                .map_err(|source| EditorEventExecutionError::Layout { source })?;
            let mut shell = controller.shell().lock();
            execute_layout_command(&mut shell, &manager, command)
                .map_err(|source| EditorEventExecutionError::Layout { source })
        }
        EditorEvent::Selection(event) => execute_selection(&mut controller.shell().lock(), event)
            .map_err(|source| EditorEventExecutionError::Selection { source }),
        EditorEvent::Hierarchy(event) => {
            execute_hierarchy_event(&mut controller.shell().lock(), event)
                .map_err(|source| EditorEventExecutionError::Hierarchy { source })
        }
        EditorEvent::Asset(event) => {
            execute_asset_event(controller, event).map_err(EditorEventExecutionError::from)
        }
        EditorEvent::Draft(event) => execute_draft_event(&mut controller.shell().lock(), event)
            .map_err(|source| EditorEventExecutionError::Draft { source }),
        EditorEvent::Animation(event) => {
            let manager = controller
                .enter_editor_manager()
                .map_err(|source| EditorEventExecutionError::Animation { source })?;
            let mut shell = controller.shell().lock();
            execute_animation_event(&mut shell, &manager, event)
                .map_err(|source| EditorEventExecutionError::Animation { source })
        }
        EditorEvent::Inspector(event) => {
            execute_inspector_event(&mut controller.shell().lock(), event)
                .map_err(|source| EditorEventExecutionError::Inspector { source })
        }
        EditorEvent::Viewport(event) => execute_viewport_event(controller, event)
            .map_err(|source| EditorEventExecutionError::Viewport { source }),
        EditorEvent::Operation(event) => match event {
            EditorOperationEvent::ControlFailure {
                operation_id,
                error,
            } => Err(EditorEventExecutionError::RecordedOperationControlFailure {
                operation_id: operation_id.clone(),
                message: error.clone(),
            }),
            EditorOperationEvent::CommandExecuted { .. } => Ok(ExecutionOutcome {
                changed: true,
                effects: vec![EditorEventEffect::PresentationChanged],
            }),
            EditorOperationEvent::NativeCommandExecuted { .. } => Ok(ExecutionOutcome {
                changed: false,
                effects: vec![EditorEventEffect::PresentationChanged],
            }),
            EditorOperationEvent::EditQueued { .. } => Ok(ExecutionOutcome {
                changed: false,
                effects: vec![
                    EditorEventEffect::PresentationChanged,
                    EditorEventEffect::ReflectionChanged,
                ],
            }),
        },
        EditorEvent::Transient(update) => {
            let mut shell = controller.shell().lock();
            shell.transient.apply(update);
            let effects = match update {
                EditorEventTransient::OpenCommandPalette => {
                    vec![EditorEventEffect::CommandPaletteOpenRequested]
                }
                EditorEventTransient::OpenSettingsWindow => {
                    vec![EditorEventEffect::SettingsWindowOpenRequested]
                }
                EditorEventTransient::ToggleContentDrawer => {
                    vec![EditorEventEffect::ContentDrawerToggleRequested]
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
