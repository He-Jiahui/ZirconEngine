use super::*;
use crate::core::commands::{CommandEvalCtx, EditorCommandRegistry};

#[test]
fn every_controller_state_projects_the_same_admission_as_commands() {
    let registry = EditorCommandRegistry::default_workbench();
    for (state, show_stop, enter_enabled, stop_enabled, label) in [
        (PlayStateKind::Edit, false, true, false, "Play"),
        (PlayStateKind::Building, false, false, false, "Building…"),
        (PlayStateKind::Playing, true, false, true, "Stop"),
        (PlayStateKind::CleanupFailed, true, false, true, "Retry Stop"),
    ] {
        let context = CommandEvalCtx::interactive()
            .with_project_open(true)
            .with_play_state(state);
        let model = RunControlsModel::from_commands(&registry, &context);
        assert_eq!(model.state, state);
        let projection = toolbar_run_projection(model);
        assert_eq!(projection.play.visible, !show_stop, "{state:?}");
        assert_eq!(projection.stop.visible, show_stop, "{state:?}");
        assert_eq!(projection.play.enabled, enter_enabled, "{state:?}");
        assert_eq!(projection.stop.enabled, stop_enabled, "{state:?}");
        assert_eq!(projection.stop.checked, state == PlayStateKind::Playing);
        assert_eq!(if show_stop { projection.stop.label } else { projection.play.label }, label);
    }
}

#[test]
fn play_stays_visible_but_disabled_without_a_project() {
    let registry = EditorCommandRegistry::default_workbench();
    let model = RunControlsModel::from_commands(&registry, &CommandEvalCtx::interactive());
    let projection = toolbar_run_projection(model);
    assert!(projection.play.visible);
    assert!(!projection.play.enabled);
    assert!(!projection.stop.visible);
    assert!(!projection.stop.enabled);
}

#[test]
fn an_old_playing_navigation_value_reopens_as_project() {
    use crate::ui::workbench::startup::EditorSessionMode;
    let mode: EditorSessionMode = serde_json::from_str("\"Playing\"").unwrap();
    assert_eq!(mode, EditorSessionMode::Project);
    assert_eq!(serde_json::to_string(&mode).unwrap(), "\"Project\"");
}
