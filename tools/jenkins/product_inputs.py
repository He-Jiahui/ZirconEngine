"""Exact compiler and validation inputs for the managed editor product.

These paths are read rights in the reader closure. They never expand the
managed writer's implementation ownership or admit checkout configuration.
"""
from __future__ import annotations

from .contracts import JenkinsError, require_digest

CAPABILITY = "managed-worktree-product-inputs-v1"
PROJECT_TEMPLATE_INPUTS = tuple("templates/projects/renderable-empty/" + path for path in (
    ".gitignore", ".zircon/autosave/.gitignore", ".zircon/cache/.gitignore",
    ".zircon/play/.gitignore", ".zircon/registry/.gitignore", ".zircon/settings.toml",
    ".zircon/thumbnails/.gitignore", "assets/materials/default.zmaterial",
    "assets/materials/default.zmaterial.zmeta", "assets/models/cube.obj",
    "assets/models/cube.obj.zmeta", "assets/scenes/main.scene.toml",
    "assets/shaders/pbr_shader.zmeta", "assets/shaders/pbr_shader/pbr.wgsl",
    "assets/shaders/pbr_shader/pbr.zshader", "export/desktop_windows.zpreset",
    "zircon-project.toml",
))
_AUDIT_ROOT = ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/"
# Reviewed direct Rust includes/reads and the static Python import closure of
# the two audit entry points. No directory or wildcard grants are accepted.
COMPILER_CONTROL_PATHS = frozenset(_AUDIT_ROOT + path for path in (
    'audit_editor_structure.py',
    'audit_runtime_structure.py',
    'editor_structure_audits/__init__.py',
    'editor_structure_audits/module_convention_boundary.py',
    'runtime_structure_audits/__init__.py',
    'runtime_structure_audits/asset_pipeline_anchor_inventory.py',
    'runtime_structure_audits/asset_pipeline_boundary.py',
    'runtime_structure_audits/asset_pipeline_markdown.py',
    'runtime_structure_audits/asset_pipeline_source_inventory.py',
    'runtime_structure_audits/core_spine_root_generated_boundary.py',
    'runtime_structure_audits/core_spine_root_generated_markdown.py',
    'runtime_structure_audits/dynamic_api_test_boundary.py',
    'runtime_structure_audits/dynamic_api_test_markdown.py',
    'runtime_structure_audits/dynamic_runtime_api_abi_inventory.py',
    'runtime_structure_audits/dynamic_runtime_api_archive_inventory.py',
    'runtime_structure_audits/dynamic_runtime_api_boundary.py',
    'runtime_structure_audits/dynamic_runtime_api_diagnostics_inventory.py',
    'runtime_structure_audits/dynamic_runtime_api_failure_inventory.py',
    'runtime_structure_audits/dynamic_runtime_api_host_request_inventory.py',
    'runtime_structure_audits/dynamic_runtime_api_markdown.py',
    'runtime_structure_audits/dynamic_runtime_api_session_lifecycle_inventory.py',
    'runtime_structure_audits/dynamic_runtime_api_ui_contract_inventory.py',
    'runtime_structure_audits/dynamic_runtime_api_validation_inventory.py',
    'runtime_structure_audits/ecs_kernel_data_anchor_inventory.py',
    'runtime_structure_audits/ecs_kernel_data_boundary.py',
    'runtime_structure_audits/ecs_kernel_data_markdown.py',
    'runtime_structure_audits/ecs_kernel_data_source_inventory.py',
    'runtime_structure_audits/ecs_query_state_boundary.py',
    'runtime_structure_audits/ecs_query_state_markdown.py',
    'runtime_structure_audits/entry_static_dependencies.py',
    'runtime_structure_audits/entry_static_dependencies_markdown.py',
    'runtime_structure_audits/generated_code_boundary.py',
    'runtime_structure_audits/generated_code_markdown.py',
    'runtime_structure_audits/hard_cutover_migration_smells.py',
    'runtime_structure_audits/hard_cutover_migration_smells_markdown.py',
    'runtime_structure_audits/input_stack_anchor_inventory.py',
    'runtime_structure_audits/input_stack_boundary.py',
    'runtime_structure_audits/input_stack_markdown.py',
    'runtime_structure_audits/input_stack_source_inventory.py',
    'runtime_structure_audits/job_system_anchor_inventory.py',
    'runtime_structure_audits/job_system_boundary.py',
    'runtime_structure_audits/job_system_markdown.py',
    'runtime_structure_audits/job_system_source_inventory.py',
    'runtime_structure_audits/large_file_ownership.py',
    'runtime_structure_audits/large_file_ownership_markdown.py',
    'runtime_structure_audits/legacy_standalone_references.py',
    'runtime_structure_audits/legacy_standalone_references_markdown.py',
    'runtime_structure_audits/module_convention_gate.py',
    'runtime_structure_audits/module_convention_gate_markdown.py',
    'runtime_structure_audits/module_family_boundary.py',
    'runtime_structure_audits/module_family_markdown.py',
    'runtime_structure_audits/module_inventory.py',
    'runtime_structure_audits/module_inventory_markdown.py',
    'runtime_structure_audits/native_plugin_public_surface.py',
    'runtime_structure_audits/native_plugin_public_surface_markdown.py',
    'runtime_structure_audits/non_network_server_naming.py',
    'runtime_structure_audits/non_network_server_naming_markdown.py',
    'runtime_structure_audits/performance_hotpath_anchor_inventory.py',
    'runtime_structure_audits/performance_hotpath_boundary.py',
    'runtime_structure_audits/performance_hotpath_markdown.py',
    'runtime_structure_audits/performance_hotpath_source_inventory.py',
    'runtime_structure_audits/plugin_runtime_gaps.py',
    'runtime_structure_audits/plugin_runtime_gaps_markdown.py',
    'runtime_structure_audits/plugin_surface_lifecycle_boundary.py',
    'runtime_structure_audits/plugin_surface_lifecycle_markdown.py',
    'runtime_structure_audits/runtime_api_boundary.py',
    'runtime_structure_audits/runtime_api_markdown.py',
    'runtime_structure_audits/runtime_naming_boundary.py',
    'runtime_structure_audits/runtime_naming_markdown.py',
    'runtime_structure_audits/runtime_root_surface.py',
    'runtime_structure_audits/runtime_root_surface_markdown.py',
    'runtime_structure_audits/runtime_scene_editor_surface.py',
    'runtime_structure_audits/runtime_scene_editor_surface_markdown.py',
    'runtime_structure_audits/scene_project_serialization_boundary.py',
    'runtime_structure_audits/scene_project_serialization_markdown.py',
    'runtime_structure_audits/schedule_frame_loop_anchor_inventory.py',
    'runtime_structure_audits/schedule_frame_loop_boundary.py',
    'runtime_structure_audits/schedule_frame_loop_markdown.py',
    'runtime_structure_audits/schedule_frame_loop_source_inventory.py',
    'runtime_structure_audits/script_binding_boundary.py',
    'runtime_structure_audits/script_binding_markdown.py',
    'runtime_structure_audits/tech_stack_anchor_inventory.py',
    'runtime_structure_audits/tech_stack_boundary.py',
    'runtime_structure_audits/tech_stack_markdown.py',
    'runtime_structure_audits/tech_stack_source_inventory.py',
    'runtime_structure_audits/ui_architecture_boundary.py',
    'runtime_structure_audits/ui_architecture_markdown.py',
))
FORBIDDEN_ROOTS = frozenset({".git", ".codex", ".jenkins", ".opencode", "target"})


