from __future__ import annotations

from runtime_structure_audits.large_file_ownership import (
    LARGE_FILE_OWNER_CLASSIFICATION_ORDER,
)


def render_large_file_hotspots_markdown(
    large_file_hotspots: list[dict[str, object]],
) -> list[str]:
    lines = ["## Large File Hotspots"]
    if not large_file_hotspots:
        lines.append("- none above threshold")
    else:
        for hotspot in large_file_hotspots:
            lines.append(
                f"- `{hotspot['path']}` ({hotspot['lines']} lines, "
                f"{hotspot['owner_class']})"
            )
    return lines


def render_large_file_ownership_classes_markdown(
    ownership_classes: dict[str, dict[str, object]],
) -> list[str]:
    lines = ["## Large File Ownership Classes"]
    if not ownership_classes:
        lines.append("- none")
    else:
        for owner_class, summary in ownership_classes.items():
            lines.append(
                f"- `{owner_class}`: {summary['count']} file(s), "
                f"max {summary['max_lines']} lines"
            )
    return lines


def render_large_file_ownership_gate_markdown(
    gate: dict[str, object],
) -> list[str]:
    lines = [
        "## Large File Ownership Gate",
        f"- threshold: {gate['threshold']} lines",
        f"- M1 gate status: {gate['m1_gate_status']}",
        f"- hotspot count: {gate['hotspot_count']}",
    ]
    classification_counts = gate["classification_counts"]
    if not classification_counts:
        lines.append("- migration debt: none")
    else:
        owner_decisions = gate["owner_decisions"]
        for classification in LARGE_FILE_OWNER_CLASSIFICATION_ORDER:
            count = classification_counts.get(classification, 0)
            if not count:
                continue
            decision = owner_decisions[classification]
            lines.append(
                f"- {classification} hotspots ({count}): {decision['target_owner']}"
            )
        for debt in gate["large_file_migration_debt"]:
            lines.append(f"- migration debt: {debt}")
        for risk in gate["risks"]:
            lines.append(f"- risk: {risk}")
        decision_groups = gate["decision_groups"]
        for classification in LARGE_FILE_OWNER_CLASSIFICATION_ORDER:
            paths = decision_groups.get(classification, [])
            if not paths:
                continue
            lines.append(f"- {classification} samples:")
            for path in paths[:5]:
                lines.append(f"  - `{path}`")
    return lines
