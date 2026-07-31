use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::core::context::EditorContextBuilder;
use crate::core::editor_message::{
    EditorMessagePayload, EditorTopic, SharedEditorMessageBus, TOPIC_JOB,
};

use super::super::{
    test_job_scheduler, test_job_system_with_bus, EditorJob, EditorJobLimits, EditorJobSpec,
    JobCategory, JobContext, JobError, JobEventKind, JobEventPumpBudget,
};

const COMPLETE_TEST_PUMP_BUDGET: JobEventPumpBudget =
    JobEventPumpBudget::new(usize::MAX, Duration::from_secs(1));

#[test]
fn worker_events_enter_the_editor_bus_only_when_the_main_thread_pumps() {
    let bus = SharedEditorMessageBus::default();
    let topic = EditorTopic::parse(TOPIC_JOB).unwrap();
    let subscriber = bus.register_subscriber([topic]).unwrap();
    let jobs = test_job_system_with_bus(bus.clone(), EditorJobLimits::default());

    jobs.submit(
        EditorJobSpec::new("progress", JobCategory::Index),
        ProgressJob,
    )
    .unwrap()
    .wait()
    .unwrap();
    assert!(bus.deliveries_for(subscriber).is_empty());

    assert_eq!(jobs.pump_events_with_budget(COMPLETE_TEST_PUMP_BUDGET), 3);
    let deliveries = bus.drain_deliveries(subscriber);
    let kinds = deliveries
        .iter()
        .map(|delivery| match delivery.message().payload() {
            EditorMessagePayload::Job(event) => event.kind().clone(),
            payload => panic!("unexpected payload: {payload:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        vec![
            JobEventKind::Started,
            JobEventKind::Progress {
                completed: 1,
                total: 2,
                message: "half".to_string(),
            },
            JobEventKind::Completed,
        ]
    );
}

#[test]
fn editor_context_jobs_publish_to_the_context_bus() {
    let context = EditorContextBuilder::new(test_job_scheduler()).build();
    let topic = EditorTopic::parse(TOPIC_JOB).unwrap();
    let subscriber = context.bus().register_subscriber([topic]).unwrap();

    context
        .jobs()
        .submit(EditorJobSpec::new("context", JobCategory::Misc), NoopJob)
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(
        context
            .jobs()
            .pump_events_with_budget(COMPLETE_TEST_PUMP_BUDGET),
        2
    );
    assert_eq!(context.bus().drain_deliveries(subscriber).len(), 2);
}

#[test]
fn count_and_time_budgets_defer_edges_without_losing_them() {
    let bus = SharedEditorMessageBus::default();
    let topic = EditorTopic::parse(TOPIC_JOB).unwrap();
    let subscriber = bus.register_subscriber([topic]).unwrap();
    let jobs = test_job_system_with_bus(bus.clone(), EditorJobLimits::default());

    let tickets = (0..4)
        .map(|index| {
            jobs.submit(
                EditorJobSpec::new(format!("edge-{index}"), JobCategory::Misc),
                NoopJob,
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    for ticket in tickets {
        ticket.wait().unwrap();
    }

    assert_eq!(
        jobs.pump_events_with_budget(JobEventPumpBudget::new(8, Duration::ZERO)),
        0
    );
    assert!(bus.deliveries_for(subscriber).is_empty());

    let mut published = 0;
    while published < 8 {
        let pumped =
            jobs.pump_events_with_budget(JobEventPumpBudget::new(3, Duration::from_secs(1)));
        assert!((1..=3).contains(&pumped));
        published += pumped;
    }
    assert_eq!(published, 8);

    let deliveries = bus.drain_deliveries(subscriber);
    for index in 0..4 {
        let label = format!("edge-{index}");
        let kinds = deliveries
            .iter()
            .filter_map(|delivery| match delivery.message().payload() {
                EditorMessagePayload::Job(event) if event.label() == label => {
                    Some(event.kind().clone())
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(kinds, vec![JobEventKind::Started, JobEventKind::Completed]);
    }
}

#[test]
fn positive_time_budget_stops_when_the_injected_elapsed_clock_expires() {
    let bus = SharedEditorMessageBus::default();
    let topic = EditorTopic::parse(TOPIC_JOB).unwrap();
    let subscriber = bus.register_subscriber([topic]).unwrap();
    let jobs = test_job_system_with_bus(bus.clone(), EditorJobLimits::default());
    let tickets = (0..4)
        .map(|index| {
            jobs.submit(
                EditorJobSpec::new(format!("timed-edge-{index}"), JobCategory::Misc),
                NoopJob,
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    for ticket in tickets {
        ticket.wait().unwrap();
    }

    let mut clock_reads = 0;
    let pumped =
        jobs.pump_events_with_elapsed(JobEventPumpBudget::new(8, Duration::from_millis(1)), || {
            clock_reads += 1;
            if clock_reads <= 2 {
                Duration::ZERO
            } else {
                Duration::from_millis(2)
            }
        });

    assert_eq!(pumped, 2);
    assert_eq!(bus.deliveries_for(subscriber).len(), 2);
    assert_eq!(jobs.pump_events_with_budget(COMPLETE_TEST_PUMP_BUDGET), 6);
}

#[test]
fn progress_burst_coalesces_to_latest_value_between_lifecycle_edges() {
    let bus = SharedEditorMessageBus::default();
    let topic = EditorTopic::parse(TOPIC_JOB).unwrap();
    let subscriber = bus.register_subscriber([topic]).unwrap();
    let jobs = test_job_system_with_bus(bus.clone(), EditorJobLimits::default());

    jobs.submit(
        EditorJobSpec::new("coalesced", JobCategory::Index),
        ProgressBurstJob,
    )
    .unwrap()
    .wait()
    .unwrap();

    assert_eq!(jobs.pump_events_with_budget(COMPLETE_TEST_PUMP_BUDGET), 3);
    let kinds = bus
        .drain_deliveries(subscriber)
        .into_iter()
        .map(|delivery| match delivery.message().payload() {
            EditorMessagePayload::Job(event) => event.kind().clone(),
            payload => panic!("unexpected payload: {payload:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        vec![
            JobEventKind::Started,
            JobEventKind::Progress {
                completed: 100,
                total: 100,
                message: "step-100".to_string(),
            },
            JobEventKind::Completed,
        ]
    );
}

#[test]
fn escaped_job_context_cannot_publish_progress_after_terminal() {
    let bus = SharedEditorMessageBus::default();
    let topic = EditorTopic::parse(TOPIC_JOB).unwrap();
    let subscriber = bus.register_subscriber([topic]).unwrap();
    let jobs = test_job_system_with_bus(bus.clone(), EditorJobLimits::default());

    let escaped = jobs
        .submit(
            EditorJobSpec::new("escaped-context", JobCategory::Misc),
            EscapingContextJob,
        )
        .unwrap()
        .wait()
        .unwrap();
    escaped.report_progress(1, 1, "too late");

    assert_eq!(jobs.pump_events_with_budget(COMPLETE_TEST_PUMP_BUDGET), 2);
    let kinds = bus
        .drain_deliveries(subscriber)
        .into_iter()
        .map(|delivery| match delivery.message().payload() {
            EditorMessagePayload::Job(event) => event.kind().clone(),
            payload => panic!("unexpected payload: {payload:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(kinds, vec![JobEventKind::Started, JobEventKind::Completed]);
}

#[test]
fn concurrent_pump_callers_preserve_each_job_lifecycle_order() {
    let bus = SharedEditorMessageBus::default();
    let topic = EditorTopic::parse(TOPIC_JOB).unwrap();
    let subscriber = bus.register_subscriber([topic]).unwrap();
    let jobs = test_job_system_with_bus(bus.clone(), EditorJobLimits::default());
    let tickets = (0..64)
        .map(|index| {
            jobs.submit(
                EditorJobSpec::new(format!("parallel-pump-{index}"), JobCategory::Misc),
                NoopJob,
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    for ticket in tickets {
        ticket.wait().unwrap();
    }

    let jobs = Arc::new(jobs);
    let pumpers = (0..8)
        .map(|_| {
            let jobs = Arc::clone(&jobs);
            thread::spawn(move || loop {
                if jobs.pump_events_with_budget(JobEventPumpBudget::new(1, Duration::from_secs(1)))
                    == 0
                {
                    break;
                }
            })
        })
        .collect::<Vec<_>>();
    for pumper in pumpers {
        pumper.join().unwrap();
    }

    let deliveries = bus.drain_deliveries(subscriber);
    for index in 0..64 {
        let label = format!("parallel-pump-{index}");
        let kinds = deliveries
            .iter()
            .filter_map(|delivery| match delivery.message().payload() {
                EditorMessagePayload::Job(event) if event.label() == label => {
                    Some(event.kind().clone())
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(kinds, vec![JobEventKind::Started, JobEventKind::Completed]);
    }
}

struct ProgressJob;

impl EditorJob for ProgressJob {
    type Output = ();

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        context.report_progress(1, 2, "half");
        Ok(())
    }
}

struct NoopJob;

impl EditorJob for NoopJob {
    type Output = ();

    fn run(self, _context: JobContext) -> Result<Self::Output, JobError> {
        Ok(())
    }
}

struct ProgressBurstJob;

impl EditorJob for ProgressBurstJob {
    type Output = ();

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        for completed in 1..=100 {
            context.report_progress(completed, 100, format!("step-{completed}"));
        }
        Ok(())
    }
}

struct EscapingContextJob;

impl EditorJob for EscapingContextJob {
    type Output = JobContext;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        Ok(context)
    }
}
