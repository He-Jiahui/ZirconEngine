use super::{RuntimeSessionArchiveMergePlan, RuntimeSessionArchiveMergeReport};

fn merge_plan(report: RuntimeSessionArchiveMergeReport) -> RuntimeSessionArchiveMergePlan<'static> {
    RuntimeSessionArchiveMergePlan {
        target_generation: 7,
        target_revision: 11,
        report,
        replacements: Vec::new(),
        inserts: Vec::new(),
    }
}

#[test]
fn runtime52_batch_consuming_merge_plan_moves_report_without_cloning() {
    let plan = merge_plan(RuntimeSessionArchiveMergeReport {
        inserted_slot_ids: vec!["inserted".to_owned()],
        replaced_slot_ids: vec!["replaced".to_owned()],
        skipped_slot_ids: vec!["skipped".to_owned()],
    });
    let inserted_buffer = plan.report.inserted_slot_ids.as_ptr();
    let replaced_buffer = plan.report.replaced_slot_ids.as_ptr();
    let skipped_buffer = plan.report.skipped_slot_ids.as_ptr();

    let report = plan.into_report();

    assert_eq!(report.inserted_slot_ids.as_ptr(), inserted_buffer);
    assert_eq!(report.replaced_slot_ids.as_ptr(), replaced_buffer);
    assert_eq!(report.skipped_slot_ids.as_ptr(), skipped_buffer);
    assert_eq!(report.inserted_slot_ids, ["inserted"]);
    assert_eq!(report.replaced_slot_ids, ["replaced"]);
    assert_eq!(report.skipped_slot_ids, ["skipped"]);
}

#[test]
fn runtime52_batch_borrowed_merge_plan_report_remains_available() {
    let plan = merge_plan(RuntimeSessionArchiveMergeReport {
        inserted_slot_ids: vec!["slot-a".to_owned()],
        ..RuntimeSessionArchiveMergeReport::default()
    });

    assert_eq!(plan.report().inserted_slot_ids, ["slot-a"]);
    assert_eq!(plan.target_generation(), 7);
    assert_eq!(plan.target_revision(), 11);
}

#[test]
fn runtime52_batch_consuming_empty_merge_plan_preserves_empty_report() {
    let report = merge_plan(RuntimeSessionArchiveMergeReport::default()).into_report();

    assert!(report.is_empty());
}
