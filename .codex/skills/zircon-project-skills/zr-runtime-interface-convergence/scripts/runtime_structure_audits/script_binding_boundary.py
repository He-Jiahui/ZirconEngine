from __future__ import annotations

from pathlib import Path


EXPECTED_FIXED_HOST_MODULES = 6
EXPECTED_FIXED_HOST_FUNCTIONS = 52
EXPECTED_TYPE_DESCRIPTORS = 2
EXPECTED_BUILTIN_CALLBACKS = 11
EXPECTED_GAMEPLAY_CALLBACKS = 39
EXPECTED_MACRO_HOST_FUNCTIONS = 2
EXPECTED_HOST_CAPABILITIES = 11
SCRIPT_LEDGER_TEST_MAX_LINES = 700
GAMEPLAY_TEST_MAX_LINES = 1000

RUNTIME_13_SOURCE_FILES = (
    "zircon_runtime/src/script/vm/host/builtin_host_modules.rs",
    "zircon_runtime/src/script/vm/gameplay_host.rs",
    "zircon_runtime/src/script/vm/gameplay_host/combat.rs",
    "zircon_runtime/src/script/vm/gameplay_host/components.rs",
    "zircon_runtime/src/script/vm/gameplay_host/error.rs",
    "zircon_runtime/src/script/vm/gameplay_host/input.rs",
    "zircon_runtime/src/script/vm/gameplay_host/lifecycle.rs",
    "zircon_runtime/src/script/vm/gameplay_host/navigation.rs",
    "zircon_runtime/src/script/vm/gameplay_host/script_bindings.rs",
    "zircon_runtime/src/script/vm/gameplay_host/transform.rs",
    "zircon_runtime/src/script/vm/gameplay_host/values.rs",
    "zircon_runtime/src/script/vm/host/bridge_host_module.rs",
    "zircon_runtime/src/script/vm/host/host_export_registry.rs",
    "zircon_runtime/src/script/vm/host/script_call_table.rs",
    "zircon_runtime/src/core/framework/script.rs",
    "zircon_runtime/src/script/vm/capability_set.rs",
    "zircon_runtime/src/script/vm/handles.rs",
    "zircon_runtime/src/script/vm/runtime_context.rs",
)
RUNTIME_13_TEST_FILES = (
    "zircon_runtime/src/tests/runtime_absorption/script_host_ledger.rs",
    "zircon_runtime/src/tests/runtime_absorption/script_binding.rs",
    "zircon_runtime/src/script/vm/gameplay_host/tests.rs",
)
RUNTIME_13_GUARD_FILES = (
    "zircon_runtime/src/tests/runtime_absorption/script_host_ledger.rs",
    "zircon_runtime/src/tests/runtime_absorption/script_binding.rs",
    "zircon_runtime/src/tests/runtime_absorption/script_host_ledger/ledger.rs",
    "zircon_runtime/src/tests/runtime_absorption/script_host_ledger/capability.rs",
    "zircon_runtime/src/tests/runtime_absorption/script_host_ledger/ecs_facade.rs",
    "zircon_runtime/src/tests/runtime_absorption/script_binding/gameplay_host.rs",
    "zircon_runtime/src/tests/runtime_absorption/script_binding/mirror_docs.rs",
    "zircon_runtime/src/script/vm/gameplay_host/tests/combat_lifecycle.rs",
    "zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/late/runtime_13.rs",
)
FIXED_HOST_MODULES = (
    "zr.zircon.foundation",
    "zr.zircon.asset",
    "zr.zircon.scene",
    "zr.zircon.render",
    "zr.zircon.math",
    "zr.zircon.gameplay",
)
HOST_CAPABILITIES = (
    "foundation.log",
    "foundation.time",
    "foundation.event",
    "asset.query",
    "scene.query",
    "scene.handle",
    "render.query",
    "gameplay.input",
    "gameplay.entity",
    "gameplay.navigation",
    "bridge.call",
)
LEDGER_DOC_ANCHORS = (
    "6 host modules, 52 fixed host functions, and 2 fixed script type descriptors",
    "`zr.zircon.bridge`",
    "dynamic module shape contract",
    "Value descriptors",
    "Host handles",
    "Serialized payloads",
    "The current script gameplay ECS path is `zr.zircon.gameplay` through `ScriptRuntimeCallContext`",
    "`ZrHostEcsApiV1` belongs to the native/plugin ABI layer",
    "host_function_registry_matches_documented_ledger",
    "host_capability_representatives_are_declared_on_registered_modules",
    "host_function_without_required_capability_is_rejected_with_explicit_error",
)
RUNTIME_13_GUARDS = (
    "host_function_registry_matches_documented_ledger",
    "host_function_registry_ledger_guard_rejects_missing_entry",
    "host_capability_representatives_are_declared_on_registered_modules",
    "host_function_without_required_capability_is_rejected_with_explicit_error",
    "script_ecs_access_path_stays_on_gameplay_facade_not_native_ecs_abi",
    "script_held_entity_handle_reports_invalid_after_despawn",
    "runtime_13_script_binding_cargo_gate_stays_visible_until_script_filters_pass",
    "runtime_13_script_binding_mirror_docs_match_structure_audit_counts",
    "runtime_13_gameplay_host_owner_split_keeps_domain_files",
)
MIRROR_DOCS_GUARD = "runtime_13_script_binding_mirror_docs_match_structure_audit_counts"
BRIDGE_ANCHORS = (
    "pub const BRIDGE_HOST_MODULE: &str = \"zr.zircon.bridge\";",
    "pub const BRIDGE_HOST_CAPABILITY: &str = \"bridge.call\";",
    "ScriptBridgeMethodDescriptor",
    "register_bridge_host_module",
    "BridgeInvocationTable",
)
GAMEPLAY_FACADE_ANCHORS = (
    "const GAMEPLAY_MODULE: &str = \"zr.zircon.gameplay\";",
    "pub fn register_gameplay_host_module(",
    "current_script_runtime_call_context()?",
    "pub struct ScriptRuntimeCallContext",
    "pub level: LevelSystem",
    "pub entity: EntityId",
)
CARGO_GATE_ANCHORS = (
    "cargo test -p zircon_runtime --lib script --locked -- --nocapture",
    "runtime_13_script_binding_cargo_gate_stays_visible_until_script_filters_pass",
    "code_static_pending_cargo",
)
FORBIDDEN_SCRIPT_NATIVE_ECS_ABI = (
    "ZrHostEcsApiV1",
    "ZrHostEcsApi",
    "HostEcsApi",
)


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _relative(root: Path, path: Path) -> str:
    return path.relative_to(root).as_posix()


