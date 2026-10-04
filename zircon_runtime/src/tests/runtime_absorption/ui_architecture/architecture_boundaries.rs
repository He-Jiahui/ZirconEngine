use super::support::{
    files_with_matching_line, matching_line_count, production_ui_file, read_repo_file,
    rust_files_under, top_level_entry_names,
};

const EXPECTED_UI_ENTRY_COUNT: usize = 23;
// Historical raw worktree count includes two directories without Rust source.
const EXPECTED_SURFACE_ENTRY_COUNT: usize = 45;
const EXPECTED_SURFACE_SOURCE_ENTRY_COUNT: usize = 43;

fn source_backed_top_level_entry_names(relative: &str, include_root_mod: bool) -> Vec<String> {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime manifest should live under the repository root");
    let directory = repo_root.join(relative);

    top_level_entry_names(relative, include_root_mod)
        .into_iter()
        .filter(|entry| {
            let path = directory.join(entry);
            if path.is_file() {
                return path.extension().and_then(|extension| extension.to_str()) == Some("rs");
            }

            path.is_dir() && !rust_files_under(&format!("{relative}/{entry}")).is_empty()
        })
        .collect()
}
const LEGACY_MIGRATION_TERMS: &[&str] = &[
    "has_legacy_or_indexed_pointer_capture_for_owner",
    "split_legacy_table_text",
    "legacy_component_interaction_fallback",
    "legacy_interactive",
    "legacy_visible",
    "LegacyZircon",
    "legacy_zircon",
    "legacy_selected_count",
];

fn matching_migration_line_count(files: &[std::path::PathBuf]) -> usize {
    files
        .iter()
        .map(|path| {
            std::fs::read_to_string(path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .lines()
                .filter(|line| {
                    LEGACY_MIGRATION_TERMS
                        .iter()
                        .any(|term| line.contains(term))
                })
                .count()
        })
        .sum()
}

fn files_with_migration_term(files: &[std::path::PathBuf]) -> Vec<std::path::PathBuf> {
    files
        .iter()
        .filter(|path| {
            std::fs::read_to_string(path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .lines()
                .any(|line| {
                    LEGACY_MIGRATION_TERMS
                        .iter()
                        .any(|term| line.contains(term))
                })
        })
        .cloned()
        .collect()
}

const RUNTIME_09_STATUS: &str = concat!(
    include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    ),
    include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/09/2026-07-09-ui-subsystem-architecture-output-records.md"
    )
);
const RUNTIME_INDEX_STATUS: &str = concat!(
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md"),
    include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/09/2026-07-09-ui-subsystem-architecture-output-records.md"
    ),
    include_str!(
        "../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md"
    )
);

