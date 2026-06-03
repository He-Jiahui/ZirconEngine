pub(super) const MODULE_TAB_CONTROLS: &[&str] = &[
    "WorkbenchModuleScene",
    "WorkbenchModuleEffect",
    "WorkbenchModuleAbility",
    "WorkbenchModuleTags",
    "WorkbenchModulePerception",
    "WorkbenchModuleMaterial",
    "WorkbenchModuleBehavior",
    "WorkbenchModuleRender",
    "WorkbenchModuleAssets",
    "WorkbenchModuleVfx",
    "WorkbenchModuleHud",
];

pub(super) const MODULE_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchModuleSave",
    "WorkbenchModuleBrowse",
    "WorkbenchModuleCompile",
    "WorkbenchModuleDiff",
    "WorkbenchModuleSimulate",
];

pub(super) const MODULE_WORKSPACE_CONTROLS: &[&str] = &[
    "WorkbenchModuleEffectWorkspace",
    "WorkbenchModuleAbilityWorkspace",
    "WorkbenchModuleTagsWorkspace",
    "WorkbenchModulePerceptionWorkspace",
    "WorkbenchModuleMaterialWorkspace",
    "WorkbenchModuleBehaviorWorkspace",
    "WorkbenchModuleRenderWorkspace",
    "WorkbenchModuleAssetsWorkspace",
    "WorkbenchModuleVfxWorkspace",
    "WorkbenchModuleHudWorkspace",
];

pub(super) const MODULE_PANEL_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchEffectApplyButton",
    "WorkbenchMaterialCompileButton",
    "WorkbenchBehaviorValidateButton",
    "WorkbenchAssetsImportButton",
    "WorkbenchVfxSimulateButton",
    "WorkbenchAbilityPlaytestButton",
    "WorkbenchTagsAddButton",
    "WorkbenchTagsRenameButton",
    "WorkbenchPerceptionSimulateButton",
    "WorkbenchRenderCompileButton",
    "WorkbenchHudPreviewButton",
];

const EFFECT_PANEL_TAB_CONTROLS: &[&str] = &[
    "WorkbenchEffectRulesTab",
    "WorkbenchEffectTagsTab",
    "WorkbenchEffectSimulationTab",
];
const MATERIAL_PANEL_TAB_CONTROLS: &[&str] = &[
    "WorkbenchMaterialGraphTab",
    "WorkbenchMaterialPreviewTab",
    "WorkbenchMaterialShaderTab",
];
const BEHAVIOR_PANEL_TAB_CONTROLS: &[&str] = &[
    "WorkbenchBehaviorTreeTab",
    "WorkbenchBehaviorBlackboardTab",
    "WorkbenchBehaviorDebugTab",
];
const ASSETS_PANEL_TAB_CONTROLS: &[&str] = &[
    "WorkbenchAssetsBrowserTab",
    "WorkbenchAssetsImportTab",
    "WorkbenchAssetsValidationTab",
];
const VFX_PANEL_TAB_CONTROLS: &[&str] = &[
    "WorkbenchVfxEmittersTab",
    "WorkbenchVfxTimelineTab",
    "WorkbenchVfxCompileTab",
];
const ABILITY_PANEL_TAB_CONTROLS: &[&str] = &[
    "WorkbenchAbilityGraphTab",
    "WorkbenchAbilityTasksTab",
    "WorkbenchAbilityDebugTab",
];
const TAGS_PANEL_TAB_CONTROLS: &[&str] = &[
    "WorkbenchTagsRegistryTab",
    "WorkbenchTagsValidationTab",
    "WorkbenchTagsSourcesTab",
];
const PERCEPTION_PANEL_TAB_CONTROLS: &[&str] = &[
    "WorkbenchPerceptionSightTab",
    "WorkbenchPerceptionHearingTab",
    "WorkbenchPerceptionDebugTab",
];
const RENDER_PANEL_TAB_CONTROLS: &[&str] = &[
    "WorkbenchRenderGraphTab",
    "WorkbenchRenderResourcesTab",
    "WorkbenchRenderWarningsTab",
];
const HUD_PANEL_TAB_CONTROLS: &[&str] = &[
    "WorkbenchHudCanvasTab",
    "WorkbenchHudBindingsTab",
    "WorkbenchHudValidationTab",
];

