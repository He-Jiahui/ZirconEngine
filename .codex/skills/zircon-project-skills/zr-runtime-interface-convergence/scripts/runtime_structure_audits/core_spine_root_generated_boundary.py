from __future__ import annotations

import re
from pathlib import Path

from runtime_structure_audits.generated_code_boundary import generated_code_boundary_audit
from runtime_structure_audits.runtime_root_surface import runtime_root_surface_audit


EXPECTED_CORE_ROOT_ENTRIES = (
    "framework",
    "manager",
    "math",
    "mod.rs",
    "resource",
    "runtime",
)

EXPECTED_CORE_PUBLIC_MODULES = (
    "runtime",
    "framework",
    "manager",
    "math",
    "resource",
)

EXPECTED_RETIRED_CORE_ROOT_ENTRIES = (
    "channel_util.rs",
    "config_store.rs",
    "diagnostics",
    "event_bus",
    "event_bus.rs",
    "frame_clock.rs",
    "job_scheduler.rs",
    "lifecycle.rs",
    "modules",
    "state",
    "tasks",
    "time.rs",
    "types.rs",
)

EXPECTED_ROOT_ENTRIES_TEST_COUNT = 13
EXPECTED_ROOT_SURFACE_TEST_COUNT = 6
EXPECTED_GENERATED_GUARD_TEST_COUNT = 7
EXPECTED_PUBLIC_MODULE_COUNT = 19
EXPECTED_PUBLIC_USE_COUNT = 2
EXPECTED_GRAPHICS_REEXPORT_COUNT = 0
EXPECTED_GENERATED_TEMPLATE_COUNT = 10
EXPECTED_GENERATED_BEHAVIOR_COUNT = 6
EXPECTED_GENERATED_ADAPTER_COUNT = 6
EXPECTED_GENERATED_MIGRATION_DEBT_COUNT = 0
EXPECTED_GENERATED_DECISION_COUNT = 3
MIRROR_DOCS_GUARD = "runtime_02_core_spine_root_generated_mirror_docs_match_structure_audit_counts"
ROOT_ENTRIES_GUARD_RELATIVES = (
    "zircon_runtime/src/tests/runtime_absorption/root_entries.rs",
    "zircon_runtime/src/tests/runtime_absorption/root_entries/core_spine.rs",
    "zircon_runtime/src/tests/runtime_absorption/root_entries/module_families.rs",
    "zircon_runtime/src/tests/runtime_absorption/root_entries/module_families/navigation.rs",
    "zircon_runtime/src/tests/runtime_absorption/root_entries/module_families/animation_backlog.rs",
    "zircon_runtime/src/tests/runtime_absorption/root_entries/module_families/animation_status_json.rs",
    "zircon_runtime/src/tests/runtime_absorption/root_entries/module_families/root_seats.rs",
    "zircon_runtime/src/tests/runtime_absorption/root_entries/module_families/mirror_docs.rs",
    "zircon_runtime/src/tests/runtime_absorption/root_entries/runtime_root.rs",
)
ROOT_SURFACE_GUARD_RELATIVES = (
    "zircon_runtime/src/tests/runtime_absorption/root_surface/public_surface.rs",
    "zircon_runtime/src/tests/runtime_absorption/root_surface/graphics_alias.rs",
    "zircon_runtime/src/tests/runtime_absorption/root_surface/docs.rs",
)
GENERATED_GUARD_RELATIVES = (
    "zircon_runtime/src/tests/runtime_absorption/generated_code_guard/markers.rs",
    "zircon_runtime/src/tests/runtime_absorption/generated_code_guard/behavior.rs",
    "zircon_runtime/src/tests/runtime_absorption/generated_code_guard/scope.rs",
    "zircon_runtime/src/tests/runtime_absorption/generated_code_guard/delegation.rs",
)
MIRROR_DOCS_GUARD_RELATIVES = (
    "zircon_runtime/src/tests/runtime_absorption/core_spine_root_generated.rs",
    "zircon_runtime/src/tests/runtime_absorption/core_spine_root_generated/mirror_docs.rs",
)

ROOT_ENTRIES_TEST_ANCHORS = (
    "core_root_retires_channel_and_service_alias_fragments",
    "core_root_retires_runtime_kernel_fragment_files",
    "core_root_splits_event_dto_from_runtime_event_bus",
    "core_root_reexports_runtime_diagnostics_without_root_directory",
    "core_module_tree_matches_decided_spine_shape",
    "runtime_crate_root_does_not_flatten_plugin_surface",
    "runtime_crate_root_does_not_flatten_builtin_module_assembly_functions",
    "builtin_root_stays_structural_after_runtime_module_split",
    "runtime_navigation_boundary_file_set_requires_doc_update",
    "runtime_animation_backlog_boundary_requires_doc_update",
    "runtime_animation_status_json_boundary_sanitizes_non_finite_values",
    "runtime_14_module_family_root_seats_match_documented_judgements",
    "runtime_14_module_family_mirror_docs_match_structure_audit_counts",
)

