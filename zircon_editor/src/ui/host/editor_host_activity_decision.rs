use thiserror::Error;

use crate::core::notifications::{DecisionNotificationError, DecisionResolveReport};
use crate::ui::activity::{ActivityDecisionSelectionError, ActivityDecisionSelectionId};

use super::EditorHostEventController;

impl EditorHostEventController {
    /// Resolves only the currently presented core Decision option.
    ///
    /// The route identifier is revalidated against the live core snapshot, so stale retained UI
    /// rows cannot resolve a replacement Decision that reused the same notification identity.
    pub(crate) fn resolve_activity_decision(
        &self,
        selection_id: &str,
    ) -> Result<DecisionResolveReport, ActivityDecisionResolutionError> {
        let selection_id = ActivityDecisionSelectionId::parse(selection_id)?;
        let selection = selection_id.selection()?;
        let center = self.context().notifications().decisions()?;
        let Some(current) = center.pending_snapshot().into_iter().next() else {
            return Err(ActivityDecisionResolutionError::NoPendingDecision);
        };
        if current.notification().id() != selection.notification_id() {
            return Err(ActivityDecisionResolutionError::NotCurrentDecision);
        }
        if !current.notification().has_option(selection.option_id()) {
            return Err(ActivityDecisionResolutionError::OptionUnavailable);
        }
        Ok(center.resolve(current.ticket(), selection.option_id())?)
    }
}

#[derive(Debug, Error)]
pub(crate) enum ActivityDecisionResolutionError {
    #[error(transparent)]
    InvalidSelection(#[from] ActivityDecisionSelectionError),
    #[error(transparent)]
    Decision(#[from] DecisionNotificationError),
    #[error("no pending editor Decision is available for resolution")]
    NoPendingDecision,
    #[error("the submitted editor Decision is no longer current")]
    NotCurrentDecision,
    #[error("the current editor Decision does not offer the submitted option")]
    OptionUnavailable,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::core::notifications::{
        DecisionNotification, DecisionOption, DecisionOptionId, NotificationId, NotificationSource,
    };
    use crate::ui::host::EditorManager;
    use crate::ui::workbench::state::EditorState;
    use zircon_runtime::core::CoreRuntime;
    use zircon_runtime::scene::DefaultLevelManager;
    use zircon_runtime_interface::math::UVec2;

    use super::{ActivityDecisionResolutionError, EditorHostEventController};

    fn controller() -> EditorHostEventController {
        let core = CoreRuntime::new();
        let manager = Arc::new(EditorManager::new(&core.handle()).unwrap());
        let state = EditorState::with_default_selection_with_context(
            DefaultLevelManager::default().create_default_level(),
            UVec2::new(1280, 720),
            Arc::clone(manager.context()),
        );
        EditorHostEventController::new(state, manager)
    }

    fn publish(controller: &EditorHostEventController, id: &str) {
        controller
            .context()
            .notifications()
            .decisions()
            .unwrap()
            .publish(
                DecisionNotification::new(
                    NotificationId::parse(id).unwrap(),
                    NotificationSource::builtin("editor.test").unwrap(),
                    "editor.play.pending_edits.title",
                    "editor.play.pending_edits.message",
                    vec![
                        DecisionOption::new(
                            DecisionOptionId::parse("apply").unwrap(),
                            "editor.play.pending_edits.apply",
                        )
                        .unwrap(),
                        DecisionOption::new(
                            DecisionOptionId::parse("discard").unwrap(),
                            "editor.play.pending_edits.discard",
                        )
                        .unwrap(),
                    ],
                )
                .unwrap(),
            )
            .unwrap();
    }

    #[test]
    fn route_cannot_skip_the_current_fifo_decision() {
        let controller = controller();
        publish(&controller, "editor.activity.first");
        publish(&controller, "editor.activity.second");

        assert!(matches!(
            controller.resolve_activity_decision("editor.activity.second:apply"),
            Err(ActivityDecisionResolutionError::NotCurrentDecision)
        ));
        assert!(controller
            .resolve_activity_decision("editor.activity.first:apply")
            .unwrap()
            .newly_resolved());
    }
}
