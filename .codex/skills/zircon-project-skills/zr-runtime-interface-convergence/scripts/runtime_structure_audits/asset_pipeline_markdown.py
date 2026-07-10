from __future__ import annotations


def render_asset_pipeline_boundary_markdown(boundary: dict[str, object]) -> list[str]:
    source_files = boundary["source_files"]
    guard_files = boundary["guard_files"]
    lines = [
        "## Runtime 04 Asset Pipeline Boundary",
        "- audited asset pipeline source files "
        f"({len(source_files)}/{boundary['expected_source_file_count']}): "
        f"{len(source_files)} files",
        "- audited Runtime 04 guard/test files "
        f"({len(guard_files)}/{boundary['expected_guard_file_count']}): "
        f"{len(guard_files)} files",
        "- worker diagnostics: "
        f"{boundary['worker_diagnostic_count']}/"
        f"{boundary['expected_worker_diagnostic_count']}",
        "- artifact-store scene roundtrip guards: "
        f"{boundary['artifact_store_roundtrip_count']}/"
        f"{boundary['expected_artifact_store_roundtrip_count']}",
        "- watcher acceptance evidence references: "
        f"{boundary['watcher_acceptance_reference_count']} "
        f"(expected watcher tests: {boundary['expected_watcher_acceptance_count']})",
        "- Runtime 04 guard anchors: "
        f"{boundary['test_anchor_count'] - len(boundary['missing_test_anchors'])}/"
        f"{boundary['test_anchor_count']}",
        "- Runtime 04 behavior-test anchors: "
        f"{boundary['behavior_test_anchor_count'] - len(boundary['missing_behavior_test_anchors'])}/"
        f"{boundary['behavior_test_anchor_count']}",
        "- retired worker-count constructor references: "
        f"{len(boundary['retired_worker_new_references'])}",
        "- retired worker request-sender references: "
        f"{len(boundary['retired_worker_request_sender_references'])}",
        "- old watch-loop WATCH_DEBOUNCE references: "
        f"{len(boundary['old_watch_debounce_references'])}",
        "- mirror-doc aggregate guard: "
        f"{'present' if boundary['mirror_docs_guard_present'] else 'missing'}",
    ]

    if boundary["missing_source_files"]:
        lines.append(
            "- missing Runtime 04 source files: "
            f"{', '.join(boundary['missing_source_files'])}"
        )
    if boundary["missing_guard_files"]:
        lines.append(
            "- missing Runtime 04 guard/test files: "
            f"{', '.join(boundary['missing_guard_files'])}"
        )
    if boundary["missing_handle_state_anchors"]:
        lines.append(
            "- missing handle/state anchors: "
            f"{', '.join(boundary['missing_handle_state_anchors'])}"
        )
    if boundary["missing_resource_reload_anchors"]:
        lines.append(
            "- missing resource reload anchors: "
            f"{', '.join(boundary['missing_resource_reload_anchors'])}"
        )
    if boundary["missing_worker_pool_anchors"]:
        lines.append(
            "- missing worker-pool anchors: "
            f"{', '.join(boundary['missing_worker_pool_anchors'])}"
        )
    if boundary["missing_watcher_anchors"]:
        lines.append(
            "- missing watcher anchors: "
            f"{', '.join(boundary['missing_watcher_anchors'])}"
        )
    if boundary["missing_artifact_cache_anchors"]:
        lines.append(
            "- missing artifact cache anchors: "
            f"{', '.join(boundary['missing_artifact_cache_anchors'])}"
        )
    if boundary["missing_test_anchors"]:
        lines.append(
            "- missing Runtime 04 test anchors: "
            f"{', '.join(boundary['missing_test_anchors'])}"
        )
    if boundary["missing_behavior_test_anchors"]:
        lines.append(
            "- missing Runtime 04 behavior-test anchors: "
            f"{', '.join(boundary['missing_behavior_test_anchors'])}"
        )
    if boundary["missing_doc_anchors"]:
        lines.append(
            "- missing Runtime 04 doc anchors: "
            f"{', '.join(boundary['missing_doc_anchors'])}"
        )
    if boundary["missing_cargo_gate_anchors"]:
        lines.append(
            "- missing pending Cargo gate anchors: "
            f"{', '.join(boundary['missing_cargo_gate_anchors'])}"
        )
    if boundary["retired_worker_new_references"]:
        lines.append("- retired worker-count constructor references:")
        for reference in boundary["retired_worker_new_references"]:
            lines.append(
                f"  - `{reference['path']}:{reference['line']}` {reference['snippet']}"
            )
    if boundary["retired_worker_request_sender_references"]:
        lines.append("- retired worker request-sender references:")
        for reference in boundary["retired_worker_request_sender_references"]:
            lines.append(
                f"  - `{reference['path']}:{reference['line']}` {reference['snippet']}"
            )
    if boundary["old_watch_debounce_references"]:
        lines.append("- old WATCH_DEBOUNCE references:")
        for reference in boundary["old_watch_debounce_references"]:
            lines.append(
                f"  - `{reference['path']}:{reference['line']}` {reference['snippet']}"
            )

    for risk in boundary["risks"]:
        lines.append(f"- risk: {risk}")

    return lines
