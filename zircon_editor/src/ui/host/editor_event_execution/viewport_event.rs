use crate::core::editor_event::{EditorEventEffect, EditorViewportEvent};
use crate::scene::viewport::ViewportFeedback;
use crate::ui::binding::ViewportCommand;
use crate::ui::host::EditorHostEventController;
use crate::ui::workbench::shell_state::WorkbenchShellStateData;
use crate::ui::workbench::state::EditorViewportStateError;

use super::execution_outcome::ExecutionOutcome;

pub(super) fn execute_viewport_event(
    _controller: &EditorHostEventController,
    shell: &mut WorkbenchShellStateData,
    event: &EditorViewportEvent,
) -> Result<ExecutionOutcome, EditorViewportStateError> {
    let command = match event {
        EditorViewportEvent::PointerMoved { x, y } => {
            ViewportCommand::PointerMoved { x: *x, y: *y }
        }
        EditorViewportEvent::LeftPressed {
            x,
            y,
            selection_mutation,
        } => ViewportCommand::LeftPressed {
            x: *x,
            y: *y,
            selection_mutation: *selection_mutation,
        },
        EditorViewportEvent::LeftReleased => ViewportCommand::LeftReleased,
        EditorViewportEvent::CancelInteraction => ViewportCommand::CancelInteraction,
        EditorViewportEvent::RightPressed { x, y } => {
            ViewportCommand::RightPressed { x: *x, y: *y }
        }
        EditorViewportEvent::RightReleased => ViewportCommand::RightReleased,
        EditorViewportEvent::MiddlePressed { x, y } => {
            ViewportCommand::MiddlePressed { x: *x, y: *y }
        }
        EditorViewportEvent::MiddleReleased => ViewportCommand::MiddleReleased,
        EditorViewportEvent::Scrolled { delta } => ViewportCommand::Scrolled { delta: *delta },
        EditorViewportEvent::Resized { width, height } => ViewportCommand::Resized {
            width: *width,
            height: *height,
        },
        EditorViewportEvent::ActivateSceneMode { mode } => {
            ViewportCommand::ActivateSceneMode(mode.clone())
        }
        EditorViewportEvent::SetTransformSpace { space } => {
            ViewportCommand::SetTransformSpace(*space)
        }
        EditorViewportEvent::SetPivotMode { mode } => ViewportCommand::SetPivotMode(*mode),
        EditorViewportEvent::SetProjectionMode { mode } => {
            ViewportCommand::SetProjectionMode(*mode)
        }
        EditorViewportEvent::AlignView { orientation } => ViewportCommand::AlignView(*orientation),
        EditorViewportEvent::SetDisplayMode { mode } => ViewportCommand::SetDisplayMode(*mode),
        EditorViewportEvent::SetGridMode { mode } => ViewportCommand::SetGridMode(*mode),
        EditorViewportEvent::SetTranslateSnap { step } => ViewportCommand::SetTranslateSnap(*step),
        EditorViewportEvent::SetRotateSnapDegrees { step } => {
            ViewportCommand::SetRotateSnapDegrees(*step)
        }
        EditorViewportEvent::SetScaleSnap { step } => ViewportCommand::SetScaleSnap(*step),
        EditorViewportEvent::SetPreviewLighting { enabled } => {
            ViewportCommand::SetPreviewLighting(*enabled)
        }
        EditorViewportEvent::SetPreviewSkybox { enabled } => {
            ViewportCommand::SetPreviewSkybox(*enabled)
        }
        EditorViewportEvent::SetGizmosEnabled { enabled } => {
            ViewportCommand::SetGizmosEnabled(*enabled)
        }
        EditorViewportEvent::ToggleOverlayProvider { provider_id } => {
            ViewportCommand::ToggleOverlayProvider {
                provider_id: provider_id.clone(),
            }
        }
        EditorViewportEvent::FrameSelection => ViewportCommand::FrameSelection,
    };
    let feedback = shell.state.apply_viewport_command(&command)?;
    let structural_viewport_change = structural_viewport_event(event, &feedback);
    let chrome_projection_change = event.changes_chrome_projection();
    let changed = structural_viewport_change
        || feedback.camera_updated
        || feedback.transformed_node.is_some()
        || feedback.hovered_axis.is_some()
        || feedback.interaction_extract_stale;
    Ok(ExecutionOutcome {
        changed,
        effects: viewport_effects(
            event,
            &feedback,
            structural_viewport_change,
            chrome_projection_change,
        ),
    })
}

