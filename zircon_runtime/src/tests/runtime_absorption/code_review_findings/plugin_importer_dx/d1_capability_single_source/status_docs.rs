use super::support::assert_contains_all;

pub(super) fn assert_d1_status_docs_are_synced() {
    let review_findings = concat!(
        include_str!("../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"),
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md")
    );
    let d1_row = review_findings
        .lines()
        .find(|line| line.starts_with("| D1 |"))
        .expect("engine code review findings should keep a D1 row");
    assert_contains_all(
        "D1 review finding row is closed by capability single-source and SDK builder evidence",
        review_findings,
        &[
            "已闭合",
            "15 个 trait-backed first-party runtime roots",
            "plugins_12_runtime_capability_single_source_guard_passed",
            "plugins_12_capability_single_source_conformance",
            "m4_runtime_capability_gate_status = runtime-capability-single-source-clean",
            "capability_source_mismatches = 0",
            "m4_t2_builder_mirror_gate_status = sdk-builder-mirror-clean",
            "sdk_builder_mirror_violations = 0",
            "review_d1_plugin_capabilities_use_single_source_and_sdk_builder_mirror",
            "d1_capability_single_source_review_synced_static_passed_cargo_deferred",
        ],
    );
    assert!(
        !d1_row.contains("改一个名动 6 处") && !d1_row.contains("重复 3 次"),
        "D1 row should no longer describe capability duplication as an open issue"
    );

    assert_contains_all(
        "D1 numbered review output",
        review_findings,
        &[
            "D1 capability single-source review/status sync",
            "d1_capability_single_source_review_synced_static_passed_cargo_deferred",
            "review_d1_plugin_capabilities_use_single_source_and_sdk_builder_mirror",
            "plugins_12_runtime_capability_single_source_guard_passed",
            "plugins_12_capability_single_source_conformance",
            "m4_runtime_capability_gate_status = runtime-capability-single-source-clean",
            "capability_source_mismatches = 0",
            "m4_t2_builder_mirror_gate_status = sdk-builder-mirror-clean",
            "sdk_builder_mirror_violations = 0",
        ],
    );

    assert_contains_all(
        "D1 numbered folder-backed output",
        review_findings,
        &[
            super::D1_FOLDER_BACKED_SLICE,
            super::D1_FOLDER_BACKED_STATUS,
            super::D1_FRAMEWORKS_STATUS,
            super::D1_FOLDER_BACKED_GUARD,
            "code_review_findings/plugin_importer_dx/d1_capability_single_source.rs",
            "code_review_findings/plugin_importer_dx/d1_capability_single_source/runtime_roots.rs",
            "code_review_findings/plugin_importer_dx/d1_capability_single_source/split_layout.rs",
        ],
    );
}
