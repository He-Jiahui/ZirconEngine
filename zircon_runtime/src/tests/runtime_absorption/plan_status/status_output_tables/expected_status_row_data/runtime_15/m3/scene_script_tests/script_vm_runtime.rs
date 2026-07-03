type Slice = super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 script VM test folder split",
        &[
            "runtime_15_script_vm_tests_folder_split_static_passed_cargo_timeout_no_result",
            "script/vm/tests.rs",
            "script/vm/tests/host_exports.rs",
            "script/vm/tests/reflection_docs.rs",
            "runtime_15_script_vm_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 script VM primary guard child-owner split",
        &[
            "runtime_15_script_vm_primary_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests/primary.rs",
            "script/vm/tests.rs",
            "script/vm/tests/host_exports.rs",
            "script/vm/tests/reflection_docs.rs",
            "runtime_15_script_vm_tests_are_folder_backed",
            "runtime_15_script_vm_primary_guard_is_child_owner",
            "32 script VM tests",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 script VM hot-reload coordinator test folder split",
        &[
            "runtime_15_script_vm_hot_reload_coordinator_tests_folder_split_static_passed_cargo_deferred",
            "script/vm/runtime/hot_reload_coordinator.rs",
            "script/vm/runtime/hot_reload_coordinator/tests.rs",
            "runtime_15_script_vm_hot_reload_coordinator_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 script VM hot-reload guard child-owner split",
        &[
            "runtime_15_script_vm_hot_reload_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests/hot_reload.rs",
            "script/vm/runtime/hot_reload_coordinator.rs",
            "script/vm/runtime/hot_reload_coordinator/tests.rs",
            "runtime_15_script_vm_hot_reload_coordinator_tests_are_folder_backed",
            "runtime_15_script_vm_hot_reload_guard_is_child_owner",
            "5 hot-reload coordinator tests",
            "Cargo gate deferred",
        ],
    ),
];
