from __future__ import annotations


def render_module_convention_gate_markdown(
    gate: dict[str, object],
) -> list[str]:
    lines = [
        "## Module Convention Gate",
        f"- M1 gate status: {gate['m1_gate_status']}",
        f"- migration debt count: {gate['migration_debt_count']}",
        f"- render-scoped migration debt count: {gate['render_scoped_migration_debt_count']}",
        f"- non-render migration debt count: {gate['non_render_migration_debt_count']}",
        f"- exempt entries: {gate['exempt_count']}",
    ]
    lines.append("- source gate statuses:")
    for source, status in gate["source_gate_statuses"].items():
        lines.append(f"  - {source}: {status}")

    lines.append("- violation fields:")
    for field, count in gate["violation_fields"].items():
        lines.append(f"  - {field}: {count}")

    classification_counts = gate["classification_counts"]
    if classification_counts:
        lines.append("- classification counts:")
        for classification, count in classification_counts.items():
            lines.append(f"  - {classification}: {count}")

    for debt in gate["migration_debt"][:20]:
        lines.append(f"- migration debt: {debt}")
    for debt in gate["non_render_migration_debt"][:20]:
        lines.append(f"- non-render migration debt: {debt}")
    for risk in gate["risks"][:20]:
        lines.append(f"- risk: {risk}")
    return lines
