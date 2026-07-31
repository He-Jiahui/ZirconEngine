from __future__ import annotations

import re
from pathlib import Path

from runtime_structure_audits.native_plugin_public_surface import (
    native_plugin_public_surface_audit,
)


EXPECTED_SOURCE_FILE_COUNT = 17
EXPECTED_DOC_FILE_COUNT = 5
EXPECTED_ROOT_REEXPORT_COUNT = 0
EXPECTED_NATIVE_NAMESPACE_REEXPORT_COUNT = 74
EXPECTED_NATIVE_PUBLIC_SURFACE_DEBT_GROUPS = 0
EXPECTED_NATIVE_NAMESPACE_SYMBOL_GROUPS = 6
EXPECTED_UNCLASSIFIED_NATIVE_SYMBOLS = 0
EXPECTED_ROOT_PUBLIC_NATIVE_REEXPORT_LOCATIONS = 0
EXPECTED_PUBLIC_NATIVE_REEXPORT_LOCATIONS = 1
EXPECTED_APP_NATIVE_PLUGIN_FILE_COUNT = 8
EXPECTED_NATIVE_LOADER_V1_V2_FILE_COUNT = 0
EXPECTED_PLUGIN_V1_V2_USAGE_FILES: tuple[str, ...] = ()
EXPECTED_EXPORT_BUILD_PLAN_V1_V2_USAGE_COUNT = 0
EXPECTED_NATIVE_LOADER_TEST_FILE_COUNT = 4
EXPECTED_NATIVE_TEST_NAMESPACE_IMPORT_FILE_COUNT = 3
EXPECTED_NATIVE_TEST_ROOT_IMPORT_LEAK_COUNT = 0
EXPECTED_LIFECYCLE_FALLBACK_TEST_COUNT = 4
EXPECTED_RUNTIME_06_STATUS = "in_progress"
EXPECTED_RUNTIME_06_LAST_REFINED = "2026-07-31"
EXPECTED_M4_GATE_STATUS = "classified-and-clear"
MIRROR_DOCS_GUARD = "runtime_06_plugin_surface_lifecycle_mirror_docs_match_structure_audit_counts"
NATIVE_TEST_NAMESPACE_GUARD = "runtime_06_native_loader_tests_use_isolated_plugin_native_namespace"
LIFECYCLE_FALLBACK_TEST_GUARD = "runtime_06_vm_lifecycle_fallback_failure_tests_are_folder_backed"

RUNTIME_06_SOURCE_FILES = (
    "zircon_runtime/src/plugin/mod.rs",
    "zircon_runtime/src/plugin/native.rs",
    "zircon_runtime/src/plugin/native_plugin_loader/mod.rs",
    "zircon_runtime/src/plugin/native_plugin_loader/abi_declarations.rs",
    "zircon_runtime/src/plugin/native_plugin_loader/native_plugin_abi.rs",
    "zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs",
    "zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/hot_reload.rs",
    "zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_failures.rs",
    "zircon_plugins/zr_vm_language/runtime/src/real_backend/instance.rs",
    "zircon_runtime/src/script/vm/tests.rs",
    "zircon_runtime/src/script/vm/tests/lifecycle_failures.rs",
    "zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/early/runtime_06.rs",
    "zircon_runtime/src/tests/runtime_absorption/plugin_surface_lifecycle.rs",
    "zircon_runtime/src/tests/runtime_absorption/plugin_surface_lifecycle/lifecycle_fallback.rs",
    "zircon_runtime/src/tests/runtime_absorption/plugin_surface_lifecycle/mirror_docs.rs",
    "zircon_runtime/src/tests/runtime_absorption/plugin_surface_lifecycle/native_loader_namespace.rs",
    "zircon_plugins/native_dynamic_fixture/native/src/lib.rs",
)

RUNTIME_06_DOC_FILES = (
    "docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md",
    "docs/plans/zircon_runtime/runtime/index.md",
    "docs/engine-architecture/native-plugin-boundary.md",
    "docs/engine-architecture/runtime-interface-convergence.md",
    "docs/engine-architecture/runtime-architecture-review-m0.md",
)

