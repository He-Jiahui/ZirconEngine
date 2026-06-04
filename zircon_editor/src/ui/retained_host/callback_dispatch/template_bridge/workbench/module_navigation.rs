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
        "workbench.module.scene.select" => Some("WorkbenchModuleScene"),
        "workbench.module.effect.select" => Some("WorkbenchModuleEffect"),
        "workbench.module.ability.select" => Some("WorkbenchModuleAbility"),
        "workbench.module.tags.select" => Some("WorkbenchModuleTags"),
        "workbench.module.perception.select" => Some("WorkbenchModulePerception"),
        "workbench.module.material.select" => Some("WorkbenchModuleMaterial"),
        "workbench.module.behavior.select" => Some("WorkbenchModuleBehavior"),
        "workbench.module.render.select" => Some("WorkbenchModuleRender"),
        "workbench.module.assets.select" => Some("WorkbenchModuleAssets"),
        "workbench.module.vfx.select" => Some("WorkbenchModuleVfx"),
        "workbench.module.hud.select" => Some("WorkbenchModuleHud"),
        _ => None,
    }
}

pub(super) fn workbench_module_command_control_id(action_id: &str) -> Option<&'static str> {
    match action_id {
        "workbench.module.save.invoke" => Some("WorkbenchModuleSave"),
        "workbench.module.browse.invoke" => Some("WorkbenchModuleBrowse"),
        "workbench.module.compile.invoke" => Some("WorkbenchModuleCompile"),
        "workbench.module.diff.invoke" => Some("WorkbenchModuleDiff"),
        "workbench.module.simulate.invoke" => Some("WorkbenchModuleSimulate"),
        _ => None,
    }
}

pub(super) fn workbench_module_workspace_control_id(action_id: &str) -> Option<&'static str> {
    match action_id {
        "workbench.module.effect.select" => Some("WorkbenchModuleEffectWorkspace"),
        "workbench.module.ability.select" => Some("WorkbenchModuleAbilityWorkspace"),
        "workbench.module.tags.select" => Some("WorkbenchModuleTagsWorkspace"),
        "workbench.module.perception.select" => Some("WorkbenchModulePerceptionWorkspace"),
        "workbench.module.material.select" => Some("WorkbenchModuleMaterialWorkspace"),
        "workbench.module.behavior.select" => Some("WorkbenchModuleBehaviorWorkspace"),
        "workbench.module.render.select" => Some("WorkbenchModuleRenderWorkspace"),
        "workbench.module.assets.select" => Some("WorkbenchModuleAssetsWorkspace"),
        "workbench.module.vfx.select" => Some("WorkbenchModuleVfxWorkspace"),
        "workbench.module.hud.select" => Some("WorkbenchModuleHudWorkspace"),
        _ => None,
    }
}

