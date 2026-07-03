use super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 code review findings status-doc guard child-owner split",
        &[
            "runtime_15_code_review_findings_status_docs_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs.rs",
            "runtime_15_code_review_findings_status_docs_are_child_owner",
            "runtime_15_code_review_findings_tests_are_folder_backed",
            "runtime_15_code_review_findings_structure_guard_children_are_mounted",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 code review findings status-doc guard folder-backed split",
        &[
            "runtime_15_code_review_findings_status_docs_folder_backed_static_passed_cargo_deferred",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/sync.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/source_anchor_guard.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchor_guard.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/delegation.rs",
            "runtime_15_code_review_findings_status_docs_are_child_owner",
            "runtime_15_code_review_findings_status_docs_source_anchors_are_child_owner",
            "runtime_15_code_review_findings_status_docs_status_anchors_are_child_owner",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 code review findings status-doc source anchors child-owner split",
        &[
            "runtime_15_code_review_findings_status_docs_source_anchors_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/source_anchors.rs",
            "runtime_15_code_review_findings_status_docs_source_anchors_are_child_owner",
            "runtime_15_code_review_findings_status_docs_are_child_owner",
            "runtime_15_code_review_findings_tests_are_folder_backed",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 code review findings status-doc source anchors folder-backed split",
        &[
            "runtime_15_code_review_findings_status_docs_source_anchors_folder_backed_static_passed_cargo_deferred",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/source_anchors.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/source_anchors/review_sources.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/source_anchors/native_typed_error.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/source_anchors/runtime_surface.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/source_anchors/structure_owners.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/source_anchors/status_mirrors.rs",
            "runtime_15_code_review_findings_status_docs_source_anchors_are_child_owner",
            "runtime_15_code_review_findings_status_docs_source_anchors_folder_backed_status_is_current",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 code review findings status-doc status anchors child-owner split",
        &[
            "runtime_15_code_review_findings_status_docs_status_anchors_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchors.rs",
            "runtime_15_code_review_findings_status_docs_status_anchors_are_child_owner",
            "runtime_15_code_review_findings_status_docs_are_child_owner",
            "runtime_15_code_review_findings_tests_are_folder_backed",
            "Cargo gate deferred",
        ],
    ),
];