#[test]
fn runtime_09_ui_architecture_doc_records_current_boundaries() {
    let architecture_doc =
        include_str!("../../../../../docs/crates/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = RUNTIME_09_STATUS;
    let runtime_index = RUNTIME_INDEX_STATUS;

    for required_anchor in [
        "runtime_09_m0_ui_architecture_static_passed",
        "Module Boundary Map",
        "`ui/` top-level entries: 23",
        "`surface/` entries: 45",
        "surface_source_module_entry_count = 43",
        "two local directories, `navigation/` and `pointer/`, that contain no Rust source.",
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
        EXPECTED_UI_ENTRY_COUNT,
        "Runtime 09 M0 architecture doc must be refreshed when ui/ top-level entries change"
    );
    for required_entry in [
        "accessibility",
        "binding",
        "component",
        "dispatch",
        "editable_text_composition.rs",
        "event_ui",
        "icon_atlas",
        "layout",
        "module",
        "module.rs",
        "platform_input",
        "prelude.rs",
        "public_runtime_frame.rs",
        "secure_text_policy.rs",
        "style",
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
    let source_surface_entries =
        source_backed_top_level_entry_names("zircon_runtime/src/ui/surface", true);
    assert_eq!(
        source_surface_entries.len(),
        EXPECTED_SURFACE_SOURCE_ENTRY_COUNT,
        "Runtime 09 source-backed surface map must count only entries in the sealed source tree"
    );
    assert!(
        surface_entries.len() == EXPECTED_SURFACE_SOURCE_ENTRY_COUNT
            || surface_entries.len() == EXPECTED_SURFACE_ENTRY_COUNT,
        "raw surface paths should be either the sealed 43 source entries or include the two known directories without Rust source"
    );
    let non_source_entries = surface_entries
        .iter()
        .filter(|entry| !source_surface_entries.iter().any(|source| source == *entry))
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        non_source_entries.is_empty()
            || non_source_entries == vec!["navigation".to_owned(), "pointer".to_owned()],
        "only the navigation/ and pointer/ directories without Rust source may lack source-backed entries: {non_source_entries:?}"
    );
    for entry in &non_source_entries {
        assert!(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .expect("zircon_runtime manifest should live under the repository root")
                .join("zircon_runtime/src/ui/surface")
                .join(entry)
                .is_dir(),
            "unbacked surface entry `{entry}` should be a directory without Rust source"
        );
        assert!(
            rust_files_under(&format!("zircon_runtime/src/ui/surface/{entry}")).is_empty(),
            "unbacked surface entry `{entry}` must not contain Rust source"
        );
    }
    for required_entry in [
        "input",
        "render",
        "control_index.rs",
        "focus",
        "host_font_assets.rs",
        "invalidation.rs",
        "surface.rs",
        "mod.rs",
    ] {
        assert!(
            source_surface_entries
                .iter()
                .any(|entry| entry == required_entry),
            "Runtime 09 source-backed surface map should include `{required_entry}`"
        );
    }
}

#[test]
fn runtime_09_ui_architecture_cargo_gate_stays_visible_until_ui_owner_validation() {
    let runtime_09_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md"
    );
    for command in [
        "cargo check -p zircon_runtime --lib --locked",
        "cargo test -p zircon_runtime --lib ui --locked",
        "cargo test -p zircon_runtime --lib input --locked -- --nocapture",
        "cargo test -p zircon_runtime --lib naming_boundary --locked",
        "cargo test -p zircon_runtime --lib layout --locked -- --nocapture",
        "cargo test -p zircon_runtime --lib template --locked -- --nocapture",
    ] {
        assert!(
            runtime_09_plan.contains(command),
            "Runtime 09 plan should retain the UI owner validation gate `{command}`"
        );
    }
}