const EFFECT_PANEL_ROW_CONTROLS: &[&str] = &[
    "WorkbenchEffectDurationRow",
    "WorkbenchEffectPeriodRow",
    "WorkbenchEffectStackRow",
    "WorkbenchEffectCueRow",
];
const EFFECT_ASSET_ROW_CONTROLS: &[&str] = &[
    "WorkbenchEffectHealthRegenRow",
    "WorkbenchEffectDamageFireRow",
];
const EFFECT_TABLE_ROW_CONTROLS: &[&str] = &[
    "WorkbenchEffectModifierHealthRow",
    "WorkbenchEffectModifierHealingRow",
    "WorkbenchEffectModifierCapRow",
    "WorkbenchEffectGraphRow",
    "WorkbenchEffectAttributePreviewRow",
    "WorkbenchEffectOutputRow",
];
const MATERIAL_PANEL_ROW_CONTROLS: &[&str] = &[
    "WorkbenchMaterialBaseColorRow",
    "WorkbenchMaterialNormalRow",
    "WorkbenchMaterialNodeRow01",
    "WorkbenchMaterialNodeRow02",
    "WorkbenchMaterialNodeRow03",
    "WorkbenchMaterialOutputRow",
];
const BEHAVIOR_PANEL_ROW_CONTROLS: &[&str] = &[
    "WorkbenchBehaviorSelectorRow",
    "WorkbenchBehaviorAttackRow",
    "WorkbenchBehaviorNodeRow01",
    "WorkbenchBehaviorNodeRow02",
    "WorkbenchBehaviorNodeRow03",
    "WorkbenchBehaviorOutputRow",
];
const ASSETS_PANEL_ROW_CONTROLS: &[&str] = &[
    "WorkbenchAssetsForestRow",
    "WorkbenchAssetsMaterialRow",
    "WorkbenchAssetsTableRow01",
    "WorkbenchAssetsTableRow02",
    "WorkbenchAssetsTableRow03",
    "WorkbenchAssetsOutputRow",
];
const VFX_PANEL_ROW_CONTROLS: &[&str] = &[
    "WorkbenchVfxEmitterRow",
    "WorkbenchVfxCurveRow",
    "WorkbenchVfxSpawnRow",
    "WorkbenchVfxLifetimeRow",
    "WorkbenchVfxMaterialRow",
    "WorkbenchVfxOutputRow",
];
const ABILITY_TASK_ROW_CONTROLS: &[&str] = &[
    "WorkbenchAbilityTaskActivateRow",
    "WorkbenchAbilityTaskCostRow",
    "WorkbenchAbilityAssetRow",
];
const ABILITY_GRAPH_ROW_CONTROLS: &[&str] = &[
    "WorkbenchAbilityPhaseActivateRow",
    "WorkbenchAbilityPhaseCostRow",
    "WorkbenchAbilityGraphRow",
    "WorkbenchAbilityOutputRow",
];
const TAGS_PANEL_ROW_CONTROLS: &[&str] = &[
    "WorkbenchTagsSourceRow",
    "WorkbenchTagsAbilityActivateRow",
    "WorkbenchTagsStateStunnedRow",
    "WorkbenchTagsValidationRow",
];
const PERCEPTION_PANEL_ROW_CONTROLS: &[&str] = &[
    "WorkbenchPerceptionGuardRow",
    "WorkbenchPerceptionSniperRow",
    "WorkbenchPerceptionSightConeRow",
    "WorkbenchPerceptionHearingPulseRow",
    "WorkbenchPerceptionStimulusRow",
    "WorkbenchPerceptionEventRow",
];
const RENDER_PANEL_ROW_CONTROLS: &[&str] = &[
    "WorkbenchRenderLightingPassRow",
    "WorkbenchRenderBloomPassRow",
    "WorkbenchRenderFrameStartRow",
    "WorkbenchRenderLightingNodeRow",
    "WorkbenchRenderSceneColorRow",
    "WorkbenchRenderCaptureRow",
];
const HUD_PANEL_ROW_CONTROLS: &[&str] = &[
    "WorkbenchHudWidgetTextRow",
    "WorkbenchHudWidgetButtonRow",
    "WorkbenchHudMinimapRow",
    "WorkbenchHudAmmoPanelRow",
    "WorkbenchHudBindingRow",
    "WorkbenchHudValidationRow",
];

