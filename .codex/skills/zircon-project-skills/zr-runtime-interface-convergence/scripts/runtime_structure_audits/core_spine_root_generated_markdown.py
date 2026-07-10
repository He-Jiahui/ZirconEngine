from __future__ import annotations


def render_core_spine_root_generated_boundary_markdown(
    boundary: dict[str, object],
) -> list[str]:
    lines = [
        "## Runtime 02 Core Spine Root Generated Boundary",
        "- core root entries: "
        f"{len(boundary['core_root_entries'])}/{len(boundary['expected_core_root_entries'])} "
        f"({', '.join(boundary['core_root_entries'])})",
        "- core public modules: "
        f"{len(boundary['core_public_modules'])}/{len(boundary['expected_core_public_modules'])} "
        f"({', '.join(boundary['core_public_modules'])})",
        "- retired core root entries present: "
        f"{', '.join(boundary['retired_core_entries_present']) if boundary['retired_core_entries_present'] else 'none'}",
        "- runtime root public modules: "
        f"{boundary['root_public_module_count']}/{boundary['expected_root_public_module_count']}",
        "- runtime root public use sites: "
        f"{boundary['root_public_use_count']}/{boundary['expected_root_public_use_count']}",
        "- crate-visible graphics alias debt: "
        f"{boundary['root_graphics_reexport_count']}/{boundary['expected_root_graphics_reexport_count']}",
        f"- root-surface M1 gate status: {boundary['root_surface_m1_gate_status']}",
        "- generated export templates: "
        f"{boundary['generated_template_file_count']}/{boundary['expected_generated_template_file_count']}",
        "- generated behavior locations: "
        f"{boundary['generated_behavior_location_count']}/{boundary['expected_generated_behavior_location_count']}",
        "- generated allowed-adapter locations: "
        f"{boundary['generated_allowed_adapter_location_count']}/{boundary['expected_generated_allowed_adapter_location_count']}",
        "- generated migration-debt locations: "
        f"{boundary['generated_migration_debt_location_count']}/{boundary['expected_generated_migration_debt_location_count']}",
        f"- generated-code M1 gate status: {boundary['generated_m1_gate_status']}",
        "- root_entries guard tests: "
        f"{boundary['root_entries_test_count']}/{boundary['expected_root_entries_test_count']}",
        "- root_surface guard tests: "
        f"{boundary['root_surface_test_count']}/{boundary['expected_root_surface_test_count']}",
        "- generated-code guard tests: "
        f"{boundary['generated_guard_test_count']}/{boundary['expected_generated_guard_test_count']}",
        "- Runtime 02 guard test anchors: "
        f"{boundary['guard_test_anchor_count'] - len(boundary['missing_guard_test_anchors'])}/"
        f"{boundary['guard_test_anchor_count']}",
        "- mirror-doc aggregate guard: "
        f"{'present' if boundary['mirror_docs_guard_present'] else 'missing'}",
    ]

    missing_groups = (
        ("missing root_entries anchors", boundary["missing_root_entries_anchors"]),
        ("missing root_surface anchors", boundary["missing_root_surface_anchors"]),
        ("missing generated-code anchors", boundary["missing_generated_guard_anchors"]),
        ("missing Runtime 02 pending gate anchors", boundary["missing_pending_gate_anchors"]),
        ("missing Runtime 02 doc anchors", boundary["missing_doc_anchors"]),
    )
    for label, values in missing_groups:
        if values:
            lines.append(f"- {label}: {', '.join(values)}")

    for risk in boundary["risks"]:
        lines.append(f"- risk: {risk}")

    return lines
