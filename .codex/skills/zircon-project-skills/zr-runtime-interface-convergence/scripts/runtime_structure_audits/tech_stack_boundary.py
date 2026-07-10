from __future__ import annotations

from pathlib import Path

from .tech_stack_anchor_inventory import (
    CARGO_GATE_ANCHORS,
    EDITOR_BACKLOG_ANCHORS,
    EXPECTED_TECH_STACK_GUARD_COUNT,
    MIRROR_DOCS_GUARD,
    PHYSICS_DECISION_ANCHORS,
    TECH_STACK_BEHAVIOR_TEST_ANCHORS,
    TECH_STACK_DOC_ANCHORS,
    TECH_STACK_GUARDS,
    TEXT_STACK_DOC_ANCHORS,
)
from .tech_stack_source_inventory import (
    EXPECTED_EDITOR_ONLY_CANDIDATE_COUNT,
    EXPECTED_MANIFEST_COUNT,
    EXPECTED_NON_DEPENDENCY_COUNT,
    EXPECTED_ZIP_DEPENDENCY_COUNT,
    MANIFEST_FILES,
    NON_DEPENDENCIES,
    REQUIRED_VERSION_ANCHORS,
    ZIP_DEPENDENCY_LINE,
)


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


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


def _manifest_sources(root: Path) -> list[str]:
    skip_dirs = {".git", "target", "dev", "node_modules"}
    pending = [root]
    sources: list[str] = []

    while pending:
        current = pending.pop()
        try:
            entries = list(current.iterdir())
        except OSError:
            continue

        for entry in entries:
            if entry.is_dir():
                if entry.name in skip_dirs:
                    continue
                pending.append(entry)
            elif entry.name == "Cargo.toml":
                sources.append(_read_text(entry))

    return sources


def _manifest_declares_dependency(source: str, crate_name: str) -> bool:
    for raw_line in source.splitlines():
        line = raw_line.strip()
        if (
            line == f"[dependencies.{crate_name}]"
            or line == f"[workspace.dependencies.{crate_name}]"
            or line.startswith(f"{crate_name} =")
            or line.startswith(f"{crate_name}.workspace")
        ):
            return True
    return False


def _missing_snippets(sources: tuple[str, ...], snippets: tuple[str, ...]) -> list[str]:
    return [
        snippet
        for snippet in snippets
        if not any(snippet in source for source in sources)
    ]


def _declared_dependencies(
    sources: list[str],
    dependencies: tuple[str, ...],
) -> list[str]:
    return [
        dependency
        for dependency in dependencies
        if any(_manifest_declares_dependency(source, dependency) for source in sources)
    ]


