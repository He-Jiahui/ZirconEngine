from __future__ import annotations


def render_performance_hotpath_boundary_markdown(
    boundary: dict[str, object],
) -> list[str]:
    source_files = boundary["source_files"]
    test_files = boundary["test_files"]
    lines = [
        "## Runtime 07 Performance Hotpath Boundary",
        "- audited performance hotpath source files "
        f"({len(source_files)}/{boundary['expected_source_file_count']}): "
        f"{len(source_files)} files",
        "- audited Runtime 07 guard/test files "
        f"({len(test_files)}/{boundary['expected_test_file_count']}): "
        f"{len(test_files)} files",
        "- frame span anchors: "
        f"{boundary['frame_span_anchor_count'] - len(boundary['missing_frame_span_anchors'])}/"
        f"{boundary['frame_span_anchor_count']}",
        "- QueryState telemetry anchors: "
        f"{boundary['query_counter_anchor_count'] - len(boundary['missing_query_counter_anchors'])}/"
        f"{boundary['query_counter_anchor_count']}",
        "- change-detection telemetry anchors: "
        f"{boundary['change_counter_anchor_count'] - len(boundary['missing_change_counter_anchors'])}/"
        f"{boundary['change_counter_anchor_count']}",
        "- extract telemetry anchors: "
        f"{boundary['extract_counter_anchor_count'] - len(boundary['missing_extract_counter_anchors'])}/"
        f"{boundary['extract_counter_anchor_count']}",
        "- asset-worker candidate telemetry anchors: "
        f"{boundary['asset_worker_anchor_count'] - len(boundary['missing_asset_worker_anchors'])}/"
        f"{boundary['asset_worker_anchor_count']}",
        "- animation scene telemetry anchors: "
        f"{boundary['animation_scene_anchor_count'] - len(boundary['missing_animation_scene_anchors'])}/"
        f"{boundary['animation_scene_anchor_count']}",
        "- profile counter hotspot export anchors: "
        f"{boundary['profile_counter_hotspot_anchor_count'] - len(boundary['missing_profile_counter_hotspot_anchors'])}/"
        f"{boundary['profile_counter_hotspot_anchor_count']}",
        "- hotspot guard anchors: "
        f"{boundary['hotspot_guard_anchor_count'] - len(boundary['missing_hotspot_guard_anchors'])}/"
        f"{boundary['hotspot_guard_anchor_count']}",
        "- Runtime 07 counter assertion anchors: "
        f"{boundary['test_anchor_count'] - len(boundary['missing_test_anchors'])}/"
        f"{boundary['test_anchor_count']}",
        "- Runtime 07 doc anchors: "
        f"{boundary['doc_anchor_count'] - len(boundary['missing_doc_anchors'])}/"
        f"{boundary['doc_anchor_count']}",
        "- pending Cargo/profiling/FPS gate anchors: "
        f"{boundary['cargo_gate_anchor_count'] - len(boundary['missing_cargo_gate_anchors'])}/"
        f"{boundary['cargo_gate_anchor_count']}",
        "- mirror-doc aggregate guard: "
        f"{'present' if boundary['mirror_docs_guard_present'] else 'missing'}",
        "- stale hotspot top3 placeholder present: "
        f"{boundary['stale_hotspot_placeholder_present']}",
        "- large-file owner gate: "
        f"{boundary['large_file_m1_gate_status']} "
        f"(threshold {boundary['large_file_hotspot_threshold']}, "
        f"hotspots {boundary['large_file_hotspot_count']}, "
        f"debt groups {boundary['large_file_migration_debt_count']}, "
        f"unclassified {boundary['large_file_unclassified_hotspot_count']})",
        "- large-file owner classes: "
        f"{boundary['large_file_owner_class_count']} "
        f"({', '.join(boundary['large_file_owner_classes'])})",
    ]

    if boundary["missing_source_files"]:
        lines.append(
            "- missing Runtime 07 source files: "
            f"{', '.join(boundary['missing_source_files'])}"
        )
    if boundary["missing_test_files"]:
        lines.append(
            "- missing Runtime 07 guard/test files: "
            f"{', '.join(boundary['missing_test_files'])}"
        )
    if boundary["missing_frame_span_anchors"]:
        lines.append(
            "- missing frame span anchors: "
            f"{', '.join(boundary['missing_frame_span_anchors'])}"
        )
    if boundary["missing_query_counter_anchors"]:
        lines.append(
            "- missing QueryState telemetry anchors: "
            f"{', '.join(boundary['missing_query_counter_anchors'])}"
        )
    if boundary["missing_change_counter_anchors"]:
        lines.append(
            "- missing change-detection telemetry anchors: "
            f"{', '.join(boundary['missing_change_counter_anchors'])}"
        )
    if boundary["missing_extract_counter_anchors"]:
        lines.append(
            "- missing extract telemetry anchors: "
            f"{', '.join(boundary['missing_extract_counter_anchors'])}"
        )
    if boundary["missing_asset_worker_anchors"]:
        lines.append(
            "- missing asset-worker candidate telemetry anchors: "
            f"{', '.join(boundary['missing_asset_worker_anchors'])}"
        )
    if boundary["missing_animation_scene_anchors"]:
        lines.append(
            "- missing animation scene telemetry anchors: "
            f"{', '.join(boundary['missing_animation_scene_anchors'])}"
        )
    if boundary["missing_profile_counter_hotspot_anchors"]:
        lines.append(
            "- missing profile counter hotspot export anchors: "
            f"{', '.join(boundary['missing_profile_counter_hotspot_anchors'])}"
        )
    if boundary["missing_hotspot_guard_anchors"]:
        lines.append(
            "- missing hotspot guard anchors: "
            f"{', '.join(boundary['missing_hotspot_guard_anchors'])}"
        )
    if boundary["missing_test_anchors"]:
        lines.append(
            "- missing Runtime 07 counter assertion anchors: "
            f"{', '.join(boundary['missing_test_anchors'])}"
        )
    if boundary["missing_doc_anchors"]:
        lines.append(
            "- missing Runtime 07 doc anchors: "
            f"{', '.join(boundary['missing_doc_anchors'])}"
        )
    if boundary["missing_cargo_gate_anchors"]:
        lines.append(
            "- missing pending Cargo/profiling/FPS gate anchors: "
            f"{', '.join(boundary['missing_cargo_gate_anchors'])}"
        )
    if boundary["missing_large_file_owner_classes"]:
        lines.append(
            "- missing expected large-file owner classes: "
            f"{', '.join(boundary['missing_large_file_owner_classes'])}"
        )

    for risk in boundary["risks"]:
        lines.append(f"- risk: {risk}")

    return lines
