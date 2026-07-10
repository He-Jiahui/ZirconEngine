from __future__ import annotations


def render_module_family_boundary_markdown(boundary: dict[str, object]) -> list[str]:
    families = boundary["families"]
    lines = [
        "## Runtime Module Family Boundary",
        "- documented module-family roots "
        f"({len(families)}/{boundary['expected_family_count']}): "
        f"{', '.join(family['family'] for family in families) if families else 'none'}",
        "- root-seat aggregate guard: "
        f"{'present' if boundary['root_seat_guard_present'] else 'missing'}",
        "- mirror-doc aggregate guard: "
        f"{'present' if boundary['mirror_docs_guard_present'] else 'missing'}",
        "- animation status JSON boundary guard: "
        f"{'present' if boundary['animation_status_json_guard_present'] else 'missing'}; "
        f"anchors {boundary['animation_status_json_anchor_count'] - len(boundary['missing_animation_status_json_anchors'])}/"
        f"{boundary['animation_status_json_anchor_count']}",
        "- module-family guard anchors: "
        f"{boundary['module_family_guard_anchor_count'] - len(boundary['missing_module_family_guard_anchors'])}/"
        f"{boundary['module_family_guard_anchor_count']}",
        "- pending module-family Cargo gate anchors: "
        f"{boundary['cargo_gate_anchor_count'] - len(boundary['missing_cargo_gate_anchors'])}/"
        f"{boundary['cargo_gate_anchor_count']}",
    ]

    for family in families:
        lines.append(
            "- "
            f"`{family['family']}`: root_seat={family['root_seat']}, "
            f"rust_files={family['rust_file_count']}/{family['expected_file_count']}, "
            f"guard=`{family['required_guard']}`"
        )

    if boundary["missing_doc_anchors"]:
        lines.append("- missing doc anchors:")
        for entry in boundary["missing_doc_anchors"]:
            lines.append(
                f"  - `{entry['doc']}` missing `{entry['anchor']}` for `{entry['family']}`"
            )

    if boundary["missing_cargo_gate_anchors"]:
        lines.append(
            "- missing pending Cargo gate anchors: "
            f"{', '.join(boundary['missing_cargo_gate_anchors'])}"
        )
    if boundary["missing_animation_status_json_anchors"]:
        lines.append(
            "- missing animation status JSON anchors: "
            f"{', '.join(boundary['missing_animation_status_json_anchors'])}"
        )
    if boundary["missing_module_family_guard_anchors"]:
        lines.append(
            "- missing module-family guard anchors: "
            f"{', '.join(boundary['missing_module_family_guard_anchors'])}"
        )

    for risk in boundary["risks"]:
        lines.append(f"- risk: {risk}")

    return lines