def product_path(path):
    return path.split("/", 1)[0] not in FORBIDDEN_ROOTS or path in COMPILER_CONTROL_PATHS


def compiler_controls(value, hashes, owned, *, managed):
    if value is None:
        return None
    if not managed or not isinstance(value, dict) or set(value) != COMPILER_CONTROL_PATHS:
        raise JenkinsError("compiler_control_scope_invalid",
                           "Managed product control inputs require the exact reviewed file set")
    if set(owned) & COMPILER_CONTROL_PATHS:
        raise JenkinsError("compiler_control_ownership_invalid",
                           "Compiler audit inputs are validation dependencies")
    for name, checksum in value.items():
        if require_digest(checksum, "compilerControlHash") != hashes.get(name):
            raise JenkinsError("compiler_control_hash_mismatch", "Compiler input is absent from the exact source closure")
    return dict(sorted(value.items()))


def validate_hash_paths(hashes, controls):
    admitted = set(controls or {})
    if any(path.split("/", 1)[0] in FORBIDDEN_ROOTS and path not in admitted for path in hashes):
        raise JenkinsError("current_source_control_forbidden",
                           "Product inputs contain checkout controls or compiler output")


def require_product_inputs(hashes, controls):
    compiler_controls(controls, hashes, (), managed=True)
    missing = [name for name in (*PROJECT_TEMPLATE_INPUTS, *COMPILER_CONTROL_PATHS)
               if not hashes.get(name)]
    if missing:
        raise JenkinsError("editor_inputs_missing", "Managed product compiler inputs are incomplete",
                           details={"paths": sorted(missing)})