pub(super) fn is_workbench_module_action(action_id: &str) -> bool {
    workbench_module_tab_control_id(action_id).is_some()
        || workbench_module_command_control_id(action_id).is_some()
        || workbench_module_panel_tab_control_id(action_id).is_some()
        || workbench_module_panel_row_control_id(action_id).is_some()
        || workbench_module_panel_command_control_id(action_id).is_some()
        || workbench_module_panel_field_action(action_id)
}

pub(super) fn workbench_module_tab_control_id(action_id: &str) -> Option<&'static str> {
    match action_id {
        "SelectWorkbenchModuleScene" => Some("WorkbenchModuleScene"),
        "SelectWorkbenchModuleEffect" => Some("WorkbenchModuleEffect"),
        "SelectWorkbenchModuleAbility" => Some("WorkbenchModuleAbility"),
        "SelectWorkbenchModuleTags" => Some("WorkbenchModuleTags"),
        "SelectWorkbenchModulePerception" => Some("WorkbenchModulePerception"),
        "SelectWorkbenchModuleMaterial" => Some("WorkbenchModuleMaterial"),
        "SelectWorkbenchModuleBehavior" => Some("WorkbenchModuleBehavior"),
        "SelectWorkbenchModuleRender" => Some("WorkbenchModuleRender"),
        "SelectWorkbenchModuleAssets" => Some("WorkbenchModuleAssets"),
        "SelectWorkbenchModuleVfx" => Some("WorkbenchModuleVfx"),
        "SelectWorkbenchModuleHud" => Some("WorkbenchModuleHud"),
        _ => None,
    }
}

pub(super) fn workbench_module_command_control_id(action_id: &str) -> Option<&'static str> {
    match action_id {
        "InvokeWorkbenchModuleSave" => Some("WorkbenchModuleSave"),
        "InvokeWorkbenchModuleBrowse" => Some("WorkbenchModuleBrowse"),
        "InvokeWorkbenchModuleCompile" => Some("WorkbenchModuleCompile"),
        "InvokeWorkbenchModuleDiff" => Some("WorkbenchModuleDiff"),
        "InvokeWorkbenchModuleSimulate" => Some("WorkbenchModuleSimulate"),
        _ => None,
    }
}

pub(super) fn workbench_module_workspace_control_id(action_id: &str) -> Option<&'static str> {
    match action_id {
        "SelectWorkbenchModuleEffect" => Some("WorkbenchModuleEffectWorkspace"),
        "SelectWorkbenchModuleAbility" => Some("WorkbenchModuleAbilityWorkspace"),
        "SelectWorkbenchModuleTags" => Some("WorkbenchModuleTagsWorkspace"),
        "SelectWorkbenchModulePerception" => Some("WorkbenchModulePerceptionWorkspace"),
        "SelectWorkbenchModuleMaterial" => Some("WorkbenchModuleMaterialWorkspace"),
        "SelectWorkbenchModuleBehavior" => Some("WorkbenchModuleBehaviorWorkspace"),
        "SelectWorkbenchModuleRender" => Some("WorkbenchModuleRenderWorkspace"),
        "SelectWorkbenchModuleAssets" => Some("WorkbenchModuleAssetsWorkspace"),
        "SelectWorkbenchModuleVfx" => Some("WorkbenchModuleVfxWorkspace"),
        "SelectWorkbenchModuleHud" => Some("WorkbenchModuleHudWorkspace"),
        _ => None,
    }
}

