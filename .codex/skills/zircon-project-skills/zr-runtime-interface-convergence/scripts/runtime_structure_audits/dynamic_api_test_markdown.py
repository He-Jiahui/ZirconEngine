from __future__ import annotations


def render_dynamic_api_test_boundary_markdown(boundary: dict[str, object]) -> list[str]:
    test_modules = boundary["test_modules"]
    lines = [
        "## Dynamic API Test Boundary",
        "- folder-backed owner modules "
        f"({len(test_modules)}/{boundary['expected_module_count']}): "
        f"{', '.join(module['path'] for module in test_modules) if test_modules else 'none'}",
        "- legacy `zircon_runtime/src/dynamic_api/tests.rs`: "
        f"{'present' if boundary['legacy_tests_file_exists'] else 'absent'}",
    ]

    if boundary["missing_modules"]:
        lines.append("- missing owner modules:")
        lines.extend(f"  - `{missing_module}`" for missing_module in boundary["missing_modules"])
    if boundary["missing_mod_declarations"]:
        lines.append(
            "- missing mod declarations: "
            f"{', '.join(boundary['missing_mod_declarations'])}"
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
