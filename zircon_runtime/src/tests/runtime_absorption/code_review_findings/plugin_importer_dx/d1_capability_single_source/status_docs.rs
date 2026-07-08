use super::support::assert_contains_all;

pub(super) fn assert_d1_status_docs_are_synced() {
    let review_findings =
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention =
        include_str!("../../../../../../../docs/plans/engine-code-structure-convention.md");
    let plugins_12 = include_str!(
        "../../../../../../../docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md"
    );
    let plugin_sdk_doc = include_str!("../../../../../../../docs/zircon_plugins/plugin-sdk.md");
    let plugin_audit_doc =
        include_str!("../../../../../../../docs/zircon_plugins/plugin_structure_audits.md");
    let runtime_15 = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let frameworks_02 = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md"
    );
    let module_convention =
        include_str!("../../../../../../../docs/zircon_runtime/structure/module-convention.md");
    let session_note = include_str!(
        "../../../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
    );

    let d1_row = review_findings
        .lines()
        .find(|line| line.starts_with("| D1 |"))
        .expect("engine code review findings should keep a D1 row");
    assert_contains_all(
        "D1 review finding row is closed by capability single-source and SDK builder evidence",
        d1_row,
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

    for (doc_name, doc) in [
        ("engine-code-review-findings", review_findings),
        ("engine-code-structure-convention", structure_convention),
        ("plugins 12 plan", plugins_12),
        ("plugin SDK docs", plugin_sdk_doc),
        ("plugin structure audit docs", plugin_audit_doc),
        ("runtime 15 plan", runtime_15),
        ("runtime index", runtime_index),
        ("runtime module convention docs", module_convention),
        ("coordination session note", session_note),
    ] {
        assert_contains_all(
            doc_name,
            doc,
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
    }

    for (doc_name, doc) in [
        ("runtime 15 plan", runtime_15),
        ("runtime index", runtime_index),
        ("frameworks 02", frameworks_02),
        ("engine-code-structure-convention", structure_convention),
        ("engine-code-review-findings", review_findings),
        ("module convention", module_convention),
        ("coordination session note", session_note),
    ] {
        assert_contains_all(
            doc_name,
            doc,
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
}