pub(super) fn workbench_module_panel_tab_control_id(action_id: &str) -> Option<&'static str> {
    match action_id {
        "SelectWorkbenchEffectRulesTab" => Some("WorkbenchEffectRulesTab"),
        "SelectWorkbenchEffectTagsTab" => Some("WorkbenchEffectTagsTab"),
        "SelectWorkbenchEffectSimulationTab" => Some("WorkbenchEffectSimulationTab"),
        "SelectWorkbenchMaterialGraphTab" => Some("WorkbenchMaterialGraphTab"),
        "SelectWorkbenchMaterialPreviewTab" => Some("WorkbenchMaterialPreviewTab"),
        "SelectWorkbenchMaterialShaderTab" => Some("WorkbenchMaterialShaderTab"),
        "SelectWorkbenchBehaviorTreeTab" => Some("WorkbenchBehaviorTreeTab"),
        "SelectWorkbenchBehaviorBlackboardTab" => Some("WorkbenchBehaviorBlackboardTab"),
        "SelectWorkbenchBehaviorDebugTab" => Some("WorkbenchBehaviorDebugTab"),
        "SelectWorkbenchAssetsBrowserTab" => Some("WorkbenchAssetsBrowserTab"),
        "SelectWorkbenchAssetsImportTab" => Some("WorkbenchAssetsImportTab"),
        "SelectWorkbenchAssetsValidationTab" => Some("WorkbenchAssetsValidationTab"),
        "SelectWorkbenchVfxEmittersTab" => Some("WorkbenchVfxEmittersTab"),
        "SelectWorkbenchVfxTimelineTab" => Some("WorkbenchVfxTimelineTab"),
        "SelectWorkbenchVfxCompileTab" => Some("WorkbenchVfxCompileTab"),
        "SelectWorkbenchAbilityGraphTab" => Some("WorkbenchAbilityGraphTab"),
        "SelectWorkbenchAbilityTasksTab" => Some("WorkbenchAbilityTasksTab"),
        "SelectWorkbenchAbilityDebugTab" => Some("WorkbenchAbilityDebugTab"),
        "SelectWorkbenchTagsRegistryTab" => Some("WorkbenchTagsRegistryTab"),
        "SelectWorkbenchTagsValidationTab" => Some("WorkbenchTagsValidationTab"),
        "SelectWorkbenchTagsSourcesTab" => Some("WorkbenchTagsSourcesTab"),
        "SelectWorkbenchPerceptionSightTab" => Some("WorkbenchPerceptionSightTab"),
        "SelectWorkbenchPerceptionHearingTab" => Some("WorkbenchPerceptionHearingTab"),
        "SelectWorkbenchPerceptionDebugTab" => Some("WorkbenchPerceptionDebugTab"),
        "SelectWorkbenchRenderGraphTab" => Some("WorkbenchRenderGraphTab"),
        "SelectWorkbenchRenderResourcesTab" => Some("WorkbenchRenderResourcesTab"),
        "SelectWorkbenchRenderWarningsTab" => Some("WorkbenchRenderWarningsTab"),
        "SelectWorkbenchHudCanvasTab" => Some("WorkbenchHudCanvasTab"),
        "SelectWorkbenchHudBindingsTab" => Some("WorkbenchHudBindingsTab"),
        "SelectWorkbenchHudValidationTab" => Some("WorkbenchHudValidationTab"),
        _ => None,
    }
}

pub(super) fn workbench_module_panel_tab_group(action_id: &str) -> &'static [&'static str] {
    match action_id {
        "SelectWorkbenchEffectRulesTab"
        | "SelectWorkbenchEffectTagsTab"
        | "SelectWorkbenchEffectSimulationTab" => EFFECT_PANEL_TAB_CONTROLS,
        "SelectWorkbenchMaterialGraphTab"
        | "SelectWorkbenchMaterialPreviewTab"
        | "SelectWorkbenchMaterialShaderTab" => MATERIAL_PANEL_TAB_CONTROLS,
        "SelectWorkbenchBehaviorTreeTab"
        | "SelectWorkbenchBehaviorBlackboardTab"
        | "SelectWorkbenchBehaviorDebugTab" => BEHAVIOR_PANEL_TAB_CONTROLS,
        "SelectWorkbenchAssetsBrowserTab"
        | "SelectWorkbenchAssetsImportTab"
        | "SelectWorkbenchAssetsValidationTab" => ASSETS_PANEL_TAB_CONTROLS,
        "SelectWorkbenchVfxEmittersTab"
        | "SelectWorkbenchVfxTimelineTab"
        | "SelectWorkbenchVfxCompileTab" => VFX_PANEL_TAB_CONTROLS,
        "SelectWorkbenchAbilityGraphTab"
        | "SelectWorkbenchAbilityTasksTab"
        | "SelectWorkbenchAbilityDebugTab" => ABILITY_PANEL_TAB_CONTROLS,
        "SelectWorkbenchTagsRegistryTab"
        | "SelectWorkbenchTagsValidationTab"
        | "SelectWorkbenchTagsSourcesTab" => TAGS_PANEL_TAB_CONTROLS,
        "SelectWorkbenchPerceptionSightTab"
        | "SelectWorkbenchPerceptionHearingTab"
        | "SelectWorkbenchPerceptionDebugTab" => PERCEPTION_PANEL_TAB_CONTROLS,
        "SelectWorkbenchRenderGraphTab"
        | "SelectWorkbenchRenderResourcesTab"
        | "SelectWorkbenchRenderWarningsTab" => RENDER_PANEL_TAB_CONTROLS,
        "SelectWorkbenchHudCanvasTab"
        | "SelectWorkbenchHudBindingsTab"
        | "SelectWorkbenchHudValidationTab" => HUD_PANEL_TAB_CONTROLS,
        _ => &[],
    }
}

