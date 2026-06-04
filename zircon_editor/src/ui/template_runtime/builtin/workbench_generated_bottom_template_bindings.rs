use std::collections::BTreeMap;

use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};

pub(super) fn insert_workbench_generated_bottom_bindings(
    bindings: &mut BTreeMap<String, EditorUiBinding>,
) {
    for (control_id, action_id) in [
        ("OpenPanel", "workbench.generated_bottom.open_panel.invoke"),
        ("PinPanel", "workbench.generated_bottom.pin_panel.invoke"),
        (
            "ModeOutput",
            "workbench.generated_bottom.mode_output.select",
        ),
        ("ModeBuild", "workbench.generated_bottom.mode_build.select"),
        (
            "ModeValidation",
            "workbench.generated_bottom.mode_validation.select",
        ),
        (
            "ModeRuntime",
            "workbench.generated_bottom.mode_runtime.select",
        ),
        (
            "ModeReview",
            "workbench.generated_bottom.mode_review.select",
        ),
        (
            "SceneConsole",
            "workbench.generated_bottom.scene_console.select",
        ),
        (
            "SceneValidation",
            "workbench.generated_bottom.scene_validation.select",
        ),
        (
            "GameplayEffectAttributeDelta",
            "workbench.generated_bottom.gameplay_effect_attribute_delta.select",
        ),
        (
            "GameplayEffectValidation",
            "workbench.generated_bottom.gameplay_effect_validation.select",
        ),
        (
            "GameplayEffectCompileLog",
            "workbench.generated_bottom.gameplay_effect_compile_log.select",
        ),
        (
            "GameplayAbilityCompileLog",
            "workbench.generated_bottom.gameplay_ability_compile_log.select",
        ),
        (
            "GameplayAbilityGameplayEventLog",
            "workbench.generated_bottom.gameplay_ability_gameplay_event_log.select",
        ),
        (
            "GameplayAbilitySimulationConsole",
            "workbench.generated_bottom.gameplay_ability_simulation_console.select",
        ),
        (
            "GameplayTagsReferenceScan",
            "workbench.generated_bottom.gameplay_tags_reference_scan.select",
        ),
        (
            "GameplayTagsMigrationPreview",
            "workbench.generated_bottom.gameplay_tags_migration_preview.select",
        ),
        (
            "GameplayTagsCompileLog",
            "workbench.generated_bottom.gameplay_tags_compile_log.select",
        ),
        (
            "AiPerceptionDebugLog",
            "workbench.generated_bottom.ai_perception_debug_log.select",
        ),
        (
            "AiPerceptionQueryOutput",
            "workbench.generated_bottom.ai_perception_query_output.select",
        ),
        (
            "AiPerceptionValidation",
            "workbench.generated_bottom.ai_perception_validation.select",
        ),
        (
            "AiPerceptionCompileLog",
            "workbench.generated_bottom.ai_perception_compile_log.select",
        ),
        (
            "MaterialPreviewVariants",
            "workbench.generated_bottom.material_preview_variants.select",
        ),
        (
            "MaterialWarnings",
            "workbench.generated_bottom.material_warnings.select",
        ),
        (
            "BehaviorTreeRuntimeTrace",
            "workbench.generated_bottom.behavior_tree_runtime_trace.select",
        ),
        (
            "BehaviorTreeBreakpointOutput",
            "workbench.generated_bottom.behavior_tree_breakpoint_output.select",
        ),
        (
            "BehaviorTreeValidationIssues",
            "workbench.generated_bottom.behavior_tree_validation_issues.select",
        ),
        (
            "RenderPipelineCompileOutput",
            "workbench.generated_bottom.render_pipeline_compile_output.select",
        ),
        (
            "RenderPipelineResourceTransitions",
            "workbench.generated_bottom.render_pipeline_resource_transitions.select",
        ),
        (
            "RenderPipelineWarnings",
            "workbench.generated_bottom.render_pipeline_warnings.select",
        ),
        (
            "RenderPipelineErrors",
            "workbench.generated_bottom.render_pipeline_errors.select",
        ),
        (
            "RenderPipelineCompileLog",
            "workbench.generated_bottom.render_pipeline_compile_log.select",
        ),
        (
            "AssetBrowserOutput",
            "workbench.generated_bottom.asset_browser_output.select",
        ),
        (
            "AssetBrowserValidation",
            "workbench.generated_bottom.asset_browser_validation.select",
        ),
        (
            "AssetBrowserCook",
            "workbench.generated_bottom.asset_browser_cook.select",
        ),
        (
            "AssetBrowserPackage",
            "workbench.generated_bottom.asset_browser_package.select",
        ),
        ("VfxCurves", "workbench.generated_bottom.vfx_curves.select"),
        (
            "VfxNiagaraLog",
            "workbench.generated_bottom.vfx_niagara_log.select",
        ),
        (
            "VfxCompileOutput",
            "workbench.generated_bottom.vfx_compile_output.select",
        ),
        (
            "VfxEventLog",
            "workbench.generated_bottom.vfx_event_log.select",
        ),
        (
            "HudEditorBindingErrors",
            "workbench.generated_bottom.hud_editor_binding_errors.select",
        ),
        (
            "HudEditorPreviewLog",
            "workbench.generated_bottom.hud_editor_preview_log.select",
        ),
        (
            "HudEditorPerformance",
            "workbench.generated_bottom.hud_editor_performance.select",
        ),
        (
            "HudEditorCompileLog",
            "workbench.generated_bottom.hud_editor_compile_log.select",
        ),
    ] {
        insert_click(
            bindings,
            "WorkbenchGeneratedBottom",
            control_id,
            EditorUiBindingPayload::menu_action(action_id),
        );
    }

    for (control_id, action_id, event_kind) in [
        (
            "FilterEdit",
            "workbench.generated_bottom.filter.edit",
            EditorUiEventKind::Change,
        ),
        (
            "FilterCommit",
            "workbench.generated_bottom.filter.commit",
            EditorUiEventKind::Submit,
        ),
        (
            "ModeEdit",
            "workbench.generated_bottom.mode.edit",
            EditorUiEventKind::Change,
        ),
        (
            "ModeCommit",
            "workbench.generated_bottom.mode.commit",
            EditorUiEventKind::Submit,
        ),
    ] {
        insert_event(
            bindings,
            "WorkbenchGeneratedBottom",
            control_id,
            event_kind,
            EditorUiBindingPayload::menu_action(action_id),
        );
    }
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
