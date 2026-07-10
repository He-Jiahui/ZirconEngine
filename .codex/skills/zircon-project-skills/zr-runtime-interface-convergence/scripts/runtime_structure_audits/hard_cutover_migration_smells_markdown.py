from __future__ import annotations

from runtime_structure_audits.hard_cutover_migration_smells import (
    HARD_CUTOVER_CLASSIFICATION_DECISIONS,
    HARD_CUTOVER_CLASSIFICATION_ORDER,
)


def render_hard_cutover_migration_smells_markdown(
    boundary: dict[str, object],
) -> list[str]:
    lines = [
        "## Hard-Cutover Migration Smells",
        f"- source files scanned: {boundary['source_file_count']}",
        f"- hard-cutover gate status: {boundary['hard_cutover_gate_status']}",
        f"- legacy references: {boundary['legacy_reference_count']}",
        f"- compat references: {boundary['compat_reference_count']}",
        f"- shim references: {boundary['shim_reference_count']}",
        f"- bridge references: {boundary['bridge_reference_count']}",
        "- allowed business bridge references: "
        f"{boundary['allowed_business_bridge_reference_count']}",
        f"- migration-context bridge references: {boundary['migration_bridge_smell_count']}",
    ]
    classification_counts = boundary["classification_counts"]
    if not classification_counts:
        lines.append("- migration smell references: none")
    else:
        for classification in HARD_CUTOVER_CLASSIFICATION_ORDER:
            count = classification_counts.get(classification, 0)
            if not count:
                continue
            decision = HARD_CUTOVER_CLASSIFICATION_DECISIONS[classification]
            lines.append(
                f"- {classification} references ({count}): "
                f"{decision['target_owner']}"
            )
        for debt in boundary["hard_cutover_migration_debt"]:
            lines.append(f"- migration debt: {debt}")
        for risk in boundary["risks"]:
            lines.append(f"- risk: {risk}")
        samples = boundary["classification_samples"]
        for classification in HARD_CUTOVER_CLASSIFICATION_ORDER:
            locations = samples.get(classification, [])
            if not locations:
                continue
            lines.append(f"- {classification} samples:")
            for location in locations:
                lines.append(
                    f"  - `{location['path']}:{location['line']}` "
                    f"{location['term']} {location['snippet']}"
                )
    allowed_bridge_samples = boundary["allowed_business_bridge_samples"]
    if allowed_bridge_samples:
        lines.append("- allowed business bridge samples:")
        for location in allowed_bridge_samples:
            lines.append(
                f"  - `{location['path']}:{location['line']}` "
                f"{location['snippet']}"
            )
    return lines