pub(super) fn workbench_module_panel_tab_control_id(action_id: &str) -> Option<&'static str> {
    match action_id {
        "workbench.module.effect.rules_tab.select" => Some("WorkbenchEffectRulesTab"),
        "workbench.module.effect.tags_tab.select" => Some("WorkbenchEffectTagsTab"),
        "workbench.module.effect.simulation_tab.select" => Some("WorkbenchEffectSimulationTab"),
        "workbench.module.material.graph_tab.select" => Some("WorkbenchMaterialGraphTab"),
        "workbench.module.material.preview_tab.select" => Some("WorkbenchMaterialPreviewTab"),
        "workbench.module.material.shader_tab.select" => Some("WorkbenchMaterialShaderTab"),
        "workbench.module.behavior.tree_tab.select" => Some("WorkbenchBehaviorTreeTab"),
        "workbench.module.behavior.blackboard_tab.select" => Some("WorkbenchBehaviorBlackboardTab"),
        "workbench.module.behavior.debug_tab.select" => Some("WorkbenchBehaviorDebugTab"),
        "workbench.module.assets.browser_tab.select" => Some("WorkbenchAssetsBrowserTab"),
        "workbench.module.assets.import_tab.select" => Some("WorkbenchAssetsImportTab"),
        "workbench.module.assets.validation_tab.select" => Some("WorkbenchAssetsValidationTab"),
        "workbench.module.vfx.emitters_tab.select" => Some("WorkbenchVfxEmittersTab"),
        "workbench.module.vfx.timeline_tab.select" => Some("WorkbenchVfxTimelineTab"),
        "workbench.module.vfx.compile_tab.select" => Some("WorkbenchVfxCompileTab"),
        "workbench.module.ability.graph_tab.select" => Some("WorkbenchAbilityGraphTab"),
        "workbench.module.ability.tasks_tab.select" => Some("WorkbenchAbilityTasksTab"),
        "workbench.module.ability.debug_tab.select" => Some("WorkbenchAbilityDebugTab"),
        "workbench.module.tags.registry_tab.select" => Some("WorkbenchTagsRegistryTab"),
        "workbench.module.tags.validation_tab.select" => Some("WorkbenchTagsValidationTab"),
        "workbench.module.tags.sources_tab.select" => Some("WorkbenchTagsSourcesTab"),
        "workbench.module.perception.sight_tab.select" => Some("WorkbenchPerceptionSightTab"),
        "workbench.module.perception.hearing_tab.select" => Some("WorkbenchPerceptionHearingTab"),
        "workbench.module.perception.debug_tab.select" => Some("WorkbenchPerceptionDebugTab"),
        "workbench.module.render.graph_tab.select" => Some("WorkbenchRenderGraphTab"),
        "workbench.module.render.resources_tab.select" => Some("WorkbenchRenderResourcesTab"),
        "workbench.module.render.warnings_tab.select" => Some("WorkbenchRenderWarningsTab"),
        "workbench.module.hud.canvas_tab.select" => Some("WorkbenchHudCanvasTab"),
        "workbench.module.hud.bindings_tab.select" => Some("WorkbenchHudBindingsTab"),
        "workbench.module.hud.validation_tab.select" => Some("WorkbenchHudValidationTab"),
        _ => None,
    }
}

pub(super) fn workbench_module_panel_tab_group(action_id: &str) -> &'static [&'static str] {
    match action_id {
        "workbench.module.effect.rules_tab.select"
        | "workbench.module.effect.tags_tab.select"
        | "workbench.module.effect.simulation_tab.select" => EFFECT_PANEL_TAB_CONTROLS,
        "workbench.module.material.graph_tab.select"
        | "workbench.module.material.preview_tab.select"
        | "workbench.module.material.shader_tab.select" => MATERIAL_PANEL_TAB_CONTROLS,
        "workbench.module.behavior.tree_tab.select"
        | "workbench.module.behavior.blackboard_tab.select"
        | "workbench.module.behavior.debug_tab.select" => BEHAVIOR_PANEL_TAB_CONTROLS,
        "workbench.module.assets.browser_tab.select"
        | "workbench.module.assets.import_tab.select"
        | "workbench.module.assets.validation_tab.select" => ASSETS_PANEL_TAB_CONTROLS,
        "workbench.module.vfx.emitters_tab.select"
        | "workbench.module.vfx.timeline_tab.select"
        | "workbench.module.vfx.compile_tab.select" => VFX_PANEL_TAB_CONTROLS,
        "workbench.module.ability.graph_tab.select"
        | "workbench.module.ability.tasks_tab.select"
        | "workbench.module.ability.debug_tab.select" => ABILITY_PANEL_TAB_CONTROLS,
        "workbench.module.tags.registry_tab.select"
        | "workbench.module.tags.validation_tab.select"
        | "workbench.module.tags.sources_tab.select" => TAGS_PANEL_TAB_CONTROLS,
        "workbench.module.perception.sight_tab.select"
        | "workbench.module.perception.hearing_tab.select"
        | "workbench.module.perception.debug_tab.select" => PERCEPTION_PANEL_TAB_CONTROLS,
        "workbench.module.render.graph_tab.select"
        | "workbench.module.render.resources_tab.select"
        | "workbench.module.render.warnings_tab.select" => RENDER_PANEL_TAB_CONTROLS,
        "workbench.module.hud.canvas_tab.select"
        | "workbench.module.hud.bindings_tab.select"
        | "workbench.module.hud.validation_tab.select" => HUD_PANEL_TAB_CONTROLS,
        _ => &[],
    }
}

