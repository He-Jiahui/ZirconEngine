use super::types::{action, spec, ActionControl, ExtensionNavigationSpec};

const LOBBY_EDITOR_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionLobbyEditorDefaultLobbyRow",
    "WorkbenchExtensionLobbyEditorLeaderSlotRow",
    "WorkbenchExtensionLobbyEditorRegionAutoRow",
    "WorkbenchExtensionLobbyEditorLeaderSlotTableRow",
    "WorkbenchExtensionLobbyEditorGuest01TableRow",
    "WorkbenchExtensionLobbyEditorGuest02TableRow",
    "WorkbenchExtensionLobbyEditorCrossplayRuleTableRow",
];
const LOBBY_EDITOR_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.lobby_editor.default_lobby_row.select",
        "WorkbenchExtensionLobbyEditorDefaultLobbyRow",
    ),
    action(
        "workbench.extension.lobby_editor.leader_slot_row.select",
        "WorkbenchExtensionLobbyEditorLeaderSlotRow",
    ),
    action(
        "workbench.extension.lobby_editor.region_auto_row.select",
        "WorkbenchExtensionLobbyEditorRegionAutoRow",
    ),
    action(
        "workbench.extension.lobby_editor.leader_slot_table_row.select",
        "WorkbenchExtensionLobbyEditorLeaderSlotTableRow",
    ),
    action(
        "workbench.extension.lobby_editor.guest_01_table_row.select",
        "WorkbenchExtensionLobbyEditorGuest01TableRow",
    ),
    action(
        "workbench.extension.lobby_editor.guest_02_table_row.select",
        "WorkbenchExtensionLobbyEditorGuest02TableRow",
    ),
    action(
        "workbench.extension.lobby_editor.crossplay_rule_table_row.select",
        "WorkbenchExtensionLobbyEditorCrossplayRuleTableRow",
    ),
];
const LOBBY_EDITOR_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAssetsGameplayToolsMenu",
    "WorkbenchExtensionLobbyEditorSimulateLobbyButton",
    "WorkbenchExtensionLobbyEditorValidateLobbyButton",
];
const LOBBY_EDITOR_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.lobby_editor.open",
        "WorkbenchAssetsGameplayToolsMenu",
    ),
    action(
        "workbench.extension.lobby_editor.simulate_lobby.invoke",
        "WorkbenchExtensionLobbyEditorSimulateLobbyButton",
    ),
    action(
        "workbench.extension.lobby_editor.validate_lobby.invoke",
        "WorkbenchExtensionLobbyEditorValidateLobbyButton",
    ),
];
const LOBBY_EDITOR_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.lobby_editor.template.edit",
    "workbench.extension.lobby_editor.template.commit",
    "workbench.extension.lobby_editor.region.edit",
    "workbench.extension.lobby_editor.region.commit",
    "workbench.extension.lobby_editor.max_players.edit",
    "workbench.extension.lobby_editor.max_players.commit",
];

pub(super) const LOBBY_EDITOR_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.lobby_editor.open",
    "WorkbenchExtensionLobbyEditorWorkspace",
    LOBBY_EDITOR_ROW_CONTROLS,
    LOBBY_EDITOR_ROW_ACTIONS,
    LOBBY_EDITOR_COMMAND_CONTROLS,
    LOBBY_EDITOR_COMMAND_ACTIONS,
    LOBBY_EDITOR_FIELD_ACTIONS,
);

const MATCHMAKING_EDITOR_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionMatchmakingEditorPlaylistRankedRow",
    "WorkbenchExtensionMatchmakingEditorQueueSoloRow",
    "WorkbenchExtensionMatchmakingEditorRuleLatencyRow",
    "WorkbenchExtensionMatchmakingEditorBronzeBucketTableRow",
    "WorkbenchExtensionMatchmakingEditorGoldBucketTableRow",
    "WorkbenchExtensionMatchmakingEditorDiamondBucketTableRow",
    "WorkbenchExtensionMatchmakingEditorBackfillTableRow",
];
const MATCHMAKING_EDITOR_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.matchmaking_editor.playlist_ranked_row.select",
        "WorkbenchExtensionMatchmakingEditorPlaylistRankedRow",
    ),
    action(
        "workbench.extension.matchmaking_editor.queue_solo_row.select",
        "WorkbenchExtensionMatchmakingEditorQueueSoloRow",
    ),
    action(
        "workbench.extension.matchmaking_editor.rule_latency_row.select",
        "WorkbenchExtensionMatchmakingEditorRuleLatencyRow",
    ),
    action(
        "workbench.extension.matchmaking_editor.bronze_bucket_table_row.select",
        "WorkbenchExtensionMatchmakingEditorBronzeBucketTableRow",
    ),
    action(
        "workbench.extension.matchmaking_editor.gold_bucket_table_row.select",
        "WorkbenchExtensionMatchmakingEditorGoldBucketTableRow",
    ),
    action(
        "workbench.extension.matchmaking_editor.diamond_bucket_table_row.select",
        "WorkbenchExtensionMatchmakingEditorDiamondBucketTableRow",
    ),
    action(
        "workbench.extension.matchmaking_editor.backfill_table_row.select",
        "WorkbenchExtensionMatchmakingEditorBackfillTableRow",
    ),
];
const MATCHMAKING_EDITOR_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAssetsGameplayToolsMenu",
    "WorkbenchExtensionMatchmakingEditorSimulateMatchButton",
    "WorkbenchExtensionMatchmakingEditorValidateRulesButton",
];
const MATCHMAKING_EDITOR_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.matchmaking_editor.open",
        "WorkbenchAssetsGameplayToolsMenu",
    ),
    action(
        "workbench.extension.matchmaking_editor.simulate_match.invoke",
        "WorkbenchExtensionMatchmakingEditorSimulateMatchButton",
    ),
    action(
        "workbench.extension.matchmaking_editor.validate_rules.invoke",
        "WorkbenchExtensionMatchmakingEditorValidateRulesButton",
    ),
];
const MATCHMAKING_EDITOR_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.matchmaking_editor.playlist.edit",
    "workbench.extension.matchmaking_editor.playlist.commit",
    "workbench.extension.matchmaking_editor.region.edit",
    "workbench.extension.matchmaking_editor.region.commit",
    "workbench.extension.matchmaking_editor.max_wait.edit",
    "workbench.extension.matchmaking_editor.max_wait.commit",
];

pub(super) const MATCHMAKING_EDITOR_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.matchmaking_editor.open",
    "WorkbenchExtensionMatchmakingEditorWorkspace",
    MATCHMAKING_EDITOR_ROW_CONTROLS,
    MATCHMAKING_EDITOR_ROW_ACTIONS,
    MATCHMAKING_EDITOR_COMMAND_CONTROLS,
    MATCHMAKING_EDITOR_COMMAND_ACTIONS,
    MATCHMAKING_EDITOR_FIELD_ACTIONS,
);
