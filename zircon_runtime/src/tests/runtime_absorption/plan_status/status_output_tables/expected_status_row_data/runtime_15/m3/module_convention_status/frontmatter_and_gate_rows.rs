type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 module convention module-doc frontmatter uniqueness guard",
        &[
            "runtime_15_module_convention_module_doc_frontmatter_uniqueness_static_passed_cargo_deferred",
            "docs/zircon_runtime/structure/module-convention.md",
            "structure_convention/module_convention_gate.rs",
            "runtime_15_module_convention_module_doc_frontmatter_has_unique_entries",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/module_convention_status/frontmatter_and_gate_rows.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs",
            "related_code duplicates 0",
            "implementation_files duplicates 0",
            "frontmatter duplicate count 0",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 module convention gate guard folder-backed split",
        &[
            "runtime_15_module_convention_gate_guard_folder_backed_static_passed_cargo_deferred",
            "structure_convention/module_convention_gate.rs",
            "structure_convention/module_convention_gate/helpers.rs",
            "structure_convention/module_convention_gate/module_doc_frontmatter.rs",
            "structure_convention/module_convention_gate/output_contract.rs",
            "structure_convention/module_convention_gate/debt_boundary.rs",
            "structure_convention/module_convention_gate/audit_status.rs",
            "structure_convention/module_convention_gate/split_layout.rs",
            "runtime_15_module_convention_gate_guard_is_folder_backed",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 module-convention guard source reconciliation",
        &[
            "runtime_15_module_convention_guard_source_reconciliation_static_passed_cargo_deferred",
            "structure_convention/module_convention_gate/module_doc_frontmatter.rs",
            "structure_convention/module_convention_gate/split_layout.rs",
            "structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_moved_rows/module_convention_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/module_convention_status/frontmatter_and_gate_rows.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/lock_poison_module_maps.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/lock_poison_module_maps.rs",
            "runtime_15_module_convention_module_doc_frontmatter_has_unique_entries",
            "runtime_15_module_convention_gate_guard_is_folder_backed",
            "runtime_15_m3_child_group_moved_module_convention_rows_are_child_owned",
            "module_convention --test-threads=1 passed 11/11",
            "runtime_15_m3_child_groups --test-threads=1 passed 23/23",
            "Cargo gate deferred",
        ],
    ),
];
