use super::support::{
    files_with_matching_line, matching_line_count, production_ui_file, read_repo_file,
    rust_files_under, top_level_entry_names,
};

#[test]
fn runtime_09_ui_architecture_doc_records_current_boundaries() {
    let architecture_doc = include_str!("../../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");

    for required_anchor in [
        "runtime_09_m0_ui_architecture_static_passed",
        "Module Boundary Map",
        "`ui/` top-level entries: 18",
        "`surface/` entries: 20",
        "No M0 blocker-level owner inversion",
        "completed_static_passed",
    ] {
        assert!(
            architecture_doc.contains(required_anchor)
                || runtime_09_plan.contains(required_anchor)
                || runtime_index.contains(required_anchor),
            "Runtime 09 M0 docs/index should retain boundary anchor `{required_anchor}`"
        );
    }

    let ui_entries = top_level_entry_names("zircon_runtime/src/ui", false);
    assert_eq!(
        ui_entries.len(),
        18,
        "Runtime 09 M0 architecture doc must be refreshed when ui/ top-level entries change"
    );
    for required_entry in [
        "accessibility",
        "binding",
        "component",
        "dispatch",
        "event_ui",
        "icon_atlas",
        "layout",
        "module.rs",
        "prelude.rs",
        "public_runtime_frame.rs",
        "style.rs",
        "surface",
        "template",
        "tests",
        "text",
        "theme",
        "tree",
        "v2",
    ] {
        assert!(
            ui_entries.iter().any(|entry| entry == required_entry),
            "Runtime 09 UI top-level map should include `{required_entry}`"
        );
    }

    let surface_entries = top_level_entry_names("zircon_runtime/src/ui/surface", true);
    assert_eq!(
        surface_entries.len(),
        20,
        "Runtime 09 M0 architecture doc must be refreshed when surface/ entries change"
    );
    for required_entry in [
        "input",
        "pointer",
        "navigation",
        "render",
        "surface.rs",
        "mod.rs",
    ] {
        assert!(
            surface_entries.iter().any(|entry| entry == required_entry),
            "Runtime 09 surface map should include `{required_entry}`"
        );
    }
}

