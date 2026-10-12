use super::protect_selected_slot;
use crate::scene::dynamic_scene::session::RuntimeSessionArchivePruneReport;

fn report() -> RuntimeSessionArchivePruneReport {
    RuntimeSessionArchivePruneReport {
        retained_slot_ids: ["autosave", "manual-new"].map(str::to_owned).into(),
        removed_slot_ids: ["manual-mid", "manual-old"].map(str::to_owned).into(),
    }
}

#[test]
fn runtime52_batch_incremental_selected_protection_preserves_canonical_order() {
    let report = protect_selected_slot(report(), "manual-mid");

    assert_eq!(
        report.retained_slot_ids,
        ["autosave", "manual-mid", "manual-new"]
    );
    assert_eq!(report.removed_slot_ids, ["manual-old"]);
}

#[test]
fn runtime52_batch_incremental_selected_protection_is_noop_when_not_removed() {
    let expected = report();

    assert_eq!(
        protect_selected_slot(expected.clone(), "autosave"),
        expected
    );
}

#[test]
fn runtime52_batch_incremental_selected_protection_keeps_partition_unique() {
    let mut report = report();
    report.retained_slot_ids.insert(1, "manual-mid".to_owned());

    let report = protect_selected_slot(report, "manual-mid");

    assert_eq!(
        report
            .retained_slot_ids
            .iter()
            .filter(|slot_id| slot_id.as_str() == "manual-mid")
            .count(),
        1
    );
    assert!(!report
        .removed_slot_ids
        .iter()
        .any(|slot_id| slot_id == "manual-mid"));
}
