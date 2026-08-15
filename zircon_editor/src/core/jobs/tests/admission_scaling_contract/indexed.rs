use super::support::{GateJob, ValueJob};
use super::{
    EditorJobLimits, EditorJobSpec, JobCategory, JobPriority, test_job_system_with_limits,
};

const MAX_BUCKET_PROBES_PER_PASS: usize = 6 * JobCategory::ALL.len();

#[test]
fn indexed_pending_admission_scales_linearly_through_enqueue_and_completion() {
    let small = admission_probe_sample(1_000);
    let large = admission_probe_sample(10_000);

    assert!(
        small <= (1_000 * 2 + 2) * MAX_BUCKET_PROBES_PER_PASS,
        "unexpected 1k enqueue+completion probe count: {small}"
    );
    assert!(
        large <= (10_000 * 2 + 2) * MAX_BUCKET_PROBES_PER_PASS,
        "unexpected 10k enqueue+completion probe count: {large}"
    );
    assert!(
        large <= small.saturating_mul(11),
        "10k enqueue+completion promotion grew faster than linear: 1k={small}, 10k={large}"
    );
}

#[test]
fn ready_bucket_selection_cannot_regress_to_a_linear_job_scan() {
    let source = include_str!("../system/pending.rs");
    let take_next = source
        .split("pub(super) fn take_next")
        .nth(1)
        .and_then(|source| {
            source
                .split("pub(super) fn mark_dependency_schedulable")
                .next()
        })
        .expect("take_next source section");

    assert!(source.contains("BTreeSet<JobId>"));
    assert!(take_next.contains("ids.first().copied()"));
    for retired_scan in [".iter()", ".position(", ".min_by_key(", ".remove(index)"] {
        assert!(
            !take_next.contains(retired_scan),
            "ready selection restored a linear pending scan: {retired_scan}"
        );
    }
}

#[test]
fn category_admission_projection_uses_maintained_indexes() {
    let source = include_str!("../system/pending.rs");
    let category_projection = source
        .split("if let Some(category) = category {")
        .nth(1)
        .and_then(|source| {
            source
                .split("return EditorJobAdmissionSnapshot::new(")
                .next()
        })
        .expect("category admission projection should remain available");

    assert!(category_projection.contains("pending_ids_by_category"));
    assert!(category_projection.contains("estimated_bytes_by_category"));
    for retired_scan in [".values()", ".filter(", "for pending"] {
        assert!(
            !category_projection.contains(retired_scan),
            "category admission projection restored a pending-wide scan: {retired_scan}"
        );
    }
}

#[test]
fn promotion_uses_a_bounded_dispatch_batch_without_holding_the_state_mutex_for_schedule() {
    let source = include_str!("../system/scheduling.rs");
    let promote = source
        .split("pub(super) fn promote(self: &Arc<Self>)")
        .nth(1)
        .and_then(|body| body.split("pub(super) fn finish").next())
        .expect("promotion body should remain available");

    assert!(promote.contains("MAX_PROMOTION_DISPATCH_BATCH"));
    assert!(promote.contains("let _promotion ="));
    let state_selection = promote
        .find("let dispatch = {")
        .expect("pending selection must be scoped under the state mutex");
    let schedule = promote
        .find("let handle = self.scheduler.schedule_after")
        .expect("selected work must schedule through the runtime");
    assert!(state_selection < schedule);
    assert!(
        promote[state_selection..schedule].contains("let (pending, dependencies) = dispatch"),
        "runtime scheduling must begin after the state-lock selection scope ends"
    );
}

#[test]
fn admission_bucket_inventory_has_one_enum_owned_source() {
    let pending_source = include_str!("../system/pending.rs");
    let category_source = include_str!("../category.rs");

    assert!(pending_source.contains("FAIR_ADMISSION_SLOTS"));
    assert!(pending_source.contains("JobCategory::ALL"));
    assert!(!pending_source.contains("const PRIORITIES"));
    assert!(!pending_source.contains("const CATEGORIES"));
    assert_eq!(category_source.matches("define_job_enum! {").count(), 2);
    assert!(category_source.contains("[Self; [$(stringify!($variant)),+].len()]"));
    assert!(category_source.contains("[$(Self::$variant),+]"));
    assert!(!category_source.contains("pub const ALL: [Self; 3]"));
    assert!(!category_source.contains("pub const ALL: [Self; 8]"));
    assert_eq!(JobPriority::ALL.len(), 3);
    assert_eq!(JobCategory::ALL.len(), 8);
}

fn admission_probe_sample(job_count: usize) -> usize {
    let jobs =
        test_job_system_with_limits(EditorJobLimits::default().with_limit(JobCategory::Export, 1));
    let (started_sender, started_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();
    let blocker = jobs
        .submit(
            EditorJobSpec::new("probe-blocker", JobCategory::Export),
            GateJob::new(started_sender, release_receiver),
        )
        .unwrap();
    started_receiver.recv().unwrap();
    let baseline = jobs.admission_probe_count();

    let tickets = (0..job_count)
        .map(|index| {
            jobs.submit(
                EditorJobSpec::new(format!("probe-{index}"), JobCategory::Export)
                    .with_priority(JobPriority::Background),
                ValueJob(index as u32),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();

    release_sender.send(()).unwrap();
    assert_eq!(blocker.wait(), Ok(()));
    for (index, ticket) in tickets.into_iter().enumerate() {
        assert_eq!(ticket.wait(), Ok(index as u32 + 1));
    }
    assert_eq!(jobs.pending_job_count(), 0);
    assert_eq!(jobs.running_job_count(), 0);

    jobs.admission_probe_count().saturating_sub(baseline)
}
