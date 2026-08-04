from __future__ import annotations

from pathlib import Path


EXPECTED_UI_ENTRY_COUNT = 19
EXPECTED_SURFACE_ENTRY_COUNT = 23
EXPECTED_LEGACY_FULL_HITS = 54
EXPECTED_LEGACY_PRODUCTION_HITS = 0
EXPECTED_LEGACY_PRODUCTION_FILE_COUNT = 0
EXPECTED_TAFFY_PRODUCTION_HITS = 175
EXPECTED_TAFFY_PRODUCTION_FILE_COUNT = 10

SOURCE_FILES = (
    "zircon_runtime/src/ui/mod.rs",
    "zircon_runtime/src/ui/v2/mod.rs",
    "zircon_runtime_interface/src/ui/v2/mod.rs",
    "zircon_runtime_interface/src/ui/layout/engine.rs",
    "zircon_runtime/src/ui/layout/mod.rs",
    "zircon_runtime/src/ui/layout/style_mapping.rs",
    "zircon_runtime/src/ui/layout/scroll.rs",
    "zircon_runtime/src/ui/layout/virtualization.rs",
    "zircon_runtime/src/ui/layout/pass/mod.rs",
    "zircon_runtime/src/ui/layout/pass/pipeline.rs",
    "zircon_runtime/src/ui/layout/pass/layout_tree.rs",
    "zircon_runtime/src/ui/layout/pass/incremental.rs",
    "zircon_runtime/src/ui/layout/pass/arrange.rs",
    "zircon_runtime/src/ui/layout/pass/responsive_mui.rs",
    "zircon_runtime/src/ui/layout/pass/taffy_arrange.rs",
    "zircon_runtime/src/ui/layout/taffy_bridge/mod.rs",
    "zircon_runtime/src/ui/layout/taffy_bridge/compute.rs",
    "zircon_runtime/src/ui/tree/node/scroll.rs",
    "zircon_runtime/src/ui/tests/scroll_virtualization.rs",
    "zircon_runtime/src/ui/template/mod.rs",
    "zircon_runtime/src/ui/template/pipeline.rs",
    "zircon_runtime/src/ui/template/loader.rs",
    "zircon_runtime/src/ui/template/validate.rs",
    "zircon_runtime/src/ui/template/instance.rs",
    "zircon_runtime/src/ui/template/build/interaction.rs",
    "zircon_runtime/src/ui/template/build/surface_builder.rs",
    "zircon_runtime/src/ui/template/asset/compiler/package/artifact.rs",
    "zircon_runtime/src/ui/tests/template_pipeline.rs",
    "zircon_runtime/src/ui/surface/input/mod.rs",
    "zircon_runtime/src/ui/surface/input/dispatch.rs",
    "zircon_runtime/src/ui/surface/input/route_authority.rs",
    "zircon_runtime/src/ui/surface/input/navigation.rs",
    "zircon_runtime/src/ui/surface/input/pointer.rs",
    "zircon_runtime/src/ui/surface/input/pointer_reply.rs",
    "zircon_runtime/src/ui/surface/input/state/pointer_capture.rs",
    "zircon_runtime/src/ui/surface/input/effect/focus_pointer.rs",
    "zircon_runtime/src/ui/surface/render/collection_rows/table.rs",
    "zircon_runtime/src/ui/surface/property_mutation.rs",
    "zircon_runtime/src/ui/surface/surface/default_interactions.rs",
    "zircon_runtime/src/ui/accessibility/extract.rs",
    "zircon_runtime/src/tests/runtime_absorption/ui_architecture.rs",
    "docs/zircon_runtime/ui/architecture.md",
    "docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md",
    "docs/plans/zircon_runtime/runtime/index.md",
    "docs/engine-architecture/runtime-architecture-review-m0.md",
    "docs/engine-architecture/runtime-interface-convergence.md",
    "docs/zircon_runtime/ui/layout/pass.md",
    "docs/zircon_runtime/ui/surface/default_interactions.md",
    "docs/zircon_runtime/ui/template/pipeline.md",
    "docs/ui-and-layout/shared-ui-template-runtime.md",
    "docs/zircon_runtime/ui/v2.md",
)
EXPECTED_SOURCE_FILE_COUNT = 52

EXPECTED_UI_ENTRIES = (
    "accessibility",
    "binding",
    "component",
    "dispatch",
    "event_ui",
    "icon_atlas",
    "layout",
    "module.rs",
    "platform_input",
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
)

