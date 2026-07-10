from __future__ import annotations


def render_ui_architecture_boundary_markdown(
    boundary: dict[str, object],
) -> list[str]:
    source_files = boundary["source_files"]
    lines = [
        "## Runtime 09 UI Architecture Boundary",
        "- audited Runtime 09 UI architecture source/doc files "
        f"({len(source_files)}/{boundary['expected_source_file_count']}): "
        f"{len(source_files)} files",
        "- ui/ top-level entries: "
        f"{len(boundary['ui_entries'])}/{boundary['expected_ui_entry_count']}",
        "- surface/ entries: "
        f"{len(boundary['surface_entries'])}/"
        f"{boundary['expected_surface_entry_count']}",
        "- UI legacy full-tree hits: "
        f"{boundary['legacy_full_hits']}/{boundary['expected_legacy_full_hits']}",
        "- UI legacy production hits: "
        f"{boundary['legacy_production_hits']}/"
        f"{boundary['expected_legacy_production_hits']}",
        "- UI legacy production files: "
        f"{len(boundary['legacy_production_files'])}/"
        f"{boundary['expected_legacy_production_file_count']}",
        "- UI taffy production hits: "
        f"{boundary['taffy_production_hits']}/"
        f"{boundary['expected_taffy_production_hits']}",
        "- UI taffy production files: "
        f"{len(boundary['taffy_production_files'])}/"
        f"{boundary['expected_taffy_production_file_count']}",
        "- runtime ui::v2 anchors: "
        f"{boundary['runtime_v2_anchor_count'] - len(boundary['missing_runtime_v2_anchors'])}/"
        f"{boundary['runtime_v2_anchor_count']}",
        "- interface ui::v2 anchors: "
        f"{boundary['interface_v2_anchor_count'] - len(boundary['missing_interface_v2_anchors'])}/"
        f"{boundary['interface_v2_anchor_count']}",
        "- Runtime 09 guard anchors: "
        f"{boundary['guard_anchor_count'] - len(boundary['missing_guard_anchors'])}/"
        f"{boundary['guard_anchor_count']}",
        "- Runtime 09 pending UI owner/Cargo gate anchors: "
        f"{boundary['cargo_gate_anchor_count'] - len(boundary['missing_cargo_gate_anchors'])}/"
        f"{boundary['cargo_gate_anchor_count']}",
        "- Runtime 09 doc anchors: "
        f"{boundary['doc_anchor_count'] - len(boundary['missing_doc_anchors'])}/"
        f"{boundary['doc_anchor_count']}",
        "- mirror-doc aggregate guard: "
        f"{'present' if boundary['mirror_docs_guard_present'] else 'missing'}",
    ]

    if boundary["ui_missing_entries"]:
        lines.append(
            "- missing ui/ entries: " f"{', '.join(boundary['ui_missing_entries'])}"
        )
    if boundary["ui_unexpected_entries"]:
        lines.append(
            "- unexpected ui/ entries: " f"{', '.join(boundary['ui_unexpected_entries'])}"
        )
    if boundary["surface_missing_entries"]:
        lines.append(
            "- missing surface/ entries: "
            f"{', '.join(boundary['surface_missing_entries'])}"
        )
    if boundary["surface_unexpected_entries"]:
        lines.append(
            "- unexpected surface/ entries: "
            f"{', '.join(boundary['surface_unexpected_entries'])}"
        )
    if boundary["baseline_mismatches"]:
        lines.append("- baseline mismatches:")
        for mismatch in boundary["baseline_mismatches"]:
            lines.append(
                "  - "
                f"{mismatch['name']}: actual={mismatch['actual']} "
                f"expected={mismatch['expected']}"
            )
    if boundary["missing_runtime_v2_anchors"]:
        lines.append(
            "- missing runtime ui::v2 anchors: "
            f"{', '.join(boundary['missing_runtime_v2_anchors'])}"
        )
    if boundary["missing_interface_v2_anchors"]:
        lines.append(
            "- missing interface ui::v2 anchors: "
            f"{', '.join(boundary['missing_interface_v2_anchors'])}"
        )
    if boundary["missing_guard_anchors"]:
        lines.append(
            "- missing Runtime 09 guard anchors: "
            f"{', '.join(boundary['missing_guard_anchors'])}"
        )
    if boundary["missing_cargo_gate_anchors"]:
        lines.append(
            "- missing Runtime 09 pending UI owner/Cargo gate anchors: "
            f"{', '.join(boundary['missing_cargo_gate_anchors'])}"
        )
    if boundary["missing_doc_anchors"]:
        lines.append(
            "- missing Runtime 09 doc anchors: "
            f"{', '.join(boundary['missing_doc_anchors'])}"
        )
    if boundary["missing_required_doc_mentions"]:
        lines.append("- missing required Runtime 09 doc mentions:")
        for entry in boundary["missing_required_doc_mentions"]:
            lines.append(f"  - `{entry['path']}` missing `{entry['snippet']}`")

    for risk in boundary["risks"]:
        lines.append(f"- risk: {risk}")

    return lines
