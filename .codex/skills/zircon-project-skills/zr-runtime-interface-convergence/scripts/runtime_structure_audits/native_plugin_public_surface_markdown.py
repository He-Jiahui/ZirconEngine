from __future__ import annotations

from runtime_structure_audits.native_plugin_public_surface import (
    NATIVE_PLUGIN_SYMBOL_CLASSIFICATION_ORDER,
)


def render_native_plugin_public_surface_markdown(
    boundary: dict[str, object],
) -> list[str]:
    lines = [
        "## Native Plugin Public Surface",
        "- root native loader/ABI re-export count: "
        f"{boundary['root_reexport_count']}",
        "- native namespace re-export count: "
        f"{boundary['native_namespace_reexport_count']}",
        f"- M4 gate status: {boundary['m4_gate_status']}",
    ]
    decision_groups = boundary["symbol_decision_groups"]
    for classification in NATIVE_PLUGIN_SYMBOL_CLASSIFICATION_ORDER:
        symbols = decision_groups.get(classification, [])
        if symbols:
            lines.append(
                f"- {classification} symbols ({len(symbols)}): {', '.join(symbols)}"
            )
    for debt in boundary["native_plugin_public_surface_migration_debt"]:
        lines.append(f"- migration debt: {debt}")
    if boundary["root_reexport_symbols_sample"]:
        lines.append(
            "- root re-export sample: "
            f"{', '.join(boundary['root_reexport_symbols_sample'])}"
        )
    if boundary["native_namespace_symbols_sample"]:
        lines.append(
            "- native namespace sample: "
            f"{', '.join(boundary['native_namespace_symbols_sample'])}"
        )
    root_native_locations = boundary["root_public_reexport_locations"]
    if root_native_locations:
        lines.append(
            "- root public native re-export locations "
            f"({boundary['root_public_reexport_location_count']}):"
        )
        for location in root_native_locations:
            lines.append(
                f"  - `{location['path']}:{location['line']}` "
                f"{location['snippet']}"
            )
    public_native_locations = boundary["public_reexport_locations"]
    if public_native_locations:
        lines.append(
            "- public native namespace re-export locations "
            f"({boundary['public_reexport_location_count']}):"
        )
        for location in public_native_locations:
            lines.append(
                f"  - `{location['path']}:{location['line']}` "
                f"{location['snippet']}"
            )
    for risk in boundary["risks"]:
        lines.append(f"- risk: {risk}")

    return lines
