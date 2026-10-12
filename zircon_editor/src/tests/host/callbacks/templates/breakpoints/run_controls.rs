use super::super::super::support::{
    env_lock, BuiltinWorkbenchWindowTemplateSurfaceBridge, EditorUiBindingPayload, UiEventKind,
    UiSize,
};
use super::super::support::control_visibility;
use super::support::{
    assert_frame_value, workbench_window_node, FULL_WORKBENCH_HEIGHT, FULL_WORKBENCH_WIDTH,
};
use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::startup::EditorSessionMode;
use zircon_runtime_interface::ui::style::ButtonColor;
use zircon_runtime_interface::ui::tree::UiVisibility;

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

#[test]
fn mvp_run_and_layout_controls_remain_reachable_across_all_layout_tiers() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    for (width, height, tools_visible) in [
        (420.0, 360.0, false),
        (640.0, 520.0, false),
        (900.0, 620.0, false),
        (1260.0, 780.0, true),
    ] {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(width, height))
                .unwrap_or_else(|error| {
                    panic!("{width}px workbench bridge should build: {error:?}")
                });
        let command_row = bridge
            .control_frame("WorkbenchToolbarCommandRow")
            .unwrap_or_else(|| panic!("{width}px toolbar should expose its command row"));
        let module_commands = bridge
            .control_frame("WorkbenchModuleCommands")
            .unwrap_or_else(|| panic!("{width}px toolbar should expose primary module commands"));
        let run_group = bridge
            .control_frame("WorkbenchToolbarRunGroup")
            .unwrap_or_else(|| panic!("{width}px toolbar should keep the MVP run group reachable"));
        let play = bridge
            .control_frame("WorkbenchRunPlay")
            .unwrap_or_else(|| panic!("{width}px toolbar should keep Play reachable"));
        let run_mode = bridge
            .control_frame("WorkbenchRunMode")
            .unwrap_or_else(|| panic!("{width}px toolbar should keep Run Mode reachable"));
        let layout_group = bridge
            .control_frame("WorkbenchToolbarLayoutGroup")
            .unwrap_or_else(|| panic!("{width}px toolbar should keep Layout and Theme reachable"));

        assert_frame_value("compact MVP run group width", run_group.width, 70.0);
        assert_frame_value("compact layout group width", layout_group.width, 68.0);
        assert_frame_value("Play to Run Mode gap", run_mode.x - play.right(), 4.0);
        assert!(
            run_group.x >= module_commands.right(),
            "{width}px MVP run controls should follow primary commands without overlap"
        );
        assert!(
            run_group.right() <= command_row.right(),
            "{width}px MVP run controls should remain inside the command row"
        );
        for control_id in ["WorkbenchLayoutGrid", "WorkbenchThemeToggle"] {
            assert_eq!(
                control_visibility(&bridge, control_id),
                Some(UiVisibility::Visible),
                "{width}px toolbar should keep the iconized {control_id} command reachable"
            );
        }
        assert!(
            layout_group.x >= run_group.right(),
            "{width}px layout commands should follow Run without overlap"
        );
        assert!(
            layout_group.right() <= command_row.right(),
            "{width}px layout commands should remain inside the command row"
        );
        assert_eq!(
            control_visibility(&bridge, "WorkbenchToolbarToolGroup"),
            Some(if tools_visible {
                UiVisibility::Visible
            } else {
                UiVisibility::Collapsed
            }),
            "{width}px toolbar should project the transform-tool group by business priority"
        );
        if tools_visible {
            let tools = bridge
                .control_frame("WorkbenchToolbarToolGroup")
                .unwrap_or_else(|| panic!("{width}px toolbar should expose transform tools"));
            assert!(
                tools.x >= module_commands.right() && tools.right() <= run_group.x,
                "{width}px transform tools should fit between module and run groups"
            );
        }

        let play_binding = bridge
            .dispatch_control_state("WorkbenchRunPlay", UiEventKind::Click)
            .unwrap_or_else(|error| panic!("{width}px Play should dispatch: {error:?}"))
            .unwrap_or_else(|| panic!("{width}px Play should expose a binding"));
        assert!(matches!(
            play_binding.payload(),
            EditorUiBindingPayload::EditorCommand { command_id }
                if command_id == "runtime.play_mode.enter"
        ));
    }
}
