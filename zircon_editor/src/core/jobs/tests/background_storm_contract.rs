use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::core::editor_message::{
    EditorMessagePayload, EditorTopic, SharedEditorMessageBus, TOPIC_JOB,
};

use super::super::{
    DEFAULT_JOB_EVENT_PUMP_BUDGET, EditorJob, EditorJobAdmissionLimits, EditorJobLimits,
    EditorJobSpec, JobCategory, JobContext, JobError, JobEventKind, JobId, JobPriority,
    JobSubmitError, test_job_system_with_bus, test_job_system_with_limits,
};

const THUMBNAIL_JOB_COUNT: usize = 1_000;
const THUMBNAIL_JOB_LIMIT: usize = 2;
const EVENTS_PER_JOB: usize = 3;
const STORM_WATCHDOG: Duration = Duration::from_secs(60);

#[test]
fn thumbnail_storm_preserves_quota_and_records_main_thread_pump_baseline() {
    let bus = SharedEditorMessageBus::default();
    let job_topic = EditorTopic::parse(TOPIC_JOB).expect("job topic must remain valid");
    let subscriber = bus.register_subscriber([job_topic]).unwrap();
    let limits = EditorJobLimits::default().with_limit(JobCategory::Thumbnail, THUMBNAIL_JOB_LIMIT);
    assert_eq!(limits.limit(JobCategory::Thumbnail), THUMBNAIL_JOB_LIMIT);
    let jobs = test_job_system_with_bus(bus.clone(), limits);

    let gate = Arc::new(StormGate::default());
    let active = Arc::new(AtomicUsize::new(0));
    let maximum_active = Arc::new(AtomicUsize::new(0));
    let completed = Arc::new(AtomicUsize::new(0));
    let mut tickets = Vec::with_capacity(THUMBNAIL_JOB_COUNT);
    let mut submit_samples_ns = Vec::with_capacity(THUMBNAIL_JOB_COUNT);

    let submit_total_started = Instant::now();
    for index in 0..THUMBNAIL_JOB_COUNT {
        let submit_started = Instant::now();
        let ticket = jobs
            .submit(
                EditorJobSpec::new(format!("thumbnail-storm-{index}"), JobCategory::Thumbnail)
                    .with_priority(JobPriority::Background),
                StormThumbnailJob {
                    gate: Arc::clone(&gate),
                    active: Arc::clone(&active),
                    maximum_active: Arc::clone(&maximum_active),
                    completed: Arc::clone(&completed),
                },
            )
            .expect("the open editor job system must accept every storm job");
        submit_samples_ns.push(submit_started.elapsed().as_nanos());
        tickets.push(ticket);
    }
    let submit_total_ns = submit_total_started.elapsed().as_nanos();

    // The closed gate makes this deterministic: two Thumbnail jobs are admitted and all
    // remaining work stays in the editor queue, independent of worker scheduling speed.
    assert_eq!(jobs.scheduled_record_count(), THUMBNAIL_JOB_LIMIT);
    assert_eq!(
        jobs.pending_job_count(),
        THUMBNAIL_JOB_COUNT - THUMBNAIL_JOB_LIMIT
    );

    gate.release();
    let release_started = Instant::now();
    let watchdog_deadline = release_started + STORM_WATCHDOG;
    let expected_event_count = THUMBNAIL_JOB_COUNT * EVENTS_PER_JOB;
    let mut pumped_total = 0usize;
    let mut delivered_total = 0usize;
    let mut tick_samples_ns = Vec::new();
    let mut idle_tick_samples_ns = Vec::new();
    let mut nonempty_tick_samples_ns = Vec::new();
    let mut nonempty_batch_samples = Vec::new();
    let mut events_by_job = BTreeMap::<JobId, StormJobEventCounts>::new();

    while !storm_is_settled(&jobs, completed.load(Ordering::SeqCst), pumped_total) {
        assert!(
            Instant::now() < watchdog_deadline,
            "thumbnail storm exceeded its liveness watchdog"
        );

        // This is the retained main-loop job phase only: the production tick owns the same
        // pump call. Full UI/GPU work is intentionally excluded from this scheduler baseline.
        let tick_started = Instant::now();
        let pumped = jobs.pump_events();
        assert!(pumped <= DEFAULT_JOB_EVENT_PUMP_BUDGET.max_events());
        let deliveries = bus.drain_deliveries(subscriber);
        let elapsed_ns = tick_started.elapsed().as_nanos();
        let delivered = deliveries.len();

        assert_eq!(delivered, pumped);
        record_storm_deliveries(&deliveries, &mut events_by_job);
        pumped_total += pumped;
        delivered_total += delivered;
        assert!(
            pumped_total <= expected_event_count,
            "thumbnail jobs emitted more events than their Started/Progress/Completed contract"
        );
        tick_samples_ns.push(elapsed_ns);
        if pumped == 0 {
            idle_tick_samples_ns.push(elapsed_ns);
        } else {
            nonempty_tick_samples_ns.push(elapsed_ns);
            nonempty_batch_samples.push(pumped as u128);
        }
        // A fixed diagnostic cadence bounds the 60-second watchdog to roughly 60k samples,
        // preventing sample-vector memory growth if the scheduler ever stops making progress.
        // The sleep remains outside the measured pump-and-drain interval above.
        thread::sleep(Duration::from_millis(1));
    }
    let release_elapsed_ns = release_started.elapsed().as_nanos();

    assert_eq!(completed.load(Ordering::SeqCst), THUMBNAIL_JOB_COUNT);
    assert_eq!(pumped_total, expected_event_count);
    assert_eq!(delivered_total, expected_event_count);
    assert_eq!(active.load(Ordering::SeqCst), 0);
    assert!((1..=THUMBNAIL_JOB_LIMIT).contains(&maximum_active.load(Ordering::SeqCst)));

    let ticket_ids = tickets
        .iter()
        .map(|ticket| ticket.id())
        .collect::<BTreeSet<_>>();
    let event_ids = events_by_job.keys().copied().collect::<BTreeSet<_>>();
    assert_eq!(events_by_job.len(), THUMBNAIL_JOB_COUNT);
    assert_eq!(event_ids, ticket_ids);
    for (id, counts) in &events_by_job {
        assert_eq!(
            counts,
            &StormJobEventCounts {
                started: 1,
                progress: 1,
                completed: 1,
            },
            "job {id:?} must emit one Started/Progress/Completed sequence"
        );
    }

    for ticket in &tickets {
        assert_eq!(ticket.try_take(), Some(Ok(())));
        assert_eq!(ticket.try_take(), None);
    }

    let final_pending = jobs.pending_job_count();
    let final_running = jobs.running_job_count();
    let final_scheduled = jobs.scheduled_record_count();
    let final_mutex_tails = jobs.mutex_group_tail_count();
    assert_eq!(final_pending, 0);
    assert_eq!(final_running, 0);
    assert_eq!(final_scheduled, 0);
    assert_eq!(final_mutex_tails, 0);

    let submit = SampleDistribution::from_samples(&submit_samples_ns);
    let ticks = SampleDistribution::from_samples(&tick_samples_ns);
    let idle_ticks = SampleDistribution::from_samples_or_zero(&idle_tick_samples_ns);
    let nonempty_ticks = SampleDistribution::from_samples(&nonempty_tick_samples_ns);
    let batches = SampleDistribution::from_samples(&nonempty_batch_samples);
    let max_active = maximum_active.load(Ordering::SeqCst);

    // Runtime 03 currently defines no numeric frame-time/P95 budget. These wall-clock values
    // are observations only. Runtime 07's two-run <20% rule is reserved for external baseline
    // repeatability during the Windows testing stage; it is not a pass/fail threshold here.
    eprintln!(
        "EDITOR_JOB_STORM_BASELINE job_count={} category=Thumbnail priority=Background limit={} \
         submit_total_ns={} submit_p50_ns={} submit_p95_ns={} submit_max_ns={} \
         ticks={} idle_ticks={} nonempty_ticks={} tick_p50_ns={} tick_p95_ns={} tick_max_ns={} \
         idle_tick_p50_ns={} idle_tick_p95_ns={} idle_tick_max_ns={} \
         nonempty_tick_p50_ns={} nonempty_tick_p95_ns={} nonempty_tick_max_ns={} \
         batch_p50={} batch_p95={} batch_max={} pumped_total={} delivered_total={} \
         event_job_count={} release_elapsed_ns={} max_active={} final_pending={} final_running={} \
         final_scheduled={} final_mutex_tails={} pump_count_budget={} pump_time_budget_us={}",
        THUMBNAIL_JOB_COUNT,
        THUMBNAIL_JOB_LIMIT,
        submit_total_ns,
        submit.p50,
        submit.p95,
        submit.max,
        tick_samples_ns.len(),
        idle_tick_samples_ns.len(),
        nonempty_tick_samples_ns.len(),
        ticks.p50,
        ticks.p95,
        ticks.max,
        idle_ticks.p50,
        idle_ticks.p95,
        idle_ticks.max,
        nonempty_ticks.p50,
        nonempty_ticks.p95,
        nonempty_ticks.max,
        batches.p50,
        batches.p95,
        batches.max,
        pumped_total,
        delivered_total,
        events_by_job.len(),
        release_elapsed_ns,
        max_active,
        final_pending,
        final_running,
        final_scheduled,
        final_mutex_tails,
        DEFAULT_JOB_EVENT_PUMP_BUDGET.max_events(),
        DEFAULT_JOB_EVENT_PUMP_BUDGET.max_elapsed().as_micros(),
    );
}

