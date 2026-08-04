from __future__ import annotations

import tomllib
from pathlib import Path

from .tech_stack_anchor_inventory import (
    CARGO_GATE_ANCHORS,
    EDITOR_BACKLOG_ANCHORS,
    EXPECTED_TECH_STACK_GUARD_COUNT,
    KIRA_TECH_STACK_DOC_ANCHORS,
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
    KIRA_DEPENDENCY_VERSION,
    KIRA_OWNER_MANIFEST,
    MANIFEST_FILES,
    NON_DEPENDENCIES,
    REQUIRED_VERSION_ANCHORS,
    ZR_VM_BACKEND_FEATURE,
    ZR_VM_BINDING_DEPENDENCY_PREFIX,
    ZR_VM_EXTERNAL_PATH_PREFIX,
    ZR_VM_PLUGIN_MANIFEST,
    ZIP_DEPENDENCY_LINE,
)


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _read_toml(path: Path) -> dict[str, object]:
    with path.open("rb") as source:
        return tomllib.load(source)


def _manifest_dependency_specs(
    manifest: dict[str, object],
) -> list[tuple[str, object]]:
    declarations: list[tuple[str, object]] = []

    def collect(owner: object) -> None:
        if not isinstance(owner, dict):
            return
        for table_name in ("dependencies", "dev-dependencies", "build-dependencies"):
            table = owner.get(table_name)
            if isinstance(table, dict):
                declarations.extend(table.items())

    collect(manifest)
    collect(manifest.get("workspace"))
    targets = manifest.get("target")
    if isinstance(targets, dict):
        for target in targets.values():
            collect(target)
    return declarations


def _manifest_runtime_dependency_specs(
    manifest: dict[str, object],
) -> list[tuple[str, object]]:
    declarations: list[tuple[str, object]] = []

    def collect(owner: object) -> None:
        if not isinstance(owner, dict):
            return
        table = owner.get("dependencies")
        if isinstance(table, dict):
            declarations.extend(table.items())

    collect(manifest)
    targets = manifest.get("target")
    if isinstance(targets, dict):
        for target in targets.values():
            collect(target)
    return declarations


def _dependency_package_name(name: str, spec: object) -> str:
    if isinstance(spec, dict) and isinstance(spec.get("package"), str):
        return spec["package"]
    return name


def _dependency_version(spec: object) -> str | None:
    if isinstance(spec, str):
        return spec
    if isinstance(spec, dict) and isinstance(spec.get("version"), str):
        return spec["version"]
    return None


def _manifest_package_declarations(
    source: str,
    package_name: str,
) -> list[tuple[str, object]]:
    try:
        manifest = tomllib.loads(source)
    except tomllib.TOMLDecodeError:
        return []
    return [
        (name, spec)
        for name, spec in _manifest_dependency_specs(manifest)
        if _dependency_package_name(name, spec) == package_name
    ]


def _manifest_has_exact_single_package_pin(
    source: str,
    package_name: str,
    expected_version: str,
) -> bool:
    try:
        manifest = tomllib.loads(source)
    except tomllib.TOMLDecodeError:
        return False
    declarations = _manifest_package_declarations(source, package_name)
    runtime_declarations = [
        (name, spec)
        for name, spec in _manifest_runtime_dependency_specs(manifest)
        if _dependency_package_name(name, spec) == package_name
    ]
    return (
        len(declarations) == 1
        and len(runtime_declarations) == 1
        and _dependency_version(runtime_declarations[0][1]) == expected_version
    )


def _is_zr_vm_binding_dependency(name: str, spec: object) -> bool:
    return _dependency_package_name(name, spec).startswith(
        ZR_VM_BINDING_DEPENDENCY_PREFIX
    )


def _path_is_within(path: Path, owner: Path) -> bool:
    try:
        path.resolve(strict=False).relative_to(owner.resolve(strict=False))
    except ValueError:
        return False
    return True


def _is_zr_vm_feature_name(name: str) -> bool:
    normalized = name.casefold().replace("_", "-")
    return "zr-vm" in normalized or "zrvm" in normalized


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


