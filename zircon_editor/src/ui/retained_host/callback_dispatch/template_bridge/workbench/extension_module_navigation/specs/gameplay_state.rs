use super::types::{action, spec, ActionControl, ExtensionNavigationSpec};

const SPAWN_RULES_TAB_CONTROLS: &[&str] = &[
    "WorkbenchExtensionSpawnRulesRulesTab",
    "WorkbenchExtensionSpawnRulesStateTab",
    "WorkbenchExtensionSpawnRulesValidationTab",
];
const SPAWN_RULES_TAB_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.spawn_rules.rules_tab.select",
        "WorkbenchExtensionSpawnRulesRulesTab",
    ),
    action(
        "workbench.extension.spawn_rules.state_tab.select",
        "WorkbenchExtensionSpawnRulesStateTab",
    ),
    action(
        "workbench.extension.spawn_rules.validation_tab.select",
        "WorkbenchExtensionSpawnRulesValidationTab",
    ),
];
const SPAWN_RULES_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionSpawnRulesEnemyRow",
    "WorkbenchExtensionSpawnRulesZoneARow",
    "WorkbenchExtensionSpawnRulesConditionNightRow",
    "WorkbenchExtensionSpawnRulesZoneATableRow",
    "WorkbenchExtensionSpawnRulesConditionNightTableRow",
    "WorkbenchExtensionSpawnRulesTagCombatTableRow",
    "WorkbenchExtensionSpawnRulesConflictTableRow",
    "WorkbenchExtensionSpawnRulesOutputRow",
];
const SPAWN_RULES_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.spawn_rules.enemy_row.select",
        "WorkbenchExtensionSpawnRulesEnemyRow",
    ),
    action(
        "workbench.extension.spawn_rules.zone_a_row.select",
        "WorkbenchExtensionSpawnRulesZoneARow",
    ),
    action(
        "workbench.extension.spawn_rules.condition_night_row.select",
        "WorkbenchExtensionSpawnRulesConditionNightRow",
    ),
    action(
        "workbench.extension.spawn_rules.zone_a_table_row.select",
        "WorkbenchExtensionSpawnRulesZoneATableRow",
    ),
    action(
        "workbench.extension.spawn_rules.condition_night_table_row.select",
        "WorkbenchExtensionSpawnRulesConditionNightTableRow",
    ),
    action(
        "workbench.extension.spawn_rules.tag_combat_table_row.select",
        "WorkbenchExtensionSpawnRulesTagCombatTableRow",
    ),
    action(
        "workbench.extension.spawn_rules.conflict_table_row.select",
        "WorkbenchExtensionSpawnRulesConflictTableRow",
    ),
    action(
        "workbench.extension.spawn_rules.output.select",
        "WorkbenchExtensionSpawnRulesOutputRow",
    ),
];
const SPAWN_RULES_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAssetsSpawnRulesButton",
    "WorkbenchExtensionSpawnRulesSimulateButton",
    "WorkbenchExtensionSpawnRulesValidateButton",
];
const SPAWN_RULES_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.spawn_rules.open",
        "WorkbenchAssetsSpawnRulesButton",
    ),
    action(
        "workbench.extension.spawn_rules.simulate.invoke",
        "WorkbenchExtensionSpawnRulesSimulateButton",
    ),
    action(
        "workbench.extension.spawn_rules.validate.invoke",
        "WorkbenchExtensionSpawnRulesValidateButton",
    ),
];
const SPAWN_RULES_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.spawn_rules.rule_set.edit",
    "workbench.extension.spawn_rules.rule_set.commit",
    "workbench.extension.spawn_rules.authority.edit",
    "workbench.extension.spawn_rules.authority.commit",
    "workbench.extension.spawn_rules.seed.edit",
    "workbench.extension.spawn_rules.seed.commit",
];

pub(super) const SPAWN_RULES_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.spawn_rules.open",
    "WorkbenchExtensionSpawnRulesWorkspace",
    SPAWN_RULES_TAB_CONTROLS,
    SPAWN_RULES_TAB_ACTIONS,
    SPAWN_RULES_ROW_CONTROLS,
    SPAWN_RULES_ROW_ACTIONS,
    SPAWN_RULES_COMMAND_CONTROLS,
    SPAWN_RULES_COMMAND_ACTIONS,
    SPAWN_RULES_FIELD_ACTIONS,
);