pub(super) fn workbench_module_panel_row_control_id(action_id: &str) -> Option<&'static str> {
    match action_id {
        "SelectWorkbenchEffectDurationRow" => Some("WorkbenchEffectDurationRow"),
        "SelectWorkbenchEffectPeriodRow" => Some("WorkbenchEffectPeriodRow"),
        "SelectWorkbenchEffectHealthRegenRow" => Some("WorkbenchEffectHealthRegenRow"),
        "SelectWorkbenchEffectDamageFireRow" => Some("WorkbenchEffectDamageFireRow"),
        "SelectWorkbenchEffectStackRow" => Some("WorkbenchEffectStackRow"),
        "SelectWorkbenchEffectCueRow" => Some("WorkbenchEffectCueRow"),
        "SelectWorkbenchEffectModifierHealth" => Some("WorkbenchEffectModifierHealthRow"),
        "SelectWorkbenchEffectModifierHealing" => Some("WorkbenchEffectModifierHealingRow"),
        "SelectWorkbenchEffectModifierCap" => Some("WorkbenchEffectModifierCapRow"),
        "SelectWorkbenchEffectGraphSelect" => Some("WorkbenchEffectGraphRow"),
        "SelectWorkbenchEffectAttributePreview" => Some("WorkbenchEffectAttributePreviewRow"),
        "SelectWorkbenchEffectOutput" => Some("WorkbenchEffectOutputRow"),
        "SelectWorkbenchMaterialBaseColorRow" => Some("WorkbenchMaterialBaseColorRow"),
        "SelectWorkbenchMaterialNormalRow" => Some("WorkbenchMaterialNormalRow"),
        "SelectWorkbenchMaterialNodeAlbedo" => Some("WorkbenchMaterialNodeRow01"),
        "SelectWorkbenchMaterialNodeRoughness" => Some("WorkbenchMaterialNodeRow02"),
        "SelectWorkbenchMaterialNodeNormal" => Some("WorkbenchMaterialNodeRow03"),
        "SelectWorkbenchMaterialOutput" => Some("WorkbenchMaterialOutputRow"),
        "SelectWorkbenchBehaviorSelectorRow" => Some("WorkbenchBehaviorSelectorRow"),
        "SelectWorkbenchBehaviorAttackRow" => Some("WorkbenchBehaviorAttackRow"),
        "SelectWorkbenchBehaviorNodeSelector" => Some("WorkbenchBehaviorNodeRow01"),
        "SelectWorkbenchBehaviorNodeAttack" => Some("WorkbenchBehaviorNodeRow02"),
        "SelectWorkbenchBehaviorNodeCooldown" => Some("WorkbenchBehaviorNodeRow03"),
        "SelectWorkbenchBehaviorOutput" => Some("WorkbenchBehaviorOutputRow"),
        "SelectWorkbenchAssetsForestRow" => Some("WorkbenchAssetsForestRow"),
        "SelectWorkbenchAssetsMaterialRow" => Some("WorkbenchAssetsMaterialRow"),
        "SelectWorkbenchAssetsTableTree" => Some("WorkbenchAssetsTableRow01"),
        "SelectWorkbenchAssetsTableMaterial" => Some("WorkbenchAssetsTableRow02"),
        "SelectWorkbenchAssetsTableTexture" => Some("WorkbenchAssetsTableRow03"),
        "SelectWorkbenchAssetsOutput" => Some("WorkbenchAssetsOutputRow"),
        "SelectWorkbenchVfxEmitterRow" => Some("WorkbenchVfxEmitterRow"),
        "SelectWorkbenchVfxCurveRow" => Some("WorkbenchVfxCurveRow"),
        "SelectWorkbenchVfxSpawnRow" => Some("WorkbenchVfxSpawnRow"),
        "SelectWorkbenchVfxLifetimeRow" => Some("WorkbenchVfxLifetimeRow"),
        "SelectWorkbenchVfxMaterialRow" => Some("WorkbenchVfxMaterialRow"),
        "SelectWorkbenchVfxOutput" => Some("WorkbenchVfxOutputRow"),
        "SelectWorkbenchAbilityTaskActivate" => Some("WorkbenchAbilityTaskActivateRow"),
        "SelectWorkbenchAbilityTaskCost" => Some("WorkbenchAbilityTaskCostRow"),
        "SelectWorkbenchAbilityAssetDash" => Some("WorkbenchAbilityAssetRow"),
        "SelectWorkbenchAbilityPhaseActivate" => Some("WorkbenchAbilityPhaseActivateRow"),
        "SelectWorkbenchAbilityPhaseCost" => Some("WorkbenchAbilityPhaseCostRow"),
        "SelectWorkbenchAbilityGraphSelect" => Some("WorkbenchAbilityGraphRow"),
        "SelectWorkbenchAbilityOutput" => Some("WorkbenchAbilityOutputRow"),
        "SelectWorkbenchTagsSourceDefault" => Some("WorkbenchTagsSourceRow"),
        "SelectWorkbenchTagsAbilityActivate" => Some("WorkbenchTagsAbilityActivateRow"),
        "SelectWorkbenchTagsStateStunned" => Some("WorkbenchTagsStateStunnedRow"),
        "SelectWorkbenchTagsValidationSelect" => Some("WorkbenchTagsValidationRow"),
        "SelectWorkbenchPerceptionGuard" => Some("WorkbenchPerceptionGuardRow"),
        "SelectWorkbenchPerceptionSniper" => Some("WorkbenchPerceptionSniperRow"),
        "SelectWorkbenchPerceptionSightCone" => Some("WorkbenchPerceptionSightConeRow"),
        "SelectWorkbenchPerceptionHearingPulse" => Some("WorkbenchPerceptionHearingPulseRow"),
        "SelectWorkbenchPerceptionStimulus" => Some("WorkbenchPerceptionStimulusRow"),
        "SelectWorkbenchPerceptionEvent" => Some("WorkbenchPerceptionEventRow"),
        "SelectWorkbenchRenderLightingPass" => Some("WorkbenchRenderLightingPassRow"),
        "SelectWorkbenchRenderBloomPass" => Some("WorkbenchRenderBloomPassRow"),
        "SelectWorkbenchRenderFrameStart" => Some("WorkbenchRenderFrameStartRow"),
        "SelectWorkbenchRenderLightingNode" => Some("WorkbenchRenderLightingNodeRow"),
        "SelectWorkbenchRenderSceneColor" => Some("WorkbenchRenderSceneColorRow"),
        "SelectWorkbenchRenderCapture" => Some("WorkbenchRenderCaptureRow"),
        "SelectWorkbenchHudWidgetText" => Some("WorkbenchHudWidgetTextRow"),
        "SelectWorkbenchHudWidgetButton" => Some("WorkbenchHudWidgetButtonRow"),
        "SelectWorkbenchHudMinimap" => Some("WorkbenchHudMinimapRow"),
        "SelectWorkbenchHudAmmoPanel" => Some("WorkbenchHudAmmoPanelRow"),
        "SelectWorkbenchHudBindingAmmo" => Some("WorkbenchHudBindingRow"),
        "SelectWorkbenchHudValidationRow" => Some("WorkbenchHudValidationRow"),
        _ => None,
    }
}

