from __future__ import annotations


def render_runtime_api_boundary_markdown(boundary: dict[str, object]) -> list[str]:
    owner_modules = boundary["owner_modules"]
    lines = [
        "## Runtime API Boundary",
        "- folder-backed owner modules "
        f"({len(owner_modules)}/{boundary['expected_module_count']}): "
        f"{', '.join(module['path'] for module in owner_modules) if owner_modules else 'none'}",
        "- facade `zircon_runtime_interface/src/runtime_api.rs`: "
        f"{'present' if boundary['facade_exists'] else 'absent'}",
        "- facade non-empty lines: "
        f"{boundary['facade_non_empty_lines']}/{boundary['max_facade_non_empty_lines']}",
    ]

    if boundary["missing_mod_declarations"]:
        lines.append(
            "- missing mod declarations: "
            f"{', '.join(boundary['missing_mod_declarations'])}"
        )
    if boundary["missing_reexports"]:
        lines.append(
            "- missing re-exports: "
            f"{', '.join(boundary['missing_reexports'])}"
        )

    oversized_modules = boundary["oversized_modules"]
    if oversized_modules:
        lines.append(f"- oversized owner modules (>{boundary['max_module_lines']} lines):")
        for module in oversized_modules:
            lines.append(f"  - `{module['path']}` ({module['lines']} lines)")
    else:
        lines.append(f"- oversized owner modules (>{boundary['max_module_lines']} lines): none")

    for risk in boundary["risks"]:
        lines.append(f"- risk: {risk}")

    return lines
