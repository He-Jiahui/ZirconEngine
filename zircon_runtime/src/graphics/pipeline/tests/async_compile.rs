use super::{PipelineAsyncCompileError, PipelineAsyncCompiler, PipelineAsyncQueueResult};

#[test]
fn render_perf_async_pipeline_queue_deduplicates_and_recovers_completion() {
    let mut compiler = PipelineAsyncCompiler::new("pipeline-test", 2).unwrap();
    assert_eq!(
        compiler.try_queue(7_u32, || 41_u32),
        PipelineAsyncQueueResult::Queued
    );
    assert_eq!(
        compiler.try_queue(7_u32, || 99_u32),
        PipelineAsyncQueueResult::AlreadyPending
    );

    let mut completed = Vec::new();
    compiler.finish_pending(|key, result| completed.push((key, result)));

    assert_eq!(completed, vec![(7, Ok(41))]);
    assert_eq!(compiler.pending_count(), 0);
}

#[test]
fn render_perf_async_pipeline_queue_has_a_hard_in_flight_budget() {
    let mut compiler = PipelineAsyncCompiler::new("bounded-pipeline-test", 1).unwrap();
    assert!(compiler.has_available_slot());
    assert_eq!(
        compiler.try_queue(1_u32, || 1_u32),
        PipelineAsyncQueueResult::Queued
    );
    assert!(!compiler.has_available_slot());
    assert_eq!(
        compiler.try_queue(2_u32, || 2_u32),
        PipelineAsyncQueueResult::Full
    );
    compiler.finish_pending(|_, _| {});
    assert!(compiler.has_available_slot());
}

#[test]
fn render_perf_async_pipeline_target_sync_leaves_later_work_pending() {
    let mut compiler = PipelineAsyncCompiler::new("target-sync-test", 3).unwrap();
    let (release_first, wait_first) = std::sync::mpsc::sync_channel(0);
    let (release_later, wait_later) = std::sync::mpsc::sync_channel(0);
    assert_eq!(
        compiler.try_queue(1_u32, move || {
            wait_first.recv().expect("test releases the first job");
            1_u32
        }),
        PipelineAsyncQueueResult::Queued
    );
    assert_eq!(
        compiler.try_queue(2_u32, || 2_u32),
        PipelineAsyncQueueResult::Queued
    );
    assert_eq!(
        compiler.try_queue(3_u32, move || {
            wait_later.recv().expect("test releases the later job");
            3_u32
        }),
        PipelineAsyncQueueResult::Queued
    );

    release_first
        .send(())
        .expect("first worker job should still be waiting");
    let mut completed = Vec::new();
    assert_eq!(
        compiler.finish_pending_through(&2, |key, result| completed.push((key, result))),
        2
    );

    assert_eq!(completed, vec![(1, Ok(1)), (2, Ok(2))]);
    assert!(compiler.is_pending(&3));
    release_later
        .send(())
        .expect("later worker job should still be waiting");
    compiler.finish_pending(|_, _| {});
}

#[test]
fn render_perf_async_pipeline_worker_contains_job_panics() {
    let mut compiler = PipelineAsyncCompiler::<u32, u32>::new("panic-pipeline-test", 1).unwrap();
    compiler.try_queue(3, || panic!("synthetic compile panic"));

    let mut completed = Vec::new();
    compiler.finish_pending(|key, result| completed.push((key, result)));

    assert_eq!(
        completed,
        vec![(3, Err(PipelineAsyncCompileError::JobPanicked))]
    );
}
