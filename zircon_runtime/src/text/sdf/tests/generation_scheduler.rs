use std::sync::Arc;
use std::time::{Duration, Instant};

use ttf_parser::Face;

use super::*;
use crate::core::runtime::tasks::{TaskPool, TaskPoolDescriptor};

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
            Arc::new(crate::text::VariationCoords::default()),
        )
        .expect("parsed generation source"),
    )
}

fn fixture_glyph(source: &SdfGenerationSourceContext, scalar: char) -> u16 {
    source.with_face(|face: &Face<'_>| face.glyph_index(scalar).expect("fixture glyph").0)
}

#[test]
fn text_sdf_generation_scheduler_exposes_its_effective_budget_snapshot() {
    let options = SdfGenerationSchedulerOptions::new(3)
        .with_max_glyphs_per_batch(7)
        .with_max_in_flight_glyphs(11)
        .with_source_byte_budget(13)
        .with_completion_queue_depth(17)
        .with_completion_byte_budget(19);

    let expected = SdfGenerationBudgetSnapshot {
        max_in_flight_batches: 3,
        max_glyphs_per_batch: 7,
        max_in_flight_glyphs: 11,
        source_byte_budget: 13,
        completion_queue_depth: 17,
        completion_byte_budget: 19,
    };

    assert_eq!(options.budget_snapshot(), expected);
    let scheduler = SdfGenerationScheduler::new(
        TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1)),
        options,
    );
    assert_eq!(scheduler.diagnostics(0).budget, expected);
}

#[test]
fn text_sdf_generation_scheduler_bounds_deduplicates_and_cancels_work() {
    let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1));
    let (started_tx, started_rx) = std::sync::mpsc::sync_channel(0);
    let (release_tx, release_rx) = std::sync::mpsc::sync_channel(0);
    pool.spawn(move || {
        started_tx.send(()).expect("publish blocker start");
        release_rx.recv().expect("release blocker");
    });
    started_rx.recv().expect("worker blocker started");

    let scheduler = SdfGenerationScheduler::new(
        pool,
        SdfGenerationSchedulerOptions::new(1)
            .with_max_glyphs_per_batch(4)
            .with_max_in_flight_glyphs(4),
    );
    let source = fixture_source(1);
    let glyph = fixture_glyph(source.as_ref(), 'A');
    let first_id = SdfGenerationWorkId::new(3, 1);
    let second_id = SdfGenerationWorkId::new(3, 2);

    scheduler
        .try_submit(
            first_id,
            10,
            Arc::clone(&source),
            SdfBakeParams::default(),
            vec![glyph],
        )
        .expect("first bounded work");
    assert_eq!(
        scheduler.try_submit(
            first_id,
            10,
            Arc::clone(&source),
            SdfBakeParams::default(),
            vec![glyph],
        ),
        Err(SdfGenerationSubmitError::Duplicate(first_id))
    );
    assert_eq!(
        scheduler.try_submit(second_id, 10, source, SdfBakeParams::default(), vec![glyph],),
        Err(SdfGenerationSubmitError::QueueFull(second_id))
    );
    assert!(scheduler.cancel(first_id));
    assert_eq!(scheduler.diagnostics(14).oldest_in_flight_age_frames, 4);

    release_tx.send(()).expect("release worker blocker");
}

#[test]
fn text_sdf_generation_scheduler_publishes_bounded_completion() {
    let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(2));
    let scheduler = SdfGenerationScheduler::new(
        pool,
        SdfGenerationSchedulerOptions::new(4)
            .with_max_glyphs_per_batch(8)
            .with_completion_queue_depth(2)
            .with_completion_byte_budget(1024 * 1024),
    );
    let source = fixture_source(2);
    let glyph_ids = ['M', 'A', 'M']
        .into_iter()
        .map(|scalar| fixture_glyph(source.as_ref(), scalar))
        .collect::<Vec<_>>();
    let work_id = SdfGenerationWorkId::new(5, 9);
    scheduler
        .try_submit(
            work_id,
            20,
            source,
            SdfBakeParams::for_mode(SdfMode::Msdf),
            glyph_ids,
        )
        .expect("bounded generation work");

    let deadline = Instant::now() + Duration::from_secs(5);
    let completion = loop {
        let mut completions =
            scheduler.drain_completed(24, SdfGenerationCompletionDrainBudget::new(1, 1024 * 1024));
        if let Some(completion) = completions.pop() {
            break completion;
        }
        assert!(Instant::now() < deadline, "generation completion deadline");
        std::thread::yield_now();
    };

    assert_eq!(completion.id, work_id);
    assert_eq!(completion.age_frames, 4);
    assert_eq!(completion.batch.report.requested_glyph_count, 3);
    assert_eq!(completion.batch.report.unique_glyph_count, 2);
    assert_eq!(scheduler.diagnostics(24).completion_backlog_count, 0);
}

#[test]
fn text_sdf_generation_scheduler_exposes_backpressured_work_as_inactive_for_retry() {
    let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1));
    let scheduler = SdfGenerationScheduler::new(
        pool,
        SdfGenerationSchedulerOptions::new(1)
            .with_completion_queue_depth(1)
            .with_completion_byte_budget(1),
    );
    let source = fixture_source(3);
    let glyph = fixture_glyph(source.as_ref(), 'A');
    let work_id = SdfGenerationWorkId::new(7, 1);
    scheduler
        .try_submit(work_id, 30, source, SdfBakeParams::default(), vec![glyph])
        .expect("work admitted before completion backpressure");

    let deadline = Instant::now() + Duration::from_secs(5);
    while scheduler.diagnostics(31).completion_backpressured_count == 0 {
        assert!(
            Instant::now() < deadline,
            "completion backpressure deadline"
        );
        std::thread::yield_now();
    }

    assert_eq!(
        scheduler.take_inactive_work_outcomes([work_id]),
        vec![(work_id, SdfGenerationInactiveWorkOutcome::Retryable)]
    );
}
