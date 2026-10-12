use super::{action, spec, ActionControl, ExtensionNavigationSpec};

const PREFAB_EDITOR_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionPrefabEditorPrefabRootRow",
    "WorkbenchExtensionPrefabEditorMeshComponentRow",
    "WorkbenchExtensionPrefabEditorLootSocketRow",
    "WorkbenchExtensionPrefabEditorMeshTableRow",
    "WorkbenchExtensionPrefabEditorLootSocketTableRow",
    "WorkbenchExtensionPrefabEditorLightTableRow",
    "WorkbenchExtensionPrefabEditorOverrideTableRow",
];
const PREFAB_EDITOR_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.prefab_editor.prefab_root_row.select",
        "WorkbenchExtensionPrefabEditorPrefabRootRow",
    ),
    action(
        "workbench.extension.prefab_editor.mesh_component_row.select",
        "WorkbenchExtensionPrefabEditorMeshComponentRow",
    ),
    action(
        "workbench.extension.prefab_editor.loot_socket_row.select",
        "WorkbenchExtensionPrefabEditorLootSocketRow",
    ),
    action(
        "workbench.extension.prefab_editor.mesh_table_row.select",
        "WorkbenchExtensionPrefabEditorMeshTableRow",
    ),
    action(
        "workbench.extension.prefab_editor.loot_socket_table_row.select",
        "WorkbenchExtensionPrefabEditorLootSocketTableRow",
    ),
    action(
        "workbench.extension.prefab_editor.light_table_row.select",
        "WorkbenchExtensionPrefabEditorLightTableRow",
    ),
    action(
        "workbench.extension.prefab_editor.override_table_row.select",
        "WorkbenchExtensionPrefabEditorOverrideTableRow",
    ),
];
const PREFAB_EDITOR_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAssetsWorldToolsMenu",
    "WorkbenchExtensionPrefabEditorApplyButton",
    "WorkbenchExtensionPrefabEditorValidateButton",
];
const PREFAB_EDITOR_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.prefab_editor.open",
        "WorkbenchAssetsWorldToolsMenu",
    ),
    action(
        "workbench.extension.prefab_editor.apply.invoke",
        "WorkbenchExtensionPrefabEditorApplyButton",
    ),
    action(
        "workbench.extension.prefab_editor.validate.invoke",
        "WorkbenchExtensionPrefabEditorValidateButton",
    ),
];
const PREFAB_EDITOR_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.prefab_editor.prefab.edit",
    "workbench.extension.prefab_editor.prefab.commit",
    "workbench.extension.prefab_editor.variant.edit",
    "workbench.extension.prefab_editor.variant.commit",
    "workbench.extension.prefab_editor.instance.edit",
    "workbench.extension.prefab_editor.instance.commit",
];

pub(in super::super) const PREFAB_EDITOR_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.prefab_editor.open",
    "WorkbenchExtensionPrefabEditorWorkspace",
    PREFAB_EDITOR_ROW_CONTROLS,
    PREFAB_EDITOR_ROW_ACTIONS,
    PREFAB_EDITOR_COMMAND_CONTROLS,
    PREFAB_EDITOR_COMMAND_ACTIONS,
    PREFAB_EDITOR_FIELD_ACTIONS,
);

const SCATTER_EDITOR_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionScatterEditorSetRow",
    "WorkbenchExtensionScatterEditorRocksRuleRow",
    "WorkbenchExtensionScatterEditorFernsRuleRow",
    "WorkbenchExtensionScatterEditorBiomeMaskTableRow",
    "WorkbenchExtensionScatterEditorSlopeFilterTableRow",
    "WorkbenchExtensionScatterEditorSpawnRuleTableRow",
    "WorkbenchExtensionScatterEditorCollisionTestTableRow",
];
const SCATTER_EDITOR_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.scatter_editor.set_row.select",
        "WorkbenchExtensionScatterEditorSetRow",
    ),
    action(
        "workbench.extension.scatter_editor.rocks_rule_row.select",
        "WorkbenchExtensionScatterEditorRocksRuleRow",
    ),
    action(
        "workbench.extension.scatter_editor.ferns_rule_row.select",
        "WorkbenchExtensionScatterEditorFernsRuleRow",
    ),
    action(
        "workbench.extension.scatter_editor.biome_mask_table_row.select",
        "WorkbenchExtensionScatterEditorBiomeMaskTableRow",
    ),
    action(
        "workbench.extension.scatter_editor.slope_filter_table_row.select",
        "WorkbenchExtensionScatterEditorSlopeFilterTableRow",
    ),
    action(
        "workbench.extension.scatter_editor.spawn_rule_table_row.select",
        "WorkbenchExtensionScatterEditorSpawnRuleTableRow",
    ),
    action(
        "workbench.extension.scatter_editor.collision_test_table_row.select",
        "WorkbenchExtensionScatterEditorCollisionTestTableRow",
    ),
];
const SCATTER_EDITOR_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAssetsWorldToolsMenu",
    "WorkbenchExtensionScatterEditorGenerateButton",
    "WorkbenchExtensionScatterEditorValidateButton",
];
const SCATTER_EDITOR_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.scatter_editor.open",
        "WorkbenchAssetsWorldToolsMenu",
    ),
    action(
        "workbench.extension.scatter_editor.generate.invoke",
        "WorkbenchExtensionScatterEditorGenerateButton",
    ),
    action(
        "workbench.extension.scatter_editor.validate.invoke",
        "WorkbenchExtensionScatterEditorValidateButton",
    ),
];
const SCATTER_EDITOR_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.scatter_editor.rule_set.edit",
    "workbench.extension.scatter_editor.rule_set.commit",
    "workbench.extension.scatter_editor.seed.edit",
    "workbench.extension.scatter_editor.seed.commit",
    "workbench.extension.scatter_editor.density.edit",
    "workbench.extension.scatter_editor.density.commit",
];

pub(in super::super) const SCATTER_EDITOR_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.scatter_editor.open",
    "WorkbenchExtensionScatterEditorWorkspace",
    SCATTER_EDITOR_ROW_CONTROLS,
    SCATTER_EDITOR_ROW_ACTIONS,
    SCATTER_EDITOR_COMMAND_CONTROLS,
    SCATTER_EDITOR_COMMAND_ACTIONS,
    SCATTER_EDITOR_FIELD_ACTIONS,
);
