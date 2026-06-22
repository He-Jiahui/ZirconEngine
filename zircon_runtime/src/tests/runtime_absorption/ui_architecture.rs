use std::ffi::OsStr;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime manifest should live under the repository root")
        .to_path_buf()
}

fn repo_path(relative: &str) -> PathBuf {
    repo_root().join(relative)
}

fn read_repo_file(relative: &str) -> String {
    let path = repo_path(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn top_level_entry_names(relative: &str, include_root_mod: bool) -> Vec<String> {
    let dir = repo_path(relative);
    let mut entries = std::fs::read_dir(&dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| {
            let entry = entry.unwrap_or_else(|error| panic!("failed to read dir entry: {error}"));
            entry.file_name().into_string().unwrap_or_else(|name| {
                panic!("non-utf8 filename under {}: {name:?}", dir.display())
            })
        })
        .filter(|name| include_root_mod || name != "mod.rs")
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

fn rust_files_under(relative: &str) -> Vec<PathBuf> {
    let mut pending = vec![repo_path(relative)];
    let mut files = Vec::new();

    while let Some(path) = pending.pop() {
        for entry in std::fs::read_dir(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
        {
            let entry = entry.unwrap_or_else(|error| panic!("failed to read dir entry: {error}"));
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }

    files.sort();
    files
}

fn has_component(path: &Path, component: &str) -> bool {
    let component = OsStr::new(component);
    path.components()
        .any(|path_component| path_component.as_os_str() == component)
}

fn production_ui_file(path: &Path) -> bool {
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    !has_component(path, "tests")
        && !has_component(path, "test_fixtures")
        && filename != "tests.rs"
        && !filename.ends_with("_tests.rs")
}

fn matching_line_count(files: &[PathBuf], needle: &str) -> usize {
    files
        .iter()
        .map(|path| {
            std::fs::read_to_string(path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .lines()
                .filter(|line| line.contains(needle))
                .count()
        })
        .sum()
}

fn files_with_matching_line(files: &[PathBuf], needle: &str) -> Vec<PathBuf> {
    files
        .iter()
        .filter(|path| {
            std::fs::read_to_string(path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .lines()
                .any(|line| line.contains(needle))
        })
        .cloned()
        .collect()
}

#[test]
fn runtime_09_ui_architecture_doc_records_current_boundaries() {
    let architecture_doc = include_str!("../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");

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
    let architecture_doc = include_str!("../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
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
    let architecture_doc = include_str!("../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
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
    let architecture_doc = include_str!("../../../../docs/zircon_runtime/ui/architecture.md");
    let layout_pass_doc = include_str!("../../../../docs/zircon_runtime/ui/layout/pass.md");
    let v2_doc = include_str!("../../../../docs/zircon_runtime/ui/v2.md");
    let runtime_09_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
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
    let architecture_doc = include_str!("../../../../docs/zircon_runtime/ui/architecture.md");
    let layout_pass_doc = include_str!("../../../../docs/zircon_runtime/ui/layout/pass.md");
    let runtime_09_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
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
    let architecture_doc = include_str!("../../../../docs/zircon_runtime/ui/architecture.md");
    let template_pipeline_doc =
        include_str!("../../../../docs/zircon_runtime/ui/template/pipeline.md");
    let shared_template_doc =
        include_str!("../../../../docs/ui-and-layout/shared-ui-template-runtime.md");
    let runtime_09_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let generated_boundary_doc =
        include_str!("../../../../docs/engine-architecture/generated-code-boundary.md");
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

#[test]
fn runtime_09_navigation_legacy_reply_rename_reduces_ui_input_debt() {
    let navigation_input = read_repo_file("zircon_runtime/src/ui/surface/input/navigation.rs");
    let architecture_doc = include_str!("../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let status_anchor =
        "runtime_09_m1_2_navigation_legacy_reply_renamed_static_passed_cargo_pending";

    assert!(
        navigation_input.contains("routed_reply"),
        "Runtime 09 M1.2 should use semantic navigation route reply naming"
    );
    assert!(
        !navigation_input.contains("legacy"),
        "Runtime 09 M1.2 should remove legacy wording from navigation route reply code"
    );

    for (doc_name, doc_source) in [
        ("UI architecture doc", architecture_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
    ] {
        assert!(
            doc_source.contains(status_anchor),
            "{doc_name} should record Runtime 09 M1.2 navigation legacy reply rename status"
        );
    }
}

#[test]
fn runtime_09_pointer_legacy_reply_rename_reduces_ui_input_debt() {
    let pointer_input = read_repo_file("zircon_runtime/src/ui/surface/input/pointer.rs");
    let pointer_reply = read_repo_file("zircon_runtime/src/ui/surface/input/pointer_reply.rs");
    let architecture_doc = include_str!("../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let input_doc = include_str!("../../../../docs/zircon_runtime/ui/surface/input.md");
    let status_anchor = "runtime_09_m1_2_pointer_legacy_reply_renamed_static_passed_cargo_pending";
    let guard_anchor = "runtime_09_pointer_legacy_reply_rename_reduces_ui_input_debt";

    for (file_name, file_source) in [
        ("pointer.rs", pointer_input.as_str()),
        ("pointer_reply.rs", pointer_reply.as_str()),
    ] {
        assert!(
            file_source.contains("routed_result"),
            "Runtime 09 M1.2 should use semantic routed_result naming in {file_name}"
        );
        assert!(
            !file_source.contains("legacy"),
            "Runtime 09 M1.2 should remove legacy wording from {file_name}"
        );
    }

    for (doc_name, doc_source) in [
        ("UI architecture doc", architecture_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
        ("surface input doc", input_doc),
    ] {
        for required_anchor in [status_anchor, guard_anchor, "routed_result"] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should record Runtime 09 M1.2 pointer legacy reply rename anchor `{required_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_09_pointer_capture_fallback_rename_reduces_ui_input_debt() {
    let pointer_capture =
        read_repo_file("zircon_runtime/src/ui/surface/input/state/pointer_capture.rs");
    let focus_pointer =
        read_repo_file("zircon_runtime/src/ui/surface/input/effect/focus_pointer.rs");
    let architecture_doc = include_str!("../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let input_doc = include_str!("../../../../docs/zircon_runtime/ui/surface/input.md");
    let status_anchor =
        "runtime_09_m1_2_pointer_capture_fallback_renamed_static_passed_cargo_pending";
    let guard_anchor = "runtime_09_pointer_capture_fallback_rename_reduces_ui_input_debt";
    let semantic_name = "has_pointer_capture_or_unindexed_fallback_for_owner";

    for (file_name, file_source) in [
        ("state/pointer_capture.rs", pointer_capture.as_str()),
        ("effect/focus_pointer.rs", focus_pointer.as_str()),
    ] {
        assert!(
            file_source.contains(semantic_name),
            "Runtime 09 M1.2 should use semantic pointer capture fallback naming in {file_name}"
        );
        assert!(
            !file_source.contains("has_legacy_or_indexed_pointer_capture_for_owner"),
            "Runtime 09 M1.2 should remove legacy wording from the pointer capture fallback API in {file_name}"
        );
    }

    for (doc_name, doc_source) in [
        ("UI architecture doc", architecture_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
        ("surface input doc", input_doc),
    ] {
        for required_anchor in [status_anchor, guard_anchor, semantic_name] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should record Runtime 09 M1.2 pointer capture fallback rename anchor `{required_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_09_table_row_label_fallback_rename_reduces_ui_render_debt() {
    let table_rows =
        read_repo_file("zircon_runtime/src/ui/surface/render/collection_rows/table.rs");
    let architecture_doc = include_str!("../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let status_anchor =
        "runtime_09_m1_2_table_row_label_fallback_renamed_static_passed_cargo_pending";
    let guard_anchor = "runtime_09_table_row_label_fallback_rename_reduces_ui_render_debt";
    let semantic_name = "split_row_label_table_text";

    assert!(
        table_rows.contains(semantic_name),
        "Runtime 09 M1.2 should use semantic row-label fallback table splitting"
    );
    assert!(
        !table_rows.contains("split_legacy_table_text"),
        "Runtime 09 M1.2 should remove legacy wording from table row-label fallback splitting"
    );

    for (doc_name, doc_source) in [
        ("UI architecture doc", architecture_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
    ] {
        for required_anchor in [status_anchor, guard_anchor, semantic_name] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should record Runtime 09 M1.2 table row-label fallback rename anchor `{required_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_09_template_component_name_fallback_rename_reduces_ui_template_debt() {
    let interaction = read_repo_file("zircon_runtime/src/ui/template/build/interaction.rs");
    let architecture_doc = include_str!("../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let status_anchor =
        "runtime_09_m1_2_template_component_name_fallback_renamed_static_passed_cargo_pending";
    let guard_anchor =
        "runtime_09_template_component_name_fallback_rename_reduces_ui_template_debt";
    let semantic_name = "component_name_interaction_fallback";

    assert!(
        interaction.contains(semantic_name),
        "Runtime 09 M1.2 should name the template fallback after component-name inference"
    );
    for retired_name in [
        "legacy_component_interaction_fallback",
        "legacy_interactive",
    ] {
        assert!(
            !interaction.contains(retired_name),
            "Runtime 09 M1.2 should remove `{retired_name}` from template interaction inference"
        );
    }

    for (doc_name, doc_source) in [
        ("UI architecture doc", architecture_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
    ] {
        for required_anchor in [status_anchor, guard_anchor, semantic_name] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should record Runtime 09 M1.2 template component-name fallback rename anchor `{required_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_09_property_visibility_flag_rename_reduces_ui_surface_debt() {
    let property_mutation = read_repo_file("zircon_runtime/src/ui/surface/property_mutation.rs");
    let property_mutation_doc =
        include_str!("../../../../docs/zircon_runtime/ui/surface/property_mutation.md");
    let architecture_doc = include_str!("../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let status_anchor =
        "runtime_09_m1_2_property_visibility_flag_renamed_static_passed_cargo_pending";
    let guard_anchor = "runtime_09_property_visibility_flag_rename_reduces_ui_surface_debt";
    let semantic_name = "state_visible_flag";

    assert!(
        property_mutation.contains(semantic_name),
        "Runtime 09 M1.2 should name the visibility transition input after the state visible flag"
    );
    assert!(
        !property_mutation.contains("legacy_visible"),
        "Runtime 09 M1.2 should remove legacy wording from property mutation visibility transition"
    );

    for (doc_name, doc_source) in [
        ("property mutation doc", property_mutation_doc),
        ("UI architecture doc", architecture_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
    ] {
        for required_anchor in [status_anchor, guard_anchor, semantic_name] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should record Runtime 09 M1.2 property visibility flag rename anchor `{required_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_09_responsive_mui_visibility_flag_rename_reduces_ui_layout_debt() {
    let responsive_mui = read_repo_file("zircon_runtime/src/ui/layout/pass/responsive_mui.rs");
    let layout_pass_doc = include_str!("../../../../docs/zircon_runtime/ui/layout/pass.md");
    let architecture_doc = include_str!("../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let status_anchor =
        "runtime_09_m1_2_responsive_mui_visibility_flag_renamed_static_passed_cargo_pending";
    let guard_anchor = "runtime_09_responsive_mui_visibility_flag_rename_reduces_ui_layout_debt";
    let semantic_name = "state_visible_flag";

    assert!(
        responsive_mui.contains(semantic_name),
        "Runtime 09 M1.2 should name responsive visible input after the state visible flag"
    );
    assert!(
        !responsive_mui.contains("legacy_visible"),
        "Runtime 09 M1.2 should remove legacy wording from responsive MUI visibility DTO"
    );

    for (doc_name, doc_source) in [
        ("layout pass doc", layout_pass_doc),
        ("UI architecture doc", architecture_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
    ] {
        for required_anchor in [status_anchor, guard_anchor, semantic_name] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should record Runtime 09 M1.2 responsive MUI visibility flag rename anchor `{required_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_09_accessibility_open_state_fallback_rename_reduces_ui_a11y_debt() {
    let accessibility_extract = read_repo_file("zircon_runtime/src/ui/accessibility/extract.rs");
    let accessibility_doc = include_str!("../../../../docs/zircon_runtime/ui/accessibility.md");
    let architecture_doc = include_str!("../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let status_anchor =
        "runtime_09_m1_2_accessibility_open_state_fallback_renamed_static_passed_cargo_pending";
    let guard_anchor = "runtime_09_accessibility_open_state_fallback_rename_reduces_ui_a11y_debt";
    let semantic_name = "fallback_properties";

    assert!(
        accessibility_extract.contains(semantic_name),
        "Runtime 09 M1.2 should name accessibility open-state alternatives as fallback properties"
    );
    assert!(
        !accessibility_extract.contains("legacy_properties"),
        "Runtime 09 M1.2 should remove legacy wording from accessibility open-state fallback properties"
    );
    assert!(
        !accessibility_extract.contains("legacy_property"),
        "Runtime 09 M1.2 should remove legacy wording from accessibility open-state fallback locals"
    );

    for (doc_name, doc_source) in [
        ("accessibility doc", accessibility_doc),
        ("UI architecture doc", architecture_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
    ] {
        for required_anchor in [status_anchor, guard_anchor, semantic_name] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should record Runtime 09 M1.2 accessibility fallback rename anchor `{required_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_09_layout_engine_backend_name_cutover_reduces_ui_layout_debt() {
    let layout_engine_contract = read_repo_file("zircon_runtime_interface/src/ui/layout/engine.rs");
    let layout_pass_engine = read_repo_file("zircon_runtime/src/ui/layout/pass/engine.rs");
    let layout_pass_doc = include_str!("../../../../docs/zircon_runtime/ui/layout/pass.md");
    let architecture_doc = include_str!("../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let status_anchor =
        "runtime_09_m1_2_layout_engine_backend_name_cutover_static_passed_cargo_pending";
    let guard_anchor = "runtime_09_layout_engine_backend_name_cutover_reduces_ui_layout_debt";

    for forbidden_name in ["LegacyZircon", "legacy_zircon", "legacy_selected_count"] {
        for (file_name, file_source) in [
            ("layout engine contract", layout_engine_contract.as_str()),
            ("layout pass engine", layout_pass_engine.as_str()),
        ] {
            assert!(
                !file_source.contains(forbidden_name),
                "Runtime 09 M1.2 should remove old layout engine backend name `{forbidden_name}` from {file_name}"
            );
        }
    }

    assert!(
        layout_engine_contract.contains("UiLayoutEngineBackend::Zircon")
            && layout_engine_contract.contains("pub fn zircon()"),
        "Runtime 09 M1.2 layout engine contract should retain the Zircon backend and constructor"
    );
    assert!(
        layout_pass_engine.contains("UiLayoutEngineBackend::Zircon")
            && layout_pass_engine.contains("UiLayoutEngineCapability::zircon()"),
        "Runtime 09 M1.2 layout pass should consume the Zircon backend constructor"
    );
    assert!(
        layout_engine_contract.contains("zircon_selected_count"),
        "Runtime 09 M1.2 layout engine report should expose zircon_selected_count"
    );

    for (doc_name, doc_source) in [
        ("layout pass doc", layout_pass_doc),
        ("UI architecture doc", architecture_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
    ] {
        for required_anchor in [
            status_anchor,
            guard_anchor,
            "UiLayoutEngineBackend::Zircon",
            "UiLayoutEngineCapability::zircon",
            "zircon_selected_count",
        ] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should record Runtime 09 M1.2 layout engine backend cutover anchor `{required_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_09_surface_default_interaction_fallback_rename_reduces_ui_surface_debt() {
    let default_interactions =
        read_repo_file("zircon_runtime/src/ui/surface/surface/default_interactions.rs");
    let default_interactions_doc =
        include_str!("../../../../docs/zircon_runtime/ui/surface/default_interactions.md");
    let architecture_doc = include_str!("../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let status_anchor =
        "runtime_09_m1_2_surface_default_interaction_fallback_renamed_static_passed_cargo_pending";
    let guard_anchor =
        "runtime_09_surface_default_interaction_fallback_rename_reduces_ui_surface_debt";
    let semantic_name = "fallback_properties";

    assert!(
        default_interactions.contains("fn default_open_boolean_value("),
        "Runtime 09 M1.2 should keep default open-state fallback lookup in default_interactions"
    );
    assert!(
        default_interactions.contains(semantic_name)
            && default_interactions.contains("fallback_property"),
        "Runtime 09 M1.2 should name default interaction open-state alternatives as fallback properties"
    );
    for retired_name in ["legacy_properties", "legacy_property"] {
        assert!(
            !default_interactions.contains(retired_name),
            "Runtime 09 M1.2 should remove `{retired_name}` from default interaction open-state fallback lookup"
        );
    }

    for (doc_name, doc_source) in [
        ("default interactions doc", default_interactions_doc),
        ("UI architecture doc", architecture_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
    ] {
        for required_anchor in [
            status_anchor,
            guard_anchor,
            "default_open_boolean_value",
            semantic_name,
        ] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should record Runtime 09 M1.2 default interaction fallback rename anchor `{required_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_09_ui_input_events_route_through_single_dispatch_authority() {
    let dispatch_input = read_repo_file("zircon_runtime/src/ui/surface/input/dispatch.rs");
    let input_mod = read_repo_file("zircon_runtime/src/ui/surface/input/mod.rs");
    let route_authority = read_repo_file("zircon_runtime/src/ui/surface/input/route_authority.rs");
    let surface = read_repo_file("zircon_runtime/src/ui/surface/surface.rs");
    let runtime_manager =
        read_repo_file("zircon_runtime/src/ui/tests/runtime_ui_support/runtime_ui_manager.rs");
    let architecture_doc = include_str!("../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let status_anchor = "runtime_09_m1_1_ui_input_route_authority_static_passed_cargo_pending";
    let bypass_verdict = "runtime_09_m1_1_direct_pointer_navigation_routes_are_leaf_owner_helpers";

    for dispatch_anchor in [
        "mod route_authority;",
        "annotate_authoritative_input_dispatch",
        "dispatch_pointer_input",
        "dispatch_navigation_input",
        "dispatch_keyboard_input",
        "dispatch_drag_drop_input",
        "Ok(result)",
    ] {
        assert!(
            dispatch_input.contains(dispatch_anchor) || input_mod.contains(dispatch_anchor),
            "unified UiInputEvent dispatch should retain `{dispatch_anchor}`"
        );
    }

    for authority_anchor in [
        "runtime_09_m1_1_ui_input_route_authority",
        "route_authority=",
        "UI_INPUT_ROUTE_ORDER",
        "UiInputRouteStage::PointerCapture",
        "UiInputRouteStage::PopupStack",
        "UiInputRouteStage::PreviewTunnel",
        "UiInputRouteStage::DirectTarget",
        "UiInputRouteStage::BubblePath",
        "UiInputRouteStage::FocusPath",
        "UiInputRouteStage::DefaultAction",
        "route_authority_stage_names_for_policy",
    ] {
        assert!(
            route_authority.contains(authority_anchor),
            "Runtime 09 M1.1 route authority module should retain `{authority_anchor}`"
        );
    }

    assert!(
        surface.contains("pub fn dispatch_input_event(")
            && runtime_manager.contains("pub(crate) fn dispatch_input_event("),
        "surface/runtime_ui_support should keep UiInputEvent dispatch as the normalized input entry"
    );
    assert!(
        surface.contains("pub fn dispatch_pointer_event(")
            && surface.contains("pub fn dispatch_navigation_event(")
            && runtime_manager.contains("pub(crate) fn dispatch_pointer_event(")
            && runtime_manager.contains("pub(crate) fn dispatch_navigation_event("),
        "direct pointer/navigation helpers remain visible and need the documented owner verdict"
    );

    for (doc_name, doc_source) in [
        ("UI architecture doc", architecture_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
    ] {
        for required_anchor in [status_anchor, bypass_verdict] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should record Runtime 09 M1.1 route authority anchor `{required_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts() {
    let architecture_doc = include_str!("../../../../docs/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = include_str!(
        "../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    let runtime_index = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");
    let architecture_review =
        include_str!("../../../../docs/engine-architecture/runtime-architecture-review-m0.md");
    let interface_doc =
        include_str!("../../../../docs/engine-architecture/runtime-interface-convergence.md");
    let audit_script = include_str!(
        "../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_boundary.py"
    );
    let ui_guard = include_str!("ui_architecture.rs");
    let cargo_gate_guard = include_str!("plan_status/cargo_gates/middle.rs");

    for guard_anchor in [
        "runtime_09_ui_architecture_doc_records_current_boundaries",
        "runtime_09_ui_architecture_baselines_match_current_source_scan",
        "runtime_09_v2_verdict_matches_runtime_and_interface_modules",
        "runtime_09_navigation_legacy_reply_rename_reduces_ui_input_debt",
        "runtime_09_pointer_legacy_reply_rename_reduces_ui_input_debt",
        "runtime_09_pointer_capture_fallback_rename_reduces_ui_input_debt",
        "runtime_09_table_row_label_fallback_rename_reduces_ui_render_debt",
        "runtime_09_template_component_name_fallback_rename_reduces_ui_template_debt",
        "runtime_09_property_visibility_flag_rename_reduces_ui_surface_debt",
        "runtime_09_responsive_mui_visibility_flag_rename_reduces_ui_layout_debt",
        "runtime_09_accessibility_open_state_fallback_rename_reduces_ui_a11y_debt",
        "runtime_09_layout_engine_backend_name_cutover_reduces_ui_layout_debt",
        "runtime_09_surface_default_interaction_fallback_rename_reduces_ui_surface_debt",
        "runtime_09_ui_input_events_route_through_single_dispatch_authority",
        "runtime_09_taffy_layout_pass_order_uses_bridge_authority",
        "runtime_09_virtualization_scroll_boundary_records_invalidation_authority",
        "runtime_09_template_pipeline_boundary_records_compile_instance_validate_authority",
        "runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts",
        "runtime_09_ui_architecture_cargo_gate_stays_visible_until_ui_owner_validation",
    ] {
        assert!(
            ui_guard.contains(guard_anchor) || cargo_gate_guard.contains(guard_anchor),
            "Runtime 09 guard anchor `{guard_anchor}` should stay visible to ui_architecture_boundary"
        );
    }

    for audit_anchor in [
        "EXPECTED_SOURCE_FILE_COUNT = 52",
        "EXPECTED_UI_ENTRY_COUNT = 18",
        "EXPECTED_SURFACE_ENTRY_COUNT = 20",
        "EXPECTED_LEGACY_FULL_HITS = 54",
        "EXPECTED_LEGACY_PRODUCTION_HITS = 0",
        "EXPECTED_LEGACY_PRODUCTION_FILE_COUNT = 0",
        "EXPECTED_TAFFY_PRODUCTION_HITS = 175",
        "EXPECTED_TAFFY_PRODUCTION_FILE_COUNT = 10",
        "MIRROR_DOCS_GUARD",
        "\"runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts\"",
        "\"mirror_docs_guard_present\"",
    ] {
        assert!(
            audit_script.contains(audit_anchor),
            "ui_architecture_boundary should expose audit anchor `{audit_anchor}`"
        );
    }

    let mirror_docs = [
        ("UI architecture doc", architecture_doc),
        ("Runtime 09 plan", runtime_09_plan),
        ("runtime index", runtime_index),
        ("runtime architecture review", architecture_review),
        ("runtime interface convergence doc", interface_doc),
    ];

    for (doc_name, doc_source) in mirror_docs {
        for expected_anchor in [
            "ui_architecture_boundary",
            "expected_source_file_count = 52",
            "expected_ui_entry_count = 18",
            "expected_surface_entry_count = 20",
            "legacy_full_hits = 54",
            "expected_legacy_full_hits = 54",
            "legacy_production_hits = 0",
            "expected_legacy_production_hits = 0",
            "legacy_production_file_count = 0",
            "expected_legacy_production_file_count = 0",
            "taffy_production_hits = 175",
            "expected_taffy_production_hits = 175",
            "taffy_production_file_count = 10",
            "expected_taffy_production_file_count = 10",
            "runtime_v2_anchor_count = 10",
            "interface_v2_anchor_count = 9",
            "guard_anchor_count = 19",
            "cargo_gate_anchor_count = 7",
            "doc_anchor_count = 61",
            "missing_doc_anchors = []",
            "missing_cargo_gate_anchors = []",
            "mirror_docs_guard_present = true",
            "risks = []",
            "runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts",
            "runtime_09_m1_1_ui_input_route_authority_static_passed_cargo_pending",
            "runtime_09_m1_2_navigation_legacy_reply_renamed_static_passed_cargo_pending",
            "runtime_09_m1_2_pointer_legacy_reply_renamed_static_passed_cargo_pending",
            "runtime_09_m1_2_pointer_capture_fallback_renamed_static_passed_cargo_pending",
            "runtime_09_m1_2_table_row_label_fallback_renamed_static_passed_cargo_pending",
            "runtime_09_m1_2_template_component_name_fallback_renamed_static_passed_cargo_pending",
            "runtime_09_m1_2_property_visibility_flag_renamed_static_passed_cargo_pending",
            "runtime_09_m1_2_responsive_mui_visibility_flag_renamed_static_passed_cargo_pending",
            "runtime_09_m1_2_accessibility_open_state_fallback_renamed_static_passed_cargo_pending",
            "runtime_09_m1_2_layout_engine_backend_name_cutover_static_passed_cargo_pending",
            "runtime_09_m1_2_surface_default_interaction_fallback_renamed_static_passed_cargo_pending",
            "runtime_09_m2_1_taffy_bridge_pass_order_static_passed_cargo_pending",
            "runtime_09_m2_2_virtualization_scroll_boundary_static_passed_cargo_pending",
            "runtime_09_m3_1_template_compile_instance_validate_boundary_static_passed_cargo_pending",
        ] {
            assert!(
                doc_source.contains(expected_anchor),
                "{doc_name} should mirror Runtime 09 UI architecture audit anchor `{expected_anchor}`"
            );
        }
    }
}
