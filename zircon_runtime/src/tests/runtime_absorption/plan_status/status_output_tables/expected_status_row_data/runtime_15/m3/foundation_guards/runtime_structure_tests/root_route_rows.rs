type Slice = super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 root entries guard child-owner split",
        &[
            "runtime_15_root_entries_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/root_entries.rs",
            "tests/runtime_absorption/root_entries/core_spine.rs",
            "tests/runtime_absorption/root_entries/module_families.rs",
            "tests/runtime_absorption/root_entries/runtime_root.rs",
            "runtime_15_root_entries_guard_child_owners_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 root entries module-families guard folder-backed split",
        &[
            "runtime_15_root_entries_module_families_guard_folder_backed_static_passed_cargo_deferred",
            "tests/runtime_absorption/root_entries/module_families.rs",
            "tests/runtime_absorption/root_entries/module_families/navigation.rs",
            "tests/runtime_absorption/root_entries/module_families/animation_backlog.rs",
            "tests/runtime_absorption/root_entries/module_families/animation_status_json.rs",
            "tests/runtime_absorption/root_entries/module_families/root_seats.rs",
            "tests/runtime_absorption/root_entries/module_families/mirror_docs.rs",
            "tests/runtime_absorption/root_entries/module_families/split_layout.rs",
            "runtime_15_root_entries_module_families_guard_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 core spine root/generated route-owner split",
        &[
            "runtime_15_core_spine_root_generated_route_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/core_spine_root_generated.rs",
            "tests/runtime_absorption/core_spine_root_generated/inventory.rs",
            "tests/runtime_absorption/core_spine_root_generated/mirror_docs.rs",
            "tests/runtime_absorption/core_spine_root_generated/generated_behavior.rs",
            "tests/runtime_absorption/core_spine_root_generated/source_helpers.rs",
            "runtime_15_core_spine_root_generated_is_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 core spine root/generated audit source sync",
        &[
            "runtime_15_core_spine_root_generated_audit_source_sync_static_passed_cargo_deferred",
            ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/core_spine_root_generated_boundary.py",
            "tests/runtime_absorption/core_spine_root_generated/inventory.rs",
            "tests/runtime_absorption/core_spine_root_generated/mirror_docs.rs",
            "tests/runtime_absorption/root_surface/public_surface.rs",
            "tests/runtime_absorption/generated_code_guard/markers.rs",
            "runtime_02_core_spine_root_generated_mirror_docs_match_structure_audit_counts",
            "risks = []",
        ],
    ),
];