def _manifest_scan_error(root: Path, path: Path, error: OSError) -> str:
    try:
        relative_path = path.relative_to(root).as_posix() or "."
    except ValueError:
        relative_path = path.as_posix()
    return f"{relative_path}: {type(error).__name__}: {error}"


def _manifest_source_entries(
    root: Path,
) -> tuple[list[tuple[str, str]], list[str]]:
    skip_dirs = {".git", "target", "dev", "node_modules"}
    sources: list[tuple[str, str]] = []
    errors: list[str] = []
    workspace_manifest = root / "Cargo.toml"
    if workspace_manifest.is_file():
        try:
            sources.append(("Cargo.toml", _read_text(workspace_manifest)))
        except OSError as error:
            errors.append(_manifest_scan_error(root, workspace_manifest, error))

    try:
        pending = [
            entry
            for entry in root.iterdir()
            if entry.is_dir() and entry.name.startswith("zircon_")
        ]
    except OSError as error:
        errors.append(_manifest_scan_error(root, root, error))
        pending = []

    while pending:
        current = pending.pop()
        try:
            entries = list(current.iterdir())
        except OSError as error:
            errors.append(_manifest_scan_error(root, current, error))
            continue

        for entry in entries:
            if entry.is_dir():
                if entry.name in skip_dirs:
                    continue
                pending.append(entry)
            elif entry.name == "Cargo.toml":
                try:
                    sources.append((entry.relative_to(root).as_posix(), _read_text(entry)))
                except OSError as error:
                    errors.append(_manifest_scan_error(root, entry, error))

    return (
        sorted(sources, key=lambda item: item[0]),
        sorted(errors),
    )


def _manifest_dependency_owners(root: Path, crate_name: str) -> list[str]:
    entries, _ = _manifest_source_entries(root)
    return [
        path
        for path, source in entries
        if _manifest_declares_package(source, crate_name)
    ]


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


def _manifest_declares_package(source: str, package_name: str) -> bool:
    try:
        manifest = tomllib.loads(source)
    except tomllib.TOMLDecodeError:
        return _manifest_declares_dependency(source, package_name)
    return any(
        _dependency_package_name(name, spec) == package_name
        for name, spec in _manifest_dependency_specs(manifest)
    )


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
        if any(_manifest_declares_package(source, dependency) for source in sources)
    ]


