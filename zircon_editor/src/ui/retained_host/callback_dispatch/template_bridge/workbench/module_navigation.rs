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
const EFFECT_MODIFIER_ROW_CONTROLS: &[&str] = &[
    "WorkbenchEffectModifierHealthRow",
    "WorkbenchEffectModifierHealingRow",
    "WorkbenchEffectModifierCapRow",
];
const EFFECT_GRAPH_ROW_CONTROLS: &[&str] = &["WorkbenchEffectGraphRow"];
const EFFECT_PREVIEW_ROW_CONTROLS: &[&str] = &["WorkbenchEffectAttributePreviewRow"];
const MATERIAL_PARAMETER_ROW_CONTROLS: &[&str] = &[
    "WorkbenchMaterialBaseColorRow",
    "WorkbenchMaterialRoughnessRow",
    "WorkbenchMaterialNormalRow",
];
const MATERIAL_GRAPH_ROW_CONTROLS: &[&str] = &[
    "WorkbenchMaterialNodeRow01",
    "WorkbenchMaterialNodeRow02",
    "WorkbenchMaterialNodeRow03",
];
const BEHAVIOR_PANEL_ROW_CONTROLS: &[&str] = &[
    "WorkbenchBehaviorSelectorRow",
    "WorkbenchBehaviorAttackRow",
    "WorkbenchBehaviorNodeRow01",
    "WorkbenchBehaviorNodeRow02",
    "WorkbenchBehaviorNodeRow03",
];
const ASSETS_PANEL_ROW_CONTROLS: &[&str] = &[
    "WorkbenchAssetsForestRow",
    "WorkbenchAssetsMaterialRow",
    "WorkbenchAssetsTableRow01",
    "WorkbenchAssetsTableRow02",
    "WorkbenchAssetsTableRow03",
];
const VFX_CONTEXT_ROW_CONTROLS: &[&str] = &["WorkbenchVfxEmitterRow", "WorkbenchVfxCurveRow"];
const VFX_PARAMETER_ROW_CONTROLS: &[&str] = &[
    "WorkbenchVfxSpawnRow",
    "WorkbenchVfxLifetimeRow",
    "WorkbenchVfxMaterialRow",
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
];
const TAGS_SOURCE_ROW_CONTROLS: &[&str] = &["WorkbenchTagsSourceRow"];
const TAGS_TABLE_ROW_CONTROLS: &[&str] = &[
    "WorkbenchTagsAbilityActivateRow",
    "WorkbenchTagsStateStunnedRow",
];
const PERCEPTION_AGENT_ROW_CONTROLS: &[&str] = &[
    "WorkbenchPerceptionGuardRow",
    "WorkbenchPerceptionSniperRow",
];
const PERCEPTION_MAP_ROW_CONTROLS: &[&str] = &[
    "WorkbenchPerceptionSightConeRow",
    "WorkbenchPerceptionHearingPulseRow",
    "WorkbenchPerceptionStimulusRow",
];
const RENDER_PASS_ROW_CONTROLS: &[&str] = &[
    "WorkbenchRenderLightingPassRow",
    "WorkbenchRenderBloomPassRow",
];
const RENDER_GRAPH_ROW_CONTROLS: &[&str] = &[
    "WorkbenchRenderFrameStartRow",
    "WorkbenchRenderLightingNodeRow",
    "WorkbenchRenderSceneColorRow",
];
const HUD_WIDGET_ROW_CONTROLS: &[&str] =
    &["WorkbenchHudWidgetTextRow", "WorkbenchHudWidgetButtonRow"];
const HUD_CANVAS_ROW_CONTROLS: &[&str] = &[
    "WorkbenchHudMinimapRow",
    "WorkbenchHudAmmoPanelRow",
    "WorkbenchHudBindingRow",
];