#[test]
fn thumbnail_storm_reports_backpressure_before_retaining_an_unbounded_ticket_set() {
    const MAX_PENDING_ENTRIES: usize = 8;
    let limits = EditorJobLimits::default()
        .with_limit(JobCategory::Thumbnail, THUMBNAIL_JOB_LIMIT)
        .with_admission_limits(EditorJobAdmissionLimits::new(
            MAX_PENDING_ENTRIES,
            1_024,
            Duration::from_secs(60),
        ));
    let jobs = test_job_system_with_limits(limits);
    let gate = Arc::new(StormGate::default());
    let active = Arc::new(AtomicUsize::new(0));
    let maximum_active = Arc::new(AtomicUsize::new(0));
    let completed = Arc::new(AtomicUsize::new(0));
    let mut accepted = Vec::new();
    let mut backpressured = 0;

    for index in 0..THUMBNAIL_JOB_COUNT {
        match jobs.submit(
            EditorJobSpec::new(
                format!("thumbnail-bounded-storm-{index}"),
                JobCategory::Thumbnail,
            )
            .with_priority(JobPriority::Background)
            .with_estimated_bytes(1),
            StormThumbnailJob {
                gate: Arc::clone(&gate),
                active: Arc::clone(&active),
                maximum_active: Arc::clone(&maximum_active),
                completed: Arc::clone(&completed),
            },
        ) {
            Ok(ticket) => accepted.push(ticket),
            Err(JobSubmitError::AdmissionEntryLimitExceeded {
                limit: MAX_PENDING_ENTRIES,
            }) => backpressured += 1,
            Err(error) => {
                panic!("thumbnail storm must fail only through entry backpressure: {error}")
            }
        }
    }

    assert!(
        backpressured > 0,
        "the storm must exercise bounded admission"
    );
    assert_eq!(accepted.len() + backpressured, THUMBNAIL_JOB_COUNT);
    assert!(accepted.len() <= THUMBNAIL_JOB_LIMIT + MAX_PENDING_ENTRIES);
    assert!(jobs.pending_job_count() <= MAX_PENDING_ENTRIES);
    assert!(jobs.scheduled_record_count() <= THUMBNAIL_JOB_LIMIT + MAX_PENDING_ENTRIES);
    let admission = jobs.admission_snapshot();
    assert!(admission.pending_entries() <= MAX_PENDING_ENTRIES);
    assert!(admission.pending_estimated_bytes() <= 1_024);
    assert!(
        admission
            .oldest_pending_age()
            .is_none_or(|age| age <= Duration::from_secs(60))
    );

    let accepted_count = accepted.len();
    gate.release();
    let completion_deadline = Instant::now() + STORM_WATCHDOG;
    for ticket in accepted {
        let result = loop {
            if let Some(result) = ticket.try_take() {
                break result;
            }
            assert!(
                Instant::now() < completion_deadline,
                "bounded thumbnail storm did not settle before its liveness deadline"
            );
            thread::sleep(Duration::from_millis(1));
        };
        assert_eq!(result, Ok(()));
    }
    assert_eq!(completed.load(Ordering::SeqCst), accepted_count);
    assert_eq!(jobs.pending_job_count(), 0);
    assert_eq!(jobs.running_job_count(), 0);
    assert_eq!(jobs.scheduled_record_count(), 0);
}

