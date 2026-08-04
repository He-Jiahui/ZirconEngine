#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

from runtime_structure_audits.asset_pipeline_boundary import (
    asset_pipeline_boundary_audit,
)
from runtime_structure_audits.asset_pipeline_markdown import (
    render_asset_pipeline_boundary_markdown,
)
from runtime_structure_audits.core_spine_root_generated_boundary import (
    core_spine_root_generated_boundary_audit,
)
from runtime_structure_audits.core_spine_root_generated_markdown import (
    render_core_spine_root_generated_boundary_markdown,
)
from runtime_structure_audits.dynamic_api_test_boundary import (
    dynamic_api_test_boundary_audit,
)
from runtime_structure_audits.dynamic_api_test_markdown import (
    render_dynamic_api_test_boundary_markdown,
)
from runtime_structure_audits.dynamic_runtime_api_boundary import (
    dynamic_runtime_api_boundary_audit,
)
from runtime_structure_audits.dynamic_runtime_api_markdown import (
    render_dynamic_runtime_api_boundary_markdown,
)
from runtime_structure_audits.entry_static_dependencies import (
    entry_static_dependencies_audit,
)
from runtime_structure_audits.entry_static_dependencies_markdown import (
    render_entry_static_dependencies_markdown,
)
from runtime_structure_audits.ecs_query_state_boundary import (
    ecs_query_state_boundary_audit,
)
from runtime_structure_audits.ecs_query_state_markdown import (
    render_ecs_query_state_boundary_markdown,
)
from runtime_structure_audits.ecs_kernel_data_boundary import (
    ecs_kernel_data_boundary_audit,
)
from runtime_structure_audits.ecs_kernel_data_markdown import (
    render_ecs_kernel_data_boundary_markdown,
)
from runtime_structure_audits.generated_code_boundary import (
    generated_code_boundary_audit,
)
from runtime_structure_audits.generated_code_markdown import (
    render_generated_code_boundary_markdown,
)
from runtime_structure_audits.hard_cutover_migration_smells import (
    hard_cutover_migration_smells_audit,
)
from runtime_structure_audits.hard_cutover_migration_smells_markdown import (
    render_hard_cutover_migration_smells_markdown,
)
from runtime_structure_audits.input_stack_boundary import (
    input_stack_boundary_audit,
)
from runtime_structure_audits.input_stack_markdown import (
    render_input_stack_boundary_markdown,
)
from runtime_structure_audits.job_system_boundary import (
    job_system_boundary_audit,
)
from runtime_structure_audits.job_system_markdown import render_job_system_boundary_markdown
from runtime_structure_audits.legacy_standalone_references import (
    legacy_standalone_references,
)
from runtime_structure_audits.legacy_standalone_references_markdown import (
    render_legacy_standalone_references_markdown,
)
from runtime_structure_audits.large_file_ownership import (
    large_file_ownership_gate,
    large_file_hotspot_entries,
    large_file_ownership_classes,
)
from runtime_structure_audits.large_file_ownership_markdown import (
    render_large_file_ownership_gate_markdown,
    render_large_file_hotspots_markdown,
    render_large_file_ownership_classes_markdown,
)
from runtime_structure_audits.native_plugin_public_surface import (
    native_plugin_public_surface_audit,
)
from runtime_structure_audits.native_plugin_public_surface_markdown import (
    render_native_plugin_public_surface_markdown,
)
from runtime_structure_audits.module_inventory import (
    module_convergence_report,
    runtime_inventory,
)
from runtime_structure_audits.module_inventory_markdown import (
    render_engine_module_owner_coverage_markdown,
    render_module_classification_markdown,
    render_module_descriptor_distribution_markdown,
    render_stub_module_descriptor_usage_markdown,
    render_support_crates_markdown,
)
from runtime_structure_audits.module_family_boundary import (
    module_family_boundary_audit,
)
from runtime_structure_audits.module_family_markdown import (
    render_module_family_boundary_markdown,
)
from runtime_structure_audits.module_convention_gate import module_convention_gate
from runtime_structure_audits.module_convention_gate_markdown import (
    render_module_convention_gate_markdown,
)
from runtime_structure_audits.non_network_server_naming import (
    non_network_server_references,
)
from runtime_structure_audits.non_network_server_naming_markdown import (
    render_non_network_server_naming_markdown,
)
from runtime_structure_audits.performance_hotpath_boundary import (
    performance_hotpath_boundary_audit,
)
from runtime_structure_audits.performance_hotpath_markdown import (
    render_performance_hotpath_boundary_markdown,
)
from runtime_structure_audits.plugin_runtime_gaps import (
    plugin_runtime_gaps,
)
from runtime_structure_audits.plugin_runtime_gaps_markdown import (
    render_plugin_runtime_gaps_markdown,
)
from runtime_structure_audits.plugin_surface_lifecycle_boundary import (
    plugin_surface_lifecycle_boundary_audit,
)
from runtime_structure_audits.plugin_surface_lifecycle_markdown import (
    render_plugin_surface_lifecycle_boundary_markdown,
)
from runtime_structure_audits.runtime_api_boundary import (
    runtime_api_boundary_audit,
)
from runtime_structure_audits.runtime_api_markdown import (
    render_runtime_api_boundary_markdown,
)
from runtime_structure_audits.runtime_naming_boundary import (
    runtime_naming_boundary_audit,
)
from runtime_structure_audits.runtime_naming_markdown import (
    render_runtime_naming_boundary_markdown,
)
from runtime_structure_audits.runtime_root_surface import runtime_root_surface_audit
from runtime_structure_audits.runtime_root_surface_markdown import (
    render_runtime_root_surface_markdown,
)
from runtime_structure_audits.runtime_scene_editor_surface import (
    runtime_scene_editor_surface_audit,
)
from runtime_structure_audits.runtime_scene_editor_surface_markdown import (
    render_runtime_scene_editor_surface_markdown,
)
from runtime_structure_audits.schedule_frame_loop_boundary import (
    schedule_frame_loop_boundary_audit,
)
from runtime_structure_audits.schedule_frame_loop_markdown import (
    render_schedule_frame_loop_boundary_markdown,
)
from runtime_structure_audits.scene_project_serialization_boundary import (
    scene_project_serialization_boundary_audit,
)
from runtime_structure_audits.scene_project_serialization_markdown import (
    render_scene_project_serialization_boundary_markdown,
)
from runtime_structure_audits.script_binding_boundary import script_binding_boundary_audit
from runtime_structure_audits.script_binding_markdown import (
    render_script_binding_boundary_markdown,
)
from runtime_structure_audits.tech_stack_boundary import (
    tech_stack_boundary_audit,
)
from runtime_structure_audits.tech_stack_markdown import render_tech_stack_boundary_markdown
from runtime_structure_audits.ui_architecture_boundary import ui_architecture_boundary_audit
from runtime_structure_audits.ui_architecture_markdown import (
    render_ui_architecture_boundary_markdown,
)


