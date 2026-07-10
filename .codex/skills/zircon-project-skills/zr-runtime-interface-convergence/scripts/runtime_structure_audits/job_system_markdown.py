from __future__ import annotations


def render_job_system_boundary_markdown(boundary: dict[str, object]) -> list[str]:
    owner_modules = boundary["owner_modules"]
    lines = [
        "## JobSystem Boundary",
        "- folder-backed task owner modules "
        f"({len(owner_modules)}/{boundary['expected_module_count']}): "
        f"{', '.join(module['path'] for module in owner_modules) if owner_modules else 'none'}",
        "- direct Rayon paths "
        f"({len(boundary['direct_rayon_paths'])}/{len(boundary['expected_direct_rayon_paths'])}): "
        f"{', '.join(boundary['direct_rayon_paths']) if boundary['direct_rayon_paths'] else 'none'}",
        "- schedule executor uses dependency scheduling: "
        f"{'yes' if boundary['schedule_parallel_executor_uses_schedule_after'] else 'no'}",
        "- scheduler diagnostic anchors: "
        f"{boundary['diagnostic_anchor_count']}",
        "- behavior test anchors: "
        f"{boundary['behavior_test_anchor_count']}",
        "- mirror-doc aggregate guard: "
        f"{'present' if boundary['mirror_docs_guard_present'] else 'missing'}",
    ]

    if boundary["missing_mod_declarations"]:
        lines.append(
            "- missing mod declarations: "
            f"{', '.join(boundary['missing_mod_declarations'])}"
        )
    if boundary["missing_public_surface"]:
        lines.append(
            "- missing public surface exports: "
            f"{', '.join(boundary['missing_public_surface'])}"
        )
    if boundary["missing_api_snippets"]:
        lines.append("- missing API snippets:")
        for file_name, snippets in boundary["missing_api_snippets"].items():
            lines.append(f"  - `{file_name}`: {', '.join(snippets)}")
    if boundary["missing_behavior_test_anchors"]:
        lines.append(
            "- missing behavior test anchors: "
            f"{', '.join(boundary['missing_behavior_test_anchors'])}"
        )

    oversized_modules = boundary["oversized_modules"]
    if oversized_modules:
        lines.append(f"- oversized owner modules (>{boundary['max_module_lines']} lines):")
        for module in oversized_modules:
            lines.append(f"  - `{module['path']}` ({module['lines']} lines)")
    else:
        lines.append(
            f"- oversized owner modules (>{boundary['max_module_lines']} lines): none"
        )

    if boundary["unclassified_direct_rayon"]:
        lines.append("- unclassified direct-Rayon references:")
        for reference in boundary["unclassified_direct_rayon"]:
            lines.append(
                f"  - `{reference['path']}:{reference['line']}` {reference['snippet']}"
            )

    for risk in boundary["risks"]:
        lines.append(f"- risk: {risk}")

    return lines
