from __future__ import annotations


def render_ecs_query_state_boundary_markdown(
    query_state_boundary: dict[str, object],
) -> list[str]:
    owner_modules = query_state_boundary["owner_modules"]
    oversized_modules = query_state_boundary["oversized_modules"]

    lines = [
        "## ECS QueryState Boundary",
        "- folder-backed owner modules "
        f"({len(owner_modules)}/{query_state_boundary['expected_module_count']}): "
        f"{', '.join(module['path'] for module in owner_modules) if owner_modules else 'none'}",
        "- legacy `zircon_runtime/src/scene/ecs/query/query_state.rs`: "
        f"{'present' if query_state_boundary['legacy_file_exists'] else 'absent'}",
        "- root non-empty lines: "
        f"{query_state_boundary['root_non_empty_lines']}/{query_state_boundary['max_root_non_empty_lines']}",
    ]

    if oversized_modules:
        lines.append(
            f"- oversized owner modules (>{query_state_boundary['max_module_lines']} lines):"
        )
        for module in oversized_modules:
            lines.append(f"  - `{module['path']}` ({module['lines']} lines)")
    else:
        lines.append(
            f"- oversized owner modules (>{query_state_boundary['max_module_lines']} lines): none"
        )

    lines.extend(f"- risk: {risk}" for risk in query_state_boundary["risks"])
    return lines
