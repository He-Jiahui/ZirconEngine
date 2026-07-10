from __future__ import annotations


def render_scene_project_serialization_boundary_markdown(
    boundary: dict[str, object],
) -> list[str]:
    serialization_files = boundary["files"]
    lines = [
        "## Scene Project Serialization Boundary",
        "- audited scene serialization files "
        f"({len(serialization_files)}): {', '.join(serialization_files) if serialization_files else 'none'}",
    ]
    forbidden_locations = boundary["forbidden_locations"]
    if not forbidden_locations:
        lines.append("- editor authoring-state locations: none")
    else:
        lines.append(
            "- editor authoring-state locations: "
            f"{boundary['forbidden_location_count']}"
        )
        for label, locations in forbidden_locations.items():
            lines.append(f"  - `{label}`")
            for location in locations[:8]:
                lines.append(
                    f"    - `{location['path']}:{location['line']}` {location['snippet']}"
                )

    for risk in boundary["risks"]:
        lines.append(f"- risk: {risk}")

    return lines
