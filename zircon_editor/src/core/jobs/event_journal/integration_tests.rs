use std::sync::Arc;
use std::time::{Duration, Instant};

use super::EditorJobEventJournal;
use crate::core::editor_message::{
    EditorMessageInboxLimits, EditorMessagePayload, EditorTopic, SharedEditorMessageBus, TOPIC_JOB,
};
use crate::core::jobs::{
    test_job_system_with_bus, EditorJob, EditorJobEventJournalLimits, EditorJobLimits,
    EditorJobSpec, JobCategory, JobContext, JobError, JobEvent, JobEventKind, JobEventPumpBudget,
    JobId,
};

const COMPLETE_TEST_PUMP_BUDGET: JobEventPumpBudget =
    JobEventPumpBudget::new(usize::MAX, Duration::from_secs(1));

#[test]
fn paused_consumer_keeps_the_job_journal_bounded_and_publishes_a_resync_gap() {
    const MAX_ENTRIES: usize = 3;
    const MAX_RETAINED_BYTES: usize = 4 * 1024;

    let bus = SharedEditorMessageBus::default();
    let topic = EditorTopic::parse(TOPIC_JOB).unwrap();
    let subscriber = bus.register_subscriber([topic]).unwrap();
    let limits = EditorJobLimits::default().with_event_journal_limits(
        EditorJobEventJournalLimits::new(MAX_ENTRIES, MAX_RETAINED_BYTES),
    );
    let jobs = test_job_system_with_bus(bus.clone(), limits);

    for index in 0..4 {
        jobs.submit(
            EditorJobSpec::new(format!("paused-{index}"), JobCategory::Misc),
            NoopJob,
        )
        .unwrap()
        .wait()
        .unwrap();
    }

    let paused = jobs.event_journal_snapshot();
    assert!(paused.depth() <= MAX_ENTRIES);
    assert!(paused.retained_bytes() <= MAX_RETAINED_BYTES);
    assert!(paused.high_water_depth() <= MAX_ENTRIES);
    assert!(paused.high_water_retained_bytes() <= MAX_RETAINED_BYTES);
    assert!(paused.dropped_lifecycle_events() > 0);

    let pumped = jobs.pump_events_with_budget(COMPLETE_TEST_PUMP_BUDGET);
    let deliveries = bus.drain_deliveries(subscriber);
    assert_eq!(deliveries.len(), pumped);
    let gaps = deliveries
        .iter()
        .filter_map(|delivery| match delivery.message().payload() {
            EditorMessagePayload::JobJournalGap(gap) => Some(gap),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(gaps.len(), 1);
    assert!(gaps[0].dropped_lifecycle_events() > 0);
    assert!(gaps[0].first_dropped_sequence() <= gaps[0].last_dropped_sequence());
    assert_eq!(jobs.event_journal_snapshot().depth(), 0);
}

#[test]
fn backpressured_job_delivery_returns_to_the_front_of_the_journal() {
    let bus = SharedEditorMessageBus::with_inbox_limits(EditorMessageInboxLimits::new(1, 1, 1));
    let topic = EditorTopic::parse(TOPIC_JOB).unwrap();
    let subscriber = bus.register_subscriber([topic]).unwrap();
    let jobs = test_job_system_with_bus(bus.clone(), EditorJobLimits::default());

    jobs.submit(EditorJobSpec::new("retry", JobCategory::Misc), NoopJob)
        .unwrap()
        .wait()
        .unwrap();

    assert_eq!(jobs.pump_events_with_budget(COMPLETE_TEST_PUMP_BUDGET), 1);
    assert_eq!(jobs.event_journal_snapshot().depth(), 1);
    assert_eq!(bus.drain_deliveries(subscriber).len(), 1);
    assert_eq!(jobs.pump_events_with_budget(COMPLETE_TEST_PUMP_BUDGET), 1);
    assert_eq!(jobs.event_journal_snapshot().depth(), 0);
    assert_eq!(bus.drain_deliveries(subscriber).len(), 1);
}

#[test]
fn oversized_lifecycle_events_collapse_to_a_gap_inside_the_byte_budget() {
    const MAX_RETAINED_BYTES: usize = 256;

    let bus = SharedEditorMessageBus::default();
    let topic = EditorTopic::parse(TOPIC_JOB).unwrap();
    let subscriber = bus.register_subscriber([topic]).unwrap();
    let limits = EditorJobLimits::default()
        .with_event_journal_limits(EditorJobEventJournalLimits::new(8, MAX_RETAINED_BYTES));
    let jobs = test_job_system_with_bus(bus.clone(), limits);

    jobs.submit(
        EditorJobSpec::new("x".repeat(4 * 1024), JobCategory::Misc),
        NoopJob,
    )
    .unwrap()
    .wait()
    .unwrap();

    let snapshot = jobs.event_journal_snapshot();
    assert_eq!(snapshot.depth(), 1);
    assert!(snapshot.retained_bytes() <= MAX_RETAINED_BYTES);
    assert_eq!(snapshot.dropped_lifecycle_events(), 2);
    assert_eq!(jobs.pump_events_with_budget(COMPLETE_TEST_PUMP_BUDGET), 1);
    let deliveries = bus.drain_deliveries(subscriber);
    assert!(matches!(
        deliveries[0].message().payload(),
        EditorMessagePayload::JobJournalGap(gap)
            if gap.dropped_lifecycle_events() == 2
    ));
}

#[test]
#[ignore = "managed Editor09 performance evidence"]
fn editor09_paused_consumer_journal_pressure_evidence() {
    const JOBS: usize = 16_384;
    const EVENTS_PER_JOB: usize = 2;

    let journal = EditorJobEventJournal::default();
    let started = Instant::now();
    for index in 1..=JOBS {
        let id = JobId::new(index as u64);
        let label = Arc::<str>::from(format!("paused-pressure-{index}"));
        journal.push(JobEvent::new(
            id,
            Arc::clone(&label),
            JobCategory::Misc,
            JobEventKind::Started,
        ));
        journal.push(JobEvent::new(
            id,
            label,
            JobCategory::Misc,
            JobEventKind::Completed,
        ));
    }
    let elapsed = started.elapsed();
    let snapshot = journal.snapshot();
    let produced_events = JOBS * EVENTS_PER_JOB;
    let retained_reduction_percent =
        100.0 * (produced_events - snapshot.depth()) as f64 / produced_events as f64;

    assert!(snapshot.depth() <= journal.limits().max_entries());
    assert!(snapshot.retained_bytes() <= journal.limits().max_retained_bytes());
    assert!(snapshot.dropped_lifecycle_events() > 0);
    println!(
        "EDITOR_JOB_BENCH_V1 kind=paused_consumer_journal jobs={} produced_events={} retained_entries={} retained_bytes={} dropped_lifecycle_events={} retained_reduction_percent={:.4} elapsed_ns={} throughput_events_per_second={:.2}",
        JOBS,
        produced_events,
        snapshot.depth(),
        snapshot.retained_bytes(),
        snapshot.dropped_lifecycle_events(),
        retained_reduction_percent,
        elapsed.as_nanos(),
        produced_events as f64 / elapsed.as_secs_f64(),
    );
}

struct NoopJob;

impl EditorJob for NoopJob {
    type Output = ();

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        Ok(())
    }
}