SOURCE_ANCHORS = (
    "pub mod native;",
    "pub use super::native_plugin_loader::{",
    'self.call_entry_lifecycle_export(&guard, "activate", &[])',
    'self.call_entry_lifecycle_export(&guard, "deactivate", &[])',
    'self.call_entry_lifecycle_export(&guard, "saveState", &[])',
    'self.call_entry_lifecycle_export(&guard, "restoreState", &[argument])',
    "mod lifecycle_failures;",
    "pub fn hot_reload_runtime_plugin(",
    "pub fn hot_reload_editor_plugin(",
    "NativePluginHotReloadState",
    "hot_reload_missing_symbol_after_reload_rolls_back_to_previous_instance",
    "hot_reload_state_restore_failure_rolls_back_and_reports",
    "ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL_V3",
    "ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_ABI_VERSION",
    "abi_unknown_version",
    '"descriptor.abi_version"',
    MIRROR_DOCS_GUARD,
    LIFECYCLE_FALLBACK_TEST_GUARD,
    NATIVE_TEST_NAMESPACE_GUARD,
)

DOC_ANCHORS = (
    "plugin_surface_lifecycle_boundary",
    "runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation",
    "native_plugin_public_surface",
    "root_reexport_count = 0",
    "native_namespace_reexport_count = 74",
    "classified-and-clear",
    "script::vm/vampire_project_session/plugin/native_plugin/app/plugins",
    "runtime real-backend",
    "fallback lifecycle failure tests 4/4",
    "app NativePlugin current call-site files: 8",
    "native loader V1/V2 implementation files 0/0",
    "`zircon_plugins` V1/V2 usage files 0/0",
    "unknown ABI rejection",
    "expected_source_file_count = 17",
    "hot reload failure injection",
    "expected_doc_file_count = 5",
    "mirror_docs_guard_present = true",
    "native loader test files 4/4",
    "native test namespace import files 3/3",
    "native test root import leaks 0/0",
    LIFECYCLE_FALLBACK_TEST_GUARD,
    NATIVE_TEST_NAMESPACE_GUARD,
    MIRROR_DOCS_GUARD,
)

CARGO_GATE_ANCHORS = (
    "cargo test -p zircon_runtime --lib script::vm --locked -- --nocapture",
    "cargo test -p zircon_runtime --lib vampire_project_session --features backend-zr-vm --locked -- --nocapture --test-threads=1",
    "cargo check -p zircon_runtime --lib --locked",
    "cargo test -p zircon_runtime --lib plugin --locked -- --nocapture",
    "cargo test -p zircon_app --locked",
    "cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked",
    "cargo test -p zircon_runtime --lib native_plugin --locked -- --nocapture",
    "cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked",
    "runtime_06_plugin_surface_lifecycle_gate_stays_visible_until_plugin_validation",
)

V1_V2_PATTERNS = (
    "NativePluginAbiV1",
    "NativePluginAbiV2",
    "DESCRIPTOR_SYMBOL_V1",
    "DESCRIPTOR_SYMBOL_V2",
    "ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V1",
    "ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2",
)

NATIVE_LOADER_TEST_PATTERNS = (
    "NativePluginAbi",
    "NativePluginEntryReport",
    "NativePluginBehavior",
    "NativePluginLoader",
    "ZIRCON_NATIVE_PLUGIN_STATUS",
)

NATIVE_TEST_NAMESPACE_PATTERNS = (
    "crate::plugin::native::",
    "zircon_runtime::plugin::native::",
)

