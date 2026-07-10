from __future__ import annotations

from runtime_structure_audits.generated_code_boundary import (
    GENERATED_BEHAVIOR_CLASSIFICATION_ORDER,
)


def render_generated_code_boundary_markdown(boundary: dict[str, object]) -> list[str]:
    template_files = boundary["template_files"]
    lines = [
        "## Generated Code Boundary",
        "- export template files "
        f"({boundary['template_file_count']}): {', '.join(template_files) if template_files else 'none'}",
        f"- M1 gate status: {boundary['m1_gate_status']}",
    ]
    decision_groups = boundary["behavior_decision_groups"]
    for classification in GENERATED_BEHAVIOR_CLASSIFICATION_ORDER:
        labels = decision_groups.get(classification, [])
        if labels:
            lines.append(
                f"- {classification} behavior ({len(labels)}): {', '.join(labels)}"
            )
    behavior_locations = boundary["behavior_locations"]
    if not behavior_locations:
        lines.append("- architecture-sensitive generated behavior locations: none")
    else:
        lines.append(
            "- architecture-sensitive generated behavior locations: "
            f"{boundary['behavior_location_count']}"
        )
        lines.append(
            "- allowed generated adapter locations: "
            f"{boundary['allowed_adapter_location_count']}"
        )
        lines.append(
            "- migration-debt generated behavior locations: "
            f"{boundary['migration_debt_location_count']}"
        )
        for label, locations in behavior_locations.items():
            lines.append(f"  - `{label}`")
            for location in locations[:8]:
                lines.append(
                    f"    - `{location['path']}:{location['line']}` "
                    f"[{location['status']}] {location['snippet']}"
                )

    for debt in boundary["generated_boundary_migration_debt"]:
        lines.append(f"- migration debt: {debt}")

    for risk in boundary["risks"]:
        lines.append(f"- risk: {risk}")

    return lines
