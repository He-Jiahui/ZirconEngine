use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 05 recent-static Runtime 02/07 status metadata guard",
        &[
            "runtime_recent_static_guard_anchors_stay_recorded_across_plan_docs",
            "runtime_02_generated_status_guard_present = true",
            "runtime_07_owner_budget_status_guard_present = true",
            "standalone recent_static 1/1",
        ],
    ),
    (
        "Runtime 05 status-output recent-static metadata row",
        &[
            "Runtime 05 recent-static Runtime 02/07 status metadata guard",
            "runtime_recent_static_guard_anchors_stay_recorded_across_plan_docs",
            "runtime_02_generated_status_guard_present = true",
            "standalone recent_static 1/1",
        ],
    ),
    (
        "Runtime 05 non-network server UI sortingMode allowlist",
        &[
            "sortingMode = \"server\"",
            "allowed_context_count 99",
            "unclassified_location_count 0",
            "aggregate `audit_runtime_structure.py --json` non-network assertions",
        ],
    ),
    (
        "Runtime 05 status-output non-network server allowlist row",
        &[
            "Runtime 05 non-network server UI sortingMode allowlist",
            "sortingMode = \"server\"",
            "allowed_context_count 99",
            "unclassified_location_count 0",
        ],
    ),
    (
        "Runtime 05 naming_boundary non-network server Rust guard",
        &[
            "runtime_non_network_server_naming_is_classified_by_owner",
            "runtime-ui-component-catalog-editor-controls",
            "standalone naming_boundary 2/2",
            "sortingMode = \"server\"",
        ],
    ),
    (
        "Runtime 05 non-network server Markdown renderer split",
        &[
            "non_network_server_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "non_network_server_naming_markdown.py",
            "allowed_context_count 94",
            "observer_false_positive_count 95",
        ],
    ),
    (
        "Runtime 05 runtime naming Markdown renderer split",
        &[
            "runtime_naming_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "runtime_naming_markdown.py",
            "editor_unclassified = 0",
            "legacy_unclassified = 0",
        ],
    ),
    (
        "Runtime 05 texture importer DDS caps policy wording",
        &[
            "DDSCAPS2_CUBEMAP caps2 policy",
            "legacy_reference_count = 148",
            "hard_cutover_migration_debt_count = 5",
            "DDS debt bucket absent",
        ],
    ),
    (
        "Runtime M0 entry static dependencies Markdown renderer split",
        &[
            "entry_static_dependencies_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "entry_static_dependencies_markdown.py",
            "optional runtime plugin path dependency count 0",
            "standalone `plan_status.rs` 33/33",
        ],
    ),
    (
        "Runtime M0 legacy standalone references Markdown renderer split",
        &[
            "legacy_standalone_references_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "legacy_standalone_references_markdown.py",
            "reference_count=0",
            "standalone `plan_status.rs` 33/33",
        ],
    ),
    (
        "Runtime M0 module inventory Markdown renderer split",
        &[
            "module_inventory_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "module_inventory_markdown.py",
            "module_crates=3",
            "standalone `plan_status.rs` 33/33",
        ],
    ),
    (
        "Runtime M0 plugin runtime gaps Markdown renderer split",
        &[
            "plugin_runtime_gaps_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "plugin_runtime_gaps_markdown.py",
            "plugin_gap_count=0",
            "standalone `plan_status.rs` 33/33",
        ],
    ),
    (
        "Runtime M0 large-file ownership Markdown renderer split",
        &[
            "large_file_ownership_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "large_file_ownership_markdown.py",
            "unclassified_hotspot_count=0",
            "standalone `plan_status.rs` 33/33",
        ],
    ),
    (
        "Runtime 06 native plugin public-surface Markdown renderer split",
        &[
            "native_plugin_public_surface_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "native_plugin_public_surface_markdown.py",
            "native_namespace_reexport_count = 64",
            "standalone `plan_status.rs` 33/33",
        ],
    ),
    (
        "Runtime 05 hard-cutover migration-smell Markdown renderer split",
        &[
            "hard_cutover_migration_markdown_split_static_passed_cargo_deferred_tests_deferred",
            "hard_cutover_migration_smells_markdown.py",
            "legacy-runtime-scene-document-debt",
            "unclassified_location_count 0",
        ],
    ),
];