const WORLD_STATE_TAB_CONTROLS: &[&str] = &[
    "WorkbenchExtensionWorldStateKeysTab",
    "WorkbenchExtensionWorldStateScenarioTab",
    "WorkbenchExtensionWorldStateTimelineTab",
];
const WORLD_STATE_TAB_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.world_state.keys_tab.select",
        "WorkbenchExtensionWorldStateKeysTab",
    ),
    action(
        "workbench.extension.world_state.scenario_tab.select",
        "WorkbenchExtensionWorldStateScenarioTab",
    ),
    action(
        "workbench.extension.world_state.timeline_tab.select",
        "WorkbenchExtensionWorldStateTimelineTab",
    ),
];
const WORLD_STATE_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionWorldStateNightRaidRow",
    "WorkbenchExtensionWorldStateGlobalLayerRow",
    "WorkbenchExtensionWorldStateAlarmKeyRow",
    "WorkbenchExtensionWorldStateAlarmActiveTableRow",
    "WorkbenchExtensionWorldStateWeatherModeTableRow",
    "WorkbenchExtensionWorldStateAiAlertTableRow",
    "WorkbenchExtensionWorldStateQuestFlagTableRow",
    "WorkbenchExtensionWorldStateOutputRow",
];
const WORLD_STATE_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.world_state.night_raid_row.select",
        "WorkbenchExtensionWorldStateNightRaidRow",
    ),
    action(
        "workbench.extension.world_state.global_layer_row.select",
        "WorkbenchExtensionWorldStateGlobalLayerRow",
    ),
    action(
        "workbench.extension.world_state.alarm_key_row.select",
        "WorkbenchExtensionWorldStateAlarmKeyRow",
    ),
    action(
        "workbench.extension.world_state.alarm_active_table_row.select",
        "WorkbenchExtensionWorldStateAlarmActiveTableRow",
    ),
    action(
        "workbench.extension.world_state.weather_mode_table_row.select",
        "WorkbenchExtensionWorldStateWeatherModeTableRow",
    ),
    action(
        "workbench.extension.world_state.ai_alert_table_row.select",
        "WorkbenchExtensionWorldStateAiAlertTableRow",
    ),
    action(
        "workbench.extension.world_state.quest_flag_table_row.select",
        "WorkbenchExtensionWorldStateQuestFlagTableRow",
    ),
    action(
        "workbench.extension.world_state.output.select",
        "WorkbenchExtensionWorldStateOutputRow",
    ),
];
const WORLD_STATE_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAssetsWorldStateButton",
    "WorkbenchExtensionWorldStateSimulateButton",
    "WorkbenchExtensionWorldStateValidateButton",
];
const WORLD_STATE_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.world_state.open",
        "WorkbenchAssetsWorldStateButton",
    ),
    action(
        "workbench.extension.world_state.simulate.invoke",
        "WorkbenchExtensionWorldStateSimulateButton",
    ),
    action(
        "workbench.extension.world_state.validate.invoke",
        "WorkbenchExtensionWorldStateValidateButton",
    ),
];
const WORLD_STATE_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.world_state.scenario.edit",
    "workbench.extension.world_state.scenario.commit",
    "workbench.extension.world_state.layer.edit",
    "workbench.extension.world_state.layer.commit",
    "workbench.extension.world_state.authority.edit",
    "workbench.extension.world_state.authority.commit",
];

pub(super) const WORLD_STATE_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.world_state.open",
    "WorkbenchExtensionWorldStateWorkspace",
    WORLD_STATE_TAB_CONTROLS,
    WORLD_STATE_TAB_ACTIONS,
    WORLD_STATE_ROW_CONTROLS,
    WORLD_STATE_ROW_ACTIONS,
    WORLD_STATE_COMMAND_CONTROLS,
    WORLD_STATE_COMMAND_ACTIONS,
    WORLD_STATE_FIELD_ACTIONS,
);

