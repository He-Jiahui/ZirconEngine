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
