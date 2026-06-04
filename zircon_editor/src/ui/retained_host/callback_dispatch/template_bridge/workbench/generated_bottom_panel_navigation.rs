pub(super) const GENERATED_BOTTOM_MODE_CONTROLS: &[&str] = &[
    "WorkbenchGeneratedBottomModeOutput",
    "WorkbenchGeneratedBottomModeBuild",
    "WorkbenchGeneratedBottomModeValidation",
    "WorkbenchGeneratedBottomModeRuntime",
    "WorkbenchGeneratedBottomModeReview",
];

pub(super) const GENERATED_BOTTOM_ROUTE_CONTROLS: &[&str] = &[
    "WorkbenchGeneratedBottomSceneConsoleRow",
    "WorkbenchGeneratedBottomSceneValidationRow",
    "WorkbenchGeneratedBottomGameplayEffectAttributeDeltaRow",
    "WorkbenchGeneratedBottomGameplayEffectValidationRow",
    "WorkbenchGeneratedBottomGameplayEffectCompileLogRow",
    "WorkbenchGeneratedBottomGameplayAbilityCompileLogRow",
    "WorkbenchGeneratedBottomGameplayAbilityGameplayEventLogRow",
    "WorkbenchGeneratedBottomGameplayAbilitySimulationConsoleRow",
    "WorkbenchGeneratedBottomGameplayTagsReferenceScanRow",
    "WorkbenchGeneratedBottomGameplayTagsMigrationPreviewRow",
    "WorkbenchGeneratedBottomGameplayTagsCompileLogRow",
    "WorkbenchGeneratedBottomAiPerceptionDebugLogRow",
    "WorkbenchGeneratedBottomAiPerceptionQueryOutputRow",
    "WorkbenchGeneratedBottomAiPerceptionValidationRow",
    "WorkbenchGeneratedBottomAiPerceptionCompileLogRow",
    "WorkbenchGeneratedBottomMaterialPreviewVariantsRow",
    "WorkbenchGeneratedBottomMaterialWarningsRow",
    "WorkbenchGeneratedBottomBehaviorTreeRuntimeTraceRow",
    "WorkbenchGeneratedBottomBehaviorTreeBreakpointOutputRow",
    "WorkbenchGeneratedBottomBehaviorTreeValidationIssuesRow",
    "WorkbenchGeneratedBottomRenderPipelineCompileOutputRow",
    "WorkbenchGeneratedBottomRenderPipelineResourceTransitionsRow",
    "WorkbenchGeneratedBottomRenderPipelineWarningsRow",
    "WorkbenchGeneratedBottomRenderPipelineErrorsRow",
    "WorkbenchGeneratedBottomRenderPipelineCompileLogRow",
    "WorkbenchGeneratedBottomAssetBrowserOutputRow",
    "WorkbenchGeneratedBottomAssetBrowserValidationRow",
    "WorkbenchGeneratedBottomAssetBrowserCookRow",
    "WorkbenchGeneratedBottomAssetBrowserPackageRow",
    "WorkbenchGeneratedBottomVfxCurvesRow",
    "WorkbenchGeneratedBottomVfxNiagaraLogRow",
    "WorkbenchGeneratedBottomVfxCompileOutputRow",
    "WorkbenchGeneratedBottomVfxEventLogRow",
    "WorkbenchGeneratedBottomHudEditorBindingErrorsRow",
    "WorkbenchGeneratedBottomHudEditorPreviewLogRow",
    "WorkbenchGeneratedBottomHudEditorPerformanceRow",
    "WorkbenchGeneratedBottomHudEditorCompileLogRow",
];

#[derive(Clone, Copy)]
pub(super) struct GeneratedBottomRouteTarget {
    pub(super) action_id: &'static str,
    pub(super) control_id: &'static str,
    pub(super) panel_route: &'static str,
    pub(super) module_label: &'static str,
    pub(super) panel_label: &'static str,
    pub(super) mode_control_id: &'static str,
}

pub(super) fn is_workbench_generated_bottom_action(action_id: &str) -> bool {
    workbench_generated_bottom_route_target(action_id).is_some()
        || workbench_generated_bottom_mode_control_id(action_id).is_some()
        || matches!(
            action_id,
            "workbench.generated_bottom.open_panel.invoke"
                | "workbench.generated_bottom.pin_panel.invoke"
                | "workbench.generated_bottom.filter.edit"
                | "workbench.generated_bottom.filter.commit"
                | "workbench.generated_bottom.mode.edit"
                | "workbench.generated_bottom.mode.commit"
        )
}

