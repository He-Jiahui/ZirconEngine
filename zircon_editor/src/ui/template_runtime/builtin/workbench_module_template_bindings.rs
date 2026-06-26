use std::collections::BTreeMap;

use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};

pub(super) fn insert_workbench_module_bindings(bindings: &mut BTreeMap<String, EditorUiBinding>) {
    for (control_id, action_id) in [
        ("Scene", "workbench.module.scene.select"),
        ("Effect", "workbench.module.effect.select"),
        ("Ability", "workbench.module.ability.select"),
        ("Tags", "workbench.module.tags.select"),
        ("Perception", "workbench.module.perception.select"),
        ("Material", "workbench.module.material.select"),
        ("Behavior", "workbench.module.behavior.select"),
        ("Render", "workbench.module.render.select"),
        ("Assets", "workbench.module.assets.select"),
        ("Vfx", "workbench.module.vfx.select"),
        ("Hud", "workbench.module.hud.select"),
        ("More", "workbench.module.more.open"),
        ("Save", "workbench.module.save.invoke"),
        ("Browse", "workbench.module.browse.invoke"),
        ("Compile", "workbench.module.compile.invoke"),
        ("Diff", "workbench.module.diff.invoke"),
        ("Simulate", "workbench.module.simulate.invoke"),
        ("EffectRulesTab", "workbench.module.effect.rules_tab.select"),
        ("EffectTagsTab", "workbench.module.effect.tags_tab.select"),
        (
            "EffectSimulationTab",
            "workbench.module.effect.simulation_tab.select",
        ),
        (
            "EffectDurationRow",
            "workbench.module.effect.duration_row.select",
        ),
        (
            "EffectPeriodRow",
            "workbench.module.effect.period_row.select",
        ),
        (
            "EffectHealthRegenRow",
            "workbench.module.effect.health_regen_row.select",
        ),
        (
            "EffectDamageFireRow",
            "workbench.module.effect.damage_fire_row.select",
        ),
        ("EffectStackRow", "workbench.module.effect.stack_row.select"),
        ("EffectCueRow", "workbench.module.effect.cue_row.select"),
        ("EffectOutput", "workbench.module.effect.output.select"),
        ("EffectApply", "workbench.module.effect.apply.invoke"),
        (
            "EffectModifierHealth",
            "workbench.module.effect.modifier_health.select",
        ),
        (
            "EffectModifierHealing",
            "workbench.module.effect.modifier_healing.select",
        ),
        (
            "EffectModifierCap",
            "workbench.module.effect.modifier_cap.select",
        ),
        (
            "EffectGraphSelect",
            "workbench.module.effect.graph_select.select",
        ),
        (
            "EffectAttributePreview",
            "workbench.module.effect.attribute_preview.select",
        ),
        (
            "MaterialGraphTab",
            "workbench.module.material.graph_tab.select",
        ),
        (
            "MaterialPreviewTab",
            "workbench.module.material.preview_tab.select",
        ),
        (
            "MaterialShaderTab",
            "workbench.module.material.shader_tab.select",
        ),
        (
            "MaterialBaseColorRow",
            "workbench.module.material.base_color_row.select",
        ),
        (
            "MaterialNormalRow",
            "workbench.module.material.normal_row.select",
        ),
        (
            "MaterialNodeAlbedo",
            "workbench.module.material.node_albedo.select",
        ),
        (
            "MaterialNodeRoughness",
            "workbench.module.material.node_roughness.select",
        ),
        (
            "MaterialNodeNormal",
            "workbench.module.material.node_normal.select",
        ),
        ("MaterialOutput", "workbench.module.material.output.select"),
        (
            "MaterialCompile",
            "workbench.module.material.compile.invoke",
        ),
        (
            "BehaviorTreeTab",
            "workbench.module.behavior.tree_tab.select",
        ),
        (
            "BehaviorBlackboardTab",
            "workbench.module.behavior.blackboard_tab.select",
        ),
        (
            "BehaviorDebugTab",
            "workbench.module.behavior.debug_tab.select",
        ),
        (
            "BehaviorSelectorRow",
            "workbench.module.behavior.selector_row.select",
        ),
        (
            "BehaviorAttackRow",
            "workbench.module.behavior.attack_row.select",
        ),
        (
            "BehaviorNodeSelector",
            "workbench.module.behavior.node_selector.select",
        ),
        (
            "BehaviorNodeAttack",
            "workbench.module.behavior.node_attack.select",
        ),
        (
            "BehaviorNodeCooldown",
            "workbench.module.behavior.node_cooldown.select",
        ),
        ("BehaviorOutput", "workbench.module.behavior.output.select"),
        (
            "BehaviorValidate",
            "workbench.module.behavior.validate.invoke",
        ),
        (
            "AssetsBrowserTab",
            "workbench.module.assets.browser_tab.select",
        ),
        (
            "AssetsImportTab",
            "workbench.module.assets.import_tab.select",
        ),
        (
            "AssetsValidationTab",
            "workbench.module.assets.validation_tab.select",
        ),
        (
            "AssetsForestRow",
            "workbench.module.assets.forest_row.select",
        ),
        (
            "AssetsMaterialRow",
            "workbench.module.assets.material_row.select",
        ),
        (
            "AssetsTableTree",
            "workbench.module.assets.table_tree.select",
        ),
        (
            "AssetsTableMaterial",
            "workbench.module.assets.table_material.select",
        ),
        (
            "AssetsTableTexture",
            "workbench.module.assets.table_texture.select",
        ),
        ("AssetsOutput", "workbench.module.assets.output.select"),
        ("AssetsImport", "workbench.module.assets.import.invoke"),
        ("VfxEmittersTab", "workbench.module.vfx.emitters_tab.select"),
        ("VfxTimelineTab", "workbench.module.vfx.timeline_tab.select"),
        ("VfxCompileTab", "workbench.module.vfx.compile_tab.select"),
        ("VfxEmitterRow", "workbench.module.vfx.emitter_row.select"),
        ("VfxCurveRow", "workbench.module.vfx.curve_row.select"),
        ("VfxSpawnRow", "workbench.module.vfx.spawn_row.select"),
        ("VfxLifetimeRow", "workbench.module.vfx.lifetime_row.select"),
        ("VfxMaterialRow", "workbench.module.vfx.material_row.select"),
        ("VfxOutput", "workbench.module.vfx.output.select"),
        ("VfxSimulate", "workbench.module.vfx.simulate.invoke"),
        (
            "AbilityGraphTab",
            "workbench.module.ability.graph_tab.select",
        ),
        (
            "AbilityTasksTab",
            "workbench.module.ability.tasks_tab.select",
        ),
        (
            "AbilityDebugTab",
            "workbench.module.ability.debug_tab.select",
        ),
        (
            "AbilityTaskActivate",
            "workbench.module.ability.task_activate.select",
        ),
        (
            "AbilityTaskCost",
            "workbench.module.ability.task_cost.select",
        ),
        (
            "AbilityAssetDash",
            "workbench.module.ability.asset_dash.select",
        ),
        (
            "AbilityPlaytest",
            "workbench.module.ability.playtest.invoke",
        ),
        (
            "AbilityPhaseActivate",
            "workbench.module.ability.phase_activate.select",
        ),
        (
            "AbilityPhaseCost",
            "workbench.module.ability.phase_cost.select",
        ),
        (
            "AbilityGraphSelect",
            "workbench.module.ability.graph_select.select",
        ),
        ("AbilityOutput", "workbench.module.ability.output.select"),
        (
            "TagsRegistryTab",
            "workbench.module.tags.registry_tab.select",
        ),
        (
            "TagsValidationTab",
            "workbench.module.tags.validation_tab.select",
        ),
        ("TagsSourcesTab", "workbench.module.tags.sources_tab.select"),
        ("TagsAdd", "workbench.module.tags.add.invoke"),
        ("TagsRename", "workbench.module.tags.rename.invoke"),
        (
            "TagsSourceDefault",
            "workbench.module.tags.source_default.select",
        ),
        (
            "TagsAbilityActivate",
            "workbench.module.tags.ability_activate.select",
        ),
        (
            "TagsStateStunned",
            "workbench.module.tags.state_stunned.select",
        ),
        (
            "TagsValidationSelect",
            "workbench.module.tags.validation_select.select",
        ),
        (
            "PerceptionSightTab",
            "workbench.module.perception.sight_tab.select",
        ),
        (
            "PerceptionHearingTab",
            "workbench.module.perception.hearing_tab.select",
        ),
        (
            "PerceptionDebugTab",
            "workbench.module.perception.debug_tab.select",
        ),
        (
            "PerceptionGuard",
            "workbench.module.perception.guard.select",
        ),
        (
            "PerceptionSniper",
            "workbench.module.perception.sniper.select",
        ),
        (
            "PerceptionSimulate",
            "workbench.module.perception.simulate.invoke",
        ),
        (
            "PerceptionSightCone",
            "workbench.module.perception.sight_cone.select",
        ),
        (
            "PerceptionHearingPulse",
            "workbench.module.perception.hearing_pulse.select",
        ),
        (
            "PerceptionStimulus",
            "workbench.module.perception.stimulus.select",
        ),
        (
            "PerceptionEvent",
            "workbench.module.perception.event.select",
        ),
        ("RenderGraphTab", "workbench.module.render.graph_tab.select"),
        (
            "RenderResourcesTab",
            "workbench.module.render.resources_tab.select",
        ),
        (
            "RenderWarningsTab",
            "workbench.module.render.warnings_tab.select",
        ),
        (
            "RenderLightingPass",
            "workbench.module.render.lighting_pass.select",
        ),
        (
            "RenderBloomPass",
            "workbench.module.render.bloom_pass.select",
        ),
        ("RenderCompile", "workbench.module.render.compile.invoke"),
        (
            "RenderFrameStart",
            "workbench.module.render.frame_start.select",
        ),
        (
            "RenderLightingNode",
            "workbench.module.render.lighting_node.select",
        ),
        (
            "RenderSceneColor",
            "workbench.module.render.scene_color.select",
        ),
        ("RenderCapture", "workbench.module.render.capture.select"),
        ("HudCanvasTab", "workbench.module.hud.canvas_tab.select"),
        ("HudBindingsTab", "workbench.module.hud.bindings_tab.select"),
        (
            "HudValidationTab",
            "workbench.module.hud.validation_tab.select",
        ),
        ("HudWidgetText", "workbench.module.hud.widget_text.select"),
        (
            "HudWidgetButton",
            "workbench.module.hud.widget_button.select",
        ),
        ("HudPreview", "workbench.module.hud.preview.invoke"),
        ("HudMinimap", "workbench.module.hud.minimap.select"),
        ("HudAmmoPanel", "workbench.module.hud.ammo_panel.select"),
        ("HudBindingAmmo", "workbench.module.hud.binding_ammo.select"),
        (
            "HudValidationRow",
            "workbench.module.hud.validation_row.select",
        ),
    ] {
        insert_click(
            bindings,
            "WorkbenchModule",
            control_id,
            EditorUiBindingPayload::menu_action(action_id),
        );
    }
    insert_change(
        bindings,
        "WorkbenchModule",
        "EffectSearchEdit",
        EditorUiBindingPayload::menu_action("workbench.module.effect.search.edit"),
    );
    insert_submit(
        bindings,
        "WorkbenchModule",
        "EffectSearchCommit",
        EditorUiBindingPayload::menu_action("workbench.module.effect.search.commit"),
    );
    insert_change(
        bindings,
        "WorkbenchModule",
        "EffectMagnitudeEdit",
        EditorUiBindingPayload::menu_action("workbench.module.effect.magnitude.edit"),
    );
    insert_submit(
        bindings,
        "WorkbenchModule",
        "EffectMagnitudeCommit",
        EditorUiBindingPayload::menu_action("workbench.module.effect.magnitude.commit"),
    );
    for (control_id, action_id, event_kind) in [
        (
            "EffectPolicyEdit",
            "workbench.module.effect.policy.edit",
            EditorUiEventKind::Change,
        ),
        (
            "EffectPolicyCommit",
            "workbench.module.effect.policy.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "EffectNameEdit",
            "workbench.module.effect.name.edit",
            EditorUiEventKind::Change,
        ),
        (
            "EffectNameCommit",
            "workbench.module.effect.name.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "EffectTagEdit",
            "workbench.module.effect.tag.edit",
            EditorUiEventKind::Change,
        ),
        (
            "EffectTagCommit",
            "workbench.module.effect.tag.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "EffectStackEdit",
            "workbench.module.effect.stack.edit",
            EditorUiEventKind::Change,
        ),
        (
            "EffectStackCommit",
            "workbench.module.effect.stack.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "MaterialDomainEdit",
            "workbench.module.material.domain.edit",
            EditorUiEventKind::Change,
        ),
        (
            "MaterialDomainCommit",
            "workbench.module.material.domain.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "MaterialBlendEdit",
            "workbench.module.material.blend.edit",
            EditorUiEventKind::Change,
        ),
        (
            "MaterialBlendCommit",
            "workbench.module.material.blend.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "MaterialPreviewEdit",
            "workbench.module.material.preview.edit",
            EditorUiEventKind::Change,
        ),
        (
            "MaterialPreviewCommit",
            "workbench.module.material.preview.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "BehaviorBlackboardEdit",
            "workbench.module.behavior.blackboard.edit",
            EditorUiEventKind::Change,
        ),
        (
            "BehaviorBlackboardCommit",
            "workbench.module.behavior.blackboard.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "BehaviorAiEdit",
            "workbench.module.behavior.ai.edit",
            EditorUiEventKind::Change,
        ),
        (
            "BehaviorAiCommit",
            "workbench.module.behavior.ai.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "BehaviorStateEdit",
            "workbench.module.behavior.state.edit",
            EditorUiEventKind::Change,
        ),
        (
            "BehaviorStateCommit",
            "workbench.module.behavior.state.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "AssetsTypeEdit",
            "workbench.module.assets.type.edit",
            EditorUiEventKind::Change,
        ),
        (
            "AssetsTypeCommit",
            "workbench.module.assets.type.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "AssetsPathEdit",
            "workbench.module.assets.path.edit",
            EditorUiEventKind::Change,
        ),
        (
            "AssetsPathCommit",
            "workbench.module.assets.path.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "AssetsOwnerEdit",
            "workbench.module.assets.owner.edit",
            EditorUiEventKind::Change,
        ),
        (
            "AssetsOwnerCommit",
            "workbench.module.assets.owner.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "VfxSystemEdit",
            "workbench.module.vfx.system.edit",
            EditorUiEventKind::Change,
        ),
        (
            "VfxSystemCommit",
            "workbench.module.vfx.system.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "VfxBoundsEdit",
            "workbench.module.vfx.bounds.edit",
            EditorUiEventKind::Change,
        ),
        (
            "VfxBoundsCommit",
            "workbench.module.vfx.bounds.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "VfxSortEdit",
            "workbench.module.vfx.sort.edit",
            EditorUiEventKind::Change,
        ),
        (
            "VfxSortCommit",
            "workbench.module.vfx.sort.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "AbilityNameEdit",
            "workbench.module.ability.name.edit",
            EditorUiEventKind::Change,
        ),
        (
            "AbilityNameCommit",
            "workbench.module.ability.name.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "AbilityNetPolicyEdit",
            "workbench.module.ability.net_policy.edit",
            EditorUiEventKind::Change,
        ),
        (
            "AbilityNetPolicyCommit",
            "workbench.module.ability.net_policy.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "AbilityCooldownEdit",
            "workbench.module.ability.cooldown.edit",
            EditorUiEventKind::Change,
        ),
        (
            "AbilityCooldownCommit",
            "workbench.module.ability.cooldown.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "TagsSearchEdit",
            "workbench.module.tags.search.edit",
            EditorUiEventKind::Change,
        ),
        (
            "TagsSearchCommit",
            "workbench.module.tags.search.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "TagsRedirectEdit",
            "workbench.module.tags.redirect.edit",
            EditorUiEventKind::Change,
        ),
        (
            "TagsRedirectCommit",
            "workbench.module.tags.redirect.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "TagsOwnerEdit",
            "workbench.module.tags.owner.edit",
            EditorUiEventKind::Change,
        ),
        (
            "TagsOwnerCommit",
            "workbench.module.tags.owner.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "PerceptionConfigEdit",
            "workbench.module.perception.config.edit",
            EditorUiEventKind::Change,
        ),
        (
            "PerceptionConfigCommit",
            "workbench.module.perception.config.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "PerceptionLosEdit",
            "workbench.module.perception.los.edit",
            EditorUiEventKind::Change,
        ),
        (
            "PerceptionLosCommit",
            "workbench.module.perception.los.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "PerceptionTeamEdit",
            "workbench.module.perception.team.edit",
            EditorUiEventKind::Change,
        ),
        (
            "PerceptionTeamCommit",
            "workbench.module.perception.team.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "RenderPipelineEdit",
            "workbench.module.render.pipeline.edit",
            EditorUiEventKind::Change,
        ),
        (
            "RenderPipelineCommit",
            "workbench.module.render.pipeline.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "RenderPlatformEdit",
            "workbench.module.render.platform.edit",
            EditorUiEventKind::Change,
        ),
        (
            "RenderPlatformCommit",
            "workbench.module.render.platform.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "RenderFrameEdit",
            "workbench.module.render.frame.edit",
            EditorUiEventKind::Change,
        ),
        (
            "RenderFrameCommit",
            "workbench.module.render.frame.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "HudScreenEdit",
            "workbench.module.hud.screen.edit",
            EditorUiEventKind::Change,
        ),
        (
            "HudScreenCommit",
            "workbench.module.hud.screen.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "HudDpiEdit",
            "workbench.module.hud.dpi.edit",
            EditorUiEventKind::Change,
        ),
        (
            "HudDpiCommit",
            "workbench.module.hud.dpi.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "HudLocaleEdit",
            "workbench.module.hud.locale.edit",
            EditorUiEventKind::Change,
        ),
        (
            "HudLocaleCommit",
            "workbench.module.hud.locale.commit",
            EditorUiEventKind::Submit,
        ),
    ] {
        let payload = EditorUiBindingPayload::menu_action(action_id);
        match event_kind {
            EditorUiEventKind::Change => {
                insert_change(bindings, "WorkbenchModule", control_id, payload)
            }
            EditorUiEventKind::Submit => {
                insert_submit(bindings, "WorkbenchModule", control_id, payload)
            }
            _ => {}
        }
    }
}

