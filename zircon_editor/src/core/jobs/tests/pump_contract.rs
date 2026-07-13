use crate::core::context::EditorContextBuilder;
use crate::core::editor_message::{
    EditorMessagePayload, EditorTopic, SharedEditorMessageBus, TOPIC_JOB,
};

use super::super::{
    test_job_scheduler, test_job_system_with_bus, EditorJob, EditorJobLimits, EditorJobSpec,
    JobCategory, JobContext, JobError, JobEventKind,
};

#[test]
fn worker_events_enter_the_editor_bus_only_when_the_main_thread_pumps() {
    let bus = SharedEditorMessageBus::default();
    let topic = EditorTopic::parse(TOPIC_JOB).unwrap();
    let subscriber = bus.register_subscriber([topic]);
    let jobs = test_job_system_with_bus(bus.clone(), EditorJobLimits::default());

    jobs.submit(
        EditorJobSpec::new("progress", JobCategory::Index),
        ProgressJob,
    )
    .unwrap()
    .wait()
    .unwrap();
    assert!(bus.deliveries_for(subscriber).is_empty());

    assert_eq!(jobs.pump_events(), 3);
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
    let subscriber = context.bus().register_subscriber([topic]);

    context
        .jobs()
        .submit(EditorJobSpec::new("context", JobCategory::Misc), NoopJob)
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(context.jobs().pump_events(), 2);
    assert_eq!(context.bus().drain_deliveries(subscriber).len(), 2);
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