def _file_line_count(path: Path) -> int:
    return len(_read_text(path).splitlines())


def _count_occurrences(source: str, needle: str) -> int:
    return source.count(needle)


def _missing_snippets(sources: tuple[str, ...], snippets: tuple[str, ...]) -> list[str]:
    return [
        snippet
        for snippet in snippets
        if not any(snippet in source for source in sources)
    ]


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


def _script_native_abi_references(root: Path) -> list[dict[str, object]]:
    script_root = root / "zircon_runtime/src/script"
    if not script_root.is_dir():
        return []

    references: list[dict[str, object]] = []
    for path in sorted(script_root.rglob("*.rs")):
        source = _read_text(path)
        relative = _relative(root, path)
        for forbidden in FORBIDDEN_SCRIPT_NATIVE_ECS_ABI:
            if forbidden not in source:
                continue
            for line_no, line in enumerate(source.splitlines(), start=1):
                if forbidden in line:
                    references.append(
                        {
                            "path": relative,
                            "line": line_no,
                            "symbol": forbidden,
                            "snippet": line.strip(),
                        }
                    )
    return references


def script_binding_boundary_audit(root: Path) -> dict[str, object]:
    builtin_host = root / "zircon_runtime/src/script/vm/host/builtin_host_modules.rs"
    gameplay_host = root / "zircon_runtime/src/script/vm/gameplay_host.rs"
    bridge_host = root / "zircon_runtime/src/script/vm/host/bridge_host_module.rs"
    runtime_context = root / "zircon_runtime/src/script/vm/runtime_context.rs"
    script_host_ledger_guard = (
        root / "zircon_runtime/src/tests/runtime_absorption/script_host_ledger.rs"
    )
    script_binding_guard = (
        root / "zircon_runtime/src/tests/runtime_absorption/script_binding.rs"
    )
    gameplay_host_tests = root / "zircon_runtime/src/script/vm/gameplay_host/tests.rs"
    cargo_gate_guard = (
        root
        / "zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/late.rs"
    )
    runtime_13_plan = (
        root
        / "docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md"
    )
    runtime_index = root / "docs/plans/zircon_runtime/runtime/index.md"
    function_ledger = root / "docs/zircon_runtime/script/vm/host/function_ledger.md"
    review = root / "docs/engine-architecture/runtime-architecture-review-m0.md"

    builtin_source = _read_text(builtin_host) if builtin_host.exists() else ""
    gameplay_source = _read_text(gameplay_host) if gameplay_host.exists() else ""
    bridge_source = _read_text(bridge_host) if bridge_host.exists() else ""
    runtime_context_source = _read_text(runtime_context) if runtime_context.exists() else ""
    script_host_ledger_source = (
        _read_text(script_host_ledger_guard)
        if script_host_ledger_guard.exists()
        else ""
    )
    script_binding_guard_source = (
        _read_text(script_binding_guard) if script_binding_guard.exists() else ""
    )
    gameplay_host_tests_source = (
        _read_text(gameplay_host_tests) if gameplay_host_tests.exists() else ""
    )
    cargo_gate_guard_source = (
        _read_text(cargo_gate_guard) if cargo_gate_guard.exists() else ""
    )
    guard_paths = tuple(root / path for path in RUNTIME_13_GUARD_FILES)
    missing_guard_files = [
        path.relative_to(root).as_posix() for path in guard_paths if not path.exists()
    ]
    guard_source = "\n".join(_read_text(path) for path in guard_paths if path.exists())
    doc_sources = tuple(
        _read_text(path)
        for path in (runtime_13_plan, runtime_index, function_ledger, review)
        if path.exists()
    )

    source_files, missing_source_files = _file_entries(root, RUNTIME_13_SOURCE_FILES)
    test_files, missing_test_files = _file_entries(root, RUNTIME_13_TEST_FILES)

    builtin_callback_count = _count_occurrences(
        builtin_source,
        "HostExportFunction::new(",
    )
    gameplay_callback_count = _count_occurrences(
        gameplay_source,
        "HostExportFunction::new(",
    )
    macro_host_function_count = _count_occurrences(
        builtin_source,
        "#[crate::zircon_host_function(",
    )
    documented_fixed_modules = [
        module for module in FIXED_HOST_MODULES if any(module in source for source in doc_sources)
    ]
    documented_capabilities = [
        capability
        for capability in HOST_CAPABILITIES
        if any(capability in source for source in doc_sources)
    ]
    missing_fixed_modules = [
        module for module in FIXED_HOST_MODULES if module not in documented_fixed_modules
    ]
    missing_capabilities = [
        capability
        for capability in HOST_CAPABILITIES
        if capability not in documented_capabilities
    ]
    missing_ledger_doc_anchors = _missing_snippets(doc_sources, LEDGER_DOC_ANCHORS)
    missing_runtime_13_guards = _missing_snippets(
        (guard_source,),
        RUNTIME_13_GUARDS,
    )
    missing_bridge_anchors = _missing_snippets((bridge_source,), BRIDGE_ANCHORS)
    missing_gameplay_facade_anchors = _missing_snippets(
        (gameplay_source, runtime_context_source),
        GAMEPLAY_FACADE_ANCHORS,
    )
    missing_cargo_gate_anchors = _missing_snippets(doc_sources, CARGO_GATE_ANCHORS)
    native_ecs_abi_references = _script_native_abi_references(root)

    oversized_test_files = [
        file
        for file in test_files
        if (
            file["path"] == "zircon_runtime/src/tests/runtime_absorption/script_host_ledger.rs"
            and file["lines"] > SCRIPT_LEDGER_TEST_MAX_LINES
        )
        or (
            file["path"] == "zircon_runtime/src/script/vm/gameplay_host/tests.rs"
            and file["lines"] > GAMEPLAY_TEST_MAX_LINES
        )
    ]

    risks: list[str] = []
    if missing_source_files:
        risks.append("Runtime 13 source files for the script binding boundary are missing.")
    if missing_test_files:
        risks.append("Runtime 13 guard or gameplay test files are missing.")
    if builtin_callback_count != EXPECTED_BUILTIN_CALLBACKS:
        risks.append("Runtime 13 builtin host callback count changed without ledger sync.")
    if gameplay_callback_count != EXPECTED_GAMEPLAY_CALLBACKS:
        risks.append("Runtime 13 gameplay host callback count changed without ledger sync.")
    if macro_host_function_count != EXPECTED_MACRO_HOST_FUNCTIONS:
        risks.append("Runtime 13 macro host-function count changed without ledger sync.")
    if missing_fixed_modules:
        risks.append("Runtime 13 fixed host modules are missing from docs.")
    if missing_capabilities:
        risks.append("Runtime 13 host capability anchors are missing from docs.")
    if missing_ledger_doc_anchors:
        risks.append("Runtime 13 ledger or architecture docs are missing required anchors.")
    if missing_runtime_13_guards:
        risks.append("Runtime 13 Rust guard anchors are missing.")
    if missing_guard_files:
        risks.append("Runtime 13 script-binding guard/test owner files are missing.")
    if MIRROR_DOCS_GUARD not in guard_source:
        risks.append("Runtime 13 script-binding mirror-doc aggregate guard is missing.")
    if missing_bridge_anchors:
        risks.append("Runtime 13 bridge dynamic-module shape anchors are missing.")
    if missing_gameplay_facade_anchors:
        risks.append("Runtime 13 gameplay facade anchors are missing.")
    if missing_cargo_gate_anchors:
        risks.append("Runtime 13 pending Cargo gate anchors are missing.")
    if native_ecs_abi_references:
        risks.append("Runtime 13 script source references native ECS ABI symbols.")
    if oversized_test_files:
        risks.append("Runtime 13 guard test files exceed their owner line budget.")

    return {
        "source_files": source_files,
        "expected_source_file_count": len(RUNTIME_13_SOURCE_FILES),
        "missing_source_files": missing_source_files,
        "test_files": test_files,
        "expected_test_file_count": len(RUNTIME_13_TEST_FILES),
        "missing_test_files": missing_test_files,
        "fixed_host_module_count": len(FIXED_HOST_MODULES),
        "expected_fixed_host_module_count": EXPECTED_FIXED_HOST_MODULES,
        "fixed_host_function_count": EXPECTED_FIXED_HOST_FUNCTIONS,
        "type_descriptor_count": EXPECTED_TYPE_DESCRIPTORS,
        "builtin_callback_count": builtin_callback_count,
        "expected_builtin_callback_count": EXPECTED_BUILTIN_CALLBACKS,
        "gameplay_callback_count": gameplay_callback_count,
        "expected_gameplay_callback_count": EXPECTED_GAMEPLAY_CALLBACKS,
        "macro_host_function_count": macro_host_function_count,
        "expected_macro_host_function_count": EXPECTED_MACRO_HOST_FUNCTIONS,
        "host_capability_count": len(HOST_CAPABILITIES),
        "expected_host_capability_count": EXPECTED_HOST_CAPABILITIES,
        "missing_fixed_modules": missing_fixed_modules,
        "missing_capabilities": missing_capabilities,
        "missing_ledger_doc_anchors": missing_ledger_doc_anchors,
        "missing_runtime_13_guards": missing_runtime_13_guards,
        "guard_anchor_count": len(RUNTIME_13_GUARDS),
        "expected_guard_file_count": len(RUNTIME_13_GUARD_FILES),
        "missing_guard_files": missing_guard_files,
        "missing_bridge_anchors": missing_bridge_anchors,
        "missing_gameplay_facade_anchors": missing_gameplay_facade_anchors,
        "missing_cargo_gate_anchors": missing_cargo_gate_anchors,
        "mirror_docs_guard": MIRROR_DOCS_GUARD,
        "mirror_docs_guard_present": MIRROR_DOCS_GUARD in guard_source,
        "native_ecs_abi_references": native_ecs_abi_references,
        "max_script_ledger_test_lines": SCRIPT_LEDGER_TEST_MAX_LINES,
        "max_gameplay_test_lines": GAMEPLAY_TEST_MAX_LINES,
        "oversized_test_files": oversized_test_files,
        "risks": risks,
    }
