use super::types::{ActionControl, ExtensionNavigationSpec, action, spec};

const LOBBY_EDITOR_TAB_CONTROLS: &[&str] = &[
    "WorkbenchExtensionLobbyEditorFlowTab",
    "WorkbenchExtensionLobbyEditorSlotsTab",
    "WorkbenchExtensionLobbyEditorTelemetryTab",
];
const LOBBY_EDITOR_TAB_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.lobby_editor.flow_tab.select",
        "WorkbenchExtensionLobbyEditorFlowTab",
    ),
    action(
        "workbench.extension.lobby_editor.slots_tab.select",
        "WorkbenchExtensionLobbyEditorSlotsTab",
    ),
    action(
        "workbench.extension.lobby_editor.telemetry_tab.select",
        "WorkbenchExtensionLobbyEditorTelemetryTab",
    ),
];
const LOBBY_EDITOR_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionLobbyEditorDefaultLobbyRow",
    "WorkbenchExtensionLobbyEditorLeaderSlotRow",
    "WorkbenchExtensionLobbyEditorRegionAutoRow",
    "WorkbenchExtensionLobbyEditorLeaderSlotTableRow",
    "WorkbenchExtensionLobbyEditorGuest01TableRow",
    "WorkbenchExtensionLobbyEditorGuest02TableRow",
    "WorkbenchExtensionLobbyEditorCrossplayRuleTableRow",
    "WorkbenchExtensionLobbyEditorOutputRow",
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
    action(
        "workbench.extension.lobby_editor.output.select",
        "WorkbenchExtensionLobbyEditorOutputRow",
    ),
];
const LOBBY_EDITOR_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAssetsLobbyEditorButton",
    "WorkbenchExtensionLobbyEditorSimulateLobbyButton",
    "WorkbenchExtensionLobbyEditorValidateLobbyButton",
];
const LOBBY_EDITOR_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.lobby_editor.open",
        "WorkbenchAssetsLobbyEditorButton",
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
    LOBBY_EDITOR_TAB_CONTROLS,
    LOBBY_EDITOR_TAB_ACTIONS,
    LOBBY_EDITOR_ROW_CONTROLS,
    LOBBY_EDITOR_ROW_ACTIONS,
    LOBBY_EDITOR_COMMAND_CONTROLS,
    LOBBY_EDITOR_COMMAND_ACTIONS,
    LOBBY_EDITOR_FIELD_ACTIONS,
);

const MATCHMAKING_EDITOR_TAB_CONTROLS: &[&str] = &[
    "WorkbenchExtensionMatchmakingEditorQueuesTab",
    "WorkbenchExtensionMatchmakingEditorRulesTab",
    "WorkbenchExtensionMatchmakingEditorTelemetryTab",
];
const MATCHMAKING_EDITOR_TAB_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.matchmaking_editor.queues_tab.select",
        "WorkbenchExtensionMatchmakingEditorQueuesTab",
    ),
    action(
        "workbench.extension.matchmaking_editor.rules_tab.select",
        "WorkbenchExtensionMatchmakingEditorRulesTab",
    ),
    action(
        "workbench.extension.matchmaking_editor.telemetry_tab.select",
        "WorkbenchExtensionMatchmakingEditorTelemetryTab",
    ),
];
const MATCHMAKING_EDITOR_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionMatchmakingEditorPlaylistRankedRow",
    "WorkbenchExtensionMatchmakingEditorQueueSoloRow",
    "WorkbenchExtensionMatchmakingEditorRuleLatencyRow",
    "WorkbenchExtensionMatchmakingEditorBronzeBucketTableRow",
    "WorkbenchExtensionMatchmakingEditorGoldBucketTableRow",
    "WorkbenchExtensionMatchmakingEditorDiamondBucketTableRow",
    "WorkbenchExtensionMatchmakingEditorBackfillTableRow",
    "WorkbenchExtensionMatchmakingEditorOutputRow",
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
    action(
        "workbench.extension.matchmaking_editor.output.select",
        "WorkbenchExtensionMatchmakingEditorOutputRow",
    ),
];
const MATCHMAKING_EDITOR_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAssetsMatchmakingEditorButton",
    "WorkbenchExtensionMatchmakingEditorSimulateMatchButton",
    "WorkbenchExtensionMatchmakingEditorValidateRulesButton",
];
const MATCHMAKING_EDITOR_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.matchmaking_editor.open",
        "WorkbenchAssetsMatchmakingEditorButton",
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
    MATCHMAKING_EDITOR_TAB_CONTROLS,
    MATCHMAKING_EDITOR_TAB_ACTIONS,
    MATCHMAKING_EDITOR_ROW_CONTROLS,
    MATCHMAKING_EDITOR_ROW_ACTIONS,
    MATCHMAKING_EDITOR_COMMAND_CONTROLS,
    MATCHMAKING_EDITOR_COMMAND_ACTIONS,
    MATCHMAKING_EDITOR_FIELD_ACTIONS,
);