def tech_stack_boundary_audit(root: Path) -> dict[str, object]:
    manifest_entries, missing_manifest_files = _file_entries(root, MANIFEST_FILES)
    manifest_source_entries, manifest_scan_errors = _manifest_source_entries(root)
    manifest_sources = [source for _, source in manifest_source_entries]

    workspace_manifest = root / "Cargo.toml"
    runtime_manifest = root / "zircon_runtime/Cargo.toml"
    interface_manifest = root / "zircon_runtime_interface/Cargo.toml"
    editor_manifest = root / "zircon_editor/Cargo.toml"
    physics_manifest = root / "zircon_plugins/physics/runtime/Cargo.toml"
    zr_vm_plugin_manifest = root / ZR_VM_PLUGIN_MANIFEST
    zr_vm_external_root = (zr_vm_plugin_manifest.parent / ZR_VM_EXTERNAL_PATH_PREFIX).resolve(
        strict=False
    )
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
    review = root / "docs/engine-architecture/runtime-architecture-review-m0.md"

    workspace_source = _read_text(workspace_manifest) if workspace_manifest.exists() else ""
    runtime_source = _read_text(runtime_manifest) if runtime_manifest.exists() else ""
    interface_source = _read_text(interface_manifest) if interface_manifest.exists() else ""
    editor_source = _read_text(editor_manifest) if editor_manifest.exists() else ""
    physics_manifest_source = _read_text(physics_manifest) if physics_manifest.exists() else ""
    runtime_manifest_data = _read_toml(runtime_manifest) if runtime_manifest.exists() else {}
    zr_vm_plugin_manifest_data = (
        _read_toml(zr_vm_plugin_manifest) if zr_vm_plugin_manifest.exists() else {}
    )
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
    review_source = _read_text(review) if review.exists() else ""

    declared_removed_dependencies = _declared_dependencies(
        manifest_sources,
        NON_DEPENDENCIES,
    )
    kira_dependency_owners = [
        path
        for path, source in manifest_source_entries
        if _manifest_declares_package(source, "kira")
    ]
    kira_owner_source = next(
        (
            source
            for path, source in manifest_source_entries
            if path == KIRA_OWNER_MANIFEST
        ),
        "",
    )
    kira_owner_declarations = _manifest_package_declarations(kira_owner_source, "kira")
    kira_owner_dependency_versions = [
        _dependency_version(spec) for _, spec in kira_owner_declarations
    ]
    kira_owner_version_pinned = _manifest_has_exact_single_package_pin(
        kira_owner_source,
        "kira",
        KIRA_DEPENDENCY_VERSION,
    )
    kira_owner_violations = [
        path for path in kira_dependency_owners if path != KIRA_OWNER_MANIFEST
    ]
    zip_dependency_count = sum(
        1
        for source in manifest_sources
        if _manifest_declares_package(source, "zip")
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
    missing_kira_tech_stack_doc_anchors = _missing_snippets(
        (tech_stack_source,),
        KIRA_TECH_STACK_DOC_ANCHORS,
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
            review_source,
        ),
        CARGO_GATE_ANCHORS,
    )

    zr_vm_plugin_features = zr_vm_plugin_manifest_data.get("features")
    if not isinstance(zr_vm_plugin_features, dict):
        zr_vm_plugin_features = {}
    zr_vm_backend_feature = zr_vm_plugin_features.get(ZR_VM_BACKEND_FEATURE)
    zr_vm_plugin_dependencies = [
        (name, spec)
        for name, spec in _manifest_dependency_specs(zr_vm_plugin_manifest_data)
        if _is_zr_vm_binding_dependency(name, spec)
    ]
    zr_vm_plugin_feature_entries = (
        {entry for entry in zr_vm_backend_feature if isinstance(entry, str)}
        if isinstance(zr_vm_backend_feature, list)
        else set()
    )
    zr_vm_plugin_binding_dependencies_optional = bool(zr_vm_plugin_dependencies) and all(
        isinstance(spec, dict) and spec.get("optional") is True
        for _, spec in zr_vm_plugin_dependencies
    )
    zr_vm_plugin_binding_dependencies_external = bool(zr_vm_plugin_dependencies) and all(
        isinstance(spec, dict)
        and isinstance(spec.get("path"), str)
        and _path_is_within(zr_vm_plugin_manifest.parent / spec["path"], zr_vm_external_root)
        for _, spec in zr_vm_plugin_dependencies
    )
    zr_vm_plugin_backend_feature_gates_bindings = bool(zr_vm_plugin_dependencies) and all(
        f"dep:{name}" in zr_vm_plugin_feature_entries
        for name, _ in zr_vm_plugin_dependencies
    )
    runtime_features = runtime_manifest_data.get("features")
    if not isinstance(runtime_features, dict):
        runtime_features = {}
    runtime_zr_vm_dependencies = [
        name
        for name, spec in _manifest_dependency_specs(runtime_manifest_data)
        if _is_zr_vm_binding_dependency(name, spec)
    ]
    runtime_zr_vm_feature_entries = {
        entry
        for feature_entries in runtime_features.values()
        if isinstance(feature_entries, list)
        for entry in feature_entries
        if isinstance(entry, str)
    }
    runtime_zr_vm_owner_absent = (
        not any(_is_zr_vm_feature_name(name) for name in runtime_features)
        and not runtime_zr_vm_dependencies
        and not any(
            ZR_VM_BACKEND_FEATURE in entry
            or ZR_VM_BINDING_DEPENDENCY_PREFIX in entry
            for entry in runtime_zr_vm_feature_entries
        )
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
    if not zr_vm_plugin_manifest.exists():
        dependency_boundary_violations.append("ZrVM plugin-owned manifest is missing")
    if ZR_VM_BACKEND_FEATURE not in zr_vm_plugin_features:
        dependency_boundary_violations.append("ZrVM plugin backend feature is missing")
    if not zr_vm_plugin_dependencies:
        dependency_boundary_violations.append("ZrVM plugin binding dependencies are missing")
    elif not zr_vm_plugin_binding_dependencies_optional:
        dependency_boundary_violations.append("ZrVM plugin binding dependencies are not optional")
    if zr_vm_plugin_dependencies and not zr_vm_plugin_binding_dependencies_external:
        dependency_boundary_violations.append("ZrVM plugin binding path dependencies drifted")
    if zr_vm_plugin_dependencies and not zr_vm_plugin_backend_feature_gates_bindings:
        dependency_boundary_violations.append(
            "ZrVM plugin backend feature does not gate every binding dependency"
        )
    if not runtime_zr_vm_owner_absent:
        dependency_boundary_violations.append(
            "zircon_runtime declares plugin-owned ZrVM backend state"
        )
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
    if manifest_scan_errors:
        risks.append("Runtime 01 product manifest scan was incomplete.")
    if len(manifest_entries) != EXPECTED_MANIFEST_COUNT:
        risks.append("Runtime 01 audited manifest count changed without audit sync.")
    if declared_removed_dependencies:
        risks.append("Removed or editor-only dependencies entered Cargo manifests.")
    if KIRA_OWNER_MANIFEST not in kira_dependency_owners:
        risks.append("Sound runtime Kira dependency owner is missing.")
    if kira_owner_violations:
        risks.append("Kira dependency escaped the Sound runtime owner.")
    if not kira_owner_version_pinned:
        risks.append("Sound runtime Kira dependency pin drifted.")
    if missing_version_anchors:
        risks.append("Runtime 01 required dependency version anchors are missing.")
    if dependency_boundary_violations:
        risks.append("Runtime 01 dependency boundary guards drifted.")
    if zip_dependency_violations:
        risks.append("Runtime 01 ZIP archive dependency policy drifted.")
    if missing_tech_stack_doc_anchors:
        risks.append("Runtime tech-stack authority doc anchors are missing.")
    if missing_kira_tech_stack_doc_anchors:
        risks.append("Runtime 01 Kira Sound owner documentation drifted.")
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
        "manifest_scan_errors": manifest_scan_errors,
        "declared_removed_dependencies": declared_removed_dependencies,
        "expected_non_dependency_count": EXPECTED_NON_DEPENDENCY_COUNT,
        "kira_dependency_owners": kira_dependency_owners,
        "kira_owner_manifest": KIRA_OWNER_MANIFEST,
        "kira_owner_version_pinned": kira_owner_version_pinned,
        "kira_owner_dependency_declaration_count": len(kira_owner_declarations),
        "kira_owner_dependency_versions": kira_owner_dependency_versions,
        "kira_owner_violations": kira_owner_violations,
        "zip_dependency_count": zip_dependency_count,
        "expected_zip_dependency_count": EXPECTED_ZIP_DEPENDENCY_COUNT,
        "zip_dependency_violations": zip_dependency_violations,
        "missing_version_anchors": missing_version_anchors,
        "dependency_boundary_violations": dependency_boundary_violations,
        "missing_tech_stack_doc_anchors": missing_tech_stack_doc_anchors,
        "missing_kira_tech_stack_doc_anchors": missing_kira_tech_stack_doc_anchors,
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
        "zr_vm_plugin_manifest_present": zr_vm_plugin_manifest.exists(),
        "zr_vm_plugin_backend_feature_present": (
            ZR_VM_BACKEND_FEATURE in zr_vm_plugin_features
        ),
        "zr_vm_plugin_binding_dependency_count": len(zr_vm_plugin_dependencies),
        "zr_vm_plugin_binding_dependencies_optional": (
            zr_vm_plugin_binding_dependencies_optional
        ),
        "zr_vm_plugin_binding_dependencies_external": (
            zr_vm_plugin_binding_dependencies_external
        ),
        "zr_vm_plugin_backend_feature_gates_bindings": (
            zr_vm_plugin_backend_feature_gates_bindings
        ),
        "runtime_zr_vm_owner_absent": runtime_zr_vm_owner_absent,
        "rapier_or_avian_dependencies": rapier_or_avian_dependencies,
        "editor_only_candidate_count": EXPECTED_EDITOR_ONLY_CANDIDATE_COUNT,
        "risks": risks,
    }
