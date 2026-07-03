type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 graphics dead-code guard module split",
        &[
            "runtime_15_graphics_dead_code_guard_module_split_static_passed_cargo_lock_blocked",
            "graphics_dead_code/module_layout.rs",
            "graphics_dead_code/renderer_output_accessors.rs",
            "runtime_15_graphics_dead_code_guard_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 graphics dead-code guard child-owner split",
        &[
            "runtime_15_graphics_dead_code_guard_child_owner_split_static_passed_cargo_deferred",
            "graphics_dead_code/backend_owners.rs",
            "graphics_dead_code/gpu_resource_owners.rs",
            "graphics_dead_code/resource_streamer_cleanup.rs",
        ],
    ),
    (
        "Runtime 15 M3 graphics dead-code guard forbidden attribute literal cleanup",
        &[
            "runtime_15_graphics_dead_code_guard_literal_cleanup_static_passed_cargo_deferred",
            "graphics_dead_code/mod.rs",
            "DEAD_CODE_ALLOW_ATTRIBUTE",
            "runtime_15_graphics_dead_code_guard_forbidden_attribute_literal_is_constant_backed",
        ],
    ),
    (
        "Runtime 15 M3 provider boilerplate guard module split",
        &[
            "runtime_15_provider_boilerplate_guard_module_split_static_passed_cargo_lock_blocked",
            "structure_convention/provider_boilerplate.rs",
            "runtime_15_provider_boilerplate_guard_is_folder_backed",
            "runtime_15_provider_registration_uses_shared_owner",
        ],
    ),
    (
        "Runtime 15 M3 provider boilerplate guard child-owner split",
        &[
            "runtime_15_provider_boilerplate_guard_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/provider_boilerplate.rs",
            "structure_convention/provider_boilerplate/module_layout.rs",
            "structure_convention/provider_boilerplate/full_audit.rs",
            "runtime_15_provider_boilerplate_guard_child_owner_split",
        ],
    ),
    (
        "Runtime 15 M3 facade surface guard module split",
        &[
            "runtime_15_facade_surface_guard_module_split_static_passed_cargo_lock_blocked",
            "structure_convention/facade_surface.rs",
            "runtime_15_facade_surface_guard_is_folder_backed",
            "runtime_15_prelude_covers_required_types",
        ],
    ),
    (
        "Runtime 15 M3 runtime dead-code guard module split",
        &[
            "runtime_15_runtime_dead_code_guard_module_split_static_passed_cargo_lock_blocked",
            "structure_convention/runtime_dead_code/mod.rs",
            "runtime_15_runtime_dead_code_guard_is_folder_backed",
            "runtime_15_runtime_ui_dead_code_surface_is_test_support",
        ],
    ),
    (
        "Runtime 15 M3 runtime dead-code guard forbidden attribute literal cleanup",
        &[
            "runtime_15_runtime_dead_code_guard_literal_cleanup_static_passed_cargo_deferred",
            "structure_convention/runtime_dead_code/mod.rs",
            "DEAD_CODE_ALLOW_ATTRIBUTE",
            "runtime_15_runtime_dead_code_guard_forbidden_attribute_literal_is_constant_backed",
        ],
    ),
    (
        "Runtime 15 M3 runtime dead-code guard child-owner split",
        &[
            "runtime_15_runtime_dead_code_guard_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/runtime_dead_code/mod.rs",
            "structure_convention/runtime_dead_code/runtime_ui.rs",
            "structure_convention/runtime_dead_code/production_scan.rs",
            "runtime_15_runtime_dead_code_guard_children_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 runtime dead-code documentation anchor cleanup",
        &[
            "runtime_15_runtime_dead_code_documentation_anchor_cleanup_static_passed_cargo_deferred",
            "structure_convention/runtime_dead_code/mod.rs",
            "structure_convention/runtime_dead_code/runtime_ui.rs",
            "structure_convention/runtime_dead_code/production_scan.rs",
            "structure_convention/runtime_dead_code/status_anchor_cleanup.rs",
            "runtime_15_runtime_dead_code_documentation_anchors_use_folder_owner",
        ],
    ),
    (
        "Runtime 15 M3 runtime dead-code module-gate status wording cleanup",
        &[
            "runtime_15_runtime_dead_code_module_gate_status_wording_static_passed_cargo_deferred",
            "structure_convention/runtime_dead_code/status_anchor_cleanup.rs",
            "runtime_15_runtime_dead_code_current_rows_keep_module_gate_audit_clear",
            "module_convention_gate audit clear",
            "full Cargo sweep",
        ],
    ),
    (
        "Runtime 15 M3 runtime dead-code production-gate status wording cleanup",
        &[
            "runtime_15_runtime_dead_code_production_gate_status_wording_static_passed_cargo_deferred",
            "structure_convention/runtime_dead_code/status_anchor_cleanup.rs",
            "runtime_15_runtime_dead_code_current_rows_use_production_gate_name",
            "runtime_15_production_sources_do_not_allow_dead_code_suppression",
            "Runtime 15 M5 production dead-code suppression global gate",
        ],
    ),
];
