use super::*;
use crate::ui::binding::EditorUiBindingPayload;
use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::startup::EditorSessionMode;
use zircon_runtime_interface::ui::binding::UiEventKind;
use zircon_runtime_interface::ui::style::ButtonColor;

#[test]
fn toolbar_run_control_tracks_the_editor_play_session() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };
    let shell_size = UiSize::new(FULL_WORKBENCH_WIDTH as f32, FULL_WORKBENCH_HEIGHT as f32);
    let metrics = WorkbenchChromeMetrics::default();
    let registry = crate::core::commands::EditorCommandRegistry::default_workbench();
    let mut chrome = default_preview_fixture().build_chrome();
    chrome.session_mode = EditorSessionMode::Project;
    let edit_model = WorkbenchViewModel::build(&registry, &chrome);
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(shell_size)
        .expect("full workbench should build");

    bridge
        .recompute_layout_with_workbench_model(shell_size, &edit_model, &metrics)
        .expect("edit-mode workbench should recompute");
    assert_eq!(
        control_visibility(&bridge, "WorkbenchRunPlay"),
        Some(UiVisibility::Visible)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchRunStop"),
        Some(UiVisibility::Collapsed)
    );
    let play_frame = bridge
        .control_frame("WorkbenchRunPlay")
        .expect("Play should occupy the run-control slot in edit mode");
    let play_binding = bridge
        .binding_for_control("WorkbenchRunPlay", UiEventKind::Click)
        .expect("Play should keep its enter-play binding");
    assert_eq!(
        play_binding.payload(),
        &EditorUiBindingPayload::editor_command("runtime.play_mode.enter")
    );

    chrome.session_mode = EditorSessionMode::Playing;
    let playing_model = WorkbenchViewModel::build(&registry, &chrome);
    bridge
        .recompute_layout_with_workbench_model(shell_size, &playing_model, &metrics)
        .expect("playing workbench should recompute");
    assert_eq!(
        control_visibility(&bridge, "WorkbenchRunPlay"),
        Some(UiVisibility::Collapsed)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchRunStop"),
        Some(UiVisibility::Visible)
    );
    let stop_frame = bridge
        .control_frame("WorkbenchRunStop")
        .expect("Stop should occupy the run-control slot while playing");
    assert_eq!(
        workbench_window_node(&bridge, "WorkbenchRunStop")
            .button_style
            .color,
        ButtonColor::Error
    );
    assert_frame_value("run-control slot x", stop_frame.x, play_frame.x);
    assert_frame_value("run-control slot width", stop_frame.width, play_frame.width);
    let stop_binding = bridge
        .binding_for_control("WorkbenchRunStop", UiEventKind::Click)
        .expect("Stop should expose the canonical exit-play binding");
    assert_eq!(
        stop_binding.payload(),
        &EditorUiBindingPayload::editor_command("runtime.play_mode.exit")
    );
}