fn record_storm_deliveries(
    deliveries: &[crate::core::editor_message::EditorMessageDelivery],
    events_by_job: &mut BTreeMap<JobId, StormJobEventCounts>,
) {
    for delivery in deliveries {
        let EditorMessagePayload::Job(event) = delivery.message().payload() else {
            panic!("job subscriber received a non-job editor message");
        };
        assert_eq!(event.category(), JobCategory::Thumbnail);
        let counts = events_by_job.entry(event.id()).or_default();
        match event.kind() {
            JobEventKind::Started => {
                assert_eq!(
                    counts,
                    &StormJobEventCounts::default(),
                    "job {:?} emitted Started out of order",
                    event.id()
                );
                counts.started = 1;
            }
            JobEventKind::Progress {
                completed,
                total,
                message,
            } => {
                assert_eq!(
                    (counts.started, counts.progress, counts.completed),
                    (1, 0, 0)
                );
                assert_eq!((*completed, *total), (1, 1));
                assert_eq!(message.as_str(), "thumbnail ready");
                counts.progress = 1;
            }
            JobEventKind::Completed => {
                assert_eq!(
                    (counts.started, counts.progress, counts.completed),
                    (1, 1, 0)
                );
                counts.completed = 1;
            }
            JobEventKind::Failed { message } => {
                panic!("thumbnail storm job failed unexpectedly: {message}")
            }
            JobEventKind::Cancelled => panic!("thumbnail storm job was cancelled unexpectedly"),
        }
    }
}