#[test]
fn runtime_09_ui_architecture_baselines_match_current_source_scan() {
    let architecture_doc = include_str!("../../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let all_ui_files = rust_files_under("zircon_runtime/src/ui");
    let production_ui_files = all_ui_files
        .iter()
        .filter(|path| production_ui_file(path))
        .cloned()
        .collect::<Vec<_>>();

    let legacy_full_hits = matching_line_count(&all_ui_files, "legacy");
    let legacy_production_hits = matching_line_count(&production_ui_files, "legacy");
    let legacy_production_files = files_with_matching_line(&production_ui_files, "legacy");
    let taffy_production_hits = matching_line_count(&production_ui_files, "taffy");
    let taffy_production_files = files_with_matching_line(&production_ui_files, "taffy");

    assert_eq!(
        legacy_full_hits, 54,
        "update Runtime 09 docs if full legacy baseline changes"
    );
    assert_eq!(
        legacy_production_hits, 0,
        "update Runtime 09 docs if production legacy hit baseline changes"
    );
    assert_eq!(
        legacy_production_files.len(),
        0,
        "update Runtime 09 docs if production legacy file baseline changes"
    );
    assert_eq!(
        taffy_production_hits, 175,
        "update Runtime 09 docs if production taffy hit baseline changes"
    );
    assert_eq!(
        taffy_production_files.len(),
        10,
        "update Runtime 09 docs if production taffy file baseline changes"
    );

    for required_anchor in [
        "ui_legacy_hits=54",
        "ui_legacy_production_hits=0",
        "ui_legacy_production_files=0",
        "ui_taffy_production_hits=175",
        "ui_taffy_production_files=10",
    ] {
        assert!(
            architecture_doc.contains(required_anchor)
                || runtime_09_plan.contains(required_anchor)
                || runtime_index.contains(required_anchor),
            "Runtime 09 docs/index should retain source-scan baseline `{required_anchor}`"
        );
    }
}

#[test]
fn runtime_09_v2_verdict_matches_runtime_and_interface_modules() {
    let architecture_doc = include_str!("../../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let runtime_v2_mod = read_repo_file("zircon_runtime/src/ui/v2/mod.rs");
    let interface_v2_mod = read_repo_file("zircon_runtime_interface/src/ui/v2/mod.rs");

    for required_runtime_anchor in [
        "mod cache;",
        "mod compiler;",
        "mod file_cache;",
        "mod loader;",
        "mod style;",
        "mod surface_builder;",
        "mod surface_tree;",
        "UiV2PrototypeStoreFileCache",
        "UiV2SurfaceBuilder",
        "UiZuiAssetLoader",
    ] {
        assert!(
            runtime_v2_mod.contains(required_runtime_anchor),
            "runtime ui::v2 module should retain `{required_runtime_anchor}`"
        );
    }

    for required_interface_anchor in [
        "mod arena;",
        "mod asset;",
        "mod compiled;",
        "mod graph;",
        "mod repeat;",
        "mod style;",
        "UiV2AssetDocument",
        "UiV2CompiledDocument",
        "UiV2ResolvedStyle",
    ] {
        assert!(
            interface_v2_mod.contains(required_interface_anchor),
            "interface ui::v2 module should retain `{required_interface_anchor}`"
        );
    }

    for required_verdict_anchor in [
        "v2-replacement-mainline",
        ".zui",
        ".v2.ui.toml",
        "replacement mainline",
        "migration/test-only",
        "old recursive template",
    ] {
        assert!(
            architecture_doc.contains(required_verdict_anchor)
                || runtime_09_plan.contains(required_verdict_anchor)
                || runtime_index.contains(required_verdict_anchor),
            "Runtime 09 docs/index should retain v2 verdict anchor `{required_verdict_anchor}`"
        );
    }
}

#[test]
fn runtime_09_taffy_layout_pass_order_uses_bridge_authority() {
    let layout_mod = read_repo_file("zircon_runtime/src/ui/layout/mod.rs");
    let pass_mod = read_repo_file("zircon_runtime/src/ui/layout/pass/mod.rs");
    let pipeline = read_repo_file("zircon_runtime/src/ui/layout/pass/pipeline.rs");
    let layout_tree = read_repo_file("zircon_runtime/src/ui/layout/pass/layout_tree.rs");
    let incremental = read_repo_file("zircon_runtime/src/ui/layout/pass/incremental.rs");
    let taffy_arrange = read_repo_file("zircon_runtime/src/ui/layout/pass/taffy_arrange.rs");
    let taffy_bridge_mod = read_repo_file("zircon_runtime/src/ui/layout/taffy_bridge/mod.rs");
    let taffy_bridge_compute =
        read_repo_file("zircon_runtime/src/ui/layout/taffy_bridge/compute.rs");
    let style_mapping = read_repo_file("zircon_runtime/src/ui/layout/style_mapping.rs");
    let architecture_doc = include_str!("../../../../../docs/zircon_runtime/ui/architecture.md");
    let layout_pass_doc = include_str!("../../../../../docs/zircon_runtime/ui/layout/pass.md");
    let v2_doc = include_str!("../../../../../docs/zircon_runtime/ui/v2.md");
    let runtime_09_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let status_anchor = "runtime_09_m2_1_taffy_bridge_pass_order_static_passed_cargo_pending";
    let dto_anchor = "runtime_09_m2_1_style_mapping_remains_taffy_dto_adapter";

    for required_anchor in [
        "UI_LAYOUT_PASS_ORDER",
        "UiLayoutPassStage::ResponsiveStyleResolution",
        "UiLayoutPassStage::Measurement",
        "UiLayoutPassStage::BackendSelection",
        "UiLayoutPassStage::TaffyBridgeArrangement",
        "UiLayoutPassStage::ZirconFallbackArrangement",
        "UiLayoutPassStage::ClipAndVirtualWindowPropagation",
        "UiLayoutPassStage::SelectionReport",
        "ui_layout_pass_stage_names",
        "assert_layout_pass_stage",
    ] {
        assert!(
            pipeline.contains(required_anchor),
            "Runtime 09 M2.1 pipeline should retain `{required_anchor}`"
        );
    }

    for (file_name, source) in [
        ("layout_tree.rs", layout_tree.as_str()),
        ("incremental.rs", incremental.as_str()),
    ] {
        for required_anchor in [
            "assert_layout_pass_stage(UiLayoutPassStage::ResponsiveStyleResolution, 0)",
            "assert_layout_pass_stage(UiLayoutPassStage::Measurement, 1)",
            "assert_layout_pass_stage(UiLayoutPassStage::BackendSelection, 2)",
            "assert_layout_pass_stage(UiLayoutPassStage::TaffyBridgeArrangement, 3)",
            "assert_layout_pass_stage(UiLayoutPassStage::ZirconFallbackArrangement, 4)",
            "assert_layout_pass_stage(UiLayoutPassStage::ClipAndVirtualWindowPropagation, 5)",
            "assert_layout_pass_stage(UiLayoutPassStage::SelectionReport, 6)",
        ] {
            assert!(
                source.contains(required_anchor),
                "{file_name} should consume authoritative layout pass stage `{required_anchor}`"
            );
        }
    }

    assert!(
        layout_mod.contains("ui_layout_pass_stage_names")
            && layout_mod.contains("UI_LAYOUT_PASS_ORDER")
            && pass_mod.contains("mod pipeline;")
            && pass_mod.contains("pub use pipeline::"),
        "layout/pass modules should expose the authoritative layout pass order"
    );
    assert!(
        taffy_arrange.contains("compute_taffy_child_frames")
            && taffy_arrange.contains("TaffyChildLayoutInput")
            && !taffy_arrange.contains("TaffyTree::new")
            && !taffy_arrange.contains(".compute_layout(")
            && !taffy_arrange.contains("use taffy::"),
        "pass/taffy_arrange.rs should dispatch through the bridge instead of owning Taffy compute"
    );
    assert!(
        taffy_bridge_mod.contains("mod compute;")
            && taffy_bridge_mod.contains("taffy_style_for_container")
            && taffy_bridge_compute.contains("pub(crate) fn compute_taffy_child_frames")
            && taffy_bridge_compute.contains("TaffyTree::new()")
            && taffy_bridge_compute.contains("disable_rounding")
            && taffy_bridge_compute.contains(".compute_layout(")
            && taffy_bridge_compute.contains("TaffyChildLayoutInput"),
        "taffy_bridge should own Taffy tree build and compute"
    );
    assert!(
        style_mapping.contains("taffy_style_from_ui_layout_style")
            && style_mapping.contains("UiLayoutStyle"),
        "style_mapping should remain the Taffy DTO adapter documented by M2.1"
    );

    for (doc_name, doc_source) in [
        ("UI architecture doc", architecture_doc),
        ("layout pass doc", layout_pass_doc),
        ("v2 doc", v2_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
    ] {
        for required_anchor in [
            status_anchor,
            dto_anchor,
            "UI_LAYOUT_PASS_ORDER",
            "compute_taffy_child_frames",
        ] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should record Runtime 09 M2.1 Taffy bridge/pass-order anchor `{required_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_09_virtualization_scroll_boundary_records_invalidation_authority() {
    let layout_mod = read_repo_file("zircon_runtime/src/ui/layout/mod.rs");
    let layout_scroll = read_repo_file("zircon_runtime/src/ui/layout/scroll.rs");
    let layout_virtualization = read_repo_file("zircon_runtime/src/ui/layout/virtualization.rs");
    let arrange = read_repo_file("zircon_runtime/src/ui/layout/pass/arrange.rs");
    let tree_scroll = read_repo_file("zircon_runtime/src/ui/tree/node/scroll.rs");
    let scroll_virtualization_test =
        read_repo_file("zircon_runtime/src/ui/tests/scroll_virtualization.rs");
    let architecture_doc = include_str!("../../../../../docs/zircon_runtime/ui/architecture.md");
    let layout_pass_doc = include_str!("../../../../../docs/zircon_runtime/ui/layout/pass.md");
    let runtime_09_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let status_anchor =
        "runtime_09_m2_2_virtualization_scroll_boundary_static_passed_cargo_pending";

    for required_anchor in [
        "UiScrollVirtualizationPlan",
        "plan_scrollable_virtual_window",
        "visible_range_changed",
        "virtualization_enabled",
        "previous_state.viewport_extent",
        "previous_state.content_extent",
        "virtual_window_for_scrollable_box",
    ] {
        assert!(
            layout_scroll.contains(required_anchor),
            "layout/scroll.rs should retain Runtime 09 M2.2 virtualization/scroll owner anchor `{required_anchor}`"
        );
    }
    assert!(
        layout_mod.contains("plan_scrollable_virtual_window")
            && layout_virtualization.contains("compute_virtual_list_window"),
        "layout module should expose the scroll virtualization planner and keep window math in virtualization.rs"
    );
    assert!(
        arrange.contains("plan_scrollable_virtual_window")
            && arrange.contains("node.layout_cache.virtual_window = Some(visible_window)")
            && arrange.contains("node.dirty.visible_range |= plan.visible_range_changed")
            && arrange.contains("hide_subtree_layout"),
        "layout arrange should consume the scroll virtualization plan and cache the resulting visible window"
    );
    assert!(
        tree_scroll.contains("plan_scrollable_virtual_window")
            && tree_scroll.contains("node.dirty.visible_range |= plan.visible_range_changed")
            && !tree_scroll.contains("node.dirty.visible_range = previous_window"),
        "tree scroll mutation should OR the planner's visible-range invalidation instead of overwriting existing dirty state"
    );
    for test_anchor in [
        "virtualized_list_only_materializes_visible_window",
        "scroll_offset_invalidates_virtualization_window",
        "non_virtualized_scroll_offset_keeps_full_window_dirty_domain",
    ] {
        assert!(
            scroll_virtualization_test.contains(test_anchor),
            "scroll virtualization tests should retain `{test_anchor}`"
        );
    }

    for (doc_name, doc_source) in [
        ("UI architecture doc", architecture_doc),
        ("layout pass doc", layout_pass_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
    ] {
        for required_anchor in [
            status_anchor,
            "UiScrollVirtualizationPlan",
            "plan_scrollable_virtual_window",
            "virtualized_list_only_materializes_visible_window",
            "scroll_offset_invalidates_virtualization_window",
            "non_virtualized_scroll_offset_keeps_full_window_dirty_domain",
        ] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should record Runtime 09 M2.2 virtualization/scroll anchor `{required_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_09_template_pipeline_boundary_records_compile_instance_validate_authority() {
    let template_mod = read_repo_file("zircon_runtime/src/ui/template/mod.rs");
    let pipeline = read_repo_file("zircon_runtime/src/ui/template/pipeline.rs");
    let loader = read_repo_file("zircon_runtime/src/ui/template/loader.rs");
    let validate = read_repo_file("zircon_runtime/src/ui/template/validate.rs");
    let instance = read_repo_file("zircon_runtime/src/ui/template/instance.rs");
    let surface_builder = read_repo_file("zircon_runtime/src/ui/template/build/surface_builder.rs");
    let artifact =
        read_repo_file("zircon_runtime/src/ui/template/asset/compiler/package/artifact.rs");
    let template_pipeline_test = read_repo_file("zircon_runtime/src/ui/tests/template_pipeline.rs");
    let architecture_doc = include_str!("../../../../../docs/zircon_runtime/ui/architecture.md");
    let template_pipeline_doc =
        include_str!("../../../../../docs/zircon_runtime/ui/template/pipeline.md");
    let shared_template_doc =
        include_str!("../../../../../docs/ui-and-layout/shared-ui-template-runtime.md");
    let runtime_09_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let generated_boundary_doc =
        include_str!("../../../../../docs/engine-architecture/generated-code-boundary.md");
    let status_anchor =
        "runtime_09_m3_1_template_compile_instance_validate_boundary_static_passed_cargo_pending";
    let artifact_policy = "runtime_09_m3_1_binary_leaf_dto_artifact_not_generated_source";
    let generated_marker = "// @generated <generator> - do not edit by hand";

    assert!(
        template_mod.contains("mod pipeline;")
            && template_mod.contains("UiTemplateRuntimePipeline")
            && template_mod.contains("UI_TEMPLATE_RUNTIME_PIPELINE_STAGES"),
        "template root should expose the Runtime 09 M3.1 pipeline boundary"
    );
    for pipeline_anchor in [
        "UI_TEMPLATE_RUNTIME_PIPELINE_STAGES",
        r#"["load", "validate", "instance", "build"]"#,
        "UiTemplateRuntimePipelineError::Load",
        "UiTemplateRuntimePipelineError::Validate",
        "UiTemplateRuntimePipelineError::Instance",
        "UiTemplateRuntimePipelineError::Build",
        "load_document_from_toml_str",
        "instantiate_document",
        "build_surface_from_document",
        "build_surface_from_toml_str",
    ] {
        assert!(
            pipeline.contains(pipeline_anchor),
            "template pipeline should retain `{pipeline_anchor}`"
        );
    }
    assert!(
        loader.contains("UiTemplateLoader")
            && validate.contains("UiTemplateValidator")
            && instance.contains("from_validated_document")
            && surface_builder.contains("UiTemplateSurfaceBuilder"),
        "load/validate/instance/build modules should keep their explicit Runtime 09 M3.1 roles"
    );
    for artifact_anchor in [
        "UI_COMPILED_ASSET_ARTIFACT_GENERATED_POLICY",
        "UI_COMPILED_ASSET_ARTIFACT_GENERATED_SOURCE_MARKER_REQUIRED",
        artifact_policy,
        "generated_policy",
        "requires_generated_source_marker",
    ] {
        assert!(
            artifact.contains(artifact_anchor),
            "compiled template artifact should retain generated policy anchor `{artifact_anchor}`"
        );
    }
    for test_anchor in [
        "template_validate_rejects_unknown_component_contract",
        "template_instance_failure_surfaces_loader_error",
        "compiled_template_artifact_stays_binary_leaf_dto_not_generated_source",
    ] {
        assert!(
            template_pipeline_test.contains(test_anchor),
            "template pipeline tests should retain `{test_anchor}`"
        );
    }

    for (doc_name, doc_source) in [
        ("UI architecture doc", architecture_doc),
        ("template pipeline doc", template_pipeline_doc),
        ("shared template runtime doc", shared_template_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
    ] {
        for required_anchor in [
            status_anchor,
            "UI_TEMPLATE_RUNTIME_PIPELINE_STAGES",
            "UiTemplateRuntimePipeline",
            "UiTemplateRuntimePipelineError",
            "template_validate_rejects_unknown_component_contract",
            "template_instance_failure_surfaces_loader_error",
            artifact_policy,
            "compiled_template_artifact_stays_binary_leaf_dto_not_generated_source",
            generated_marker,
        ] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should record Runtime 09 M3.1 template boundary anchor `{required_anchor}`"
            );
        }
    }
    assert!(
        generated_boundary_doc.contains(generated_marker),
        "generated-code boundary doc should retain the first-line generated source marker"
    );
}