LIFECYCLE_FALLBACK_TESTS = (
    "vm_lifecycle_fallback_activate_bad_entry_module_surfaces_vm_error",
    "vm_lifecycle_fallback_missing_optional_export_returns_none_not_error",
    "vm_lifecycle_fallback_deactivate_is_idempotent_after_unload",
    "vm_lifecycle_fallback_empty_arguments_do_not_require_real_backend",
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


def _frontmatter_field(source: str, field: str) -> str | None:
    match = re.search(rf"(?m)^{re.escape(field)}:\s*(?P<value>[^\r\n]+)\s*$", source)
    if not match:
        return None
    return match.group("value").strip()


def _missing_snippets(sources: tuple[str, ...], snippets: tuple[str, ...]) -> list[str]:
    return [
        snippet
        for snippet in snippets
        if not any(snippet in source for source in sources)
    ]


def _rs_files_under(path: Path) -> list[Path]:
    if not path.exists():
        return []
    return sorted(path.rglob("*.rs"))


def _files_containing(root: Path, search_root: Path, patterns: tuple[str, ...]) -> list[str]:
    files: list[str] = []
    for path in _rs_files_under(search_root):
        source = _read_text(path)
        if any(pattern in source for pattern in patterns):
            files.append(_relative(root, path))
    return sorted(files)


def _location_count(search_root: Path, patterns: tuple[str, ...]) -> int:
    count = 0
    for path in _rs_files_under(search_root):
        source = _read_text(path)
        for pattern in patterns:
            count += source.count(pattern)
    return count


def _has_native_root_import_leak(source: str) -> bool:
    direct_patterns = (
        "crate::plugin::NativePlugin",
        "crate::plugin::ZIRCON_NATIVE_PLUGIN",
        "zircon_runtime::plugin::NativePlugin",
        "zircon_runtime::plugin::ZIRCON_NATIVE_PLUGIN",
    )
    if any(pattern in source for pattern in direct_patterns):
        return True

    for marker in ("use crate::plugin::", "use zircon_runtime::plugin::"):
        search_start = 0
        while True:
            statement_start = source.find(marker, search_start)
            if statement_start < 0:
                break
            statement_tail = source[statement_start:]
            if statement_tail.startswith("use crate::plugin::native::") or statement_tail.startswith(
                "use zircon_runtime::plugin::native::"
            ):
                search_start = statement_start + len(marker)
                continue
            statement_end = statement_tail.find(";")
            if statement_end < 0:
                statement_end = len(statement_tail)
            statement = statement_tail[:statement_end]
            if "NativePlugin" in statement or "ZIRCON_NATIVE_PLUGIN" in statement:
                return True
            search_start = statement_start + statement_end + 1

    return False


def _native_root_import_leak_files(root: Path, search_root: Path) -> list[str]:
    files: list[str] = []
    for path in _rs_files_under(search_root):
        if _has_native_root_import_leak(_read_text(path)):
            files.append(_relative(root, path))
    return sorted(files)


def plugin_surface_lifecycle_boundary_audit(root: Path) -> dict[str, object]:
    source_files, missing_source_files = _file_entries(root, RUNTIME_06_SOURCE_FILES)
    doc_files, missing_doc_files = _file_entries(root, RUNTIME_06_DOC_FILES)
    source_texts = tuple(
        _read_text(root / file_name)
        for file_name in RUNTIME_06_SOURCE_FILES
        if (root / file_name).exists()
    )
    doc_texts = tuple(
        _read_text(root / file_name)
        for file_name in RUNTIME_06_DOC_FILES
        if (root / file_name).exists()
    )
    plan_source = (
        _read_text(root / RUNTIME_06_DOC_FILES[0])
        if (root / RUNTIME_06_DOC_FILES[0]).exists()
        else ""
    )
    cargo_gate_source = (
        _read_text(
            root
            / "zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/early/runtime_06.rs"
        )
        if (
            root
            / "zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/early/runtime_06.rs"
        ).exists()
        else ""
    )
    native_surface = native_plugin_public_surface_audit(root)
    app_native_plugin_files = _files_containing(
        root,
        root / "zircon_app/src",
        ("NativePlugin",),
    )
    native_loader_v1_v2_files = _files_containing(
        root,
        root / "zircon_runtime/src/plugin/native_plugin_loader",
        V1_V2_PATTERNS,
    )
    plugin_v1_v2_usage_files = _files_containing(
        root,
        root / "zircon_plugins",
        V1_V2_PATTERNS,
    )
    export_build_plan_v1_v2_usage_count = _location_count(
        root / "zircon_runtime/src/plugin/export_build_plan",
        V1_V2_PATTERNS,
    )
    native_loader_test_files = _files_containing(
        root,
        root / "zircon_runtime/src/tests/plugin_extensions",
        NATIVE_LOADER_TEST_PATTERNS,
    )
    native_test_namespace_import_files = _files_containing(
        root,
        root / "zircon_runtime/src/tests/plugin_extensions",
        NATIVE_TEST_NAMESPACE_PATTERNS,
    )
    native_test_root_import_leak_files = _native_root_import_leak_files(
        root,
        root / "zircon_runtime/src/tests/plugin_extensions",
    )
    lifecycle_fallback_test_source = (
        _read_text(root / "zircon_runtime/src/script/vm/tests/lifecycle_failures.rs")
        if (root / "zircon_runtime/src/script/vm/tests/lifecycle_failures.rs").exists()
        else ""
    )
    missing_lifecycle_fallback_tests = [
        test_name
        for test_name in LIFECYCLE_FALLBACK_TESTS
        if test_name not in lifecycle_fallback_test_source
    ]

    missing_source_anchors = _missing_snippets(source_texts, SOURCE_ANCHORS)
    missing_doc_anchors = _missing_snippets(doc_texts, DOC_ANCHORS)
    missing_cargo_gate_anchors = _missing_snippets(
        (plan_source, cargo_gate_source), CARGO_GATE_ANCHORS
    )
    runtime_06_status = _frontmatter_field(plan_source, "status")
    runtime_06_last_refined = _frontmatter_field(plan_source, "last_refined")
    mirror_docs_guard_present = any(MIRROR_DOCS_GUARD in source for source in source_texts)

    risks: list[str] = []
    if len(source_files) != EXPECTED_SOURCE_FILE_COUNT:
        risks.append("Runtime 06 source inventory count changed without audit sync.")
    if missing_source_files:
        risks.append("Runtime 06 source files are missing.")
    if missing_doc_files:
        risks.append("Runtime 06 mirror docs are missing.")
    if runtime_06_status != EXPECTED_RUNTIME_06_STATUS:
        risks.append("Runtime 06 frontmatter status is not in_progress.")
    if runtime_06_last_refined != EXPECTED_RUNTIME_06_LAST_REFINED:
        risks.append("Runtime 06 last_refined does not cover the latest status row.")
    if int(native_surface["root_reexport_count"]) != EXPECTED_ROOT_REEXPORT_COUNT:
        risks.append("Native plugin root re-export count changed without Runtime 06 audit sync.")
    if int(native_surface["native_namespace_reexport_count"]) != EXPECTED_NATIVE_NAMESPACE_REEXPORT_COUNT:
        risks.append("Native plugin namespace re-export count changed without Runtime 06 audit sync.")
    if native_surface["m4_gate_status"] != EXPECTED_M4_GATE_STATUS:
        risks.append("Native plugin public-surface M4 gate status changed without Runtime 06 audit sync.")
    if (
        int(native_surface["native_plugin_public_surface_migration_debt_count"])
        != EXPECTED_NATIVE_PUBLIC_SURFACE_DEBT_GROUPS
    ):
        risks.append("Native plugin public-surface debt group count changed without Runtime 06 audit sync.")
    if int(native_surface["unclassified_root_reexport_symbol_count"]) != EXPECTED_UNCLASSIFIED_NATIVE_SYMBOLS:
        risks.append("Native plugin public-surface has unclassified root re-export symbols.")
    if int(native_surface["unclassified_native_namespace_symbol_count"]) != EXPECTED_UNCLASSIFIED_NATIVE_SYMBOLS:
        risks.append("Native plugin namespace has unclassified symbols.")
    if int(native_surface["native_namespace_symbol_group_count"]) != EXPECTED_NATIVE_NAMESPACE_SYMBOL_GROUPS:
        risks.append("Native plugin namespace classification group count changed without Runtime 06 audit sync.")
    if int(native_surface["root_public_reexport_location_count"]) != EXPECTED_ROOT_PUBLIC_NATIVE_REEXPORT_LOCATIONS:
        risks.append("Native plugin root public re-export location count changed without Runtime 06 audit sync.")
    if int(native_surface["public_reexport_location_count"]) != EXPECTED_PUBLIC_NATIVE_REEXPORT_LOCATIONS:
        risks.append("Native plugin namespace public re-export location count changed without Runtime 06 audit sync.")
    if len(app_native_plugin_files) != EXPECTED_APP_NATIVE_PLUGIN_FILE_COUNT:
        risks.append("zircon_app NativePlugin call-site file count changed without Runtime 06 audit sync.")
    if len(native_loader_v1_v2_files) != EXPECTED_NATIVE_LOADER_V1_V2_FILE_COUNT:
        risks.append("Native loader V1/V2 implementation file count changed without Runtime 06 audit sync.")
    if tuple(plugin_v1_v2_usage_files) != EXPECTED_PLUGIN_V1_V2_USAGE_FILES:
        risks.append("zircon_plugins V1/V2 usage is no longer limited to the native dynamic fixture.")
    if export_build_plan_v1_v2_usage_count != EXPECTED_EXPORT_BUILD_PLAN_V1_V2_USAGE_COUNT:
        risks.append("export_build_plan references retired native ABI V1/V2 symbols.")
    if len(native_loader_test_files) != EXPECTED_NATIVE_LOADER_TEST_FILE_COUNT:
        risks.append("Native loader test file count changed without Runtime 06 M2.2 audit sync.")
    if len(native_test_namespace_import_files) != EXPECTED_NATIVE_TEST_NAMESPACE_IMPORT_FILE_COUNT:
        risks.append("Native test namespace import file count changed without Runtime 06 M2.2 audit sync.")
    if len(native_test_root_import_leak_files) != EXPECTED_NATIVE_TEST_ROOT_IMPORT_LEAK_COUNT:
        risks.append("Native loader tests import native symbols from the plugin root.")
    if len(LIFECYCLE_FALLBACK_TESTS) != EXPECTED_LIFECYCLE_FALLBACK_TEST_COUNT:
        risks.append("Runtime 06 expected fallback lifecycle test inventory changed without audit sync.")
    if missing_lifecycle_fallback_tests:
        risks.append("Runtime 06 M1.2 fallback lifecycle tests are missing.")
    if missing_source_anchors:
        risks.append("Runtime 06 source anchors are missing.")
    if missing_doc_anchors:
        risks.append("Runtime 06 plan/docs are missing required mirror anchors.")
    if missing_cargo_gate_anchors:
        risks.append("Runtime 06 validation command anchors are missing from the subplan.")
    if not mirror_docs_guard_present:
        risks.append("Runtime 06 mirror-doc aggregate guard is missing.")

    return {
        "source_files": source_files,
        "expected_source_file_count": EXPECTED_SOURCE_FILE_COUNT,
        "missing_source_files": missing_source_files,
        "doc_files": doc_files,
        "expected_doc_file_count": EXPECTED_DOC_FILE_COUNT,
        "missing_doc_files": missing_doc_files,
        "runtime_06_status": runtime_06_status,
        "expected_runtime_06_status": EXPECTED_RUNTIME_06_STATUS,
        "runtime_06_last_refined": runtime_06_last_refined,
        "expected_runtime_06_last_refined": EXPECTED_RUNTIME_06_LAST_REFINED,
        "native_root_reexport_count": native_surface["root_reexport_count"],
        "expected_native_root_reexport_count": EXPECTED_ROOT_REEXPORT_COUNT,
        "native_namespace_reexport_count": native_surface["native_namespace_reexport_count"],
        "expected_native_namespace_reexport_count": EXPECTED_NATIVE_NAMESPACE_REEXPORT_COUNT,
        "native_public_surface_m4_gate_status": native_surface["m4_gate_status"],
        "expected_native_public_surface_m4_gate_status": EXPECTED_M4_GATE_STATUS,
        "native_public_surface_migration_debt_count": native_surface[
            "native_plugin_public_surface_migration_debt_count"
        ],
        "expected_native_public_surface_migration_debt_count": EXPECTED_NATIVE_PUBLIC_SURFACE_DEBT_GROUPS,
        "native_namespace_symbol_group_count": native_surface["native_namespace_symbol_group_count"],
        "expected_native_namespace_symbol_group_count": EXPECTED_NATIVE_NAMESPACE_SYMBOL_GROUPS,
        "unclassified_native_root_reexport_symbol_count": native_surface[
            "unclassified_root_reexport_symbol_count"
        ],
        "expected_unclassified_native_root_reexport_symbol_count": EXPECTED_UNCLASSIFIED_NATIVE_SYMBOLS,
        "unclassified_native_namespace_symbol_count": native_surface[
            "unclassified_native_namespace_symbol_count"
        ],
        "expected_unclassified_native_namespace_symbol_count": EXPECTED_UNCLASSIFIED_NATIVE_SYMBOLS,
        "root_public_native_reexport_location_count": native_surface[
            "root_public_reexport_location_count"
        ],
        "expected_root_public_native_reexport_location_count": EXPECTED_ROOT_PUBLIC_NATIVE_REEXPORT_LOCATIONS,
        "public_native_reexport_location_count": native_surface["public_reexport_location_count"],
        "expected_public_native_reexport_location_count": EXPECTED_PUBLIC_NATIVE_REEXPORT_LOCATIONS,
        "app_native_plugin_files": app_native_plugin_files,
        "app_native_plugin_file_count": len(app_native_plugin_files),
        "expected_app_native_plugin_file_count": EXPECTED_APP_NATIVE_PLUGIN_FILE_COUNT,
        "native_loader_v1_v2_files": native_loader_v1_v2_files,
        "native_loader_v1_v2_file_count": len(native_loader_v1_v2_files),
        "expected_native_loader_v1_v2_file_count": EXPECTED_NATIVE_LOADER_V1_V2_FILE_COUNT,
        "plugin_v1_v2_usage_files": plugin_v1_v2_usage_files,
        "expected_plugin_v1_v2_usage_files": list(EXPECTED_PLUGIN_V1_V2_USAGE_FILES),
        "export_build_plan_v1_v2_usage_count": export_build_plan_v1_v2_usage_count,
        "expected_export_build_plan_v1_v2_usage_count": EXPECTED_EXPORT_BUILD_PLAN_V1_V2_USAGE_COUNT,
        "native_loader_test_files": native_loader_test_files,
        "native_loader_test_file_count": len(native_loader_test_files),
        "expected_native_loader_test_file_count": EXPECTED_NATIVE_LOADER_TEST_FILE_COUNT,
        "native_test_namespace_import_files": native_test_namespace_import_files,
        "native_test_namespace_import_file_count": len(native_test_namespace_import_files),
        "expected_native_test_namespace_import_file_count": EXPECTED_NATIVE_TEST_NAMESPACE_IMPORT_FILE_COUNT,
        "native_test_root_import_leak_files": native_test_root_import_leak_files,
        "native_test_root_import_leak_count": len(native_test_root_import_leak_files),
        "expected_native_test_root_import_leak_count": EXPECTED_NATIVE_TEST_ROOT_IMPORT_LEAK_COUNT,
        "lifecycle_fallback_test_count": len(LIFECYCLE_FALLBACK_TESTS)
        - len(missing_lifecycle_fallback_tests),
        "expected_lifecycle_fallback_test_count": EXPECTED_LIFECYCLE_FALLBACK_TEST_COUNT,
        "missing_lifecycle_fallback_tests": missing_lifecycle_fallback_tests,
        "source_anchor_count": len(SOURCE_ANCHORS),
        "missing_source_anchors": missing_source_anchors,
        "doc_anchor_count": len(DOC_ANCHORS),
        "missing_doc_anchors": missing_doc_anchors,
        "cargo_gate_anchor_count": len(CARGO_GATE_ANCHORS),
        "missing_cargo_gate_anchors": missing_cargo_gate_anchors,
        "mirror_docs_guard": MIRROR_DOCS_GUARD,
        "mirror_docs_guard_present": mirror_docs_guard_present,
        "risks": risks,
    }
