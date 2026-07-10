from __future__ import annotations


def _render_term_markdown(term: str, report: dict[str, object]) -> list[str]:
    lines = [
        f"- {term} gate status: `{report['gate_status']}`",
        f"- {term} locations/files: {report['location_count']} / {report['file_count']}",
        f"- {term} unclassified locations: {report['unclassified_location_count']}",
    ]
    counts = report["classification_counts"]
    if counts:
        lines.append(f"- {term} classification counts:")
        for classification, count in counts.items():
            lines.append(f"  - {classification}: {count}")
    if report["migration_debt"]:
        lines.append(f"- {term} migration debt:")
        for debt in report["migration_debt"]:
            lines.append(f"  - {debt}")
    if report["unclassified_locations"]:
        lines.append(f"- {term} unclassified locations:")
        for location in report["unclassified_locations"][:20]:
            lines.append(
                f"  - `{location['path']}:{location['line']}` {location['snippet']}"
            )
    return lines


def render_runtime_naming_boundary_markdown(audit: dict[str, object]) -> list[str]:
    lines = [
        "## Runtime Naming Boundary",
        f"- gate status: `{audit['gate_status']}`",
    ]
    lines.extend(_render_term_markdown("editor", audit["editor"]))
    lines.extend(_render_term_markdown("legacy", audit["legacy"]))
    return lines