ROOT_SURFACE_TEST_ANCHORS = (
    "runtime_crate_root_public_surface_stays_curated",
    "graphics_alias_debt_is_removed_from_runtime_root",
    "graphics_type_alias_debt_symbols_are_only_available_through_graphics_namespace",
    "core_spine_and_root_surface_docs_stay_in_sync",
    "root_surface_m1_gate_matches_runtime_14_module_family_seats",
    "root_surface_interface_convergence_mirror_uses_current_audit_counts",
)

GENERATED_GUARD_TEST_ANCHORS = (
    "generated_marker_format_is_uniform_when_source_files_are_marked",
    "marked_generated_source_files_stay_leaf_data_only",
    "export_template_generated_behavior_stays_classified_by_owner",
    "export_template_generated_behavior_is_adapter_only_after_m4_cutover",
    "export_template_scan_scope_stays_folder_backed",
    "export_entry_templates_delegate_to_app_export_bootstrap_facade",
    "export_plugin_selection_template_delegates_registration_execution_to_app_providers",
)

PENDING_GATE_ANCHORS = (
    "runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation",
    "core/root/generated/export_build_plan/app/editor/plugin",
    "graphics_alias_block_removed_static_passed_cargo_pending",
    "code_static_pending_cargo",
)

DOC_ANCHORS = (
    "core spine",
    "root_surface_guard",
    "generated_code_boundary",
    "runtime_root_surface",
    "generated-code boundary",
    "M3 `lib.rs` graphics alias removal",
    "M3 Alias Cutover",
    "Runtime 02",
    "core_spine_root_generated_boundary",
    "core root entries 6/6",
    "core public modules 5/5",
    "retired core root entries 0",
    "runtime root public modules 19/19",
    "public `pub use` sites 2/2",
    "crate-visible graphics alias debt 0/0",
    "root-surface M1 gate `classified-and-clear`",
    "generated export templates 10/10",
    "generated behavior 6/6",
    "generated allowed adapters 6/6",
    "generated migration debt 0/0",
    "generated-code M1 gate `classified-and-clear`",
    "root_entries guard tests 13",
    "root_surface guard tests 6/6",
    "generated-code guard tests 7/7",
    "guard_test_anchor_count = 26",
    "missing_guard_test_anchors = []",
    "mirror_docs_guard_present = true",
    "risks = []",
    MIRROR_DOCS_GUARD,
)


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _missing_snippets(sources: tuple[str, ...], snippets: tuple[str, ...]) -> list[str]:
    return [
        snippet
        for snippet in snippets
        if not any(snippet in source for source in sources)
    ]


def _rust_test_names(source: str) -> list[str]:
    return re.findall(
        r"(?m)#\s*\[test\]\s*fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(",
        source,
    )


def _core_public_modules(core_mod_source: str) -> list[str]:
    return re.findall(r"(?m)^pub\s+mod\s+([A-Za-z_][A-Za-z0-9_]*);", core_mod_source)


