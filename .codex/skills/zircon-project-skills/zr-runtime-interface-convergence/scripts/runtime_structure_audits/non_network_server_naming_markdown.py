from __future__ import annotations

from runtime_structure_audits.non_network_server_naming import (
    SERVER_REFERENCE_CLASSIFICATION_DECISIONS,
    SERVER_REFERENCE_CLASSIFICATION_ORDER,
)


def render_non_network_server_naming_markdown(
    server_refs: dict[str, object],
) -> list[str]:
    lines = [
        "## Non-Network Server Naming",
        f"- M1 gate status: {server_refs['m1_gate_status']}",
        f"- ignored observer false positives: {server_refs['observer_false_positive_count']}",
        f"- ignored allowed server contexts: {server_refs['allowed_context_count']}",
    ]
    if not server_refs["count"]:
        lines.append("- none")
    else:
        lines.append(f"- suspect references: {server_refs['count']}")
        classification_counts = server_refs["classification_counts"]
        for classification in SERVER_REFERENCE_CLASSIFICATION_ORDER:
            count = classification_counts.get(classification, 0)
            if not count:
                continue
            decision = SERVER_REFERENCE_CLASSIFICATION_DECISIONS[classification]
            lines.append(
                f"- {classification} references ({count}): "
                f"{decision['target_owner']}"
            )
        for debt in server_refs["non_network_server_migration_debt"]:
            lines.append(f"- migration debt: {debt}")
        for risk in server_refs["risks"]:
            lines.append(f"- risk: {risk}")
        for location in server_refs["sample_locations"]:
            lines.append(
                f"  - `{location['path']}:{location['line']}` "
                f"{location['classification']} {location['snippet']}"
            )
    return lines