pub(super) fn workbench_module_panel_row_control_id(action_id: &str) -> Option<&'static str> {
    match action_id {
        "workbench.module.effect.duration_row.select" => Some("WorkbenchEffectDurationRow"),
        "workbench.module.effect.period_row.select" => Some("WorkbenchEffectPeriodRow"),
        "workbench.module.effect.health_regen_row.select" => Some("WorkbenchEffectHealthRegenRow"),
        "workbench.module.effect.damage_fire_row.select" => Some("WorkbenchEffectDamageFireRow"),
        "workbench.module.effect.stack_row.select" => Some("WorkbenchEffectStackRow"),
        "workbench.module.effect.cue_row.select" => Some("WorkbenchEffectCueRow"),
        "workbench.module.effect.modifier_health.select" => {
            Some("WorkbenchEffectModifierHealthRow")
        }
        "workbench.module.effect.modifier_healing.select" => {
            Some("WorkbenchEffectModifierHealingRow")
        }
        "workbench.module.effect.modifier_cap.select" => Some("WorkbenchEffectModifierCapRow"),
        "workbench.module.effect.graph_select.select" => Some("WorkbenchEffectGraphRow"),
        "workbench.module.effect.attribute_preview.select" => {
            Some("WorkbenchEffectAttributePreviewRow")
        }
        "workbench.module.effect.output.select" => Some("WorkbenchEffectOutputRow"),
        "workbench.module.material.base_color_row.select" => Some("WorkbenchMaterialBaseColorRow"),
        "workbench.module.material.normal_row.select" => Some("WorkbenchMaterialNormalRow"),
        "workbench.module.material.node_albedo.select" => Some("WorkbenchMaterialNodeRow01"),
        "workbench.module.material.node_roughness.select" => Some("WorkbenchMaterialNodeRow02"),
        "workbench.module.material.node_normal.select" => Some("WorkbenchMaterialNodeRow03"),
        "workbench.module.material.output.select" => Some("WorkbenchMaterialOutputRow"),
        "workbench.module.behavior.selector_row.select" => Some("WorkbenchBehaviorSelectorRow"),
        "workbench.module.behavior.attack_row.select" => Some("WorkbenchBehaviorAttackRow"),
        "workbench.module.behavior.node_selector.select" => Some("WorkbenchBehaviorNodeRow01"),
        "workbench.module.behavior.node_attack.select" => Some("WorkbenchBehaviorNodeRow02"),
        "workbench.module.behavior.node_cooldown.select" => Some("WorkbenchBehaviorNodeRow03"),
        "workbench.module.behavior.output.select" => Some("WorkbenchBehaviorOutputRow"),
        "workbench.module.assets.forest_row.select" => Some("WorkbenchAssetsForestRow"),
        "workbench.module.assets.material_row.select" => Some("WorkbenchAssetsMaterialRow"),
        "workbench.module.assets.table_tree.select" => Some("WorkbenchAssetsTableRow01"),
        "workbench.module.assets.table_material.select" => Some("WorkbenchAssetsTableRow02"),
        "workbench.module.assets.table_texture.select" => Some("WorkbenchAssetsTableRow03"),
        "workbench.module.assets.output.select" => Some("WorkbenchAssetsOutputRow"),
        "workbench.module.vfx.emitter_row.select" => Some("WorkbenchVfxEmitterRow"),
        "workbench.module.vfx.curve_row.select" => Some("WorkbenchVfxCurveRow"),
        "workbench.module.vfx.spawn_row.select" => Some("WorkbenchVfxSpawnRow"),
        "workbench.module.vfx.lifetime_row.select" => Some("WorkbenchVfxLifetimeRow"),
        "workbench.module.vfx.material_row.select" => Some("WorkbenchVfxMaterialRow"),
        "workbench.module.vfx.output.select" => Some("WorkbenchVfxOutputRow"),
        "workbench.module.ability.task_activate.select" => Some("WorkbenchAbilityTaskActivateRow"),
        "workbench.module.ability.task_cost.select" => Some("WorkbenchAbilityTaskCostRow"),
        "workbench.module.ability.asset_dash.select" => Some("WorkbenchAbilityAssetRow"),
        "workbench.module.ability.phase_activate.select" => {
            Some("WorkbenchAbilityPhaseActivateRow")
        }
        "workbench.module.ability.phase_cost.select" => Some("WorkbenchAbilityPhaseCostRow"),
        "workbench.module.ability.graph_select.select" => Some("WorkbenchAbilityGraphRow"),
        "workbench.module.ability.output.select" => Some("WorkbenchAbilityOutputRow"),
        "workbench.module.tags.source_default.select" => Some("WorkbenchTagsSourceRow"),
        "workbench.module.tags.ability_activate.select" => Some("WorkbenchTagsAbilityActivateRow"),
        "workbench.module.tags.state_stunned.select" => Some("WorkbenchTagsStateStunnedRow"),
        "workbench.module.tags.validation_select.select" => Some("WorkbenchTagsValidationRow"),
        "workbench.module.perception.guard.select" => Some("WorkbenchPerceptionGuardRow"),
        "workbench.module.perception.sniper.select" => Some("WorkbenchPerceptionSniperRow"),
        "workbench.module.perception.sight_cone.select" => Some("WorkbenchPerceptionSightConeRow"),
        "workbench.module.perception.hearing_pulse.select" => {
            Some("WorkbenchPerceptionHearingPulseRow")
        }
        "workbench.module.perception.stimulus.select" => Some("WorkbenchPerceptionStimulusRow"),
        "workbench.module.perception.event.select" => Some("WorkbenchPerceptionEventRow"),
        "workbench.module.render.lighting_pass.select" => Some("WorkbenchRenderLightingPassRow"),
        "workbench.module.render.bloom_pass.select" => Some("WorkbenchRenderBloomPassRow"),
        "workbench.module.render.frame_start.select" => Some("WorkbenchRenderFrameStartRow"),
        "workbench.module.render.lighting_node.select" => Some("WorkbenchRenderLightingNodeRow"),
        "workbench.module.render.scene_color.select" => Some("WorkbenchRenderSceneColorRow"),
        "workbench.module.render.capture.select" => Some("WorkbenchRenderCaptureRow"),
        "workbench.module.hud.widget_text.select" => Some("WorkbenchHudWidgetTextRow"),
        "workbench.module.hud.widget_button.select" => Some("WorkbenchHudWidgetButtonRow"),
        "workbench.module.hud.minimap.select" => Some("WorkbenchHudMinimapRow"),
        "workbench.module.hud.ammo_panel.select" => Some("WorkbenchHudAmmoPanelRow"),
        "workbench.module.hud.binding_ammo.select" => Some("WorkbenchHudBindingRow"),
        "workbench.module.hud.validation_row.select" => Some("WorkbenchHudValidationRow"),
        _ => None,
    }
}