pub(super) fn workbench_generated_bottom_mode_control_id(action_id: &str) -> Option<&'static str> {
    match action_id {
        "workbench.generated_bottom.mode_output.select" => {
            Some("WorkbenchGeneratedBottomModeOutput")
        }
        "workbench.generated_bottom.mode_build.select" => Some("WorkbenchGeneratedBottomModeBuild"),
        "workbench.generated_bottom.mode_validation.select" => {
            Some("WorkbenchGeneratedBottomModeValidation")
        }
        "workbench.generated_bottom.mode_runtime.select" => {
            Some("WorkbenchGeneratedBottomModeRuntime")
        }
        "workbench.generated_bottom.mode_review.select" => {
            Some("WorkbenchGeneratedBottomModeReview")
        }
        _ => None,
    }
}

pub(super) fn workbench_generated_bottom_route_control_id(action_id: &str) -> Option<&'static str> {
    Some(workbench_generated_bottom_route_target(action_id)?.control_id)
}

pub(super) fn workbench_generated_bottom_route_target(
    action_id: &str,
) -> Option<GeneratedBottomRouteTarget> {
    for target in GENERATED_BOTTOM_ROUTE_TARGETS {
        if target.action_id == action_id {
            return Some(*target);
        }
    }
    None
}

pub(super) const GENERATED_BOTTOM_ROUTE_TARGETS: &[GeneratedBottomRouteTarget] = &[
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.scene_console.select",
        control_id: "WorkbenchGeneratedBottomSceneConsoleRow",
        panel_route: "module-bottom-scene:console",
        module_label: "Scene",
        panel_label: "Console",
        mode_control_id: "WorkbenchGeneratedBottomModeRuntime",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.scene_validation.select",
        control_id: "WorkbenchGeneratedBottomSceneValidationRow",
        panel_route: "module-bottom-scene:validation",
        module_label: "Scene",
        panel_label: "Validation",
        mode_control_id: "WorkbenchGeneratedBottomModeValidation",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.gameplay_effect_attribute_delta.select",
        control_id: "WorkbenchGeneratedBottomGameplayEffectAttributeDeltaRow",
        panel_route: "module-bottom-gameplay-effect:attribute-delta",
        module_label: "Gameplay Effect",
        panel_label: "Attribute Delta",
        mode_control_id: "WorkbenchGeneratedBottomModeOutput",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.gameplay_effect_validation.select",
        control_id: "WorkbenchGeneratedBottomGameplayEffectValidationRow",
        panel_route: "module-bottom-gameplay-effect:validation",
        module_label: "Gameplay Effect",
        panel_label: "Validation",
        mode_control_id: "WorkbenchGeneratedBottomModeValidation",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.gameplay_effect_compile_log.select",
        control_id: "WorkbenchGeneratedBottomGameplayEffectCompileLogRow",
        panel_route: "module-bottom-gameplay-effect:compile-log",
        module_label: "Gameplay Effect",
        panel_label: "Compile Log",
        mode_control_id: "WorkbenchGeneratedBottomModeBuild",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.gameplay_ability_compile_log.select",
        control_id: "WorkbenchGeneratedBottomGameplayAbilityCompileLogRow",
        panel_route: "module-bottom-gameplay-ability:compile-log",
        module_label: "Gameplay Ability",
        panel_label: "Compile Log",
        mode_control_id: "WorkbenchGeneratedBottomModeBuild",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.gameplay_ability_gameplay_event_log.select",
        control_id: "WorkbenchGeneratedBottomGameplayAbilityGameplayEventLogRow",
        panel_route: "module-bottom-gameplay-ability:gameplay-event-log",
        module_label: "Gameplay Ability",
        panel_label: "Gameplay Event Log",
        mode_control_id: "WorkbenchGeneratedBottomModeRuntime",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.gameplay_ability_simulation_console.select",
        control_id: "WorkbenchGeneratedBottomGameplayAbilitySimulationConsoleRow",
        panel_route: "module-bottom-gameplay-ability:simulation-console",
        module_label: "Gameplay Ability",
        panel_label: "Simulation Console",
        mode_control_id: "WorkbenchGeneratedBottomModeRuntime",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.gameplay_tags_reference_scan.select",
        control_id: "WorkbenchGeneratedBottomGameplayTagsReferenceScanRow",
        panel_route: "module-bottom-gameplay-tags:reference-scan",
        module_label: "Gameplay Tags",
        panel_label: "Reference Scan",
        mode_control_id: "WorkbenchGeneratedBottomModeReview",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.gameplay_tags_migration_preview.select",
        control_id: "WorkbenchGeneratedBottomGameplayTagsMigrationPreviewRow",
        panel_route: "module-bottom-gameplay-tags:migration-preview",
        module_label: "Gameplay Tags",
        panel_label: "Migration Preview",
        mode_control_id: "WorkbenchGeneratedBottomModeValidation",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.gameplay_tags_compile_log.select",
        control_id: "WorkbenchGeneratedBottomGameplayTagsCompileLogRow",
        panel_route: "module-bottom-gameplay-tags:compile-log",
        module_label: "Gameplay Tags",
        panel_label: "Compile Log",
        mode_control_id: "WorkbenchGeneratedBottomModeBuild",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.ai_perception_debug_log.select",
        control_id: "WorkbenchGeneratedBottomAiPerceptionDebugLogRow",
        panel_route: "module-bottom-ai-perception:debug-log",
        module_label: "AI Perception",
        panel_label: "Debug Log",
        mode_control_id: "WorkbenchGeneratedBottomModeRuntime",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.ai_perception_query_output.select",
        control_id: "WorkbenchGeneratedBottomAiPerceptionQueryOutputRow",
        panel_route: "module-bottom-ai-perception:query-output",
        module_label: "AI Perception",
        panel_label: "Query Output",
        mode_control_id: "WorkbenchGeneratedBottomModeOutput",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.ai_perception_validation.select",
        control_id: "WorkbenchGeneratedBottomAiPerceptionValidationRow",
        panel_route: "module-bottom-ai-perception:validation",
        module_label: "AI Perception",
        panel_label: "Validation",
        mode_control_id: "WorkbenchGeneratedBottomModeValidation",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.ai_perception_compile_log.select",
        control_id: "WorkbenchGeneratedBottomAiPerceptionCompileLogRow",
        panel_route: "module-bottom-ai-perception:compile-log",
        module_label: "AI Perception",
        panel_label: "Compile Log",
        mode_control_id: "WorkbenchGeneratedBottomModeBuild",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.material_preview_variants.select",
        control_id: "WorkbenchGeneratedBottomMaterialPreviewVariantsRow",
        panel_route: "module-bottom-material:preview-variants",
        module_label: "Material",
        panel_label: "Preview Variants",
        mode_control_id: "WorkbenchGeneratedBottomModeReview",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.material_warnings.select",
        control_id: "WorkbenchGeneratedBottomMaterialWarningsRow",
        panel_route: "module-bottom-material:warnings",
        module_label: "Material",
        panel_label: "Warnings",
        mode_control_id: "WorkbenchGeneratedBottomModeBuild",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.behavior_tree_runtime_trace.select",
        control_id: "WorkbenchGeneratedBottomBehaviorTreeRuntimeTraceRow",
        panel_route: "module-bottom-behavior-tree:runtime-trace",
        module_label: "Behavior Tree",
        panel_label: "Runtime Trace",
        mode_control_id: "WorkbenchGeneratedBottomModeRuntime",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.behavior_tree_breakpoint_output.select",
        control_id: "WorkbenchGeneratedBottomBehaviorTreeBreakpointOutputRow",
        panel_route: "module-bottom-behavior-tree:breakpoint-output",
        module_label: "Behavior Tree",
        panel_label: "Breakpoint Output",
        mode_control_id: "WorkbenchGeneratedBottomModeOutput",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.behavior_tree_validation_issues.select",
        control_id: "WorkbenchGeneratedBottomBehaviorTreeValidationIssuesRow",
        panel_route: "module-bottom-behavior-tree:validation-issues",
        module_label: "Behavior Tree",
        panel_label: "Validation Issues",
        mode_control_id: "WorkbenchGeneratedBottomModeValidation",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.render_pipeline_compile_output.select",
        control_id: "WorkbenchGeneratedBottomRenderPipelineCompileOutputRow",
        panel_route: "module-bottom-render-pipeline:compile-output",
        module_label: "Render Pipeline",
        panel_label: "Compile Output",
        mode_control_id: "WorkbenchGeneratedBottomModeBuild",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.render_pipeline_resource_transitions.select",
        control_id: "WorkbenchGeneratedBottomRenderPipelineResourceTransitionsRow",
        panel_route: "module-bottom-render-pipeline:resource-transitions",
        module_label: "Render Pipeline",
        panel_label: "Resource Transitions",
        mode_control_id: "WorkbenchGeneratedBottomModeReview",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.render_pipeline_warnings.select",
        control_id: "WorkbenchGeneratedBottomRenderPipelineWarningsRow",
        panel_route: "module-bottom-render-pipeline:warnings",
        module_label: "Render Pipeline",
        panel_label: "Warnings",
        mode_control_id: "WorkbenchGeneratedBottomModeBuild",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.render_pipeline_errors.select",
        control_id: "WorkbenchGeneratedBottomRenderPipelineErrorsRow",
        panel_route: "module-bottom-render-pipeline:errors",
        module_label: "Render Pipeline",
        panel_label: "Errors",
        mode_control_id: "WorkbenchGeneratedBottomModeBuild",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.render_pipeline_compile_log.select",
        control_id: "WorkbenchGeneratedBottomRenderPipelineCompileLogRow",
        panel_route: "module-bottom-render-pipeline:compile-log",
        module_label: "Render Pipeline",
        panel_label: "Compile Log",
        mode_control_id: "WorkbenchGeneratedBottomModeBuild",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.asset_browser_output.select",
        control_id: "WorkbenchGeneratedBottomAssetBrowserOutputRow",
        panel_route: "module-bottom-asset-browser:output",
        module_label: "Asset Browser",
        panel_label: "Output",
        mode_control_id: "WorkbenchGeneratedBottomModeOutput",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.asset_browser_validation.select",
        control_id: "WorkbenchGeneratedBottomAssetBrowserValidationRow",
        panel_route: "module-bottom-asset-browser:validation",
        module_label: "Asset Browser",
        panel_label: "Validation",
        mode_control_id: "WorkbenchGeneratedBottomModeValidation",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.asset_browser_cook.select",
        control_id: "WorkbenchGeneratedBottomAssetBrowserCookRow",
        panel_route: "module-bottom-asset-browser:cook",
        module_label: "Asset Browser",
        panel_label: "Cook",
        mode_control_id: "WorkbenchGeneratedBottomModeBuild",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.asset_browser_package.select",
        control_id: "WorkbenchGeneratedBottomAssetBrowserPackageRow",
        panel_route: "module-bottom-asset-browser:package",
        module_label: "Asset Browser",
        panel_label: "Package",
        mode_control_id: "WorkbenchGeneratedBottomModeBuild",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.vfx_curves.select",
        control_id: "WorkbenchGeneratedBottomVfxCurvesRow",
        panel_route: "module-bottom-vfx:curves",
        module_label: "VFX",
        panel_label: "Curves",
        mode_control_id: "WorkbenchGeneratedBottomModeOutput",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.vfx_niagara_log.select",
        control_id: "WorkbenchGeneratedBottomVfxNiagaraLogRow",
        panel_route: "module-bottom-vfx:niagara-log",
        module_label: "VFX",
        panel_label: "Niagara Log",
        mode_control_id: "WorkbenchGeneratedBottomModeOutput",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.vfx_compile_output.select",
        control_id: "WorkbenchGeneratedBottomVfxCompileOutputRow",
        panel_route: "module-bottom-vfx:compile-output",
        module_label: "VFX",
        panel_label: "Compile Output",
        mode_control_id: "WorkbenchGeneratedBottomModeBuild",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.vfx_event_log.select",
        control_id: "WorkbenchGeneratedBottomVfxEventLogRow",
        panel_route: "module-bottom-vfx:event-log",
        module_label: "VFX",
        panel_label: "Event Log",
        mode_control_id: "WorkbenchGeneratedBottomModeRuntime",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.hud_editor_binding_errors.select",
        control_id: "WorkbenchGeneratedBottomHudEditorBindingErrorsRow",
        panel_route: "module-bottom-hud-editor:binding-errors",
        module_label: "HUD Editor",
        panel_label: "Binding Errors",
        mode_control_id: "WorkbenchGeneratedBottomModeValidation",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.hud_editor_preview_log.select",
        control_id: "WorkbenchGeneratedBottomHudEditorPreviewLogRow",
        panel_route: "module-bottom-hud-editor:preview-log",
        module_label: "HUD Editor",
        panel_label: "Preview Log",
        mode_control_id: "WorkbenchGeneratedBottomModeReview",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.hud_editor_performance.select",
        control_id: "WorkbenchGeneratedBottomHudEditorPerformanceRow",
        panel_route: "module-bottom-hud-editor:performance",
        module_label: "HUD Editor",
        panel_label: "Performance",
        mode_control_id: "WorkbenchGeneratedBottomModeOutput",
    },
    GeneratedBottomRouteTarget {
        action_id: "workbench.generated_bottom.hud_editor_compile_log.select",
        control_id: "WorkbenchGeneratedBottomHudEditorCompileLogRow",
        panel_route: "module-bottom-hud-editor:compile-log",
        module_label: "HUD Editor",
        panel_label: "Compile Log",
        mode_control_id: "WorkbenchGeneratedBottomModeBuild",
    },
];
