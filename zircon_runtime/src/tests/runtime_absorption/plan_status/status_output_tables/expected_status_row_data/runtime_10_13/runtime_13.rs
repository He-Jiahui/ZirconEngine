use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 13 Script binding 镜像文档守卫",
        [
            "runtime_13_script_binding_mirror_docs_match_structure_audit_counts",
            "expected_source_file_count = 18",
            "standalone rustc 2/2",
            "script Cargo filters pending",
        ],
    ),
    (
        "Runtime 13 Gameplay Host Owner Split",
        [
            "gameplay_host/{combat,components,input,lifecycle,navigation,script_bindings,transform,values}.rs",
            "runtime_13_gameplay_host_owner_split_keeps_domain_files",
            "script::vm -- --nocapture` 48/48 passed",
            "script Cargo filters pending",
        ],
    ),
    (
        "Runtime 13 Gameplay host predicate functions for real ZR VM",
        [
            "gameplay.entity_exists",
            "gameplay.script_number_at_most",
            "gameplay_host_script_property_match_and_heal_update_bindings",
            "host_function_registry_matches_documented_ledger",
        ],
    ),
    (
        "Runtime 13 Script binding current audit recheck",
        [
            "script_binding_current_audit_static_passed_cargo_pending",
            "source files 18/18",
            "standalone `script_binding.rs` 2/2",
            "broader `cargo test -p zircon_runtime --lib script --locked`",
        ],
    ),
];
