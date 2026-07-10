from __future__ import annotations

import re
from pathlib import Path

from runtime_structure_audits.dynamic_runtime_api_abi_inventory import (
    EXPECTED_FUNCTION_TABLES,
    EXPECTED_SOURCE_FILE_COUNT,
    RUNTIME_SESSION_OPERATIONS,
    SOURCE_FILES,
)
from runtime_structure_audits.dynamic_runtime_api_diagnostics_inventory import (
    RUNTIME_DIAGNOSTICS_ANCHORS,
    SCENE_ASSET_RELOAD_DIAGNOSTIC_PATH_ANCHORS,
)
from runtime_structure_audits.dynamic_runtime_api_failure_inventory import (
    FFI_PANIC_ANCHORS,
    LOADER_FAILURE_ANCHORS,
)
from runtime_structure_audits.dynamic_runtime_api_host_request_inventory import (
    HOST_REQUEST_PAYLOAD_ANCHORS,
)
from runtime_structure_audits.dynamic_runtime_api_session_lifecycle_inventory import (
    HEADLESS_LIFECYCLE_ANCHORS,
)
from runtime_structure_audits.dynamic_runtime_api_ui_contract_inventory import (
    UI_CONTRACT_SINGLE_SOURCE_ANCHORS,
    UI_PENDING_GATE_ANCHORS,
    UI_V2_CONTRACT_SYNC_ANCHORS,
)
from runtime_structure_audits.dynamic_runtime_api_validation_inventory import (
    BEHAVIOR_TEST_ANCHORS,
    CARGO_GATE_ANCHORS,
    DOC_ANCHORS,
    MIRROR_DOCS_GUARD,
)


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _relative(root: Path, path: Path) -> str:
    return path.relative_to(root).as_posix()


def _existing_files(root: Path, paths: tuple[str, ...]) -> list[str]:
    return [path for path in paths if (root / path).exists()]


def _missing_files(root: Path, paths: tuple[str, ...]) -> list[str]:
    return [path for path in paths if not (root / path).exists()]


def _struct_body(source: str, struct_name: str) -> str | None:
    struct_pattern = re.compile(rf"pub\s+struct\s+{re.escape(struct_name)}\s*\{{")
    match = struct_pattern.search(source)
    if not match:
        return None
    body_start = match.end()
    body_end = source.find("\n}", body_start)
    if body_end == -1:
        return None
    return source[body_start:body_end]


def _struct_has_local_repr_c(source: str, struct_name: str) -> bool:
    struct_pattern = re.compile(rf"pub\s+struct\s+{re.escape(struct_name)}\s*\{{")
    match = struct_pattern.search(source)
    if not match:
        return False
    prefix = source[: match.start()]
    repr_index = prefix.rfind("#[repr(C)]")
    previous_struct_index = prefix.rfind("pub struct ")
    return repr_index != -1 and (
        previous_struct_index == -1 or previous_struct_index < repr_index
    )


def _field_names(source: str, struct_name: str) -> list[str]:
    body = _struct_body(source, struct_name)
    if body is None:
        return []
    fields: list[str] = []
    for line in body.splitlines():
        line = line.strip()
        if not line.startswith("pub "):
            continue
        field = line[4:].split(":", 1)[0].strip()
        if field:
            fields.append(field)
    return fields


def _missing_anchors(root: Path, anchors: tuple[tuple[str, str], ...]) -> list[dict[str, str]]:
    missing: list[dict[str, str]] = []
    for relative_path, snippet in anchors:
        path = root / relative_path
        if not path.exists() or snippet not in _read_text(path):
            missing.append({"path": relative_path, "snippet": snippet})
    return missing


