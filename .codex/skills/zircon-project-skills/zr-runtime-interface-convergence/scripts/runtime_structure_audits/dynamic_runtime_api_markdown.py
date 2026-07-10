from __future__ import annotations


def render_dynamic_runtime_api_boundary_markdown(
    boundary: dict[str, object],
) -> list[str]:
    table_count = (
        boundary["expected_function_table_count"]
        - len(boundary["missing_function_tables"])
    )
    wrapper_count = (
        boundary["runtime_session_operation_count"]
        - len({entry["operation"] for entry in boundary["missing_ffi_wrappers"]})
    )
    lines = [
        "## Runtime 10 Dynamic Runtime API Boundary",
        "- audited Runtime 10 dynamic API source files "
        f"({len(boundary['source_files'])}/{boundary['expected_source_file_count']}): "
        f"{len(boundary['source_files'])} files",
        "- function table structs: "
        f"{table_count}/{boundary['expected_function_table_count']}",
        "- function table field-count mismatches: "
        f"{len(boundary['field_count_mismatches'])}",
        "- function tables missing local #[repr(C)]: "
        f"{len(boundary['missing_repr_c_tables'])}",
        "- runtime session FFI wrappers: "
        f"{wrapper_count}/{boundary['runtime_session_operation_count']}",
        "- direct session table-entry bypasses: "
        f"{len(boundary['direct_session_table_entry_bypasses'])}",
        "- session owner extern C declarations present: "
        f"{boundary['session_owner_extern_c_present']}",
        "- headless/minimal lifecycle anchors: "
        f"{boundary['headless_lifecycle_anchor_count'] - len(boundary['missing_headless_lifecycle_anchors'])}/{boundary['headless_lifecycle_anchor_count']}",
        "- FFI panic-boundary anchors: "
        f"{boundary['ffi_panic_anchor_count'] - len(boundary['missing_ffi_panic_anchors'])}/{boundary['ffi_panic_anchor_count']}",
        "- loader failure-path anchors: "
        f"{boundary['loader_failure_anchor_count'] - len(boundary['missing_loader_failure_anchors'])}/{boundary['loader_failure_anchor_count']}",
        "- behavior test anchors: "
        f"{boundary['behavior_test_anchor_count'] - len(boundary['missing_behavior_test_anchors'])}/{boundary['behavior_test_anchor_count']}",
        "- runtime diagnostics profile-control anchors: "
        f"{boundary['runtime_diagnostics_anchor_count'] - len(boundary['missing_runtime_diagnostics_anchors'])}/{boundary['runtime_diagnostics_anchor_count']}",
        "- scene-asset reload diagnostic path anchors: "
        f"{boundary['scene_asset_reload_diagnostic_path_anchor_count'] - len(boundary['missing_scene_asset_reload_diagnostic_path_anchors'])}/{boundary['scene_asset_reload_diagnostic_path_anchor_count']}",
        "- UI contract pending-gate anchors: "
        f"{boundary['ui_pending_gate_anchor_count'] - len(boundary['missing_ui_pending_gate_anchors'])}/{boundary['ui_pending_gate_anchor_count']}",
        "- UI contract single-source anchors: "
        f"{boundary['ui_contract_single_source_anchor_count'] - len(boundary['missing_ui_contract_single_source_anchors'])}/{boundary['ui_contract_single_source_anchor_count']}",
        "- UI contract duplicate public types: "
        f"{len(boundary['ui_contract_duplicate_public_types'])}",
        "- UI v2 contract synchronization anchors: "
        f"{boundary['ui_v2_contract_sync_anchor_count'] - len(boundary['missing_ui_v2_contract_sync_anchors'])}/{boundary['ui_v2_contract_sync_anchor_count']}",
        "- host-request payload anchors: "
        f"{boundary['host_request_payload_anchor_count'] - len(boundary['missing_host_request_payload_anchors'])}/{boundary['host_request_payload_anchor_count']}",
        "- pending Cargo gate anchors: "
        f"{boundary['cargo_gate_anchor_count'] - len(boundary['missing_cargo_gate_anchors'])}/{boundary['cargo_gate_anchor_count']}",
        "- doc anchors: "
        f"{boundary['doc_anchor_count'] - len(boundary['missing_doc_anchors'])}/{boundary['doc_anchor_count']}",
        "- mirror-doc aggregate guard: "
        f"{'present' if boundary['mirror_docs_guard_present'] else 'missing'}",
    ]

    for risk in boundary["risks"]:
        lines.append(f"- risk: {risk}")

    return lines