pub(super) fn workbench_module_panel_row_group(action_id: &str) -> &'static [&'static str] {
    match action_id {
        "SelectWorkbenchEffectHealthRegenRow" | "SelectWorkbenchEffectDamageFireRow" => {
            EFFECT_ASSET_ROW_CONTROLS
        }
        "SelectWorkbenchEffectDurationRow"
        | "SelectWorkbenchEffectPeriodRow"
        | "SelectWorkbenchEffectStackRow"
        | "SelectWorkbenchEffectCueRow" => EFFECT_PANEL_ROW_CONTROLS,
        "SelectWorkbenchEffectModifierHealth"
        | "SelectWorkbenchEffectModifierHealing"
        | "SelectWorkbenchEffectModifierCap"
        | "SelectWorkbenchEffectGraphSelect"
        | "SelectWorkbenchEffectAttributePreview"
        | "SelectWorkbenchEffectOutput" => EFFECT_TABLE_ROW_CONTROLS,
        "SelectWorkbenchMaterialBaseColorRow"
        | "SelectWorkbenchMaterialNormalRow"
        | "SelectWorkbenchMaterialNodeAlbedo"
        | "SelectWorkbenchMaterialNodeRoughness"
        | "SelectWorkbenchMaterialNodeNormal"
        | "SelectWorkbenchMaterialOutput" => MATERIAL_PANEL_ROW_CONTROLS,
        "SelectWorkbenchBehaviorSelectorRow"
        | "SelectWorkbenchBehaviorAttackRow"
        | "SelectWorkbenchBehaviorNodeSelector"
        | "SelectWorkbenchBehaviorNodeAttack"
        | "SelectWorkbenchBehaviorNodeCooldown"
        | "SelectWorkbenchBehaviorOutput" => BEHAVIOR_PANEL_ROW_CONTROLS,
        "SelectWorkbenchAssetsForestRow"
        | "SelectWorkbenchAssetsMaterialRow"
        | "SelectWorkbenchAssetsTableTree"
        | "SelectWorkbenchAssetsTableMaterial"
        | "SelectWorkbenchAssetsTableTexture"
        | "SelectWorkbenchAssetsOutput" => ASSETS_PANEL_ROW_CONTROLS,
        "SelectWorkbenchVfxEmitterRow"
        | "SelectWorkbenchVfxCurveRow"
        | "SelectWorkbenchVfxSpawnRow"
        | "SelectWorkbenchVfxLifetimeRow"
        | "SelectWorkbenchVfxMaterialRow"
        | "SelectWorkbenchVfxOutput" => VFX_PANEL_ROW_CONTROLS,
        "SelectWorkbenchAbilityTaskActivate"
        | "SelectWorkbenchAbilityTaskCost"
        | "SelectWorkbenchAbilityAssetDash" => ABILITY_TASK_ROW_CONTROLS,
        "SelectWorkbenchAbilityPhaseActivate"
        | "SelectWorkbenchAbilityPhaseCost"
        | "SelectWorkbenchAbilityGraphSelect"
        | "SelectWorkbenchAbilityOutput" => ABILITY_GRAPH_ROW_CONTROLS,
        "SelectWorkbenchTagsSourceDefault"
        | "SelectWorkbenchTagsAbilityActivate"
        | "SelectWorkbenchTagsStateStunned"
        | "SelectWorkbenchTagsValidationSelect" => TAGS_PANEL_ROW_CONTROLS,
        "SelectWorkbenchPerceptionGuard"
        | "SelectWorkbenchPerceptionSniper"
        | "SelectWorkbenchPerceptionSightCone"
        | "SelectWorkbenchPerceptionHearingPulse"
        | "SelectWorkbenchPerceptionStimulus"
        | "SelectWorkbenchPerceptionEvent" => PERCEPTION_PANEL_ROW_CONTROLS,
        "SelectWorkbenchRenderLightingPass"
        | "SelectWorkbenchRenderBloomPass"
        | "SelectWorkbenchRenderFrameStart"
        | "SelectWorkbenchRenderLightingNode"
        | "SelectWorkbenchRenderSceneColor"
        | "SelectWorkbenchRenderCapture" => RENDER_PANEL_ROW_CONTROLS,
        "SelectWorkbenchHudWidgetText"
        | "SelectWorkbenchHudWidgetButton"
        | "SelectWorkbenchHudMinimap"
        | "SelectWorkbenchHudAmmoPanel"
        | "SelectWorkbenchHudBindingAmmo"
        | "SelectWorkbenchHudValidationRow" => HUD_PANEL_ROW_CONTROLS,
        _ => &[],
    }
}