pub(super) fn workbench_module_panel_row_group(action_id: &str) -> &'static [&'static str] {
    match action_id {
        "workbench.module.effect.health_regen_row.select"
        | "workbench.module.effect.damage_fire_row.select" => EFFECT_ASSET_ROW_CONTROLS,
        "workbench.module.effect.duration_row.select"
        | "workbench.module.effect.period_row.select"
        | "workbench.module.effect.stack_row.select"
        | "workbench.module.effect.cue_row.select" => EFFECT_PANEL_ROW_CONTROLS,
        "workbench.module.effect.modifier_health.select"
        | "workbench.module.effect.modifier_healing.select"
        | "workbench.module.effect.modifier_cap.select"
        | "workbench.module.effect.graph_select.select"
        | "workbench.module.effect.attribute_preview.select"
        | "workbench.module.effect.output.select" => EFFECT_TABLE_ROW_CONTROLS,
        "workbench.module.material.base_color_row.select"
        | "workbench.module.material.normal_row.select"
        | "workbench.module.material.node_albedo.select"
        | "workbench.module.material.node_roughness.select"
        | "workbench.module.material.node_normal.select"
        | "workbench.module.material.output.select" => MATERIAL_PANEL_ROW_CONTROLS,
        "workbench.module.behavior.selector_row.select"
        | "workbench.module.behavior.attack_row.select"
        | "workbench.module.behavior.node_selector.select"
        | "workbench.module.behavior.node_attack.select"
        | "workbench.module.behavior.node_cooldown.select"
        | "workbench.module.behavior.output.select" => BEHAVIOR_PANEL_ROW_CONTROLS,
        "workbench.module.assets.forest_row.select"
        | "workbench.module.assets.material_row.select"
        | "workbench.module.assets.table_tree.select"
        | "workbench.module.assets.table_material.select"
        | "workbench.module.assets.table_texture.select"
        | "workbench.module.assets.output.select" => ASSETS_PANEL_ROW_CONTROLS,
        "workbench.module.vfx.emitter_row.select"
        | "workbench.module.vfx.curve_row.select"
        | "workbench.module.vfx.spawn_row.select"
        | "workbench.module.vfx.lifetime_row.select"
        | "workbench.module.vfx.material_row.select"
        | "workbench.module.vfx.output.select" => VFX_PANEL_ROW_CONTROLS,
        "workbench.module.ability.task_activate.select"
        | "workbench.module.ability.task_cost.select"
        | "workbench.module.ability.asset_dash.select" => ABILITY_TASK_ROW_CONTROLS,
        "workbench.module.ability.phase_activate.select"
        | "workbench.module.ability.phase_cost.select"
        | "workbench.module.ability.graph_select.select"
        | "workbench.module.ability.output.select" => ABILITY_GRAPH_ROW_CONTROLS,
        "workbench.module.tags.source_default.select"
        | "workbench.module.tags.ability_activate.select"
        | "workbench.module.tags.state_stunned.select"
        | "workbench.module.tags.validation_select.select" => TAGS_PANEL_ROW_CONTROLS,
        "workbench.module.perception.guard.select"
        | "workbench.module.perception.sniper.select"
        | "workbench.module.perception.sight_cone.select"
        | "workbench.module.perception.hearing_pulse.select"
        | "workbench.module.perception.stimulus.select"
        | "workbench.module.perception.event.select" => PERCEPTION_PANEL_ROW_CONTROLS,
        "workbench.module.render.lighting_pass.select"
        | "workbench.module.render.bloom_pass.select"
        | "workbench.module.render.frame_start.select"
        | "workbench.module.render.lighting_node.select"
        | "workbench.module.render.scene_color.select"
        | "workbench.module.render.capture.select" => RENDER_PANEL_ROW_CONTROLS,
        "workbench.module.hud.widget_text.select"
        | "workbench.module.hud.widget_button.select"
        | "workbench.module.hud.minimap.select"
        | "workbench.module.hud.ammo_panel.select"
        | "workbench.module.hud.binding_ammo.select"
        | "workbench.module.hud.validation_row.select" => HUD_PANEL_ROW_CONTROLS,
        _ => &[],
    }
}