def core_spine_root_generated_boundary_audit(root: Path) -> dict[str, object]:
    core_root = root / "zircon_runtime/src/core"
    core_mod = root / "zircon_runtime/src/core/mod.rs"
    lib_rs = root / "zircon_runtime/src/lib.rs"
    root_entries_guards = tuple(root / relative for relative in ROOT_ENTRIES_GUARD_RELATIVES)
    root_surface_guards = tuple(root / relative for relative in ROOT_SURFACE_GUARD_RELATIVES)
    generated_guards = tuple(root / relative for relative in GENERATED_GUARD_RELATIVES)
    mirror_docs_guards = tuple(root / relative for relative in MIRROR_DOCS_GUARD_RELATIVES)
    runtime_02_plan = root / "docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md"
    runtime_index = root / "docs/plans/zircon_runtime/runtime/index.md"
    root_surface_doc = root / "docs/zircon_runtime/core/root_surface.md"
    generated_doc = root / "docs/engine-architecture/generated-code-boundary.md"
    m0_review = root / "docs/engine-architecture/runtime-architecture-review-m0.md"
    convergence_doc = root / "docs/engine-architecture/runtime-interface-convergence.md"

    core_entries = sorted(path.name for path in core_root.iterdir()) if core_root.exists() else []
    core_mod_source = _read_text(core_mod) if core_mod.exists() else ""
    lib_source = _read_text(lib_rs) if lib_rs.exists() else ""
    root_entries_sources = tuple(
        _read_text(path) for path in root_entries_guards if path.exists()
    )
    root_surface_sources = tuple(
        _read_text(path) for path in root_surface_guards if path.exists()
    )
    generated_sources = tuple(_read_text(path) for path in generated_guards if path.exists())
    mirror_docs_guard_sources = tuple(
        _read_text(path) for path in mirror_docs_guards if path.exists()
    )
    doc_sources = tuple(
        _read_text(path)
        for path in (
            runtime_02_plan,
            runtime_index,
            root_surface_doc,
            generated_doc,
            m0_review,
            convergence_doc,
        )
        if path.exists()
    )

    core_public_modules = _core_public_modules(core_mod_source)
    root_surface = runtime_root_surface_audit(root)
    generated = generated_code_boundary_audit(root)
    root_entries_tests = [
        test_name
        for source in root_entries_sources
        for test_name in _rust_test_names(source)
    ]
    root_surface_tests = [
        test_name
        for source in root_surface_sources
        for test_name in _rust_test_names(source)
    ]
    generated_guard_tests = [
        test_name
        for source in generated_sources
        for test_name in _rust_test_names(source)
    ]

    retired_core_entries_present = [
        entry for entry in EXPECTED_RETIRED_CORE_ROOT_ENTRIES if (core_root / entry).exists()
    ]
    missing_root_entries_anchors = _missing_snippets(
        root_entries_sources, ROOT_ENTRIES_TEST_ANCHORS
    )
    missing_root_surface_anchors = _missing_snippets(root_surface_sources, ROOT_SURFACE_TEST_ANCHORS)
    missing_generated_guard_anchors = _missing_snippets(generated_sources, GENERATED_GUARD_TEST_ANCHORS)
    missing_guard_test_anchors = (
        missing_root_entries_anchors
        + missing_root_surface_anchors
        + missing_generated_guard_anchors
    )
    missing_pending_gate_anchors = _missing_snippets(
        doc_sources + root_surface_sources,
        PENDING_GATE_ANCHORS,
    )
    missing_doc_anchors = _missing_snippets(doc_sources, DOC_ANCHORS)
    mirror_docs_guard_present = any(
        MIRROR_DOCS_GUARD in source for source in mirror_docs_guard_sources
    )

    root_public_use_count = len(
        [
            line
            for line in lib_source.splitlines()
            if line.strip().startswith("pub use ")
        ]
    )

    risks: list[str] = []
    if tuple(core_entries) != EXPECTED_CORE_ROOT_ENTRIES:
        risks.append("Runtime 02 core root entries differ from the decided spine shape.")
    if tuple(core_public_modules) != EXPECTED_CORE_PUBLIC_MODULES:
        risks.append("Runtime 02 core public module declarations differ from the spine set.")
    if retired_core_entries_present:
        risks.append("Retired core root entries are present.")
    if int(root_surface["public_module_count"]) != EXPECTED_PUBLIC_MODULE_COUNT:
        risks.append("Runtime root public module count changed without Runtime 02 audit sync.")
    if root_public_use_count != EXPECTED_PUBLIC_USE_COUNT:
        risks.append("Runtime root public use count changed without Runtime 02 audit sync.")
    if int(root_surface["crate_visible_graphics_reexport_count"]) != EXPECTED_GRAPHICS_REEXPORT_COUNT:
        risks.append("Runtime root graphics alias debt count changed without Runtime 02 audit sync.")
    if int(generated["template_file_count"]) != EXPECTED_GENERATED_TEMPLATE_COUNT:
        risks.append("Generated template file count changed without Runtime 02 audit sync.")
    if int(generated["behavior_location_count"]) != EXPECTED_GENERATED_BEHAVIOR_COUNT:
        risks.append("Generated behavior location count changed without Runtime 02 audit sync.")
    if int(generated["allowed_adapter_location_count"]) != EXPECTED_GENERATED_ADAPTER_COUNT:
        risks.append("Generated adapter location count changed without Runtime 02 audit sync.")
    if int(generated["migration_debt_location_count"]) != EXPECTED_GENERATED_MIGRATION_DEBT_COUNT:
        risks.append("Generated migration debt reappeared in Runtime 02.")
    if int(generated["behavior_decision_count"]) != EXPECTED_GENERATED_DECISION_COUNT:
        risks.append("Generated behavior decision count changed without Runtime 02 audit sync.")
    if len(root_entries_tests) < EXPECTED_ROOT_ENTRIES_TEST_COUNT:
        risks.append("Runtime root_entries guard test coverage dropped below Runtime 02 baseline.")
    if len(root_surface_tests) != EXPECTED_ROOT_SURFACE_TEST_COUNT:
        risks.append("Runtime root_surface guard test count changed without Runtime 02 audit sync.")
    if len(generated_guard_tests) != EXPECTED_GENERATED_GUARD_TEST_COUNT:
        risks.append("Generated-code guard test count changed without Runtime 02 audit sync.")
    if missing_root_entries_anchors:
        risks.append("Runtime 02 root_entries guard anchors are missing.")
    if missing_root_surface_anchors:
        risks.append("Runtime 02 root_surface guard anchors are missing.")
    if missing_generated_guard_anchors:
        risks.append("Runtime 02 generated-code guard anchors are missing.")
    if missing_guard_test_anchors:
        risks.append("Runtime 02 guard test anchor inventory is incomplete.")
    if missing_pending_gate_anchors:
        risks.append("Runtime 02 pending gate anchors are missing from plan/docs.")
    if missing_doc_anchors:
        risks.append("Runtime 02 mirror docs are missing required anchors.")
    if not mirror_docs_guard_present:
        risks.append("Runtime 02 mirror-doc aggregate guard is missing.")

    return {
        "core_root_entries": core_entries,
        "expected_core_root_entries": list(EXPECTED_CORE_ROOT_ENTRIES),
        "core_public_modules": core_public_modules,
        "expected_core_public_modules": list(EXPECTED_CORE_PUBLIC_MODULES),
        "retired_core_entries_present": retired_core_entries_present,
        "root_public_module_count": root_surface["public_module_count"],
        "expected_root_public_module_count": EXPECTED_PUBLIC_MODULE_COUNT,
        "root_public_use_count": root_public_use_count,
        "expected_root_public_use_count": EXPECTED_PUBLIC_USE_COUNT,
        "root_graphics_reexport_count": root_surface["crate_visible_graphics_reexport_count"],
        "expected_root_graphics_reexport_count": EXPECTED_GRAPHICS_REEXPORT_COUNT,
        "root_surface_m1_gate_status": root_surface["m1_gate_status"],
        "root_surface_risk_count": len(root_surface["risks"]),
        "generated_template_file_count": generated["template_file_count"],
        "expected_generated_template_file_count": EXPECTED_GENERATED_TEMPLATE_COUNT,
        "generated_behavior_location_count": generated["behavior_location_count"],
        "expected_generated_behavior_location_count": EXPECTED_GENERATED_BEHAVIOR_COUNT,
        "generated_allowed_adapter_location_count": generated["allowed_adapter_location_count"],
        "expected_generated_allowed_adapter_location_count": EXPECTED_GENERATED_ADAPTER_COUNT,
        "generated_migration_debt_location_count": generated["migration_debt_location_count"],
        "expected_generated_migration_debt_location_count": EXPECTED_GENERATED_MIGRATION_DEBT_COUNT,
        "generated_behavior_decision_count": generated["behavior_decision_count"],
        "expected_generated_behavior_decision_count": EXPECTED_GENERATED_DECISION_COUNT,
        "generated_m1_gate_status": generated["m1_gate_status"],
        "root_entries_test_count": len(root_entries_tests),
        "expected_root_entries_test_count": EXPECTED_ROOT_ENTRIES_TEST_COUNT,
        "root_surface_test_count": len(root_surface_tests),
        "expected_root_surface_test_count": EXPECTED_ROOT_SURFACE_TEST_COUNT,
        "generated_guard_test_count": len(generated_guard_tests),
        "expected_generated_guard_test_count": EXPECTED_GENERATED_GUARD_TEST_COUNT,
        "guard_test_anchor_count": len(ROOT_ENTRIES_TEST_ANCHORS)
        + len(ROOT_SURFACE_TEST_ANCHORS)
        + len(GENERATED_GUARD_TEST_ANCHORS),
        "missing_guard_test_anchors": missing_guard_test_anchors,
        "mirror_docs_guard": MIRROR_DOCS_GUARD,
        "mirror_docs_guard_present": mirror_docs_guard_present,
        "missing_root_entries_anchors": missing_root_entries_anchors,
        "missing_root_surface_anchors": missing_root_surface_anchors,
        "missing_generated_guard_anchors": missing_generated_guard_anchors,
        "missing_pending_gate_anchors": missing_pending_gate_anchors,
        "missing_doc_anchors": missing_doc_anchors,
        "risks": risks,
    }