pub(super) fn workbench_module_panel_command_control_id(action_id: &str) -> Option<&'static str> {
    match action_id {
        "InvokeWorkbenchEffectApply" => Some("WorkbenchEffectApplyButton"),
        "InvokeWorkbenchMaterialCompile" => Some("WorkbenchMaterialCompileButton"),
        "InvokeWorkbenchBehaviorValidate" => Some("WorkbenchBehaviorValidateButton"),
        "InvokeWorkbenchAssetsImport" => Some("WorkbenchAssetsImportButton"),
        "InvokeWorkbenchVfxSimulate" => Some("WorkbenchVfxSimulateButton"),
        "InvokeWorkbenchAbilityPlaytest" => Some("WorkbenchAbilityPlaytestButton"),
        "InvokeWorkbenchTagsAdd" => Some("WorkbenchTagsAddButton"),
        "InvokeWorkbenchTagsRename" => Some("WorkbenchTagsRenameButton"),
        "InvokeWorkbenchPerceptionSimulate" => Some("WorkbenchPerceptionSimulateButton"),
        "InvokeWorkbenchRenderCompile" => Some("WorkbenchRenderCompileButton"),
        "InvokeWorkbenchHudPreview" => Some("WorkbenchHudPreviewButton"),
        _ => None,
    }
}