pub(super) fn workbench_module_panel_command_control_id(action_id: &str) -> Option<&'static str> {
    match action_id {
        "workbench.module.effect.apply.invoke" => Some("WorkbenchEffectApplyButton"),
        "workbench.module.material.compile.invoke" => Some("WorkbenchMaterialCompileButton"),
        "workbench.module.behavior.validate.invoke" => Some("WorkbenchBehaviorValidateButton"),
        "workbench.module.assets.import.invoke" => Some("WorkbenchAssetsImportButton"),
        "workbench.module.vfx.simulate.invoke" => Some("WorkbenchVfxSimulateButton"),
        "workbench.module.ability.playtest.invoke" => Some("WorkbenchAbilityPlaytestButton"),
        "workbench.module.tags.add.invoke" => Some("WorkbenchTagsAddButton"),
        "workbench.module.tags.rename.invoke" => Some("WorkbenchTagsRenameButton"),
        "workbench.module.perception.simulate.invoke" => Some("WorkbenchPerceptionSimulateButton"),
        "workbench.module.render.compile.invoke" => Some("WorkbenchRenderCompileButton"),
        "workbench.module.hud.preview.invoke" => Some("WorkbenchHudPreviewButton"),
        _ => None,
    }
}

fn workbench_module_panel_field_action(action_id: &str) -> bool {
    matches!(
        action_id,
        "workbench.module.effect.search.edit"
            | "workbench.module.effect.search.commit"
            | "workbench.module.effect.policy.edit"
            | "workbench.module.effect.policy.commit"
            | "workbench.module.effect.magnitude.edit"
            | "workbench.module.effect.magnitude.commit"
            | "workbench.module.effect.name.edit"
            | "workbench.module.effect.name.commit"
            | "workbench.module.effect.tag.edit"
            | "workbench.module.effect.tag.commit"
            | "workbench.module.effect.stack.edit"
            | "workbench.module.effect.stack.commit"
            | "workbench.module.material.domain.edit"
            | "workbench.module.material.domain.commit"
            | "workbench.module.material.blend.edit"
            | "workbench.module.material.blend.commit"
            | "workbench.module.material.preview.edit"
            | "workbench.module.material.preview.commit"
            | "workbench.module.behavior.blackboard.edit"
            | "workbench.module.behavior.blackboard.commit"
            | "workbench.module.behavior.ai.edit"
            | "workbench.module.behavior.ai.commit"
            | "workbench.module.behavior.state.edit"
            | "workbench.module.behavior.state.commit"
            | "workbench.module.assets.type.edit"
            | "workbench.module.assets.type.commit"
            | "workbench.module.assets.path.edit"
            | "workbench.module.assets.path.commit"
            | "workbench.module.assets.owner.edit"
            | "workbench.module.assets.owner.commit"
            | "workbench.module.vfx.system.edit"
            | "workbench.module.vfx.system.commit"
            | "workbench.module.vfx.bounds.edit"
            | "workbench.module.vfx.bounds.commit"
            | "workbench.module.vfx.sort.edit"
            | "workbench.module.vfx.sort.commit"
            | "workbench.module.ability.name.edit"
            | "workbench.module.ability.name.commit"
            | "workbench.module.ability.net_policy.edit"
            | "workbench.module.ability.net_policy.commit"
            | "workbench.module.ability.cooldown.edit"
            | "workbench.module.ability.cooldown.commit"
            | "workbench.module.tags.search.edit"
            | "workbench.module.tags.search.commit"
            | "workbench.module.tags.redirect.edit"
            | "workbench.module.tags.redirect.commit"
            | "workbench.module.tags.owner.edit"
            | "workbench.module.tags.owner.commit"
            | "workbench.module.perception.config.edit"
            | "workbench.module.perception.config.commit"
            | "workbench.module.perception.los.edit"
            | "workbench.module.perception.los.commit"
            | "workbench.module.perception.team.edit"
            | "workbench.module.perception.team.commit"
            | "workbench.module.render.pipeline.edit"
            | "workbench.module.render.pipeline.commit"
            | "workbench.module.render.platform.edit"
            | "workbench.module.render.platform.commit"
            | "workbench.module.render.frame.edit"
            | "workbench.module.render.frame.commit"
            | "workbench.module.hud.screen.edit"
            | "workbench.module.hud.screen.commit"
            | "workbench.module.hud.dpi.edit"
            | "workbench.module.hud.dpi.commit"
            | "workbench.module.hud.locale.edit"
            | "workbench.module.hud.locale.commit"
    )
}
