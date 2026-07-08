pub(super) const EXPECTED_CORE_ROOT_ENTRIES: &[&str] = &[
    "framework",
    "manager",
    "math",
    "mod.rs",
    "resource",
    "runtime",
];

pub(super) const EXPECTED_CORE_PUBLIC_MODULES: &[&str] =
    &["runtime", "framework", "manager", "math", "resource"];

pub(super) const RETIRED_CORE_ROOT_ENTRIES: &[&str] = &[
    "channel_util.rs",
    "config_store.rs",
    "diagnostics",
    "event_bus",
    "event_bus.rs",
    "frame_clock.rs",
    "job_scheduler.rs",
    "lifecycle.rs",
    "modules",
    "state",
    "tasks",
    "time.rs",
    "types.rs",
];

pub(super) const EXPECTED_RUNTIME_02_GUARD_TEST_ANCHORS: &[(&str, &[&str])] = &[
    (
        "zircon_runtime/src/tests/runtime_absorption/root_entries/core_spine.rs",
        &[
            "core_root_retires_channel_and_service_alias_fragments",
            "core_root_retires_runtime_kernel_fragment_files",
            "core_root_splits_event_dto_from_runtime_event_bus",
            "core_root_reexports_runtime_diagnostics_without_root_directory",
            "core_module_tree_matches_decided_spine_shape",
        ],
    ),
    (
        "zircon_runtime/src/tests/runtime_absorption/root_entries/runtime_root.rs",
        &[
            "runtime_crate_root_does_not_flatten_plugin_surface",
            "runtime_crate_root_does_not_flatten_builtin_module_assembly_functions",
            "builtin_root_stays_structural_after_runtime_module_split",
        ],
    ),
    (
        "zircon_runtime/src/tests/runtime_absorption/root_entries/module_families/navigation.rs",
        &["runtime_navigation_boundary_file_set_requires_doc_update"],
    ),
    (
        "zircon_runtime/src/tests/runtime_absorption/root_entries/module_families/animation_backlog.rs",
        &["runtime_animation_backlog_boundary_requires_doc_update"],
    ),
    (
        "zircon_runtime/src/tests/runtime_absorption/root_entries/module_families/animation_status_json.rs",
        &["runtime_animation_status_json_boundary_sanitizes_non_finite_values"],
    ),
    (
        "zircon_runtime/src/tests/runtime_absorption/root_entries/module_families/root_seats.rs",
        &["runtime_14_module_family_root_seats_match_documented_judgements"],
    ),
    (
        "zircon_runtime/src/tests/runtime_absorption/root_entries/module_families/mirror_docs.rs",
        &["runtime_14_module_family_mirror_docs_match_structure_audit_counts"],
    ),
    (
        "zircon_runtime/src/tests/runtime_absorption/root_surface/public_surface.rs",
        &["runtime_crate_root_public_surface_stays_curated"],
    ),
    (
        "zircon_runtime/src/tests/runtime_absorption/root_surface/graphics_alias.rs",
        &[
            "graphics_alias_debt_is_removed_from_runtime_root",
            "graphics_type_alias_debt_symbols_are_only_available_through_graphics_namespace",
        ],
    ),
    (
        "zircon_runtime/src/tests/runtime_absorption/root_surface/docs.rs",
        &[
            "core_spine_and_root_surface_docs_stay_in_sync",
            "root_surface_m1_gate_matches_runtime_14_module_family_seats",
            "root_surface_interface_convergence_mirror_uses_current_audit_counts",
        ],
    ),
    (
        "zircon_runtime/src/tests/runtime_absorption/generated_code_guard/markers.rs",
        &[
            "generated_marker_format_is_uniform_when_source_files_are_marked",
            "marked_generated_source_files_stay_leaf_data_only",
        ],
    ),
    (
        "zircon_runtime/src/tests/runtime_absorption/generated_code_guard/behavior.rs",
        &[
            "export_template_generated_behavior_stays_classified_by_owner",
            "export_template_generated_behavior_is_adapter_only_after_m4_cutover",
        ],
    ),
    (
        "zircon_runtime/src/tests/runtime_absorption/generated_code_guard/scope.rs",
        &["export_template_scan_scope_stays_folder_backed"],
    ),
    (
        "zircon_runtime/src/tests/runtime_absorption/generated_code_guard/delegation.rs",
        &[
            "export_entry_templates_delegate_to_app_export_bootstrap_facade",
            "export_plugin_selection_template_delegates_registration_execution_to_app_providers",
        ],
    ),
];

pub(super) const MIRROR_DOCS: &[(&str, &str)] = &[
    (
        "Runtime 02 plan",
        include_str!(
            "../../../../../docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md"
        ),
    ),
    (
        "runtime index",
        include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md"),
    ),
    (
        "root surface doc",
        include_str!("../../../../../docs/zircon_runtime/core/root_surface.md"),
    ),
    (
        "generated-code boundary",
        include_str!("../../../../../docs/engine-architecture/generated-code-boundary.md"),
    ),
    (
        "interface convergence",
        include_str!("../../../../../docs/engine-architecture/runtime-interface-convergence.md"),
    ),
    (
        "M0 review",
        include_str!("../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"),
    ),
];