def _public_ui_type_duplicates(root: Path) -> list[dict[str, str]]:
    def collect_types(relative_root: str) -> dict[tuple[str, str], list[str]]:
        types: dict[tuple[str, str], list[str]] = {}
        source_root = root / relative_root
        for path in source_root.rglob("*.rs"):
            source = _read_text(path)
            for match in re.finditer(
                r"(?m)^\s*pub\s+(struct|enum)\s+([A-Za-z_][A-Za-z0-9_]*)",
                source,
            ):
                key = (match.group(1), match.group(2))
                types.setdefault(key, []).append(_relative(root, path))
        return types

    runtime_types = collect_types("zircon_runtime/src/ui")
    interface_types = collect_types("zircon_runtime_interface/src/ui")
    duplicates: list[dict[str, str]] = []

    for (kind, name), runtime_paths in runtime_types.items():
        for interface_path in interface_types.get((kind, name), []):
            for runtime_path in runtime_paths:
                duplicates.append(
                    {
                        "kind": kind,
                        "name": name,
                        "runtime_path": runtime_path,
                        "interface_path": interface_path,
                    }
                )

    return duplicates


def _function_table_report(root: Path) -> dict[str, object]:
    tables: list[dict[str, object]] = []
    missing_tables: list[dict[str, object]] = []
    missing_repr_c: list[dict[str, object]] = []
    field_count_mismatches: list[dict[str, object]] = []

    for relative_path, table_name, expected_fields in EXPECTED_FUNCTION_TABLES:
        path = root / relative_path
        source = _read_text(path) if path.exists() else ""
        fields = _field_names(source, table_name)
        table = {
            "path": relative_path,
            "name": table_name,
            "field_count": len(fields),
            "expected_field_count": expected_fields,
            "fields": fields,
            "repr_c": _struct_has_local_repr_c(source, table_name),
        }
        if not fields:
            missing_tables.append(table)
        elif len(fields) != expected_fields:
            field_count_mismatches.append(table)
        if not table["repr_c"]:
            missing_repr_c.append(table)
        tables.append(table)

    return {
        "tables": tables,
        "expected_table_count": len(EXPECTED_FUNCTION_TABLES),
        "missing_tables": missing_tables,
        "missing_repr_c": missing_repr_c,
        "field_count_mismatches": field_count_mismatches,
    }

def _ffi_wrapper_report(root: Path) -> dict[str, object]:
    exports_source = _read_text(root / "zircon_runtime/src/dynamic_api/exports.rs")
    session_source = _read_text(root / "zircon_runtime/src/dynamic_api/session.rs")
    missing_wrappers: list[dict[str, str]] = []
    direct_session_table_entry_bypasses: list[str] = []
    missing_session_owners: list[str] = []

    for operation in RUNTIME_SESSION_OPERATIONS:
        wrapper = f"{operation}_ffi"
        required = [
            f"Some({wrapper})",
            f"fn {wrapper}(",
            f"catch_ffi_panic(|| unsafe {{ {operation}(",
        ]
        for snippet in required:
            if snippet not in exports_source:
                missing_wrappers.append({"operation": operation, "snippet": snippet})
        if f"Some({operation})," in exports_source:
            direct_session_table_entry_bypasses.append(operation)
        if f"pub(super) unsafe fn {operation}(" not in session_source:
            missing_session_owners.append(operation)

    return {
        "operation_count": len(RUNTIME_SESSION_OPERATIONS),
        "missing_wrappers": missing_wrappers,
        "direct_session_table_entry_bypasses": direct_session_table_entry_bypasses,
        "missing_session_owners": missing_session_owners,
        "session_owner_extern_c_present": 'pub(super) unsafe extern "C" fn' in session_source,
    }


