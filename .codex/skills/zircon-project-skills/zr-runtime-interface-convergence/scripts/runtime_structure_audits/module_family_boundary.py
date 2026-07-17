from __future__ import annotations

from pathlib import Path


MODULE_FAMILY_ROOTS = {
    "animation": {
        "root_module": "animation",
        "doc": "docs/zircon_runtime/animation/runtime.md",
        "expected_file_count": 28,
        "required_doc_anchors": (
            "Root motion",
            "GPU skinning",
            "editor tooling",
            "runtime_animation_backlog_boundary_requires_doc_update",
        ),
        "required_guard": "runtime_animation_backlog_boundary_requires_doc_update",
    },
    "navigation": {
        "root_module": "navigation",
        "doc": "docs/zircon_runtime/navigation/runtime.md",
        "expected_file_count": 12,
        "required_doc_anchors": (
            "built-in fallback implementation",
            "folder-backed runtime owner split",
            "runtime_navigation_boundary_file_set_requires_doc_update",
        ),
        "required_guard": "runtime_navigation_boundary_file_set_requires_doc_update",
    },
    "diagnostic_log": {
        "root_module": "diagnostic_log",
        "doc": "docs/zircon_runtime/diagnostic_log/mod.md",
        "expected_file_count": 7,
        "required_doc_anchors": (
            "numeric snapshots to process text output",
            "diagnostic_log_snapshot_bridge_stays_single_owner",
        ),
        "required_guard": "diagnostic_log_snapshot_bridge_stays_single_owner",
    },
    "engine_module": {
        "root_module": "engine_module",
        "doc": "docs/zircon_runtime/engine_module/relationship.md",
        "expected_file_count": 8,
        "required_doc_anchors": (
            "declared layering",
            "engine_module_declared_layer_does_not_own_runtime_lifecycle",
        ),
        "required_guard": "engine_module_declared_layer_does_not_own_runtime_lifecycle",
    },
}
ROOT_SEAT_GUARD = "runtime_14_module_family_root_seats_match_documented_judgements"
MIRROR_DOCS_GUARD = "runtime_14_module_family_mirror_docs_match_structure_audit_counts"
ANIMATION_STATUS_JSON_GUARD = (
    "runtime_animation_status_json_boundary_sanitizes_non_finite_values"
)
ANIMATION_STATUS_JSON_ANCHORS = (
    "AnimationPlayerRuntimeStatus::sanitized_snapshot",
    "AnimationRuntimeStatus::sanitized_snapshot",
    "serialize_sanitized_non_negative_real",
    "deserialize_sanitized_non_negative_real",
    "serialize_normalized_real",
    "deserialize_normalized_real",
    "JSON boundary",
    "JSON `null` values from `NaN` or infinite runtime floats",
)
MODULE_FAMILY_GUARD_ANCHORS = tuple(
    config["required_guard"] for config in MODULE_FAMILY_ROOTS.values()
) + (ROOT_SEAT_GUARD, MIRROR_DOCS_GUARD, ANIMATION_STATUS_JSON_GUARD)
CARGO_GATE_ANCHORS = (
    "cargo test -p zircon_runtime --lib animation --locked",
    "cargo test -p zircon_runtime --lib navigation --locked",
    "cargo test -p zircon_runtime --lib diagnostic_log --locked",
    "cargo test -p zircon_runtime --lib engine_module --locked",
    "cargo test -p zircon_runtime --lib --locked",
)
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


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _relative(root: Path, path: Path) -> str:
    return path.relative_to(root).as_posix()


def _rust_file_count(path: Path) -> int:
    return sum(1 for child in path.rglob("*.rs") if child.is_file())


def _missing_snippets(sources: tuple[str, ...], snippets: tuple[str, ...]) -> list[str]:
    return [
        snippet
        for snippet in snippets
        if not any(snippet in source for source in sources)
    ]