def tech_stack_boundary_audit(root: Path) -> dict[str, object]:
    manifest_entries, missing_manifest_files = _file_entries(root, MANIFEST_FILES)
    manifest_sources = _manifest_sources(root)

    workspace_manifest = root / "Cargo.toml"
    runtime_manifest = root / "zircon_runtime/Cargo.toml"
    interface_manifest = root / "zircon_runtime_interface/Cargo.toml"
    editor_manifest = root / "zircon_editor/Cargo.toml"
    physics_manifest = root / "zircon_plugins/physics/runtime/Cargo.toml"
    physics_backend_files = (
        root / "zircon_plugins/physics/runtime/src/backend/mod.rs",
        root / "zircon_plugins/physics/runtime/src/backend/selection.rs",
    )
    physics_jolt_backend_files = (
        root / "zircon_plugins/physics/runtime/src/backend/jolt/mod.rs",
        root / "zircon_plugins/physics/runtime/src/backend/jolt/native_world.rs",
        root / "zircon_plugins/physics/runtime/src/backend/jolt/runtime.rs",
    )
    tech_stack_doc = root / "docs/engine-architecture/runtime-tech-stack.md"
    architecture_index = root / "docs/engine-architecture/index.md"
    runtime_01_plan = root / "docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md"
    runtime_index = root / "docs/plans/zircon_runtime/runtime/index.md"
    text_doc = root / "docs/zircon_runtime/ui/text.md"
    physics_options = root / "docs/zircon_plugins/physics-plugin-options.md"
    physics_runtime_doc = root / "docs/zircon_plugins/physics/runtime.md"
    editor_backlog = root / "docs/editor-and-tooling/runtime-editor-only-dependency-backlog.md"
    editor_index = root / "docs/editor-and-tooling/index.md"
    tech_stack_guard = root / "zircon_runtime/src/tests/extensions/tech_stack_dependency_guard.rs"
    tech_stack_mirror_guard = (
        root / "zircon_runtime/src/tests/runtime_absorption/tech_stack/mirror_docs.rs"
    )
    text_shaper_tests = root / "zircon_runtime/src/ui/tests/text_shaper.rs"
    physics_contract_mod = (
        root / "zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract/mod.rs"
    )
    physics_contract_step = (
        root / "zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract/step.rs"
    )
    cargo_gate_guard = (
        root / "zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/early.rs"
    )
    recent_static_guard = (
        root / "zircon_runtime/src/tests/runtime_absorption/plan_status/recent_static_guards.rs"
    )
    review = root / "docs/engine-architecture/runtime-architecture-review-m0.md"

    workspace_source = _read_text(workspace_manifest) if workspace_manifest.exists() else ""
    runtime_source = _read_text(runtime_manifest) if runtime_manifest.exists() else ""
    interface_source = _read_text(interface_manifest) if interface_manifest.exists() else ""
    editor_source = _read_text(editor_manifest) if editor_manifest.exists() else ""
    physics_manifest_source = _read_text(physics_manifest) if physics_manifest.exists() else ""
    physics_backend_source = "\n".join(
        _read_text(path) for path in physics_backend_files if path.exists()
    )
    physics_jolt_backend_files_present = all(
        path.exists() for path in physics_jolt_backend_files
    )
    tech_stack_source = _read_text(tech_stack_doc) if tech_stack_doc.exists() else ""
    architecture_index_source = (
        _read_text(architecture_index) if architecture_index.exists() else ""
    )
    runtime_01_plan_source = _read_text(runtime_01_plan) if runtime_01_plan.exists() else ""
    runtime_index_source = _read_text(runtime_index) if runtime_index.exists() else ""
    text_doc_source = _read_text(text_doc) if text_doc.exists() else ""
    physics_options_source = _read_text(physics_options) if physics_options.exists() else ""
    physics_runtime_doc_source = (
        _read_text(physics_runtime_doc) if physics_runtime_doc.exists() else ""
    )
    editor_backlog_source = _read_text(editor_backlog) if editor_backlog.exists() else ""
    editor_index_source = _read_text(editor_index) if editor_index.exists() else ""
    tech_stack_guard_source = (
        _read_text(tech_stack_guard) if tech_stack_guard.exists() else ""
    )
    tech_stack_mirror_guard_source = (
        _read_text(tech_stack_mirror_guard) if tech_stack_mirror_guard.exists() else ""
    )
    text_shaper_tests_source = (
        _read_text(text_shaper_tests) if text_shaper_tests.exists() else ""
    )
    physics_contract_mod_source = (
        _read_text(physics_contract_mod) if physics_contract_mod.exists() else ""
    )
    physics_contract_step_source = (
        _read_text(physics_contract_step) if physics_contract_step.exists() else ""
    )
    cargo_gate_guard_source = (
        _read_text(cargo_gate_guard) if cargo_gate_guard.exists() else ""
    )
    recent_static_guard_source = (
        _read_text(recent_static_guard) if recent_static_guard.exists() else ""
    )
    review_source = _read_text(review) if review.exists() else ""

    declared_removed_dependencies = _declared_dependencies(
        manifest_sources,
        NON_DEPENDENCIES,
    )
    zip_dependency_count = sum(
        1
        for source in manifest_sources
        if _manifest_declares_dependency(source, "zip")
    )
    runtime_jolt_feature_slot_count = runtime_source.count("backend-jolt = []")
    physics_jolt_dependency_feature_slot_count = physics_manifest_source.count(
        'backend-jolt = ["dep:joltc-sys"]'
    )
    joltc_sys_optional_dependency_present = any(
        line.strip().startswith("joltc-sys =") and "optional = true" in line
        for line in physics_manifest_source.splitlines()
    )
    jolt_feature_slot_count = (
        runtime_jolt_feature_slot_count
        + physics_jolt_dependency_feature_slot_count
    )
    rapier_or_avian_dependencies = _declared_dependencies(
        manifest_sources,
        ("rapier2d", "rapier3d", "avian2d", "avian3d"),
    )
    missing_version_anchors = _missing_snippets(
        (workspace_source, runtime_source),
        REQUIRED_VERSION_ANCHORS,
    )
    missing_tech_stack_doc_anchors = _missing_snippets(
        (tech_stack_source,),
        TECH_STACK_DOC_ANCHORS,
    )
    missing_text_doc_anchors = _missing_snippets(
        (text_doc_source,),
        TEXT_STACK_DOC_ANCHORS,
    )
    missing_physics_decision_anchors = _missing_snippets(
        (physics_options_source, physics_runtime_doc_source),
        PHYSICS_DECISION_ANCHORS,
    )
    missing_editor_backlog_anchors = _missing_snippets(
        (editor_backlog_source, editor_index_source),
        EDITOR_BACKLOG_ANCHORS,
    )
    missing_tech_stack_guards = _missing_snippets(
        (
            tech_stack_guard_source,
            tech_stack_mirror_guard_source,
            cargo_gate_guard_source,
            recent_static_guard_source,
            review_source,
            runtime_01_plan_source,
            runtime_index_source,
        ),
        TECH_STACK_GUARDS,
    )
    missing_behavior_test_anchors = _missing_snippets(
        (
            text_shaper_tests_source,
            physics_contract_mod_source,
            physics_contract_step_source,
        ),
        TECH_STACK_BEHAVIOR_TEST_ANCHORS,
    )
    missing_cargo_gate_anchors = _missing_snippets(
        (
            runtime_01_plan_source,
            runtime_index_source,
            cargo_gate_guard_source,
            recent_static_guard_source,
            review_source,
        ),
        CARGO_GATE_ANCHORS,
    )

    dependency_boundary_violations: list[str] = []
    if "wgpu" in interface_source:
        dependency_boundary_violations.append("zircon_runtime_interface declares wgpu")
    if "winit" in interface_source:
        dependency_boundary_violations.append("zircon_runtime_interface declares winit")
    if "wgpu" in editor_source:
        dependency_boundary_violations.append("zircon_editor declares wgpu")
    if "winit.workspace = true" not in editor_source:
        dependency_boundary_violations.append("zircon_editor direct winit boundary drifted")
    if "../../zr_vm/zr_vm_rust_binding" not in runtime_source:
        dependency_boundary_violations.append("ZrVM external path dependency anchor is missing")
    if "backend-zr-vm" not in runtime_source or "optional = true" not in runtime_source:
        dependency_boundary_violations.append("ZrVM real backend feature/optional gate drifted")
    if (
        'JOLT_BACKEND_AVAILABLE: bool = cfg!(feature = "backend-jolt")'
        not in physics_backend_source
    ):
        dependency_boundary_violations.append(
            "Jolt feature-gated backend availability anchor drifted"
        )
    if not physics_jolt_backend_files_present:
        dependency_boundary_violations.append("Physics Jolt backend owner set is incomplete")
    if (
        '#[cfg(feature = "backend-jolt")]\nmod jolt;'
        not in physics_backend_source
        or '#[cfg(feature = "backend-jolt")]\npub use jolt::JoltPhysicsBackend;'
        not in physics_backend_source
    ):
        dependency_boundary_violations.append("Physics Jolt backend module gate drifted")
    if "joltc-sys" in runtime_source:
        dependency_boundary_violations.append(
            "zircon_runtime declares plugin-owned joltc-sys"
        )
    if runtime_jolt_feature_slot_count != 1:
        dependency_boundary_violations.append("Runtime Jolt feature passthrough slot drifted")
    if physics_jolt_dependency_feature_slot_count != 1:
        dependency_boundary_violations.append(
            "Physics Jolt dependency-backed feature slot drifted"
        )
    if not joltc_sys_optional_dependency_present:
        dependency_boundary_violations.append("Physics joltc-sys optional dependency drifted")

    zip_dependency_violations: list[str] = []
    if zip_dependency_count != EXPECTED_ZIP_DEPENDENCY_COUNT:
        zip_dependency_violations.append(
            "zip dependency count is not limited to the runtime archive materializer"
        )
    if ZIP_DEPENDENCY_LINE not in runtime_source:
        zip_dependency_violations.append(
            "runtime zip dependency pin/features drifted from archive materializer policy"
        )

    risks: list[str] = []
    if missing_manifest_files:
        risks.append("Runtime 01 audited manifest file set is incomplete.")
    if len(manifest_entries) != EXPECTED_MANIFEST_COUNT:
        risks.append("Runtime 01 audited manifest count changed without audit sync.")
    if declared_removed_dependencies:
        risks.append("Removed or editor-only dependencies entered Cargo manifests.")
    if missing_version_anchors:
        risks.append("Runtime 01 required dependency version anchors are missing.")
    if dependency_boundary_violations:
        risks.append("Runtime 01 dependency boundary guards drifted.")
    if zip_dependency_violations:
        risks.append("Runtime 01 ZIP archive dependency policy drifted.")
    if missing_tech_stack_doc_anchors:
        risks.append("Runtime tech-stack authority doc anchors are missing.")
    if "runtime-tech-stack.md" not in architecture_index_source:
        risks.append("Runtime tech-stack doc is not linked from architecture index.")
    if missing_text_doc_anchors:
        risks.append("Runtime UI text stack matrix anchors are missing.")
    if missing_physics_decision_anchors:
        risks.append("Runtime 01 physics option decision anchors are missing.")
    if missing_editor_backlog_anchors:
        risks.append("Runtime 01 editor-only dependency backlog anchors are missing.")
    if missing_tech_stack_guards:
        risks.append("Runtime 01 Rust/static guard anchors are missing.")
    if missing_behavior_test_anchors:
        risks.append("Runtime 01 behavior test anchors are missing.")
    if MIRROR_DOCS_GUARD not in tech_stack_mirror_guard_source:
        risks.append("Runtime 01 tech-stack mirror-doc aggregate guard is missing.")
    if missing_cargo_gate_anchors:
        risks.append("Runtime 01 pending Cargo gate anchors are missing.")
    if jolt_feature_slot_count != 2:
        risks.append("Runtime 01 expects exactly two visible Jolt feature slots.")
    if rapier_or_avian_dependencies:
        risks.append("Rapier/Avian dependencies entered manifests without physics decision.")

    return {
        "manifest_files": manifest_entries,
        "expected_manifest_count": EXPECTED_MANIFEST_COUNT,
        "missing_manifest_files": missing_manifest_files,
        "declared_removed_dependencies": declared_removed_dependencies,
        "expected_non_dependency_count": EXPECTED_NON_DEPENDENCY_COUNT,
        "zip_dependency_count": zip_dependency_count,
        "expected_zip_dependency_count": EXPECTED_ZIP_DEPENDENCY_COUNT,
        "zip_dependency_violations": zip_dependency_violations,
        "missing_version_anchors": missing_version_anchors,
        "dependency_boundary_violations": dependency_boundary_violations,
        "missing_tech_stack_doc_anchors": missing_tech_stack_doc_anchors,
        "missing_text_doc_anchors": missing_text_doc_anchors,
        "missing_physics_decision_anchors": missing_physics_decision_anchors,
        "missing_editor_backlog_anchors": missing_editor_backlog_anchors,
        "missing_tech_stack_guards": missing_tech_stack_guards,
        "tech_stack_guard_count": len(TECH_STACK_GUARDS),
        "expected_tech_stack_guard_count": EXPECTED_TECH_STACK_GUARD_COUNT,
        "behavior_test_anchor_count": len(TECH_STACK_BEHAVIOR_TEST_ANCHORS),
        "missing_behavior_test_anchors": missing_behavior_test_anchors,
        "missing_cargo_gate_anchors": missing_cargo_gate_anchors,
        "mirror_docs_guard": MIRROR_DOCS_GUARD,
        "mirror_docs_guard_present": MIRROR_DOCS_GUARD in tech_stack_mirror_guard_source,
        "jolt_feature_slot_count": jolt_feature_slot_count,
        "runtime_jolt_feature_slot_count": runtime_jolt_feature_slot_count,
        "physics_jolt_dependency_feature_slot_count": physics_jolt_dependency_feature_slot_count,
        "joltc_sys_optional_dependency_present": joltc_sys_optional_dependency_present,
        "physics_jolt_backend_files_present": physics_jolt_backend_files_present,
        "jolt_backend_feature_gated": (
            'JOLT_BACKEND_AVAILABLE: bool = cfg!(feature = "backend-jolt")'
            in physics_backend_source
        ),
        "runtime_joltc_sys_dependency_absent": "joltc-sys" not in runtime_source,
        "rapier_or_avian_dependencies": rapier_or_avian_dependencies,
        "editor_only_candidate_count": EXPECTED_EDITOR_ONLY_CANDIDATE_COUNT,
        "risks": risks,
    }