fn workbench_module_panel_field_action(action_id: &str) -> bool {
    matches!(
        action_id,
        "EditWorkbenchEffectSearch"
            | "CommitWorkbenchEffectSearch"
            | "EditWorkbenchEffectPolicy"
            | "CommitWorkbenchEffectPolicy"
            | "EditWorkbenchEffectMagnitude"
            | "CommitWorkbenchEffectMagnitude"
            | "EditWorkbenchEffectName"
            | "CommitWorkbenchEffectName"
            | "EditWorkbenchEffectTag"
            | "CommitWorkbenchEffectTag"
            | "EditWorkbenchEffectStack"
            | "CommitWorkbenchEffectStack"
            | "EditWorkbenchMaterialDomain"
            | "CommitWorkbenchMaterialDomain"
            | "EditWorkbenchMaterialBlend"
            | "CommitWorkbenchMaterialBlend"
            | "EditWorkbenchMaterialPreview"
            | "CommitWorkbenchMaterialPreview"
            | "EditWorkbenchBehaviorBlackboard"
            | "CommitWorkbenchBehaviorBlackboard"
            | "EditWorkbenchBehaviorAi"
            | "CommitWorkbenchBehaviorAi"
            | "EditWorkbenchBehaviorState"
            | "CommitWorkbenchBehaviorState"
            | "EditWorkbenchAssetsType"
            | "CommitWorkbenchAssetsType"
            | "EditWorkbenchAssetsPath"
            | "CommitWorkbenchAssetsPath"
            | "EditWorkbenchAssetsOwner"
            | "CommitWorkbenchAssetsOwner"
            | "EditWorkbenchVfxSystem"
            | "CommitWorkbenchVfxSystem"
            | "EditWorkbenchVfxBounds"
            | "CommitWorkbenchVfxBounds"
            | "EditWorkbenchVfxSort"
            | "CommitWorkbenchVfxSort"
            | "EditWorkbenchAbilityName"
            | "CommitWorkbenchAbilityName"
            | "EditWorkbenchAbilityNetPolicy"
            | "CommitWorkbenchAbilityNetPolicy"
            | "EditWorkbenchAbilityCooldown"
            | "CommitWorkbenchAbilityCooldown"
            | "EditWorkbenchTagsSearch"
            | "CommitWorkbenchTagsSearch"
            | "EditWorkbenchTagsRedirect"
            | "CommitWorkbenchTagsRedirect"
            | "EditWorkbenchTagsOwner"
            | "CommitWorkbenchTagsOwner"
            | "EditWorkbenchPerceptionConfig"
            | "CommitWorkbenchPerceptionConfig"
            | "EditWorkbenchPerceptionLos"
            | "CommitWorkbenchPerceptionLos"
            | "EditWorkbenchPerceptionTeam"
            | "CommitWorkbenchPerceptionTeam"
            | "EditWorkbenchRenderPipeline"
            | "CommitWorkbenchRenderPipeline"
            | "EditWorkbenchRenderPlatform"
            | "CommitWorkbenchRenderPlatform"
            | "EditWorkbenchRenderFrame"
            | "CommitWorkbenchRenderFrame"
            | "EditWorkbenchHudScreen"
            | "CommitWorkbenchHudScreen"
            | "EditWorkbenchHudDpi"
            | "CommitWorkbenchHudDpi"
            | "EditWorkbenchHudLocale"
            | "CommitWorkbenchHudLocale"
    )
}