#[test]
fn runtime_09_ui_architecture_baselines_match_current_source_scan() {
    let architecture_doc =
        include_str!("../../../../../docs/crates/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = RUNTIME_09_STATUS;
    let runtime_index = RUNTIME_INDEX_STATUS;
    let all_ui_files = rust_files_under("zircon_runtime/src/ui");
    let production_ui_files = all_ui_files
        .iter()
        .filter(|path| production_ui_file(path))
        .cloned()
        .collect::<Vec<_>>();

    let legacy_full_hits = matching_migration_line_count(&all_ui_files);
    let legacy_production_hits = matching_migration_line_count(&production_ui_files);
    let legacy_production_files = files_with_migration_term(&production_ui_files);
    let taffy_production_hits = matching_line_count(&production_ui_files, "taffy");
    let taffy_production_files = files_with_matching_line(&production_ui_files, "taffy");

    assert_eq!(
        legacy_full_hits, 15,
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
        taffy_production_hits, 254,
        "update Runtime 09 docs if production taffy hit baseline changes"
    );
    assert_eq!(
        taffy_production_files.len(),
        16,
        "update Runtime 09 docs if production taffy file baseline changes"
    );

    for required_anchor in [
        "ui_legacy_hits=15",
        "ui_legacy_production_hits=0",
        "ui_legacy_production_files=0",
        "ui_taffy_production_hits=254",
        "ui_taffy_production_files=16",
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
    let architecture_doc =
        include_str!("../../../../../docs/crates/zircon_runtime/ui/architecture.md");
    let runtime_09_plan = RUNTIME_09_STATUS;
    let runtime_index = RUNTIME_INDEX_STATUS;
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
    let taffy_parent_products =
        read_repo_file("zircon_runtime/src/ui/layout/taffy_bridge/product_cache.rs");
    let style_mapping = read_repo_file("zircon_runtime/src/ui/layout/style_mapping.rs");
    let architecture_doc =
        include_str!("../../../../../docs/crates/zircon_runtime/ui/architecture.md");
    let layout_pass_doc =
        include_str!("../../../../../docs/crates/zircon_runtime/ui/layout/pass.md");
    let v2_doc = include_str!("../../../../../docs/crates/zircon_runtime/ui/v2.md");
    let runtime_09_plan = RUNTIME_09_STATUS;
    let runtime_index = RUNTIME_INDEX_STATUS;
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
            && taffy_bridge_mod.contains("mod product_cache;")
            && taffy_bridge_mod.contains("taffy_style_for_container")
            && taffy_bridge_compute.contains("pub(crate) fn compute_taffy_child_frames")
            && !taffy_bridge_compute.contains("TaffyTree::new()")
            && taffy_bridge_compute.contains("TaffyChildLayoutInput"),
        "taffy_bridge compute should adapt contracts without owning retained Taffy identity"
    );
    assert!(
        taffy_parent_products.contains("struct TaffyParentProduct")
            && taffy_parent_products.contains("TaffyTree::new()")
            && taffy_parent_products.contains("disable_rounding")
            && taffy_parent_products.contains(".compute_layout(")
            && taffy_parent_products.contains(".dirty(product.parent_node)"),
        "taffy_bridge product cache should own retained Taffy identity and solve reuse"
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
    let architecture_doc =
        include_str!("../../../../../docs/crates/zircon_runtime/ui/architecture.md");
    let layout_pass_doc =
        include_str!("../../../../../docs/crates/zircon_runtime/ui/layout/pass.md");
    let runtime_09_plan = RUNTIME_09_STATUS;
    let runtime_index = RUNTIME_INDEX_STATUS;
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
        "retained_virtual_list_only_arranges_visible_window",
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
            "retained_virtual_list_only_arranges_visible_window",
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
fn runtime_74_template_boundary_has_one_compiler_authority() {
    let template_mod = read_repo_file("zircon_runtime/src/ui/template/mod.rs");
    let interface_mod = read_repo_file("zircon_runtime_interface/src/ui/template/mod.rs");
    let template_pipeline_test = read_repo_file("zircon_runtime/src/ui/tests/template_pipeline.rs");
    let template_pipeline_doc =
        include_str!("../../../../../docs/crates/zircon_runtime/ui/template/pipeline.md");
    let optimize_record = include_str!(
        "../../../../../docs/plans/optimize/zircon_runtime/74/2026-08-22-single-template-compiler-authority.md"
    );

    for required in [
        "UiAssetLoader",
        "UiDocumentCompiler",
        "UiCompiledDocument",
        "UiTemplateSurfaceBuilder",
    ] {
        assert!(
            template_mod.contains(required),
            "template namespace should expose canonical compiler-path owner `{required}`"
        );
    }
    for forbidden in [
        "mod loader;",
        "mod pipeline;",
        "mod validate;",
        "UiTemplateLoader",
        "UiTemplateValidator",
        "UiTemplateRuntimePipeline",
        "UiTemplateDocument",
        "UiTemplateError",
    ] {
        assert!(
            !template_mod.contains(forbidden) && !interface_mod.contains(forbidden),
            "legacy compiler authority `{forbidden}` should be removed after the hard cut"
        );
    }
    for test_anchor in [
        "asset_compiler_is_the_single_template_compile_authority",
        "legacy_recursive_template_document_is_not_a_runtime_compile_input",
        "template_compiler_authority_has_bounded_p95_latency",
        "PERF-RUNTIME74-COMPILER-AUTHORITY",
        "sample_count={SAMPLE_COUNT}",
        "runtime_compiler_authorities=1",
        "legacy_runtime_pipeline_exports=0",
    ] {
        assert!(
            template_pipeline_test.contains(test_anchor),
            "single-authority tests should retain `{test_anchor}`"
        );
    }
    for (label, source) in [
        ("template pipeline doc", template_pipeline_doc),
        ("Runtime74 optimization record", optimize_record),
    ] {
        for required in [
            "RTB-P1-001",
            "UiDocumentCompiler",
            "PERF-RUNTIME74-COMPILER-AUTHORITY",
            "nearest-rank P95",
        ] {
            assert!(
                source.contains(required),
                "{label} should record single-authority anchor `{required}`"
            );
        }
    }
}
