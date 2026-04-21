use super::support::{
    open_preview_session, UI_ASSET_EDITOR_RUNTIME_DIALOG_TOML, UI_ASSET_EDITOR_RUNTIME_HUD_TOML,
    UI_ASSET_EDITOR_RUNTIME_INVENTORY_TOML, UI_ASSET_EDITOR_RUNTIME_QUEST_LOG_TOML,
    UI_ASSET_EDITOR_RUNTIME_SETTINGS_TOML,
};

fn assert_runtime_preview_session(
    route_id: &str,
    source: &str,
    hierarchy_fragment: &str,
    canvas_label: &str,
) {
    let session = open_preview_session(route_id, source);

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
        "res://ui/runtime/runtime_hud.ui.toml",
        UI_ASSET_EDITOR_RUNTIME_HUD_TOML,
        "hud_root [VerticalBox]",
        "RuntimeHudRoot",
    );
}

#[test]
fn ui_asset_editor_runtime_pause_dialog_asset_opens_as_shared_runtime_preview_session() {
    assert_runtime_preview_session(
        "res://ui/runtime/pause_dialog.ui.toml",
        UI_ASSET_EDITOR_RUNTIME_DIALOG_TOML,
        "pause_dialog_root [Overlay]",
        "PauseDialogRoot",
    );
}

#[test]
fn ui_asset_editor_runtime_settings_dialog_asset_opens_as_shared_runtime_preview_session() {
    assert_runtime_preview_session(
        "res://ui/runtime/settings_dialog.ui.toml",
        UI_ASSET_EDITOR_RUNTIME_SETTINGS_TOML,
        "settings_dialog_root [Overlay]",
        "SettingsDialogRoot",
    );
}

#[test]
fn ui_asset_editor_runtime_inventory_dialog_asset_opens_as_shared_runtime_preview_session() {
    assert_runtime_preview_session(
        "res://ui/runtime/inventory_dialog.ui.toml",
        UI_ASSET_EDITOR_RUNTIME_INVENTORY_TOML,
        "inventory_dialog_root [Overlay]",
        "InventoryDialogRoot",
    );
}

#[test]
fn ui_asset_editor_runtime_quest_log_dialog_asset_opens_as_shared_runtime_preview_session() {
    assert_runtime_preview_session(
        "res://ui/runtime/quest_log_dialog.ui.toml",
        UI_ASSET_EDITOR_RUNTIME_QUEST_LOG_TOML,
        "quest_log_dialog_root [Overlay]",
        "QuestLogDialogRoot",
    );
}
