use std::sync::Arc;

use super::*;

#[test]
fn sink_reuses_the_spec_stable_label_allocation() {
    let spec = super::super::EditorJobSpec::new("stable-job-label", JobCategory::Index);
    let sink = JobEventSink::new(
        JobId::new(1),
        Arc::clone(&spec.label),
        JobCategory::Index,
        EditorJobEventJournal::default(),
        EditorJobProgressSource::default(),
    );

    assert!(Arc::ptr_eq(&spec.label, &sink.label));
}
