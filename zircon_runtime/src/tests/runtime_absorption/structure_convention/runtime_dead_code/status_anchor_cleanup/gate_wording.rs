use super::*;

#[test]
fn runtime_15_runtime_dead_code_current_rows_keep_module_gate_audit_clear() {
    let runtime_15_plan = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                MODULE_GATE_WORDING_SLICE,
                MODULE_GATE_WORDING_STATUS,
                MODULE_GATE_WORDING_GUARD,
                MODULE_GATE_AUDIT_CLEAR,
                "full Cargo sweep",
            ],
        );
    }

    for (label, source, slice) in [
        (
            "Runtime 15 plan child-owner row",
            runtime_15_plan.as_str(),
            CHILD_OWNER_SLICE,
        ),
        (
            "Runtime 15 plan documentation row",
            runtime_15_plan.as_str(),
            SLICE,
        ),
        (
            "review findings child-owner row",
            review_findings.as_str(),
            CHILD_OWNER_SLICE,
        ),
        (
            "review findings documentation row",
            review_findings.as_str(),
            SLICE,
        ),
        (
            "module convention child-owner section",
            module_doc.as_str(),
            CHILD_OWNER_SLICE,
        ),
        (
            "module convention documentation section",
            module_doc.as_str(),
            SLICE,
        ),
    ] {
        let row = slice_entry(source, slice)
            .unwrap_or_else(|| panic!("{label} should contain slice `{slice}`"));
        assert!(
            row.contains(MODULE_GATE_AUDIT_CLEAR),
            "{label} should preserve the audit-clear module gate wording"
        );
        assert!(
            !row.contains(STALE_MODULE_GATE_PENDING),
            "{label} should not reopen module_convention_gate as pending after audit clear"
        );
    }
}

#[test]
fn runtime_15_runtime_dead_code_current_rows_use_production_gate_name() {
    let runtime_15_plan = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                PRODUCTION_GATE_WORDING_SLICE,
                PRODUCTION_GATE_WORDING_STATUS,
                PRODUCTION_GATE_WORDING_GUARD,
                CURRENT_PRODUCTION_GATE,
                "Runtime 15 M5 production dead-code suppression global gate",
            ],
        );
    }

    for (label, source, slice) in [
        (
            "Runtime 15 plan child-owner row",
            runtime_15_plan.as_str(),
            CHILD_OWNER_SLICE,
        ),
        (
            "Runtime 15 plan documentation row",
            runtime_15_plan.as_str(),
            SLICE,
        ),
        (
            "Runtime 15 plan module-gate wording row",
            runtime_15_plan.as_str(),
            MODULE_GATE_WORDING_SLICE,
        ),
        (
            "review findings child-owner row",
            review_findings.as_str(),
            CHILD_OWNER_SLICE,
        ),
        (
            "review findings documentation row",
            review_findings.as_str(),
            SLICE,
        ),
        (
            "review findings module-gate wording row",
            review_findings.as_str(),
            MODULE_GATE_WORDING_SLICE,
        ),
        (
            "module convention child-owner section",
            module_doc.as_str(),
            CHILD_OWNER_SLICE,
        ),
        (
            "module convention documentation section",
            module_doc.as_str(),
            SLICE,
        ),
        (
            "module convention module-gate wording section",
            module_doc.as_str(),
            MODULE_GATE_WORDING_SLICE,
        ),
    ] {
        let row = slice_entry(source, slice)
            .unwrap_or_else(|| panic!("{label} should contain slice `{slice}`"));
        assert!(
            row.contains(CURRENT_PRODUCTION_GATE),
            "{label} should point to the current production dead-code gate"
        );
        assert!(
            !row.contains(STALE_PRODUCTION_GATE),
            "{label} should not keep the retired production dead-code gate name"
        );
    }
}
