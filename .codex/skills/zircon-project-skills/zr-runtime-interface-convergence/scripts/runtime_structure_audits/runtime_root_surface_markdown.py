from __future__ import annotations

from runtime_structure_audits.runtime_root_surface import (
    ROOT_MODULE_CLASSIFICATION_ORDER,
)


def render_runtime_root_surface_markdown(root_audit: dict[str, object]) -> list[str]:
    lines = [
        "## Runtime Root Surface Audit",
        "- public modules "
        f"({root_audit['public_module_count']}): {', '.join(root_audit['public_modules'])}",
        f"- M1 gate status: {root_audit['m1_gate_status']}",
        f"- public `pub use` locations ({root_audit['public_use_location_count']})",
        "- crate-visible graphics re-exports: "
        f"{root_audit['crate_visible_graphics_reexport_count']}",
    ]
    decision_groups = root_audit["module_decision_groups"]
    for classification in ROOT_MODULE_CLASSIFICATION_ORDER:
        modules = decision_groups.get(classification, [])
        if modules:
            lines.append(
                f"- {classification} modules ({len(modules)}): {', '.join(modules)}"
            )
    lines.extend(
        f"- migration debt: {debt}"
        for debt in root_audit["root_surface_migration_debt"]
    )
    lines.extend(
        "- public use decision: "
        f"{decision['line']} {decision['classification']} - {decision['reason']}"
        for decision in root_audit["public_use_decisions"]
    )
    lines.extend(f"- risk: {risk}" for risk in root_audit["risks"])
    return lines
