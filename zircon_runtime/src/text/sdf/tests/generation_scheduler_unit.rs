use super::*;
use crate::core::runtime::tasks::TaskPoolDescriptor;
use crate::text::VariationCoords;
use std::sync::Arc;

fn fixture_source(handle: u64) -> Arc<SdfGenerationSourceContext> {
    let bytes = Arc::<[u8]>::from(
        std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets/fonts/FiraSans-Regular.ttf"),
        )
        .expect("Fira Sans fixture"),
    );
    Arc::new(
        SdfGenerationSourceContext::new(
            SdfGenerationSourceHandle::new(handle),
            bytes,
            0,
            Arc::new(VariationCoords::default()),
        )
        .expect("parsed generation source"),
    )
}

#[test]
fn worker_panic_is_reported_as_terminal_inactive_work() {
    let scheduler = SdfGenerationScheduler::new(
        TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1)),
        SdfGenerationSchedulerOptions::new(1),
    );
    let work_id = SdfGenerationWorkId::new(7, 3);
    let source = SdfGenerationSourceHandle::new(11);
    {
        let mut state = lock_state(&scheduler.state);
        state.active_ids.insert(work_id);
        state.in_flight.insert(
            work_id,
            InFlightBatch {
                submitted_frame: 4,
                glyph_count: 1,
                source,
                cancelled: false,
            },
        );
        state.in_flight_glyph_count = 1;
        admit_source(&mut state, source, 128);
    }

    publish_worker_result(
        scheduler.state.as_ref(),
        scheduler.shutdown.as_ref(),
        scheduler.options,
        work_id,
        Err(Box::new("test worker panic")),
    );

    assert_eq!(
        scheduler.take_inactive_work_outcomes([work_id]),
        vec![(work_id, SdfGenerationInactiveWorkOutcome::WorkerPanic)]
    );
    assert!(scheduler.take_inactive_work_outcomes([work_id]).is_empty());
    assert_eq!(scheduler.diagnostics(5).worker_panic_count, 1);
}

#[test]
fn cancelling_an_inactive_work_id_discards_its_stale_panic_outcome() {
    let scheduler = SdfGenerationScheduler::new(
        TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1)),
        SdfGenerationSchedulerOptions::new(1),
    );
    let work_id = SdfGenerationWorkId::new(7, 3);
    lock_state(&scheduler.state)
        .worker_panic_ids
        .insert(work_id);

    assert!(!scheduler.cancel(work_id));
    assert_eq!(
        scheduler.take_inactive_work_outcomes([work_id]),
        vec![(work_id, SdfGenerationInactiveWorkOutcome::Retryable)]
    );
}

#[test]
fn cancelling_all_work_discards_stale_panic_outcomes() {
    let scheduler = SdfGenerationScheduler::new(
        TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1)),
        SdfGenerationSchedulerOptions::new(1),
    );
    let work_id = SdfGenerationWorkId::new(7, 3);
    lock_state(&scheduler.state)
        .worker_panic_ids
        .insert(work_id);

    assert_eq!(scheduler.cancel_all(), 0);
    assert_eq!(
        scheduler.take_inactive_work_outcomes([work_id]),
        vec![(work_id, SdfGenerationInactiveWorkOutcome::Retryable)]
    );
}

#[test]
fn admitting_a_reused_work_id_discards_its_stale_panic_outcome() {
    let scheduler = SdfGenerationScheduler::new(
        TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1)),
        SdfGenerationSchedulerOptions::new(1),
    );
    let work_id = SdfGenerationWorkId::new(7, 3);
    lock_state(&scheduler.state)
        .worker_panic_ids
        .insert(work_id);

    scheduler
        .try_submit(
            work_id,
            1,
            fixture_source(12),
            SdfBakeParams::default(),
            vec![1],
        )
        .expect("reused work id must be admitted");

    assert!(!lock_state(&scheduler.state)
        .worker_panic_ids
        .contains(&work_id));
}