def dynamic_runtime_api_boundary_audit(root: Path) -> dict[str, object]:
    source_files = _existing_files(root, SOURCE_FILES)
    function_tables = _function_table_report(root)
    ffi_wrappers = _ffi_wrapper_report(root)

    missing_headless_lifecycle_anchors = _missing_anchors(root, HEADLESS_LIFECYCLE_ANCHORS)
    missing_ffi_panic_anchors = _missing_anchors(root, FFI_PANIC_ANCHORS)
    missing_loader_failure_anchors = _missing_anchors(root, LOADER_FAILURE_ANCHORS)
    missing_behavior_test_anchors = _missing_anchors(root, BEHAVIOR_TEST_ANCHORS)
    missing_runtime_diagnostics_anchors = _missing_anchors(root, RUNTIME_DIAGNOSTICS_ANCHORS)
    missing_scene_asset_reload_diagnostic_path_anchors = _missing_anchors(
        root, SCENE_ASSET_RELOAD_DIAGNOSTIC_PATH_ANCHORS
    )
    missing_ui_pending_gate_anchors = _missing_anchors(root, UI_PENDING_GATE_ANCHORS)
    missing_ui_contract_single_source_anchors = _missing_anchors(
        root, UI_CONTRACT_SINGLE_SOURCE_ANCHORS
    )
    missing_ui_v2_contract_sync_anchors = _missing_anchors(
        root, UI_V2_CONTRACT_SYNC_ANCHORS
    )
    ui_contract_duplicate_public_types = _public_ui_type_duplicates(root)
    missing_host_request_payload_anchors = _missing_anchors(
        root, HOST_REQUEST_PAYLOAD_ANCHORS
    )
    missing_cargo_gate_anchors = _missing_anchors(root, CARGO_GATE_ANCHORS)
    missing_doc_anchors = _missing_anchors(root, DOC_ANCHORS)
    mirror_docs_guard_present = (
        MIRROR_DOCS_GUARD
        in _read_text(
            root
            / "zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/mirror_docs.rs"
        )
    )

    risks: list[str] = []
    if _missing_files(root, SOURCE_FILES):
        risks.append("Runtime 10 dynamic runtime API boundary source files are missing.")
    if len(source_files) != EXPECTED_SOURCE_FILE_COUNT:
        risks.append("Runtime 10 dynamic runtime API source file count changed.")
    if function_tables["missing_tables"]:
        risks.append("Runtime 10 function table inventory is missing expected tables.")
    if function_tables["missing_repr_c"]:
        risks.append("Runtime 10 function table inventory has tables without local #[repr(C)].")
    if function_tables["field_count_mismatches"]:
        risks.append("Runtime 10 function table field counts drifted from the documented ABI matrix.")
    if ffi_wrappers["missing_wrappers"]:
        risks.append("Runtime 10 FFI panic wrappers are missing expected function-table anchors.")
    if ffi_wrappers["direct_session_table_entry_bypasses"]:
        risks.append("Runtime 10 function table bypasses exports.rs panic wrappers.")
    if ffi_wrappers["missing_session_owners"] or ffi_wrappers["session_owner_extern_c_present"]:
        risks.append("Runtime 10 session owner functions drifted from private Rust ABI.")
    if missing_headless_lifecycle_anchors:
        risks.append("Runtime 10 headless/minimal lifecycle anchors are incomplete.")
    if missing_ffi_panic_anchors:
        risks.append("Runtime 10 FFI panic-boundary anchors are incomplete.")
    if missing_loader_failure_anchors:
        risks.append("Runtime 10 loader failure-path anchors are incomplete.")
    if missing_behavior_test_anchors:
        risks.append("Runtime 10 behavior test anchors are incomplete.")
    if missing_runtime_diagnostics_anchors:
        risks.append("Runtime 10 runtime diagnostics profile-control anchors are incomplete.")
    if missing_scene_asset_reload_diagnostic_path_anchors:
        risks.append("Runtime 10 scene-asset reload diagnostic path anchors are incomplete.")
    if missing_ui_pending_gate_anchors:
        risks.append("Runtime 10 UI contract pending gate anchors are incomplete.")
    if missing_ui_contract_single_source_anchors:
        risks.append("Runtime 10 UI contract single-source anchors are incomplete.")
    if missing_ui_v2_contract_sync_anchors:
        risks.append("Runtime 10 UI v2 contract synchronization anchors are incomplete.")
    if ui_contract_duplicate_public_types:
        risks.append("Runtime 10 UI contract public types still have duplicate runtime/interface definitions.")
    if missing_host_request_payload_anchors:
        risks.append("Runtime 10 host-request payload ABI/app handoff anchors are incomplete.")
    if missing_cargo_gate_anchors:
        risks.append("Runtime 10 pending Cargo gate anchors are incomplete.")
    if missing_doc_anchors:
        risks.append("Runtime 10 plan or mirror docs are missing required status anchors.")
    if not mirror_docs_guard_present:
        risks.append("Runtime 10 mirror-doc aggregate guard is missing.")

    return {
        "source_files": source_files,
        "expected_source_file_count": EXPECTED_SOURCE_FILE_COUNT,
        "missing_source_files": _missing_files(root, SOURCE_FILES),
        "function_table_structs": function_tables["tables"],
        "expected_function_table_count": function_tables["expected_table_count"],
        "missing_function_tables": function_tables["missing_tables"],
        "missing_repr_c_tables": function_tables["missing_repr_c"],
        "field_count_mismatches": function_tables["field_count_mismatches"],
        "runtime_session_operation_count": ffi_wrappers["operation_count"],
        "missing_ffi_wrappers": ffi_wrappers["missing_wrappers"],
        "direct_session_table_entry_bypasses": ffi_wrappers[
            "direct_session_table_entry_bypasses"
        ],
        "missing_session_owners": ffi_wrappers["missing_session_owners"],
        "session_owner_extern_c_present": ffi_wrappers["session_owner_extern_c_present"],
        "headless_lifecycle_anchor_count": len(HEADLESS_LIFECYCLE_ANCHORS),
        "missing_headless_lifecycle_anchors": missing_headless_lifecycle_anchors,
        "ffi_panic_anchor_count": len(FFI_PANIC_ANCHORS),
        "missing_ffi_panic_anchors": missing_ffi_panic_anchors,
        "loader_failure_anchor_count": len(LOADER_FAILURE_ANCHORS),
        "missing_loader_failure_anchors": missing_loader_failure_anchors,
        "behavior_test_anchor_count": len(BEHAVIOR_TEST_ANCHORS),
        "missing_behavior_test_anchors": missing_behavior_test_anchors,
        "runtime_diagnostics_anchor_count": len(RUNTIME_DIAGNOSTICS_ANCHORS),
        "missing_runtime_diagnostics_anchors": missing_runtime_diagnostics_anchors,
        "scene_asset_reload_diagnostic_path_anchor_count": len(
            SCENE_ASSET_RELOAD_DIAGNOSTIC_PATH_ANCHORS
        ),
        "missing_scene_asset_reload_diagnostic_path_anchors": missing_scene_asset_reload_diagnostic_path_anchors,
        "ui_pending_gate_anchor_count": len(UI_PENDING_GATE_ANCHORS),
        "missing_ui_pending_gate_anchors": missing_ui_pending_gate_anchors,
        "ui_contract_single_source_anchor_count": len(UI_CONTRACT_SINGLE_SOURCE_ANCHORS),
        "missing_ui_contract_single_source_anchors": missing_ui_contract_single_source_anchors,
        "ui_contract_duplicate_public_types": ui_contract_duplicate_public_types,
        "ui_v2_contract_sync_anchor_count": len(UI_V2_CONTRACT_SYNC_ANCHORS),
        "missing_ui_v2_contract_sync_anchors": missing_ui_v2_contract_sync_anchors,
        "host_request_payload_anchor_count": len(HOST_REQUEST_PAYLOAD_ANCHORS),
        "missing_host_request_payload_anchors": missing_host_request_payload_anchors,
        "cargo_gate_anchor_count": len(CARGO_GATE_ANCHORS),
        "missing_cargo_gate_anchors": missing_cargo_gate_anchors,
        "doc_anchor_count": len(DOC_ANCHORS),
        "missing_doc_anchors": missing_doc_anchors,
        "mirror_docs_guard": MIRROR_DOCS_GUARD,
        "mirror_docs_guard_present": mirror_docs_guard_present,
        "risks": risks,
    }