const NAVMESH_AI_TAB_CONTROLS: &[&str] = &[
    "WorkbenchExtensionNavmeshAiTilesTab",
    "WorkbenchExtensionNavmeshAiAgentsTab",
    "WorkbenchExtensionNavmeshAiQueriesTab",
];
const NAVMESH_AI_TAB_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.navmesh_ai.tiles_tab.select",
        "WorkbenchExtensionNavmeshAiTilesTab",
    ),
    action(
        "workbench.extension.navmesh_ai.agents_tab.select",
        "WorkbenchExtensionNavmeshAiAgentsTab",
    ),
    action(
        "workbench.extension.navmesh_ai.queries_tab.select",
        "WorkbenchExtensionNavmeshAiQueriesTab",
    ),
];
const NAVMESH_AI_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionNavmeshAiMainNavmeshRow",
    "WorkbenchExtensionNavmeshAiHumanoidAgentRow",
    "WorkbenchExtensionNavmeshAiPatrolQueryRow",
    "WorkbenchExtensionNavmeshAiTile1208TableRow",
    "WorkbenchExtensionNavmeshAiTile1209TableRow",
    "WorkbenchExtensionNavmeshAiAgentRadiusTableRow",
    "WorkbenchExtensionNavmeshAiBlockedLinkTableRow",
    "WorkbenchExtensionNavmeshAiOutputRow",
];
const NAVMESH_AI_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.navmesh_ai.main_navmesh_row.select",
        "WorkbenchExtensionNavmeshAiMainNavmeshRow",
    ),
    action(
        "workbench.extension.navmesh_ai.humanoid_agent_row.select",
        "WorkbenchExtensionNavmeshAiHumanoidAgentRow",
    ),
    action(
        "workbench.extension.navmesh_ai.patrol_query_row.select",
        "WorkbenchExtensionNavmeshAiPatrolQueryRow",
    ),
    action(
        "workbench.extension.navmesh_ai.tile_1208_table_row.select",
        "WorkbenchExtensionNavmeshAiTile1208TableRow",
    ),
    action(
        "workbench.extension.navmesh_ai.tile_1209_table_row.select",
        "WorkbenchExtensionNavmeshAiTile1209TableRow",
    ),
    action(
        "workbench.extension.navmesh_ai.agent_radius_table_row.select",
        "WorkbenchExtensionNavmeshAiAgentRadiusTableRow",
    ),
    action(
        "workbench.extension.navmesh_ai.blocked_link_table_row.select",
        "WorkbenchExtensionNavmeshAiBlockedLinkTableRow",
    ),
    action(
        "workbench.extension.navmesh_ai.output.select",
        "WorkbenchExtensionNavmeshAiOutputRow",
    ),
];
const NAVMESH_AI_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAssetsNavmeshAiButton",
    "WorkbenchExtensionNavmeshAiRebuildButton",
    "WorkbenchExtensionNavmeshAiQueryPathButton",
];
const NAVMESH_AI_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.navmesh_ai.open",
        "WorkbenchAssetsNavmeshAiButton",
    ),
    action(
        "workbench.extension.navmesh_ai.rebuild.invoke",
        "WorkbenchExtensionNavmeshAiRebuildButton",
    ),
    action(
        "workbench.extension.navmesh_ai.query_path.invoke",
        "WorkbenchExtensionNavmeshAiQueryPathButton",
    ),
];
const NAVMESH_AI_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.navmesh_ai.agent.edit",
    "workbench.extension.navmesh_ai.agent.commit",
    "workbench.extension.navmesh_ai.area.edit",
    "workbench.extension.navmesh_ai.area.commit",
    "workbench.extension.navmesh_ai.cost.edit",
    "workbench.extension.navmesh_ai.cost.commit",
];

pub(super) const NAVMESH_AI_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.navmesh_ai.open",
    "WorkbenchExtensionNavmeshAiWorkspace",
    NAVMESH_AI_TAB_CONTROLS,
    NAVMESH_AI_TAB_ACTIONS,
    NAVMESH_AI_ROW_CONTROLS,
    NAVMESH_AI_ROW_ACTIONS,
    NAVMESH_AI_COMMAND_CONTROLS,
    NAVMESH_AI_COMMAND_ACTIONS,
    NAVMESH_AI_FIELD_ACTIONS,
);
