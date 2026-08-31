use std::sync::mpsc::{SyncSender, TrySendError};

use crate::core::jobs::{EditorJob, JobContext, JobError};
use zircon_runtime_interface::export::ExportStage;

use super::super::{
    run_export_wizard_job, EditorExportBuildError, ExportWizardCancelSignal,
    ExportWizardCommandRunner, ExportWizardJobEvent, ExportWizardJobEventKind,
    ExportWizardJobSnapshot, ExportWizardJobStatus, ExportWizardPipelinePlan,
};

const MAX_BUFFERED_STAGE_OUTPUT_EVENTS_PER_STAGE: usize = 16;
const EXPORT_STAGE_COUNT: usize = ExportStage::ALL.len();

type BufferedStageOutputCounts = [usize; EXPORT_STAGE_COUNT];

/// Adapts the export wizard runner to the shared editor job contract.
pub(super) struct ExportWizardEditorJob<R> {
    job_id: String,
    plan: ExportWizardPipelinePlan,
    runner: R,
    events: SyncSender<ExportWizardJobEvent>,
}

impl<R> ExportWizardEditorJob<R> {
    pub(super) fn new(
        job_id: String,
        plan: ExportWizardPipelinePlan,
        runner: R,
        events: SyncSender<ExportWizardJobEvent>,
    ) -> Self {
        Self {
            job_id,
            plan,
            runner,
            events,
        }
    }
}

impl<R> EditorJob for ExportWizardEditorJob<R>
where
    R: ExportWizardCommandRunner + Send + 'static,
{
    type Output = ExportWizardJobSnapshot;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        let Self {
            job_id,
            plan,
            mut runner,
            events,
        } = self;
        let cancel = ExportWizardJobCancelSignal { context: &context };
        let mut buffered_output_events = BufferedStageOutputCounts::default();
        let mut coalesced_output_events = 0_u64;
        let snapshot = run_export_wizard_job(job_id, &plan, &mut runner, &cancel, &mut |event| {
            report_event_progress(&context, &event);
            send_job_event(
                &events,
                event,
                &mut buffered_output_events,
                &mut coalesced_output_events,
            );
        });
        match snapshot.status {
            ExportWizardJobStatus::Finished => Ok(snapshot),
            ExportWizardJobStatus::Cancelled => Err(JobError::Cancelled),
            ExportWizardJobStatus::Failed => Err(snapshot
                .stages
                .iter()
                .rev()
                .find_map(|stage| stage.failure.clone())
                .map(JobError::Failed)
                .unwrap_or_else(|| {
                    let stage = snapshot
                        .stages
                        .last()
                        .expect("failed wizard without typed cause must retain its stage");
                    JobError::failed(EditorExportBuildError::WizardStageFailed {
                        stage: stage.stage,
                        exit_code: stage.exit_code,
                    })
                })),
            status => Err(JobError::failed(
                EditorExportBuildError::WizardNonTerminal {
                    job_id: snapshot.job_id.clone(),
                    status: non_terminal_status_name(status),
                },
            )),
        }
    }
}

fn send_job_event(
    events: &SyncSender<ExportWizardJobEvent>,
    mut event: ExportWizardJobEvent,
    buffered_output_events: &mut BufferedStageOutputCounts,
    coalesced_output_events: &mut u64,
) {
    event.coalesced_output_events = *coalesced_output_events;
    if event.kind != ExportWizardJobEventKind::StageOutput {
        let _ = events.send(event);
        return;
    }

    let stage = event
        .output_delta
        .as_ref()
        .expect("StageOutput events must carry one typed delta")
        .stage;
    let buffered_count = stage_output_event_count_mut(buffered_output_events, stage);
    if *buffered_count >= MAX_BUFFERED_STAGE_OUTPUT_EVENTS_PER_STAGE {
        *coalesced_output_events = coalesced_output_events.saturating_add(1);
        return;
    }

    match events.try_send(event) {
        Ok(()) => *buffered_count += 1,
        Err(TrySendError::Full(_)) => {
            *coalesced_output_events = coalesced_output_events.saturating_add(1);
        }
        Err(TrySendError::Disconnected(_)) => {}
    }
}

fn stage_output_event_count_mut(
    buffered_output_events: &mut BufferedStageOutputCounts,
    stage: ExportStage,
) -> &mut usize {
    &mut buffered_output_events[export_stage_index(stage)]
}

const fn export_stage_index(stage: ExportStage) -> usize {
    match stage {
        ExportStage::Validate => 0,
        ExportStage::SourceTemplate => 1,
        ExportStage::NativeDynamic => 2,
        ExportStage::CompileHost => 3,
        ExportStage::CookAssets => 4,
        ExportStage::Pack => 5,
        ExportStage::PlatformBundle => 6,
        ExportStage::Report => 7,
    }
}

fn non_terminal_status_name(status: ExportWizardJobStatus) -> &'static str {
    match status {
        ExportWizardJobStatus::Pending => "pending",
        ExportWizardJobStatus::Running => "running",
        ExportWizardJobStatus::Cancelling => "cancelling",
        ExportWizardJobStatus::Finished => "finished",
        ExportWizardJobStatus::Cancelled => "cancelled",
        ExportWizardJobStatus::Failed => "failed",
    }
}

struct ExportWizardJobCancelSignal<'a> {
    context: &'a JobContext,
}

impl ExportWizardCancelSignal for ExportWizardJobCancelSignal<'_> {
    fn is_cancel_requested(&self) -> bool {
        self.context.is_cancelled()
    }
}