fn insert_change(
    bindings: &mut BTreeMap<String, EditorUiBinding>,
    view_id: &str,
    control_id: &str,
    payload: EditorUiBindingPayload,
) {
    insert_event(
        bindings,
        view_id,
        control_id,
        EditorUiEventKind::Change,
        payload,
    );
}

fn insert_click(
    bindings: &mut BTreeMap<String, EditorUiBinding>,
    view_id: &str,
    control_id: &str,
    payload: EditorUiBindingPayload,
) {
    insert_event(
        bindings,
        view_id,
        control_id,
        EditorUiEventKind::Click,
        payload,
    );
}

fn insert_submit(
    bindings: &mut BTreeMap<String, EditorUiBinding>,
    view_id: &str,
    control_id: &str,
    payload: EditorUiBindingPayload,
) {
    insert_event(
        bindings,
        view_id,
        control_id,
        EditorUiEventKind::Submit,
        payload,
    );
}

fn insert_event(
    bindings: &mut BTreeMap<String, EditorUiBinding>,
    view_id: &str,
    control_id: &str,
    event_kind: EditorUiEventKind,
    payload: EditorUiBindingPayload,
) {
    bindings.insert(
        format!("{view_id}/{control_id}"),
        EditorUiBinding::new(view_id, control_id, event_kind, payload),
    );
}