def repo_root() -> Path:
    current = Path(__file__).resolve()
    for parent in current.parents:
        if (parent / "Cargo.toml").exists():
            return parent
    raise RuntimeError("Could not find repository root from audit script location.")


def main() -> int:
    parser = argparse.ArgumentParser(description="Audit zirconEngine runtime interface convergence.")
    parser.add_argument("--json", action="store_true", help="Emit JSON instead of Markdown.")
    parser.add_argument(
        "--hotspot-threshold",
        type=int,
        default=1000,
        help="Line-count threshold for large production-file hotspots.",
    )
    args = parser.parse_args()

    root = repo_root()
    inventory = runtime_inventory(root, args.hotspot_threshold)
    plugin_gaps = plugin_runtime_gaps(root, inventory.all_rs_files, inventory.zircon_crates)
    module_report = module_convergence_report(inventory, plugin_gap=bool(plugin_gaps))
    largest_hotspots = large_file_hotspot_entries(inventory.hotspots)
    large_file_gate = large_file_ownership_gate(
        inventory.hotspots,
        args.hotspot_threshold,
    )
    runtime_naming = runtime_naming_boundary_audit(root)
    hard_cutover_smells = hard_cutover_migration_smells_audit(root)
    non_network_servers = non_network_server_references(
        root,
        inventory.all_rs_files,
    )
    module_gate = module_convention_gate(
        module_classification=module_report["module_classification"],
        large_file_ownership_gate=large_file_gate,
        runtime_naming_boundary=runtime_naming,
        hard_cutover_migration_smells=hard_cutover_smells,
        non_network_server_references=non_network_servers,
    )
    report = {
        "module_convention_gate": module_gate,
        "module_descriptor_distribution": module_report["module_descriptor_distribution"],
        "stub_module_descriptor_usage": module_report["stub_module_descriptor_usage"],
        "engine_module_owner_coverage": module_report["engine_module_owner_coverage"],
        "plugin_runtime_gaps": plugin_gaps,
        "entry_static_dependencies": entry_static_dependencies_audit(root),
        "root_surface_audit": runtime_root_surface_audit(root),
        "tech_stack_boundary": tech_stack_boundary_audit(root),
        "core_spine_root_generated_boundary": core_spine_root_generated_boundary_audit(root),
        "runtime_scene_editor_surface": runtime_scene_editor_surface_audit(root),
        "scene_project_serialization_boundary": scene_project_serialization_boundary_audit(root),
        "dynamic_api_test_boundary": dynamic_api_test_boundary_audit(root),
        "runtime_api_boundary": runtime_api_boundary_audit(root),
        "dynamic_runtime_api_boundary": dynamic_runtime_api_boundary_audit(root),
        "ecs_query_state_boundary": ecs_query_state_boundary_audit(root),
        "plugin_surface_lifecycle_boundary": plugin_surface_lifecycle_boundary_audit(root),
        "performance_hotpath_boundary": performance_hotpath_boundary_audit(
            root,
            large_file_gate=large_file_gate,
        ),
        "ecs_kernel_data_boundary": ecs_kernel_data_boundary_audit(root),
        "ui_architecture_boundary": ui_architecture_boundary_audit(root),
        "schedule_frame_loop_boundary": schedule_frame_loop_boundary_audit(root),
        "asset_pipeline_boundary": asset_pipeline_boundary_audit(root),
        "job_system_boundary": job_system_boundary_audit(root),
        "input_stack_boundary": input_stack_boundary_audit(root),
        "script_binding_boundary": script_binding_boundary_audit(root),
        "module_family_boundary": module_family_boundary_audit(root),
        "generated_code_boundary": generated_code_boundary_audit(root),
        "native_plugin_public_surface": native_plugin_public_surface_audit(root),
        "runtime_naming_boundary": runtime_naming,
        "legacy_standalone_references": legacy_standalone_references(root),
        "hard_cutover_migration_smells": hard_cutover_smells,
        "non_network_server_references": non_network_servers,
        "large_file_hotspots": largest_hotspots,
        "large_file_ownership_classes": large_file_ownership_classes(inventory.hotspots),
        "large_file_ownership_gate": large_file_gate,
        "module_classification": module_report["module_classification"],
        "support_crates": module_report["support_crates"],
    }

    if args.json:
        print(json.dumps(report, indent=2, ensure_ascii=False))
        return 0

    print("# Runtime Interface Audit")
    print()
    print("\n".join(render_module_descriptor_distribution_markdown(
        report["module_descriptor_distribution"]
    )))
    print()

    print("\n".join(render_stub_module_descriptor_usage_markdown(
        report["stub_module_descriptor_usage"]
    )))
    print()

    print("\n".join(render_engine_module_owner_coverage_markdown(
        report["engine_module_owner_coverage"]
    )))
    print()

    print("\n".join(render_plugin_runtime_gaps_markdown(report["plugin_runtime_gaps"])))
    print()

    print("\n".join(render_module_convention_gate_markdown(
        report["module_convention_gate"]
    )))
    print()

    print("\n".join(render_entry_static_dependencies_markdown(report["entry_static_dependencies"])))
    print()

    print("\n".join(render_runtime_root_surface_markdown(report["root_surface_audit"])))
    print()

    for line in render_tech_stack_boundary_markdown(report["tech_stack_boundary"]):
        print(line)
    print()

    for line in render_core_spine_root_generated_boundary_markdown(
        report["core_spine_root_generated_boundary"]
    ):
        print(line)
    print()

    for line in render_runtime_scene_editor_surface_markdown(
        report["runtime_scene_editor_surface"]
    ):
        print(line)
    print()

    for line in render_scene_project_serialization_boundary_markdown(
        report["scene_project_serialization_boundary"]
    ):
        print(line)
    print()

    for line in render_dynamic_api_test_boundary_markdown(report["dynamic_api_test_boundary"]):
        print(line)
    print()

    for line in render_runtime_api_boundary_markdown(report["runtime_api_boundary"]):
        print(line)
    print()

    for line in render_dynamic_runtime_api_boundary_markdown(
        report["dynamic_runtime_api_boundary"]
    ):
        print(line)
    print()

    print("\n".join(render_ecs_query_state_boundary_markdown(
        report["ecs_query_state_boundary"]
    )))
    print()

    for line in render_plugin_surface_lifecycle_boundary_markdown(
        report["plugin_surface_lifecycle_boundary"]
    ):
        print(line)
    print()

    for line in render_performance_hotpath_boundary_markdown(
        report["performance_hotpath_boundary"]
    ):
        print(line)
    print()

    for line in render_ecs_kernel_data_boundary_markdown(
        report["ecs_kernel_data_boundary"]
    ):
        print(line)
    print()

    for line in render_ui_architecture_boundary_markdown(
        report["ui_architecture_boundary"]
    ):
        print(line)
    print()

    for line in render_schedule_frame_loop_boundary_markdown(
        report["schedule_frame_loop_boundary"]
    ):
        print(line)
    print()

    for line in render_asset_pipeline_boundary_markdown(
        report["asset_pipeline_boundary"]
    ):
        print(line)
    print()

    for line in render_job_system_boundary_markdown(report["job_system_boundary"]):
        print(line)
    print()

    for line in render_input_stack_boundary_markdown(report["input_stack_boundary"]):
        print(line)
    print()

    for line in render_script_binding_boundary_markdown(report["script_binding_boundary"]):
        print(line)
    print()

    for line in render_module_family_boundary_markdown(report["module_family_boundary"]):
        print(line)
    print()

    for line in render_generated_code_boundary_markdown(report["generated_code_boundary"]):
        print(line)
    print()

    for line in render_native_plugin_public_surface_markdown(
        report["native_plugin_public_surface"]
    ):
        print(line)
    print()

    for line in render_runtime_naming_boundary_markdown(
        report["runtime_naming_boundary"]
    ):
        print(line)
    print()

    print("\n".join(render_legacy_standalone_references_markdown(report["legacy_standalone_references"])))
    print()

    for line in render_hard_cutover_migration_smells_markdown(
        report["hard_cutover_migration_smells"]
    ):
        print(line)
    print()

    print("\n".join(render_non_network_server_naming_markdown(report["non_network_server_references"])))
    print()

    print("\n".join(render_large_file_hotspots_markdown(report["large_file_hotspots"])))
    print()

    print("\n".join(render_large_file_ownership_classes_markdown(report["large_file_ownership_classes"])))
    print()

    print("\n".join(render_large_file_ownership_gate_markdown(report["large_file_ownership_gate"])))
    print()

    print("\n".join(render_module_classification_markdown(report["module_classification"])))
    print()

    support_crate_lines = render_support_crates_markdown(report["support_crates"])
    if support_crate_lines:
        print("\n".join(support_crate_lines))

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
