use super::support::{
    open_v2_preview_session, UI_ASSET_EDITOR_RUNTIME_DIALOG_ZUI, UI_ASSET_EDITOR_RUNTIME_HUD_ZUI,
    UI_ASSET_EDITOR_RUNTIME_INVENTORY_ZUI, UI_ASSET_EDITOR_RUNTIME_QUEST_LOG_ZUI,
    UI_ASSET_EDITOR_RUNTIME_SETTINGS_ZUI,
};

fn assert_runtime_preview_session(
    route_id: &str,
    source: &str,
    hierarchy_fragment: &str,
    canvas_label: &str,
) {
    let session = open_v2_preview_session(route_id, source);

    assert!(session.diagnostics().is_empty());
    let pane = session.pane_presentation();
    assert!(pane.preview_available);
    assert!(pane
        .hierarchy_items
        .iter()
        .any(|item| item.contains(hierarchy_fragment)));
    assert!(pane
        .preview_canvas_items
        .iter()
        .any(|item| item.label == canvas_label));
}

#[test]
fn ui_asset_editor_runtime_hud_asset_opens_as_shared_runtime_preview_session() {
    assert_runtime_preview_session(
        "res://ui/runtime/fixtures/hud_overlay.zui",
        UI_ASSET_EDITOR_RUNTIME_HUD_ZUI,
        "hud_root [Overlay]",
        "HealthPanel",
    );
}

#[test]
fn ui_asset_editor_runtime_pause_dialog_asset_opens_as_shared_runtime_preview_session() {
    assert_runtime_preview_session(
        "res://ui/runtime/fixtures/pause_menu.zui",
        UI_ASSET_EDITOR_RUNTIME_DIALOG_ZUI,
        "pause_root [Overlay]",
        "PauseDialog",
    );
}

#[test]
fn ui_asset_editor_runtime_settings_dialog_asset_opens_as_shared_runtime_preview_session() {
    assert_runtime_preview_session(
        "res://ui/runtime/fixtures/settings_dialog.zui",
        UI_ASSET_EDITOR_RUNTIME_SETTINGS_ZUI,
        "settings_root [Overlay]",
        "SettingsDialog",
    );
}

#[test]
fn ui_asset_editor_runtime_inventory_dialog_asset_opens_as_shared_runtime_preview_session() {
    assert_runtime_preview_session(
        "res://ui/runtime/fixtures/inventory_list.zui",
        UI_ASSET_EDITOR_RUNTIME_INVENTORY_ZUI,
        "inventory_root [Overlay]",
        "InventoryPanel",
    );
}

#[test]
fn ui_asset_editor_runtime_quest_log_dialog_asset_opens_as_shared_runtime_preview_session() {
    assert_runtime_preview_session(
        "res://ui/runtime/fixtures/quest_log_dialog.zui",
        UI_ASSET_EDITOR_RUNTIME_QUEST_LOG_ZUI,
        "quest_root [Overlay]",
        "QuestLogDialog",
    );
}