EXPECTED_SURFACE_ENTRIES = (
    "arranged.rs",
    "component_state.rs",
    "diagnostics.rs",
    "ecs_projection.rs",
    "focus.rs",
    "frame_hit_test.rs",
    "input",
    "interaction_gate.rs",
    "mod.rs",
    "navigation",
    "node_pool.rs",
    "pointer",
    "popup_stack.rs",
    "property_mutation",
    "property_mutation.rs",
    "reflection_snapshot.rs",
    "render",
    "slots.rs",
    "surface",
    "surface.rs",
    "text_geometry.rs",
    "text_shape.rs",
    "timeline.rs",
)

RUNTIME_V2_ANCHORS = (
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
)

INTERFACE_V2_ANCHORS = (
    "mod arena;",
    "mod asset;",
    "mod compiled;",
    "mod graph;",
    "mod repeat;",
    "mod style;",
    "UiV2AssetDocument",
    "UiV2CompiledDocument",
    "UiV2ResolvedStyle",
)

RUNTIME_09_GUARD_ANCHORS = (
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
)
MIRROR_DOCS_GUARD = (
    "runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts"
)

RUNTIME_09_DOC_ANCHORS = (
    "ui_architecture_boundary",
    "runtime_09_m0_ui_architecture_static_passed",
    MIRROR_DOCS_GUARD,
    "runtime_09_m1_1_ui_input_route_authority_static_passed_cargo_pending",
    "runtime_09_m1_1_direct_pointer_navigation_routes_are_leaf_owner_helpers",
    "runtime_09_m1_2_navigation_legacy_reply_renamed_static_passed_cargo_pending",
    "runtime_09_m1_2_pointer_legacy_reply_renamed_static_passed_cargo_pending",
    "runtime_09_pointer_legacy_reply_rename_reduces_ui_input_debt",
    "routed_result",
    "runtime_09_m1_2_pointer_capture_fallback_renamed_static_passed_cargo_pending",
    "runtime_09_pointer_capture_fallback_rename_reduces_ui_input_debt",
    "has_pointer_capture_or_unindexed_fallback_for_owner",
    "runtime_09_m1_2_table_row_label_fallback_renamed_static_passed_cargo_pending",
    "runtime_09_table_row_label_fallback_rename_reduces_ui_render_debt",
    "split_row_label_table_text",
    "runtime_09_m1_2_template_component_name_fallback_renamed_static_passed_cargo_pending",
    "runtime_09_template_component_name_fallback_rename_reduces_ui_template_debt",
    "component_name_interaction_fallback",
    "runtime_09_m1_2_property_visibility_flag_renamed_static_passed_cargo_pending",
    "runtime_09_property_visibility_flag_rename_reduces_ui_surface_debt",
    "runtime_09_m1_2_responsive_mui_visibility_flag_renamed_static_passed_cargo_pending",
    "runtime_09_responsive_mui_visibility_flag_rename_reduces_ui_layout_debt",
    "state_visible_flag",
    "runtime_09_m1_2_accessibility_open_state_fallback_renamed_static_passed_cargo_pending",
    "runtime_09_accessibility_open_state_fallback_rename_reduces_ui_a11y_debt",
    "fallback_properties",
    "runtime_09_m1_2_layout_engine_backend_name_cutover_static_passed_cargo_pending",
    "runtime_09_layout_engine_backend_name_cutover_reduces_ui_layout_debt",
    "UiLayoutEngineBackend::Zircon",
    "UiLayoutEngineCapability::zircon",
    "zircon_selected_count",
    "runtime_09_m1_2_surface_default_interaction_fallback_renamed_static_passed_cargo_pending",
    "runtime_09_surface_default_interaction_fallback_rename_reduces_ui_surface_debt",
    "default_open_boolean_value",
    "runtime_09_m2_1_taffy_bridge_pass_order_static_passed_cargo_pending",
    "runtime_09_m2_1_style_mapping_remains_taffy_dto_adapter",
    "runtime_09_m2_2_virtualization_scroll_boundary_static_passed_cargo_pending",
    "runtime_09_m3_1_template_compile_instance_validate_boundary_static_passed_cargo_pending",
    "UI_LAYOUT_PASS_ORDER",
    "compute_taffy_child_frames",
    "UiScrollVirtualizationPlan",
    "plan_scrollable_virtual_window",
    "virtualized_list_only_materializes_visible_window",
    "scroll_offset_invalidates_virtualization_window",
    "non_virtualized_scroll_offset_keeps_full_window_dirty_domain",
    "UI_TEMPLATE_RUNTIME_PIPELINE_STAGES",
    "UiTemplateRuntimePipeline",
    "UiTemplateRuntimePipelineError",
    "template_validate_rejects_unknown_component_contract",
    "template_instance_failure_surfaces_loader_error",
    "runtime_09_m3_1_binary_leaf_dto_artifact_not_generated_source",
    "compiled_template_artifact_stays_binary_leaf_dto_not_generated_source",
    "// @generated <generator> - do not edit by hand",
    "v2-replacement-mainline",
    "ui_legacy_hits=54",
    "ui_legacy_production_hits=0",
    "ui_legacy_production_files=0",
    "ui_taffy_production_hits=175",
    "ui_taffy_production_files=10",
    "ui/input/naming_boundary/layout/template",
    "editor UI owner",
)

