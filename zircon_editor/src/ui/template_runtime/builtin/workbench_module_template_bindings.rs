use std::collections::BTreeMap;

use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};

pub(super) fn insert_workbench_module_bindings(bindings: &mut BTreeMap<String, EditorUiBinding>) {
    for (control_id, action_id) in [
        ("Scene", "SelectWorkbenchModuleScene"),
        ("Effect", "SelectWorkbenchModuleEffect"),
        ("Ability", "SelectWorkbenchModuleAbility"),
        ("Tags", "SelectWorkbenchModuleTags"),
        ("Perception", "SelectWorkbenchModulePerception"),
        ("Material", "SelectWorkbenchModuleMaterial"),
        ("Behavior", "SelectWorkbenchModuleBehavior"),
        ("Render", "SelectWorkbenchModuleRender"),
        ("Assets", "SelectWorkbenchModuleAssets"),
        ("Vfx", "SelectWorkbenchModuleVfx"),
        ("Hud", "SelectWorkbenchModuleHud"),
        ("Save", "InvokeWorkbenchModuleSave"),
        ("Browse", "InvokeWorkbenchModuleBrowse"),
        ("Compile", "InvokeWorkbenchModuleCompile"),
        ("Diff", "InvokeWorkbenchModuleDiff"),
        ("Simulate", "InvokeWorkbenchModuleSimulate"),
        ("EffectRulesTab", "SelectWorkbenchEffectRulesTab"),
        ("EffectTagsTab", "SelectWorkbenchEffectTagsTab"),
        ("EffectSimulationTab", "SelectWorkbenchEffectSimulationTab"),
        ("EffectDurationRow", "SelectWorkbenchEffectDurationRow"),
        ("EffectPeriodRow", "SelectWorkbenchEffectPeriodRow"),
        (
            "EffectHealthRegenRow",
            "SelectWorkbenchEffectHealthRegenRow",
        ),
        ("EffectDamageFireRow", "SelectWorkbenchEffectDamageFireRow"),
        ("EffectStackRow", "SelectWorkbenchEffectStackRow"),
        ("EffectCueRow", "SelectWorkbenchEffectCueRow"),
        ("EffectOutput", "SelectWorkbenchEffectOutput"),
        ("EffectApply", "InvokeWorkbenchEffectApply"),
        (
            "EffectModifierHealth",
            "SelectWorkbenchEffectModifierHealth",
        ),
        (
            "EffectModifierHealing",
            "SelectWorkbenchEffectModifierHealing",
        ),
        ("EffectModifierCap", "SelectWorkbenchEffectModifierCap"),
        ("EffectGraphSelect", "SelectWorkbenchEffectGraphSelect"),
        (
            "EffectAttributePreview",
            "SelectWorkbenchEffectAttributePreview",
        ),
        ("MaterialGraphTab", "SelectWorkbenchMaterialGraphTab"),
        ("MaterialPreviewTab", "SelectWorkbenchMaterialPreviewTab"),
        ("MaterialShaderTab", "SelectWorkbenchMaterialShaderTab"),
        (
            "MaterialBaseColorRow",
            "SelectWorkbenchMaterialBaseColorRow",
        ),
        ("MaterialNormalRow", "SelectWorkbenchMaterialNormalRow"),
        ("MaterialNodeAlbedo", "SelectWorkbenchMaterialNodeAlbedo"),
        (
            "MaterialNodeRoughness",
            "SelectWorkbenchMaterialNodeRoughness",
        ),
        ("MaterialNodeNormal", "SelectWorkbenchMaterialNodeNormal"),
        ("MaterialOutput", "SelectWorkbenchMaterialOutput"),
        ("MaterialCompile", "InvokeWorkbenchMaterialCompile"),
        ("BehaviorTreeTab", "SelectWorkbenchBehaviorTreeTab"),
        (
            "BehaviorBlackboardTab",
            "SelectWorkbenchBehaviorBlackboardTab",
        ),
        ("BehaviorDebugTab", "SelectWorkbenchBehaviorDebugTab"),
        ("BehaviorSelectorRow", "SelectWorkbenchBehaviorSelectorRow"),
        ("BehaviorAttackRow", "SelectWorkbenchBehaviorAttackRow"),
        (
            "BehaviorNodeSelector",
            "SelectWorkbenchBehaviorNodeSelector",
        ),
        ("BehaviorNodeAttack", "SelectWorkbenchBehaviorNodeAttack"),
        (
            "BehaviorNodeCooldown",
            "SelectWorkbenchBehaviorNodeCooldown",
        ),
        ("BehaviorOutput", "SelectWorkbenchBehaviorOutput"),
        ("BehaviorValidate", "InvokeWorkbenchBehaviorValidate"),
        ("AssetsBrowserTab", "SelectWorkbenchAssetsBrowserTab"),
        ("AssetsImportTab", "SelectWorkbenchAssetsImportTab"),
        ("AssetsValidationTab", "SelectWorkbenchAssetsValidationTab"),
        ("AssetsForestRow", "SelectWorkbenchAssetsForestRow"),
        ("AssetsMaterialRow", "SelectWorkbenchAssetsMaterialRow"),
        ("AssetsTableTree", "SelectWorkbenchAssetsTableTree"),
        ("AssetsTableMaterial", "SelectWorkbenchAssetsTableMaterial"),
        ("AssetsTableTexture", "SelectWorkbenchAssetsTableTexture"),
        ("AssetsOutput", "SelectWorkbenchAssetsOutput"),
        ("AssetsImport", "InvokeWorkbenchAssetsImport"),
        ("VfxEmittersTab", "SelectWorkbenchVfxEmittersTab"),
        ("VfxTimelineTab", "SelectWorkbenchVfxTimelineTab"),
        ("VfxCompileTab", "SelectWorkbenchVfxCompileTab"),
        ("VfxEmitterRow", "SelectWorkbenchVfxEmitterRow"),
        ("VfxCurveRow", "SelectWorkbenchVfxCurveRow"),
        ("VfxSpawnRow", "SelectWorkbenchVfxSpawnRow"),
        ("VfxLifetimeRow", "SelectWorkbenchVfxLifetimeRow"),
        ("VfxMaterialRow", "SelectWorkbenchVfxMaterialRow"),
        ("VfxOutput", "SelectWorkbenchVfxOutput"),
        ("VfxSimulate", "InvokeWorkbenchVfxSimulate"),
        ("AbilityGraphTab", "SelectWorkbenchAbilityGraphTab"),
        ("AbilityTasksTab", "SelectWorkbenchAbilityTasksTab"),
        ("AbilityDebugTab", "SelectWorkbenchAbilityDebugTab"),
        ("AbilityTaskActivate", "SelectWorkbenchAbilityTaskActivate"),
        ("AbilityTaskCost", "SelectWorkbenchAbilityTaskCost"),
        ("AbilityAssetDash", "SelectWorkbenchAbilityAssetDash"),
        ("AbilityPlaytest", "InvokeWorkbenchAbilityPlaytest"),
        (
            "AbilityPhaseActivate",
            "SelectWorkbenchAbilityPhaseActivate",
        ),
        ("AbilityPhaseCost", "SelectWorkbenchAbilityPhaseCost"),
        ("AbilityGraphSelect", "SelectWorkbenchAbilityGraphSelect"),
        ("AbilityOutput", "SelectWorkbenchAbilityOutput"),
        ("TagsRegistryTab", "SelectWorkbenchTagsRegistryTab"),
        ("TagsValidationTab", "SelectWorkbenchTagsValidationTab"),
        ("TagsSourcesTab", "SelectWorkbenchTagsSourcesTab"),
        ("TagsAdd", "InvokeWorkbenchTagsAdd"),
        ("TagsRename", "InvokeWorkbenchTagsRename"),
        ("TagsSourceDefault", "SelectWorkbenchTagsSourceDefault"),
        ("TagsAbilityActivate", "SelectWorkbenchTagsAbilityActivate"),
        ("TagsStateStunned", "SelectWorkbenchTagsStateStunned"),
        (
            "TagsValidationSelect",
            "SelectWorkbenchTagsValidationSelect",
        ),
        ("PerceptionSightTab", "SelectWorkbenchPerceptionSightTab"),
        (
            "PerceptionHearingTab",
            "SelectWorkbenchPerceptionHearingTab",
        ),
        ("PerceptionDebugTab", "SelectWorkbenchPerceptionDebugTab"),
        ("PerceptionGuard", "SelectWorkbenchPerceptionGuard"),
        ("PerceptionSniper", "SelectWorkbenchPerceptionSniper"),
        ("PerceptionSimulate", "InvokeWorkbenchPerceptionSimulate"),
        ("PerceptionSightCone", "SelectWorkbenchPerceptionSightCone"),
        (
            "PerceptionHearingPulse",
            "SelectWorkbenchPerceptionHearingPulse",
        ),
        ("PerceptionStimulus", "SelectWorkbenchPerceptionStimulus"),
        ("PerceptionEvent", "SelectWorkbenchPerceptionEvent"),
        ("RenderGraphTab", "SelectWorkbenchRenderGraphTab"),
        ("RenderResourcesTab", "SelectWorkbenchRenderResourcesTab"),
        ("RenderWarningsTab", "SelectWorkbenchRenderWarningsTab"),
        ("RenderLightingPass", "SelectWorkbenchRenderLightingPass"),
        ("RenderBloomPass", "SelectWorkbenchRenderBloomPass"),
        ("RenderCompile", "InvokeWorkbenchRenderCompile"),
        ("RenderFrameStart", "SelectWorkbenchRenderFrameStart"),
        ("RenderLightingNode", "SelectWorkbenchRenderLightingNode"),
        ("RenderSceneColor", "SelectWorkbenchRenderSceneColor"),
        ("RenderCapture", "SelectWorkbenchRenderCapture"),
        ("HudCanvasTab", "SelectWorkbenchHudCanvasTab"),
        ("HudBindingsTab", "SelectWorkbenchHudBindingsTab"),
        ("HudValidationTab", "SelectWorkbenchHudValidationTab"),
        ("HudWidgetText", "SelectWorkbenchHudWidgetText"),
        ("HudWidgetButton", "SelectWorkbenchHudWidgetButton"),
        ("HudPreview", "InvokeWorkbenchHudPreview"),
        ("HudMinimap", "SelectWorkbenchHudMinimap"),
        ("HudAmmoPanel", "SelectWorkbenchHudAmmoPanel"),
        ("HudBindingAmmo", "SelectWorkbenchHudBindingAmmo"),
        ("HudValidationRow", "SelectWorkbenchHudValidationRow"),
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
        EditorUiBindingPayload::menu_action("EditWorkbenchEffectSearch"),
    );
    insert_submit(
        bindings,
        "WorkbenchModule",
        "EffectSearchCommit",
        EditorUiBindingPayload::menu_action("CommitWorkbenchEffectSearch"),
    );
    insert_change(
        bindings,
        "WorkbenchModule",
        "EffectMagnitudeEdit",
        EditorUiBindingPayload::menu_action("EditWorkbenchEffectMagnitude"),
    );
    insert_submit(
        bindings,
        "WorkbenchModule",
        "EffectMagnitudeCommit",
        EditorUiBindingPayload::menu_action("CommitWorkbenchEffectMagnitude"),
    );
    for (control_id, action_id, event_kind) in [
        (
            "EffectPolicyEdit",
            "EditWorkbenchEffectPolicy",
            EditorUiEventKind::Change,
        ),
        (
            "EffectPolicyCommit",
            "CommitWorkbenchEffectPolicy",
            EditorUiEventKind::Submit,
        ),
        (
            "EffectNameEdit",
            "EditWorkbenchEffectName",
            EditorUiEventKind::Change,
        ),
        (
            "EffectNameCommit",
            "CommitWorkbenchEffectName",
            EditorUiEventKind::Submit,
        ),
        (
            "EffectTagEdit",
            "EditWorkbenchEffectTag",
            EditorUiEventKind::Change,
        ),
        (
            "EffectTagCommit",
            "CommitWorkbenchEffectTag",
            EditorUiEventKind::Submit,
        ),
        (
            "EffectStackEdit",
            "EditWorkbenchEffectStack",
            EditorUiEventKind::Change,
        ),
        (
            "EffectStackCommit",
            "CommitWorkbenchEffectStack",
            EditorUiEventKind::Submit,
        ),
        (
            "MaterialDomainEdit",
            "EditWorkbenchMaterialDomain",
            EditorUiEventKind::Change,
        ),
        (
            "MaterialDomainCommit",
            "CommitWorkbenchMaterialDomain",
            EditorUiEventKind::Submit,
        ),
        (
            "MaterialBlendEdit",
            "EditWorkbenchMaterialBlend",
            EditorUiEventKind::Change,
        ),
        (
            "MaterialBlendCommit",
            "CommitWorkbenchMaterialBlend",
            EditorUiEventKind::Submit,
        ),
        (
            "MaterialPreviewEdit",
            "EditWorkbenchMaterialPreview",
            EditorUiEventKind::Change,
        ),
        (
            "MaterialPreviewCommit",
            "CommitWorkbenchMaterialPreview",
            EditorUiEventKind::Submit,
        ),
        (
            "BehaviorBlackboardEdit",
            "EditWorkbenchBehaviorBlackboard",
            EditorUiEventKind::Change,
        ),
        (
            "BehaviorBlackboardCommit",
            "CommitWorkbenchBehaviorBlackboard",
            EditorUiEventKind::Submit,
        ),
        (
            "BehaviorAiEdit",
            "EditWorkbenchBehaviorAi",
            EditorUiEventKind::Change,
        ),
        (
            "BehaviorAiCommit",
            "CommitWorkbenchBehaviorAi",
            EditorUiEventKind::Submit,
        ),
        (
            "BehaviorStateEdit",
            "EditWorkbenchBehaviorState",
            EditorUiEventKind::Change,
        ),
        (
            "BehaviorStateCommit",
            "CommitWorkbenchBehaviorState",
            EditorUiEventKind::Submit,
        ),
        (
            "AssetsTypeEdit",
            "EditWorkbenchAssetsType",
            EditorUiEventKind::Change,
        ),
        (
            "AssetsTypeCommit",
            "CommitWorkbenchAssetsType",
            EditorUiEventKind::Submit,
        ),
        (
            "AssetsPathEdit",
            "EditWorkbenchAssetsPath",
            EditorUiEventKind::Change,
        ),
        (
            "AssetsPathCommit",
            "CommitWorkbenchAssetsPath",
            EditorUiEventKind::Submit,
        ),
        (
            "AssetsOwnerEdit",
            "EditWorkbenchAssetsOwner",
            EditorUiEventKind::Change,
        ),
        (
            "AssetsOwnerCommit",
            "CommitWorkbenchAssetsOwner",
            EditorUiEventKind::Submit,
        ),
        (
            "VfxSystemEdit",
            "EditWorkbenchVfxSystem",
            EditorUiEventKind::Change,
        ),
        (
            "VfxSystemCommit",
            "CommitWorkbenchVfxSystem",
            EditorUiEventKind::Submit,
        ),
        (
            "VfxBoundsEdit",
            "EditWorkbenchVfxBounds",
            EditorUiEventKind::Change,
        ),
        (
            "VfxBoundsCommit",
            "CommitWorkbenchVfxBounds",
            EditorUiEventKind::Submit,
        ),
        (
            "VfxSortEdit",
            "EditWorkbenchVfxSort",
            EditorUiEventKind::Change,
        ),
        (
            "VfxSortCommit",
            "CommitWorkbenchVfxSort",
            EditorUiEventKind::Submit,
        ),
        (
            "AbilityNameEdit",
            "EditWorkbenchAbilityName",
            EditorUiEventKind::Change,
        ),
        (
            "AbilityNameCommit",
            "CommitWorkbenchAbilityName",
            EditorUiEventKind::Submit,
        ),
        (
            "AbilityNetPolicyEdit",
            "EditWorkbenchAbilityNetPolicy",
            EditorUiEventKind::Change,
        ),
        (
            "AbilityNetPolicyCommit",
            "CommitWorkbenchAbilityNetPolicy",
            EditorUiEventKind::Submit,
        ),
        (
            "AbilityCooldownEdit",
            "EditWorkbenchAbilityCooldown",
            EditorUiEventKind::Change,
        ),
        (
            "AbilityCooldownCommit",
            "CommitWorkbenchAbilityCooldown",
            EditorUiEventKind::Submit,
        ),
        (
            "TagsSearchEdit",
            "EditWorkbenchTagsSearch",
            EditorUiEventKind::Change,
        ),
        (
            "TagsSearchCommit",
            "CommitWorkbenchTagsSearch",
            EditorUiEventKind::Submit,
        ),
        (
            "TagsRedirectEdit",
            "EditWorkbenchTagsRedirect",
            EditorUiEventKind::Change,
        ),
        (
            "TagsRedirectCommit",
            "CommitWorkbenchTagsRedirect",
            EditorUiEventKind::Submit,
        ),
        (
            "TagsOwnerEdit",
            "EditWorkbenchTagsOwner",
            EditorUiEventKind::Change,
        ),
        (
            "TagsOwnerCommit",
            "CommitWorkbenchTagsOwner",
            EditorUiEventKind::Submit,
        ),
        (
            "PerceptionConfigEdit",
            "EditWorkbenchPerceptionConfig",
            EditorUiEventKind::Change,
        ),
        (
            "PerceptionConfigCommit",
            "CommitWorkbenchPerceptionConfig",
            EditorUiEventKind::Submit,
        ),
        (
            "PerceptionLosEdit",
            "EditWorkbenchPerceptionLos",
            EditorUiEventKind::Change,
        ),
        (
            "PerceptionLosCommit",
            "CommitWorkbenchPerceptionLos",
            EditorUiEventKind::Submit,
        ),
        (
            "PerceptionTeamEdit",
            "EditWorkbenchPerceptionTeam",
            EditorUiEventKind::Change,
        ),
        (
            "PerceptionTeamCommit",
            "CommitWorkbenchPerceptionTeam",
            EditorUiEventKind::Submit,
        ),
        (
            "RenderPipelineEdit",
            "EditWorkbenchRenderPipeline",
            EditorUiEventKind::Change,
        ),
        (
            "RenderPipelineCommit",
            "CommitWorkbenchRenderPipeline",
            EditorUiEventKind::Submit,
        ),
        (
            "RenderPlatformEdit",
            "EditWorkbenchRenderPlatform",
            EditorUiEventKind::Change,
        ),
        (
            "RenderPlatformCommit",
            "CommitWorkbenchRenderPlatform",
            EditorUiEventKind::Submit,
        ),
        (
            "RenderFrameEdit",
            "EditWorkbenchRenderFrame",
            EditorUiEventKind::Change,
        ),
        (
            "RenderFrameCommit",
            "CommitWorkbenchRenderFrame",
            EditorUiEventKind::Submit,
        ),
        (
            "HudScreenEdit",
            "EditWorkbenchHudScreen",
            EditorUiEventKind::Change,
        ),
        (
            "HudScreenCommit",
            "CommitWorkbenchHudScreen",
            EditorUiEventKind::Submit,
        ),
        (
            "HudDpiEdit",
            "EditWorkbenchHudDpi",
            EditorUiEventKind::Change,
        ),
        (
            "HudDpiCommit",
            "CommitWorkbenchHudDpi",
            EditorUiEventKind::Submit,
        ),
        (
            "HudLocaleEdit",
            "EditWorkbenchHudLocale",
            EditorUiEventKind::Change,
        ),
        (
            "HudLocaleCommit",
            "CommitWorkbenchHudLocale",
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