pub(super) fn is_workbench_module_action(action_id: &str) -> bool {
    workbench_module_tab_control_id(action_id).is_some()
        || workbench_module_command_control_id(action_id).is_some()
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
        "workbench.module.material.base_color_row.select" => Some("WorkbenchMaterialBaseColorRow"),
        "workbench.module.material.roughness_row.select" => Some("WorkbenchMaterialRoughnessRow"),
        "workbench.module.material.normal_row.select" => Some("WorkbenchMaterialNormalRow"),
        "workbench.module.material.node_albedo.select" => Some("WorkbenchMaterialNodeRow01"),
        "workbench.module.material.node_roughness.select" => Some("WorkbenchMaterialNodeRow02"),
        "workbench.module.material.node_normal.select" => Some("WorkbenchMaterialNodeRow03"),
        "workbench.module.behavior.selector_row.select" => Some("WorkbenchBehaviorSelectorRow"),
        "workbench.module.behavior.attack_row.select" => Some("WorkbenchBehaviorAttackRow"),
        "workbench.module.behavior.node_selector.select" => Some("WorkbenchBehaviorNodeRow01"),
        "workbench.module.behavior.node_attack.select" => Some("WorkbenchBehaviorNodeRow02"),
        "workbench.module.behavior.node_cooldown.select" => Some("WorkbenchBehaviorNodeRow03"),
        "workbench.module.assets.forest_row.select" => Some("WorkbenchAssetsForestRow"),
        "workbench.module.assets.material_row.select" => Some("WorkbenchAssetsMaterialRow"),
        "workbench.module.assets.table_tree.select" => Some("WorkbenchAssetsTableRow01"),
        "workbench.module.assets.table_material.select" => Some("WorkbenchAssetsTableRow02"),
        "workbench.module.assets.table_texture.select" => Some("WorkbenchAssetsTableRow03"),
        "workbench.module.vfx.emitter_row.select" => Some("WorkbenchVfxEmitterRow"),
        "workbench.module.vfx.curve_row.select" => Some("WorkbenchVfxCurveRow"),
        "workbench.module.vfx.spawn_row.select" => Some("WorkbenchVfxSpawnRow"),
        "workbench.module.vfx.lifetime_row.select" => Some("WorkbenchVfxLifetimeRow"),
        "workbench.module.vfx.material_row.select" => Some("WorkbenchVfxMaterialRow"),
        "workbench.module.ability.task_activate.select" => Some("WorkbenchAbilityTaskActivateRow"),
        "workbench.module.ability.task_cost.select" => Some("WorkbenchAbilityTaskCostRow"),
        "workbench.module.ability.asset_dash.select" => Some("WorkbenchAbilityAssetRow"),
        "workbench.module.ability.phase_activate.select" => {
            Some("WorkbenchAbilityPhaseActivateRow")
        }
        "workbench.module.ability.phase_cost.select" => Some("WorkbenchAbilityPhaseCostRow"),
        "workbench.module.ability.graph_select.select" => Some("WorkbenchAbilityGraphRow"),
        "workbench.module.tags.source_default.select" => Some("WorkbenchTagsSourceRow"),
        "workbench.module.tags.ability_activate.select" => Some("WorkbenchTagsAbilityActivateRow"),
        "workbench.module.tags.state_stunned.select" => Some("WorkbenchTagsStateStunnedRow"),
        "workbench.module.perception.guard.select" => Some("WorkbenchPerceptionGuardRow"),
        "workbench.module.perception.sniper.select" => Some("WorkbenchPerceptionSniperRow"),
        "workbench.module.perception.sight_cone.select" => Some("WorkbenchPerceptionSightConeRow"),
        "workbench.module.perception.hearing_pulse.select" => {
            Some("WorkbenchPerceptionHearingPulseRow")
        }
        "workbench.module.perception.stimulus.select" => Some("WorkbenchPerceptionStimulusRow"),
        "workbench.module.render.lighting_pass.select" => Some("WorkbenchRenderLightingPassRow"),
        "workbench.module.render.bloom_pass.select" => Some("WorkbenchRenderBloomPassRow"),
        "workbench.module.render.frame_start.select" => Some("WorkbenchRenderFrameStartRow"),
        "workbench.module.render.lighting_node.select" => Some("WorkbenchRenderLightingNodeRow"),
        "workbench.module.render.scene_color.select" => Some("WorkbenchRenderSceneColorRow"),
        "workbench.module.hud.widget_text.select" => Some("WorkbenchHudWidgetTextRow"),
        "workbench.module.hud.widget_button.select" => Some("WorkbenchHudWidgetButtonRow"),
        "workbench.module.hud.minimap.select" => Some("WorkbenchHudMinimapRow"),
        "workbench.module.hud.ammo_panel.select" => Some("WorkbenchHudAmmoPanelRow"),
        "workbench.module.hud.binding_ammo.select" => Some("WorkbenchHudBindingRow"),
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
        | "workbench.module.effect.modifier_cap.select" => EFFECT_MODIFIER_ROW_CONTROLS,
        "workbench.module.effect.graph_select.select" => EFFECT_GRAPH_ROW_CONTROLS,
        "workbench.module.effect.attribute_preview.select" => EFFECT_PREVIEW_ROW_CONTROLS,
        "workbench.module.material.base_color_row.select"
        | "workbench.module.material.roughness_row.select"
        | "workbench.module.material.normal_row.select" => MATERIAL_PARAMETER_ROW_CONTROLS,
        "workbench.module.material.node_albedo.select"
        | "workbench.module.material.node_roughness.select"
        | "workbench.module.material.node_normal.select" => MATERIAL_GRAPH_ROW_CONTROLS,
        "workbench.module.behavior.selector_row.select"
        | "workbench.module.behavior.attack_row.select"
        | "workbench.module.behavior.node_selector.select"
        | "workbench.module.behavior.node_attack.select"
        | "workbench.module.behavior.node_cooldown.select" => BEHAVIOR_PANEL_ROW_CONTROLS,
        "workbench.module.assets.forest_row.select"
        | "workbench.module.assets.material_row.select"
        | "workbench.module.assets.table_tree.select"
        | "workbench.module.assets.table_material.select"
        | "workbench.module.assets.table_texture.select" => ASSETS_PANEL_ROW_CONTROLS,
        "workbench.module.vfx.emitter_row.select" | "workbench.module.vfx.curve_row.select" => {
            VFX_CONTEXT_ROW_CONTROLS
        }
        "workbench.module.vfx.spawn_row.select"
        | "workbench.module.vfx.lifetime_row.select"
        | "workbench.module.vfx.material_row.select" => VFX_PARAMETER_ROW_CONTROLS,
        "workbench.module.ability.task_activate.select"
        | "workbench.module.ability.task_cost.select"
        | "workbench.module.ability.asset_dash.select" => ABILITY_TASK_ROW_CONTROLS,
        "workbench.module.ability.phase_activate.select"
        | "workbench.module.ability.phase_cost.select"
        | "workbench.module.ability.graph_select.select" => ABILITY_GRAPH_ROW_CONTROLS,
        "workbench.module.tags.source_default.select" => TAGS_SOURCE_ROW_CONTROLS,
        "workbench.module.tags.ability_activate.select"
        | "workbench.module.tags.state_stunned.select" => TAGS_TABLE_ROW_CONTROLS,
        "workbench.module.perception.guard.select"
        | "workbench.module.perception.sniper.select" => PERCEPTION_AGENT_ROW_CONTROLS,
        "workbench.module.perception.sight_cone.select"
        | "workbench.module.perception.hearing_pulse.select"
        | "workbench.module.perception.stimulus.select" => PERCEPTION_MAP_ROW_CONTROLS,
        "workbench.module.render.lighting_pass.select"
        | "workbench.module.render.bloom_pass.select" => RENDER_PASS_ROW_CONTROLS,
        "workbench.module.render.frame_start.select"
        | "workbench.module.render.lighting_node.select"
        | "workbench.module.render.scene_color.select" => RENDER_GRAPH_ROW_CONTROLS,
        "workbench.module.hud.widget_text.select" | "workbench.module.hud.widget_button.select" => {
            HUD_WIDGET_ROW_CONTROLS
        }
        "workbench.module.hud.minimap.select"
        | "workbench.module.hud.ammo_panel.select"
        | "workbench.module.hud.binding_ammo.select" => HUD_CANVAS_ROW_CONTROLS,
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