fn structural_viewport_event(event: &EditorViewportEvent, feedback: &ViewportFeedback) -> bool {
    feedback.settings_changed
        || matches!(
            event,
            EditorViewportEvent::LeftReleased
                | EditorViewportEvent::Resized { .. }
                | EditorViewportEvent::ActivateSceneMode { .. }
                | EditorViewportEvent::AlignView { .. }
                | EditorViewportEvent::ToggleOverlayProvider { .. }
                | EditorViewportEvent::FrameSelection
        )
}

fn viewport_effects(
    _event: &EditorViewportEvent,
    feedback: &ViewportFeedback,
    structural_viewport_change: bool,
    chrome_projection_change: bool,
) -> Vec<EditorEventEffect> {
    let mut effects = Vec::new();

    if structural_viewport_change
        || feedback.camera_updated
        || feedback.transformed_node.is_some()
        || feedback.hovered_axis.is_some()
        || feedback.interaction_extract_stale
    {
        effects.push(EditorEventEffect::RenderChanged);
    }

    if (structural_viewport_change && !chrome_projection_change)
        || feedback.transformed_node.is_some()
        || feedback.hovered_axis.is_some()
    {
        effects.push(EditorEventEffect::PresentationChanged);
    }

    if structural_viewport_change || feedback.transformed_node.is_some() {
        effects.push(EditorEventEffect::ReflectionChanged);
    }

    effects
}

#[cfg(test)]
mod tests {
    use super::viewport_effects;
    use crate::core::editor_event::{EditorEventEffect, EditorViewportEvent};
    use crate::scene::viewport::{GridMode, ViewportFeedback};

    #[test]
    fn viewport_chrome_state_change_keeps_render_but_skips_full_presentation() {
        let event = EditorViewportEvent::SetGridMode {
            mode: GridMode::VisibleAndSnap,
        };

        let effects = viewport_effects(&event, &ViewportFeedback::default(), true, true);

        assert!(effects.contains(&EditorEventEffect::RenderChanged));
        assert!(effects.contains(&EditorEventEffect::ReflectionChanged));
        assert!(!effects.contains(&EditorEventEffect::PresentationChanged));
    }

    #[test]
    fn non_chrome_structural_change_still_requests_presentation() {
        let event = EditorViewportEvent::Resized {
            width: 1280,
            height: 720,
        };

        let effects = viewport_effects(&event, &ViewportFeedback::default(), true, false);

        assert!(effects.contains(&EditorEventEffect::PresentationChanged));
    }

    #[test]
    fn empty_cancel_interaction_has_no_effects() {
        let event = EditorViewportEvent::CancelInteraction;

        assert!(!super::structural_viewport_event(
            &event,
            &ViewportFeedback::default()
        ));
        assert!(viewport_effects(&event, &ViewportFeedback::default(), false, false).is_empty());
    }

    #[test]
    fn active_cancel_interaction_refreshes_viewport_projections() {
        let event = EditorViewportEvent::CancelInteraction;
        let feedback = ViewportFeedback {
            transformed_node: Some(42),
            ..ViewportFeedback::default()
        };

        let effects = viewport_effects(&event, &feedback, false, false);

        assert!(effects.contains(&EditorEventEffect::RenderChanged));
        assert!(effects.contains(&EditorEventEffect::PresentationChanged));
        assert!(effects.contains(&EditorEventEffect::ReflectionChanged));
    }

    #[test]
    fn stale_pointer_product_requests_a_render_rebuild_without_presentation_churn() {
        let feedback = ViewportFeedback {
            interaction_extract_stale: true,
            ..ViewportFeedback::default()
        };

        let effects = viewport_effects(
            &EditorViewportEvent::PointerMoved { x: 12.0, y: 24.0 },
            &feedback,
            false,
            false,
        );

        assert!(effects.contains(&EditorEventEffect::RenderChanged));
        assert!(!effects.contains(&EditorEventEffect::PresentationChanged));
        assert!(!effects.contains(&EditorEventEffect::ReflectionChanged));
    }
}