fn report_event_progress(context: &JobContext, event: &ExportWizardJobEvent) {
    let completed = bounded_stage_count(event.snapshot.stages.len());
    let total = bounded_stage_count(event.snapshot.progress.snapshots().len());
    let message = match event.snapshot.current_stage {
        Some(stage) => format!("export wizard {:?}: {:?}", event.kind, stage),
        None => format!("export wizard {:?}", event.kind),
    };
    context.report_progress(completed, total, message);
}

fn bounded_stage_count(value: usize) -> u32 {
    match u32::try_from(value) {
        Ok(value) => value,
        Err(_) => u32::MAX,
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::{
        export_wizard_pipeline_plan, ExportWizardCommandOutputLine,
        ExportWizardCommandOutputStream, ExportWizardJobState, ExportWizardPipelineOptions,
        ExportWizardStageOutputDelta,
    };
    use super::*;

    #[test]
    fn optimization_batch_20260830er_stage_output_counts_use_fixed_slots() {
        let source = include_str!("job.rs");

        assert!(source.contains(concat!(
            "type BufferedStageOutputCounts",
            " = [usize; EXPORT_STAGE_COUNT];"
        )));
        assert!(!source.contains(concat!("Vec::<(ExportStage, usize)>", "::new()")));
        assert!(!source.contains(concat!(
            ".position(|(buffered_stage, _)|",
            " *buffered_stage == stage)"
        )));
    }

    #[test]
    fn optimization_batch_20260830er_stage_output_counts_map_every_stage_to_one_slot() {
        let mut counts = BufferedStageOutputCounts::default();

        for (index, stage) in ExportStage::ALL.into_iter().enumerate() {
            *stage_output_event_count_mut(&mut counts, stage) = index + 1;
        }

        assert_eq!(counts, [1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    #[ignore = "deterministic performance evidence"]
    fn optimization_batch_20260830er_stage_output_count_benchmark_evidence() {
        const EVENT_COUNT: usize = 8_000_000;
        const SAMPLE_COUNT: usize = 11;
        const MARKER: &str = "EDITOR550_FIXED_STAGE_OUTPUT_COUNTERS_BENCH_V1";

        fn median(mut samples: Vec<std::time::Duration>) -> std::time::Duration {
            samples.sort_unstable();
            samples[samples.len() / 2]
        }

        let mut linear_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut fixed_samples = Vec::with_capacity(SAMPLE_COUNT);
        for _ in 0..SAMPLE_COUNT {
            let stage = std::hint::black_box(ExportStage::Report);
            let mut linear = ExportStage::ALL
                .into_iter()
                .map(|stage| (stage, 0_usize))
                .collect::<Vec<_>>();
            let started = std::time::Instant::now();
            for _ in 0..EVENT_COUNT {
                let count = linear
                    .iter_mut()
                    .find(|(candidate, _)| *candidate == stage)
                    .expect("all export stages are initialized");
                count.1 = std::hint::black_box(count.1.wrapping_add(1));
            }
            linear_samples.push(started.elapsed());
            std::hint::black_box(linear);

            let mut fixed = BufferedStageOutputCounts::default();
            let started = std::time::Instant::now();
            for _ in 0..EVENT_COUNT {
                let count = stage_output_event_count_mut(&mut fixed, stage);
                *count = std::hint::black_box(count.wrapping_add(1));
            }
            fixed_samples.push(started.elapsed());
            std::hint::black_box(fixed);
        }

        let linear = median(linear_samples);
        let fixed = median(fixed_samples);
        eprintln!("{MARKER} linear={linear:?} fixed={fixed:?}");
        assert!(fixed < linear, "fixed={fixed:?}, linear={linear:?}");
    }

    #[test]
    fn output_backpressure_preserves_terminal_event_and_reports_coalesced_count() {
        let mut options = ExportWizardPipelineOptions::for_test_profile(
            "windows-release",
            "zircon-project.toml",
            "D:\\zircon-export",
        );
        options.source_asset_manifest = Some("D:\\assets\\source-assets.json".to_string());
        options.host_executable = Some("D:\\zircon-export\\ZirconRuntime.exe".to_string());
        let plan = export_wizard_pipeline_plan(options);
        let snapshot = ExportWizardJobState::new("event-backpressure", &plan).into_snapshot();
        let (sender, receiver) = std::sync::mpsc::sync_channel(192);
        let mut buffered_output_events = Vec::new();
        let mut coalesced_output_events = 0;

        for line_index in 0..64 {
            send_job_event(
                &sender,
                ExportWizardJobEvent {
                    kind: ExportWizardJobEventKind::StageOutput,
                    snapshot: snapshot.event_header(),
                    output_delta: Some(ExportWizardStageOutputDelta {
                        stage: ExportStage::Validate,
                        output: ExportWizardCommandOutputLine {
                            stream: ExportWizardCommandOutputStream::Stdout,
                            line: format!("line {line_index}"),
                        },
                        progress: snapshot.progress.clone(),
                    }),
                    coalesced_output_events: 0,
                },
                &mut buffered_output_events,
                &mut coalesced_output_events,
            );
        }
        send_job_event(
            &sender,
            ExportWizardJobEvent {
                kind: ExportWizardJobEventKind::Finished,
                snapshot,
                output_delta: None,
                coalesced_output_events: 0,
            },
            &mut buffered_output_events,
            &mut coalesced_output_events,
        );

        let events = receiver.try_iter().collect::<Vec<_>>();
        assert_eq!(events.len(), 17);
        assert_eq!(
            events.last().map(|event| event.kind),
            Some(ExportWizardJobEventKind::Finished)
        );
        assert_eq!(
            events.last().map(|event| event.coalesced_output_events),
            Some(48)
        );
    }
}
