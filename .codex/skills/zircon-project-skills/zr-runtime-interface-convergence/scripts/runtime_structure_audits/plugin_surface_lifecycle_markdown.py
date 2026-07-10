from __future__ import annotations


def render_plugin_surface_lifecycle_boundary_markdown(
    boundary: dict[str, object],
) -> list[str]:
    lines = [
        "## Runtime 06 Plugin Surface Lifecycle Boundary",
        "- audited Runtime 06 source files "
        f"({len(boundary['source_files'])}/{boundary['expected_source_file_count']}): "
        f"{len(boundary['source_files'])} files",
        "- audited Runtime 06 mirror docs "
        f"({len(boundary['doc_files'])}/{boundary['expected_doc_file_count']}): "
        f"{len(boundary['doc_files'])} files",
        "- Runtime 06 frontmatter status: "
        f"{boundary['runtime_06_status']}/{boundary['expected_runtime_06_status']}",
        "- Runtime 06 last_refined: "
        f"{boundary['runtime_06_last_refined']}/{boundary['expected_runtime_06_last_refined']}",
        "- native plugin root re-export count: "
        f"{boundary['native_root_reexport_count']}/{boundary['expected_native_root_reexport_count']}",
        "- native plugin namespace re-export count: "
        f"{boundary['native_namespace_reexport_count']}/{boundary['expected_native_namespace_reexport_count']}",
        "- native plugin public-surface M4 gate status: "
        f"{boundary['native_public_surface_m4_gate_status']}/"
        f"{boundary['expected_native_public_surface_m4_gate_status']}",
        "- native plugin public-surface debt groups: "
        f"{boundary['native_public_surface_migration_debt_count']}/"
        f"{boundary['expected_native_public_surface_migration_debt_count']}",
        "- native plugin namespace symbol groups: "
        f"{boundary['native_namespace_symbol_group_count']}/"
        f"{boundary['expected_native_namespace_symbol_group_count']}",
        "- unclassified native root re-export symbols: "
        f"{boundary['unclassified_native_root_reexport_symbol_count']}/"
        f"{boundary['expected_unclassified_native_root_reexport_symbol_count']}",
        "- unclassified native namespace symbols: "
        f"{boundary['unclassified_native_namespace_symbol_count']}/"
        f"{boundary['expected_unclassified_native_namespace_symbol_count']}",
        "- root public native re-export locations: "
        f"{boundary['root_public_native_reexport_location_count']}/"
        f"{boundary['expected_root_public_native_reexport_location_count']}",
        "- public native namespace re-export locations: "
        f"{boundary['public_native_reexport_location_count']}/"
        f"{boundary['expected_public_native_reexport_location_count']}",
        "- app NativePlugin call-site files: "
        f"{boundary['app_native_plugin_file_count']}/"
        f"{boundary['expected_app_native_plugin_file_count']}",
        "- native loader V1/V2 implementation files: "
        f"{boundary['native_loader_v1_v2_file_count']}/"
        f"{boundary['expected_native_loader_v1_v2_file_count']}",
        "- zircon_plugins V1/V2 usage files: "
        f"{len(boundary['plugin_v1_v2_usage_files'])}/"
        f"{len(boundary['expected_plugin_v1_v2_usage_files'])}",
        "- export_build_plan V1/V2 usage count: "
        f"{boundary['export_build_plan_v1_v2_usage_count']}/"
        f"{boundary['expected_export_build_plan_v1_v2_usage_count']}",
        "- native loader test files: "
        f"{boundary['native_loader_test_file_count']}/"
        f"{boundary['expected_native_loader_test_file_count']}",
        "- native test namespace import files: "
        f"{boundary['native_test_namespace_import_file_count']}/"
        f"{boundary['expected_native_test_namespace_import_file_count']}",
        "- native test root import leaks: "
        f"{boundary['native_test_root_import_leak_count']}/"
        f"{boundary['expected_native_test_root_import_leak_count']}",
        "- fallback lifecycle failure tests: "
        f"{boundary['lifecycle_fallback_test_count']}/"
        f"{boundary['expected_lifecycle_fallback_test_count']}",
        "- Runtime 06 source anchors: "
        f"{boundary['source_anchor_count'] - len(boundary['missing_source_anchors'])}/"
        f"{boundary['source_anchor_count']}",
        "- Runtime 06 doc anchors: "
        f"{boundary['doc_anchor_count'] - len(boundary['missing_doc_anchors'])}/"
        f"{boundary['doc_anchor_count']}",
        "- Runtime 06 validation command anchors: "
        f"{boundary['cargo_gate_anchor_count'] - len(boundary['missing_cargo_gate_anchors'])}/"
        f"{boundary['cargo_gate_anchor_count']}",
        "- mirror-doc aggregate guard: "
        f"{'present' if boundary['mirror_docs_guard_present'] else 'missing'}",
    ]

    if boundary["missing_source_files"]:
        lines.append(
            "- missing Runtime 06 source files: "
            f"{', '.join(boundary['missing_source_files'])}"
        )
    if boundary["missing_doc_files"]:
        lines.append(
            "- missing Runtime 06 mirror docs: "
            f"{', '.join(boundary['missing_doc_files'])}"
        )
    if boundary["app_native_plugin_files"]:
        lines.append(
            "- app NativePlugin call-site files: "
            f"{', '.join(boundary['app_native_plugin_files'])}"
        )
    if boundary["native_loader_v1_v2_files"]:
        lines.append(
            "- native loader V1/V2 files: "
            f"{', '.join(boundary['native_loader_v1_v2_files'])}"
        )
    if boundary["plugin_v1_v2_usage_files"]:
        lines.append(
            "- zircon_plugins V1/V2 usage files: "
            f"{', '.join(boundary['plugin_v1_v2_usage_files'])}"
        )
    if boundary["native_loader_test_files"]:
        lines.append(
            "- native loader test files: "
            f"{', '.join(boundary['native_loader_test_files'])}"
        )
    if boundary["native_test_namespace_import_files"]:
        lines.append(
            "- native test namespace import files: "
            f"{', '.join(boundary['native_test_namespace_import_files'])}"
        )
    if boundary["native_test_root_import_leak_files"]:
        lines.append(
            "- native test root import leak files: "
            f"{', '.join(boundary['native_test_root_import_leak_files'])}"
        )
    if boundary["missing_lifecycle_fallback_tests"]:
        lines.append(
            "- missing fallback lifecycle failure tests: "
            f"{', '.join(boundary['missing_lifecycle_fallback_tests'])}"
        )
    if boundary["missing_source_anchors"]:
        lines.append(
            "- missing Runtime 06 source anchors: "
            f"{', '.join(boundary['missing_source_anchors'])}"
        )
    if boundary["missing_doc_anchors"]:
        lines.append(
            "- missing Runtime 06 doc anchors: "
            f"{', '.join(boundary['missing_doc_anchors'])}"
        )
    if boundary["missing_cargo_gate_anchors"]:
        lines.append(
            "- missing Runtime 06 validation command anchors: "
            f"{', '.join(boundary['missing_cargo_gate_anchors'])}"
        )
    for risk in boundary["risks"]:
        lines.append(f"- risk: {risk}")

    return lines
