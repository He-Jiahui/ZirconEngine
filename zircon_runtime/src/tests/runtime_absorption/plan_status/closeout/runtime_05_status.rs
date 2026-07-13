use super::super::support::{frontmatter_status, runtime_plan_source_with_archive};

#[test]
fn runtime_05_closeout_status_records_completed_scene_cargo_gate() {
    let source = runtime_plan_source_with_archive(
        "05",
        include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md"
    ),
    );
    let source = source.as_str();

    assert_eq!(
        frontmatter_status(source),
        Some("completed"),
        "Runtime 05 should be completed after the final full-scene acceptance"
    );
    for required_anchor in [
        "runtime_05_scene_1642_structure_1304_review_298_pmrem_parity_passed_closeout_acceptance_complete",
        "1642 passed / 0 failed / 5 ignored",
        "structure_convention`（1304/1304）",
        "code_review_findings`（298/298）",
        "PMREM CPU/GPU parity（1/1",
    ] {
        assert!(
            source.contains(required_anchor),
            "Runtime 05 closeout plan should record `{required_anchor}`"
        );
    }
}
