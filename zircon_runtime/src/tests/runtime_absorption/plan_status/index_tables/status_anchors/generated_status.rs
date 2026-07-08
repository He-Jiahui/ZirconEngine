use super::super::super::support::assert_contains_all;

#[test]
fn runtime_15_runtime_02_generated_status_index_anchors_are_locked() {
    let index_source =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let runtime_15_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let structure_convention =
        include_str!("../../../../../../../docs/plans/engine-code-structure-convention.md");
    let review_findings =
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let module_convention =
        include_str!("../../../../../../../docs/zircon_runtime/structure/module-convention.md");
    let session_note = include_str!(
        "../../../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
    );
    let output_anchors = include_str!(
        "../../../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_plan_status_output_anchors.py"
    );
    let runtime_02_status_row_data = include_str!(
        "../../status_output_tables/expected_status_row_data/runtime_01_04/runtime_02.rs"
    );
    let runtime_15_status_row_data = include_str!(
        "../../status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors/runtime_status_anchors.rs"
    );
    let status_map = [
        include_str!(
            "../../status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps.rs"
        ),
        include_str!(
            "../../status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps/runtime_status_anchor_maps.rs"
        ),
    ]
    .join("\n");
    let date_map = [
        include_str!(
            "../../status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps.rs"
        ),
        include_str!(
            "../../status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps/runtime_status_anchor_maps.rs"
        ),
    ]
    .join("\n");

    let runtime_02_generated_index_anchors = [
        "Runtime 02 generated template count 审计同步",
        "`template_file_count=10`",
        "generated export templates 10/10",
        "0 migration debt",
        "stale 9/9 scan",
        "Runtime 02 generated/export/app/editor/plugin Cargo gates",
    ];
    assert_contains_all(
        "runtime plan-status output anchor inventory",
        output_anchors,
        &runtime_02_generated_index_anchors,
    );
    assert_contains_all(
        "runtime index",
        index_source,
        &runtime_02_generated_index_anchors,
    );

    let runtime_02_generated_row = runtime_02_status_row_data
        .split_once("\"Runtime 02 generated template count 审计同步\"")
        .expect("Runtime 02 row-data should keep the generated template count row")
        .1
        .split_once("),")
        .expect("Runtime 02 generated template count row should end as a tuple")
        .0;
    assert_contains_all(
        "Runtime 02 generated current row data",
        runtime_02_generated_row,
        &runtime_02_generated_index_anchors[1..],
    );

    let status_anchors = [
        "Runtime 15 M3 Runtime 02 generated status anchor sync",
        "runtime_15_runtime_02_generated_status_anchor_sync_static_passed_cargo_deferred",
        "runtime_15_runtime_02_generated_status_index_anchors_are_locked",
    ];
    for (label, source) in [
        ("Runtime 15 subplan", runtime_15_plan),
        ("runtime index", index_source),
        ("engine code structure convention", structure_convention),
        ("engine code review findings", review_findings),
        ("module convention doc", module_convention),
        ("runtime implementation session note", session_note),
        ("Runtime 15 status row data", runtime_15_status_row_data),
        ("Runtime 15 expected status map", status_map.as_str()),
        ("Runtime 15 expected date map", date_map.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "Runtime 15 status row data",
        runtime_15_status_row_data,
        &runtime_02_generated_index_anchors,
    );
}