CARGO_GATE_ANCHORS = (
    "cargo check -p zircon_runtime --lib --locked",
    "cargo test -p zircon_runtime --lib ui --locked",
    "cargo test -p zircon_runtime --lib input --locked -- --nocapture",
    "cargo test -p zircon_runtime --lib naming_boundary --locked",
    "cargo test -p zircon_runtime --lib layout --locked -- --nocapture",
    "cargo test -p zircon_runtime --lib template --locked -- --nocapture",
    "runtime_09_ui_architecture_cargo_gate_stays_visible_until_ui_owner_validation",
)


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _relative(root: Path, path: Path) -> str:
    return path.relative_to(root).as_posix()


def _file_line_count(path: Path) -> int:
    return len(_read_text(path).splitlines())


def _file_entries(root: Path, files: tuple[str, ...]) -> tuple[list[dict[str, object]], list[str]]:
    entries: list[dict[str, object]] = []
    missing: list[str] = []
    for file_name in files:
        path = root / file_name
        if not path.exists():
            missing.append(file_name)
            continue
        entries.append({"path": file_name, "lines": _file_line_count(path)})
    return entries, missing


def _top_level_entry_names(
    root: Path,
    relative: str,
    *,
    include_root_mod: bool,
) -> list[str]:
    directory = root / relative
    if not directory.is_dir():
        return []

    entries = []
    for path in directory.iterdir():
        name = path.name
        if not include_root_mod and name == "mod.rs":
            continue
        entries.append(name)
    return sorted(entries)


def _rust_files_under(root: Path, relative: str) -> list[Path]:
    directory = root / relative
    if not directory.is_dir():
        return []
    return sorted(path for path in directory.rglob("*.rs") if path.is_file())


def _has_component(path: Path, component: str) -> bool:
    return component in path.parts


def _production_ui_file(path: Path) -> bool:
    filename = path.name
    return (
        not _has_component(path, "tests")
        and not _has_component(path, "test_fixtures")
        and filename != "tests.rs"
        and not filename.endswith("_tests.rs")
    )


def _matching_line_count(files: list[Path], needle: str) -> int:
    return sum(
        1
        for path in files
        for line in _read_text(path).splitlines()
        if needle in line
    )


def _files_with_matching_line(root: Path, files: list[Path], needle: str) -> list[str]:
    return [
        _relative(root, path)
        for path in files
        if any(needle in line for line in _read_text(path).splitlines())
    ]


def _missing_snippets(sources: tuple[str, ...], snippets: tuple[str, ...]) -> list[str]:
    return [
        snippet
        for snippet in snippets
        if not any(snippet in source for source in sources)
    ]


def _missing_file_snippets(
    root: Path,
    file_snippets: tuple[tuple[str, str], ...],
) -> list[dict[str, str]]:
    missing: list[dict[str, str]] = []
    for file_name, snippet in file_snippets:
        path = root / file_name
        if not path.exists() or snippet not in _read_text(path):
            missing.append({"path": file_name, "snippet": snippet})
    return missing