#[test]
fn storm_percentiles_use_nearest_rank_ordering() {
    let distribution = SampleDistribution::from_samples(&[10, 90, 20, 80, 30, 70, 40, 60, 50]);
    assert_eq!(
        distribution,
        SampleDistribution {
            p50: 50,
            p95: 90,
            max: 90
        }
    );

    assert_eq!(
        SampleDistribution::from_samples_or_zero(&[]),
        SampleDistribution::default()
    );
}

fn storm_is_settled(
    jobs: &super::super::EditorJobSystem,
    completed: usize,
    pumped_total: usize,
) -> bool {
    completed == THUMBNAIL_JOB_COUNT
        && pumped_total == THUMBNAIL_JOB_COUNT * EVENTS_PER_JOB
        && jobs.pending_job_count() == 0
        && jobs.running_job_count() == 0
        && jobs.scheduled_record_count() == 0
        && jobs.mutex_group_tail_count() == 0
}

#[derive(Default)]
struct StormGate {
    released: Mutex<bool>,
    changed: Condvar,
}

impl StormGate {
    fn wait(&self) {
        let mut released = self
            .released
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while !*released {
            released = self
                .changed
                .wait(released)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    fn release(&self) {
        *self
            .released
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = true;
        self.changed.notify_all();
    }
}

struct StormThumbnailJob {
    gate: Arc<StormGate>,
    active: Arc<AtomicUsize>,
    maximum_active: Arc<AtomicUsize>,
    completed: Arc<AtomicUsize>,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct StormJobEventCounts {
    started: usize,
    progress: usize,
    completed: usize,
}

impl EditorJob for StormThumbnailJob {
    type Output = ();

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.maximum_active.fetch_max(active, Ordering::SeqCst);
        self.gate.wait();
        context.report_progress(1, 1, "thumbnail ready");
        self.completed.fetch_add(1, Ordering::SeqCst);
        self.active.fetch_sub(1, Ordering::SeqCst);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct SampleDistribution {
    p50: u128,
    p95: u128,
    max: u128,
}

impl SampleDistribution {
    fn from_samples(samples: &[u128]) -> Self {
        assert!(!samples.is_empty(), "a percentile sample must not be empty");
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        Self {
            p50: sorted[nearest_rank_index(sorted.len(), 50)],
            p95: sorted[nearest_rank_index(sorted.len(), 95)],
            max: *sorted.last().expect("non-empty sample has a maximum"),
        }
    }

    fn from_samples_or_zero(samples: &[u128]) -> Self {
        if samples.is_empty() {
            Self::default()
        } else {
            Self::from_samples(samples)
        }
    }
}

fn nearest_rank_index(sample_count: usize, percentile: usize) -> usize {
    assert!(
        sample_count > 0,
        "nearest rank requires at least one sample"
    );
    assert!((1..=100).contains(&percentile));
    let rank = sample_count.saturating_mul(percentile).div_ceil(100);
    rank.saturating_sub(1).min(sample_count - 1)
}
