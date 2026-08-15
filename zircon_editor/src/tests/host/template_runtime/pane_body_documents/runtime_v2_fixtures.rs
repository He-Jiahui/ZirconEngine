use super::support::*;
use crate::ui::template_runtime::EditorUiHostRuntime;

#[test]
fn runtime_v2_fixture_buttons_project_interactive_metadata() {
    assert_runtime_v2_button_metadata(
        "pause_menu.zui",
        "test.runtime.pause_menu",
        &["ResumeButton", "SettingsButton", "QuitButton"],
    );
    assert_runtime_v2_button_metadata(
        "settings_dialog.zui",
        "test.runtime.settings_dialog",
        &[
            "AudioVolume",
            "GraphicsQuality",
            "ApplySettings",
            "CancelSettings",
        ],
    );
    assert_runtime_v2_button_metadata(
        "inventory_list.zui",
        "test.runtime.inventory_list",
        &["InventoryRow00", "InventoryRow11"],
    );
    assert_runtime_v2_button_metadata(
        "quest_log_dialog.zui",
        "test.runtime.quest_log_dialog",
        &["TrackQuestButton", "CloseQuestLogButton"],
    );

    let mut ui_runtime = EditorUiHostRuntime::default();
    ui_runtime
        .register_document_file(
            "test.runtime.quest_log_routes",
            runtime_v2_fixture_path("quest_log_dialog.zui"),
        )
        .unwrap();
    let surface = ui_runtime
        .build_shared_surface("test.runtime.quest_log_routes")
        .unwrap();
    assert_runtime_v2_click_route(
        &surface,
        "TrackQuestButton",
        "QuestLog/Track",
        "RuntimeAction.TrackQuest",
    );
    assert_runtime_v2_click_route(
        &surface,
        "CloseQuestLogButton",
        "QuestLog/Close",
        "RuntimeAction.CloseQuestLog",
    );
}