def module_family_boundary_audit(root: Path) -> dict[str, object]:
    lib_file = root / "zircon_runtime" / "src" / "lib.rs"
    root_entries_files = tuple(root / relative for relative in ROOT_ENTRIES_GUARD_RELATIVES)
    animation_runtime_status_file = (
        root
        / "zircon_runtime"
        / "src"
        / "core"
        / "framework"
        / "animation"
        / "runtime_status.rs"
    )
    animation_framework_tests_file = (
        root
        / "zircon_runtime"
        / "src"
        / "core"
        / "framework"
        / "animation"
        / "tests.rs"
    )
    animation_framework_doc = (
        root / "docs" / "zircon_runtime" / "core" / "framework" / "animation.md"
    )
    diagnostic_bridge_file = root / "zircon_runtime" / "src" / "diagnostic_log" / "diagnostics.rs"
    engine_module_tests_file = root / "zircon_runtime" / "src" / "engine_module" / "tests.rs"
    runtime_14_plan = (
        root
        / "docs"
        / "plans"
        / "zircon_runtime"
        / "runtime"
        / "14-runtime-module-family-closeout.md"
    )
    runtime_index = root / "docs" / "plans" / "zircon_runtime" / "runtime" / "index.md"
    architecture_review = (
        root / "docs" / "engine-architecture" / "runtime-architecture-review-m0.md"
    )
    interface_convergence = (
        root / "docs" / "engine-architecture" / "runtime-interface-convergence.md"
    )

    lib_source = _read_text(lib_file) if lib_file.exists() else ""
    root_entries_source = "\n".join(
        _read_text(path) for path in root_entries_files if path.exists()
    )
    animation_runtime_status_source = (
        _read_text(animation_runtime_status_file)
        if animation_runtime_status_file.exists()
        else ""
    )
    animation_framework_tests_source = (
        _read_text(animation_framework_tests_file)
        if animation_framework_tests_file.exists()
        else ""
    )
    animation_framework_doc_source = (
        _read_text(animation_framework_doc) if animation_framework_doc.exists() else ""
    )
    diagnostic_bridge_source = (
        _read_text(diagnostic_bridge_file) if diagnostic_bridge_file.exists() else ""
    )
    engine_module_tests_source = (
        _read_text(engine_module_tests_file) if engine_module_tests_file.exists() else ""
    )
    doc_sources = tuple(
        _read_text(path)
        for path in (
            runtime_14_plan,
            runtime_index,
            architecture_review,
            interface_convergence,
        )
        if path.exists()
    )

    family_entries: list[dict[str, object]] = []
    missing_root_seats: list[str] = []
    missing_docs: list[str] = []
    missing_doc_anchors: list[dict[str, object]] = []
    file_count_mismatches: list[dict[str, object]] = []
    missing_guards: list[str] = []

    guard_sources = {
        "root_entries": root_entries_source,
        "diagnostic_log": diagnostic_bridge_source,
        "engine_module": engine_module_tests_source,
    }

    for family, config in MODULE_FAMILY_ROOTS.items():
        source_dir = root / "zircon_runtime" / "src" / family
        doc_path = root / config["doc"]
        expected_file_count = config["expected_file_count"]
        actual_file_count = _rust_file_count(source_dir) if source_dir.is_dir() else 0
        root_module = config["root_module"]
        has_root_seat = f"pub mod {root_module};" in lib_source

        if not has_root_seat:
            missing_root_seats.append(root_module)
        if not doc_path.exists():
            missing_docs.append(config["doc"])

        doc_source = _read_text(doc_path) if doc_path.exists() else ""
        for anchor in config["required_doc_anchors"]:
            if anchor not in doc_source:
                missing_doc_anchors.append(
                    {
                        "family": family,
                        "doc": config["doc"],
                        "anchor": anchor,
                    }
                )

        if actual_file_count != expected_file_count:
            file_count_mismatches.append(
                {
                    "family": family,
                    "expected": expected_file_count,
                    "actual": actual_file_count,
                }
            )

        guard = config["required_guard"]
        if not any(guard in source for source in guard_sources.values()):
            missing_guards.append(guard)

        family_entries.append(
            {
                "family": family,
                "source_dir": _relative(root, source_dir),
                "doc": config["doc"],
                "root_seat": has_root_seat,
                "rust_file_count": actual_file_count,
                "expected_file_count": expected_file_count,
                "required_guard": guard,
            }
        )

    root_seat_guard_present = ROOT_SEAT_GUARD in root_entries_source
    mirror_docs_guard_present = MIRROR_DOCS_GUARD in root_entries_source
    animation_status_json_guard_present = (
        ANIMATION_STATUS_JSON_GUARD in root_entries_source
    )
    missing_animation_status_json_anchors = _missing_snippets(
        (
            root_entries_source,
            animation_runtime_status_source,
            animation_framework_tests_source,
            animation_framework_doc_source,
        )
        + doc_sources,
        ANIMATION_STATUS_JSON_ANCHORS,
    )
    missing_module_family_guard_anchors = _missing_snippets(
        (root_entries_source, diagnostic_bridge_source, engine_module_tests_source),
        MODULE_FAMILY_GUARD_ANCHORS,
    )
    missing_cargo_gate_anchors = _missing_snippets(doc_sources, CARGO_GATE_ANCHORS)

    risks: list[str] = []
    if not lib_file.exists():
        risks.append("zircon_runtime/src/lib.rs is missing.")
    if missing_root_seats:
        risks.append("Runtime 14 module-family crate-root seats are missing.")
    if missing_docs:
        risks.append("Runtime 14 module-family mirror docs are missing.")
    if missing_doc_anchors:
        risks.append("Runtime 14 module-family docs are missing required judgement anchors.")
    if file_count_mismatches:
        risks.append("Runtime 14 module-family source file counts changed without audit sync.")
    if missing_guards:
        risks.append("Runtime 14 module-family Rust guard anchors are missing.")
    if not root_seat_guard_present:
        risks.append("Runtime 14 root-seat aggregate guard is missing.")
    if not mirror_docs_guard_present:
        risks.append("Runtime 14 mirror-doc aggregate guard is missing.")
    if not animation_status_json_guard_present:
        risks.append("Runtime 14 animation status JSON boundary guard is missing.")
    if missing_module_family_guard_anchors:
        risks.append("Runtime 14 module-family guard anchors are missing.")
    if missing_animation_status_json_anchors:
        risks.append("Runtime 14 animation status JSON boundary anchors are missing.")
    if missing_cargo_gate_anchors:
        risks.append("Runtime 14 pending module-family Cargo gate anchors are missing.")

    return {
        "families": family_entries,
        "expected_family_count": len(MODULE_FAMILY_ROOTS),
        "missing_root_seats": missing_root_seats,
        "missing_docs": missing_docs,
        "missing_doc_anchors": missing_doc_anchors,
        "file_count_mismatches": file_count_mismatches,
        "missing_guards": missing_guards,
        "root_seat_guard": ROOT_SEAT_GUARD,
        "root_seat_guard_present": root_seat_guard_present,
        "mirror_docs_guard": MIRROR_DOCS_GUARD,
        "mirror_docs_guard_present": mirror_docs_guard_present,
        "animation_status_json_guard": ANIMATION_STATUS_JSON_GUARD,
        "animation_status_json_guard_present": animation_status_json_guard_present,
        "animation_status_json_anchor_count": len(ANIMATION_STATUS_JSON_ANCHORS),
        "missing_animation_status_json_anchors": missing_animation_status_json_anchors,
        "module_family_guard_anchor_count": len(MODULE_FAMILY_GUARD_ANCHORS),
        "missing_module_family_guard_anchors": missing_module_family_guard_anchors,
        "cargo_gate_anchor_count": len(CARGO_GATE_ANCHORS),
        "missing_cargo_gate_anchors": missing_cargo_gate_anchors,
        "risks": risks,
    }