def ui_architecture_boundary_audit(root: Path) -> dict[str, object]:
    source_files, missing_source_files = _file_entries(root, SOURCE_FILES)

    ui_entries = _top_level_entry_names(
        root,
        "zircon_runtime/src/ui",
        include_root_mod=False,
    )
    surface_entries = _top_level_entry_names(
        root,
        "zircon_runtime/src/ui/surface",
        include_root_mod=True,
    )

    all_ui_files = _rust_files_under(root, "zircon_runtime/src/ui")
    production_ui_files = [path for path in all_ui_files if _production_ui_file(path)]
    legacy_production_files = _files_with_matching_line(
        root,
        production_ui_files,
        "legacy",
    )
    taffy_production_files = _files_with_matching_line(
        root,
        production_ui_files,
        "taffy",
    )

    legacy_full_hits = _matching_line_count(all_ui_files, "legacy")
    legacy_production_hits = _matching_line_count(production_ui_files, "legacy")
    taffy_production_hits = _matching_line_count(production_ui_files, "taffy")

    runtime_v2 = root / "zircon_runtime/src/ui/v2/mod.rs"
    interface_v2 = root / "zircon_runtime_interface/src/ui/v2/mod.rs"
    runtime_v2_source = _read_text(runtime_v2) if runtime_v2.exists() else ""
    interface_v2_source = _read_text(interface_v2) if interface_v2.exists() else ""

    guard_paths = (
        root / "zircon_runtime/src/tests/runtime_absorption/ui_architecture.rs",
        root
        / "zircon_runtime/src/tests/runtime_absorption/ui_architecture/architecture_boundaries.rs",
        root
        / "zircon_runtime/src/tests/runtime_absorption/ui_architecture/legacy_renames.rs",
        root
        / "zircon_runtime/src/tests/runtime_absorption/ui_architecture/mirror_docs.rs",
    )
    guard_sources = tuple(_read_text(path) for path in guard_paths if path.exists())

    doc_paths = (
        root / "docs/zircon_runtime/ui/architecture.md",
        root / "docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md",
        root / "docs/plans/zircon_runtime/runtime/index.md",
        root / "docs/engine-architecture/runtime-architecture-review-m0.md",
        root / "docs/engine-architecture/runtime-interface-convergence.md",
    )
    doc_sources = tuple(_read_text(path) for path in doc_paths if path.exists())

    ui_missing_entries = [
        entry for entry in EXPECTED_UI_ENTRIES if entry not in ui_entries
    ]
    ui_unexpected_entries = [
        entry for entry in ui_entries if entry not in EXPECTED_UI_ENTRIES
    ]
    surface_missing_entries = [
        entry for entry in EXPECTED_SURFACE_ENTRIES if entry not in surface_entries
    ]
    surface_unexpected_entries = [
        entry for entry in surface_entries if entry not in EXPECTED_SURFACE_ENTRIES
    ]

    baseline_mismatches: list[dict[str, object]] = []
    for name, actual, expected in (
        ("ui_legacy_hits", legacy_full_hits, EXPECTED_LEGACY_FULL_HITS),
        (
            "ui_legacy_production_hits",
            legacy_production_hits,
            EXPECTED_LEGACY_PRODUCTION_HITS,
        ),
        (
            "ui_legacy_production_files",
            len(legacy_production_files),
            EXPECTED_LEGACY_PRODUCTION_FILE_COUNT,
        ),
        (
            "ui_taffy_production_hits",
            taffy_production_hits,
            EXPECTED_TAFFY_PRODUCTION_HITS,
        ),
        (
            "ui_taffy_production_files",
            len(taffy_production_files),
            EXPECTED_TAFFY_PRODUCTION_FILE_COUNT,
        ),
    ):
        if actual != expected:
            baseline_mismatches.append(
                {"name": name, "actual": actual, "expected": expected}
            )

    missing_runtime_v2_anchors = _missing_snippets(
        (runtime_v2_source,),
        RUNTIME_V2_ANCHORS,
    )
    missing_interface_v2_anchors = _missing_snippets(
        (interface_v2_source,),
        INTERFACE_V2_ANCHORS,
    )
    missing_guard_anchors = _missing_snippets(
        guard_sources + doc_sources,
        RUNTIME_09_GUARD_ANCHORS,
    )
    missing_cargo_gate_anchors = _missing_snippets(
        guard_sources + doc_sources,
        CARGO_GATE_ANCHORS,
    )
    missing_doc_anchors = _missing_snippets(
        doc_sources,
        RUNTIME_09_DOC_ANCHORS,
    )
    mirror_docs_guard_present = not _missing_snippets(
        guard_sources + doc_sources,
        (MIRROR_DOCS_GUARD,),
    )
    missing_required_doc_mentions = _missing_file_snippets(
        root,
        (
            ("docs/zircon_runtime/ui/architecture.md", "ui_architecture_boundary"),
            (
                "docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md",
                "ui_architecture_boundary",
            ),
            ("docs/plans/zircon_runtime/runtime/index.md", "ui_architecture_boundary"),
            (
                "docs/engine-architecture/runtime-architecture-review-m0.md",
                "ui_architecture_boundary",
            ),
            (
                "docs/engine-architecture/runtime-interface-convergence.md",
                "ui_architecture_boundary",
            ),
        ),
    )

    risks: list[str] = []
    if missing_source_files:
        risks.append("Runtime 09 UI architecture source/doc files are missing.")
    if ui_missing_entries or ui_unexpected_entries or len(ui_entries) != EXPECTED_UI_ENTRY_COUNT:
        risks.append("Runtime 09 ui/ top-level entry map changed without audit sync.")
    if (
        surface_missing_entries
        or surface_unexpected_entries
        or len(surface_entries) != EXPECTED_SURFACE_ENTRY_COUNT
    ):
        risks.append("Runtime 09 surface/ entry map changed without audit sync.")
    if baseline_mismatches:
        risks.append("Runtime 09 UI legacy/taffy source-scan baselines changed.")
    if missing_runtime_v2_anchors:
        risks.append("Runtime 09 runtime ui::v2 implementation anchors are missing.")
    if missing_interface_v2_anchors:
        risks.append("Runtime 09 interface ui::v2 contract anchors are missing.")
    if missing_guard_anchors:
        risks.append("Runtime 09 Rust/status guard anchors are missing.")
    if missing_cargo_gate_anchors:
        risks.append("Runtime 09 pending UI owner/Cargo gate anchors are missing.")
    if missing_doc_anchors or missing_required_doc_mentions:
        risks.append("Runtime 09 plan or mirror docs are missing required status anchors.")
    if not mirror_docs_guard_present:
        risks.append("Runtime 09 mirror-doc guard anchor is missing from docs or guards.")

    return {
        "source_files": source_files,
        "expected_source_file_count": EXPECTED_SOURCE_FILE_COUNT,
        "missing_source_files": missing_source_files,
        "ui_entries": ui_entries,
        "expected_ui_entry_count": EXPECTED_UI_ENTRY_COUNT,
        "ui_missing_entries": ui_missing_entries,
        "ui_unexpected_entries": ui_unexpected_entries,
        "surface_entries": surface_entries,
        "expected_surface_entry_count": EXPECTED_SURFACE_ENTRY_COUNT,
        "surface_missing_entries": surface_missing_entries,
        "surface_unexpected_entries": surface_unexpected_entries,
        "all_ui_rust_file_count": len(all_ui_files),
        "production_ui_rust_file_count": len(production_ui_files),
        "legacy_full_hits": legacy_full_hits,
        "expected_legacy_full_hits": EXPECTED_LEGACY_FULL_HITS,
        "legacy_production_hits": legacy_production_hits,
        "expected_legacy_production_hits": EXPECTED_LEGACY_PRODUCTION_HITS,
        "legacy_production_files": legacy_production_files,
        "expected_legacy_production_file_count": EXPECTED_LEGACY_PRODUCTION_FILE_COUNT,
        "taffy_production_hits": taffy_production_hits,
        "expected_taffy_production_hits": EXPECTED_TAFFY_PRODUCTION_HITS,
        "taffy_production_files": taffy_production_files,
        "expected_taffy_production_file_count": EXPECTED_TAFFY_PRODUCTION_FILE_COUNT,
        "baseline_mismatches": baseline_mismatches,
        "runtime_v2_anchor_count": len(RUNTIME_V2_ANCHORS),
        "missing_runtime_v2_anchors": missing_runtime_v2_anchors,
        "interface_v2_anchor_count": len(INTERFACE_V2_ANCHORS),
        "missing_interface_v2_anchors": missing_interface_v2_anchors,
        "guard_anchor_count": len(RUNTIME_09_GUARD_ANCHORS),
        "missing_guard_anchors": missing_guard_anchors,
        "cargo_gate_anchor_count": len(CARGO_GATE_ANCHORS),
        "missing_cargo_gate_anchors": missing_cargo_gate_anchors,
        "doc_anchor_count": len(RUNTIME_09_DOC_ANCHORS),
        "missing_doc_anchors": missing_doc_anchors,
        "mirror_docs_guard": MIRROR_DOCS_GUARD,
        "mirror_docs_guard_present": mirror_docs_guard_present,
        "missing_required_doc_mentions": missing_required_doc_mentions,
        "risks": risks,
    }
