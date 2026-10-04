use std::sync::{Arc, Mutex};

use crossbeam_channel::Sender;

use super::PipelinedSubmissionQueue;

#[test]
fn render_perf_pipelined_feedback_is_one_submission_late() {
    let started = Arc::new(Mutex::new(Vec::new()));
    let worker_started = Arc::clone(&started);
    let mut queue = PipelinedSubmissionQueue::new(move |frame, ready: &Sender<()>| {
        worker_started.lock().unwrap().push(frame);
        ready.send(()).unwrap();
        frame
    })
    .unwrap();

    assert_eq!(queue.submit(1).unwrap(), None);
    assert_eq!(*started.lock().unwrap(), vec![1]);
    assert_eq!(queue.submit(2).unwrap(), Some(1));
    assert_eq!(queue.finish().unwrap(), Some(2));
}

#[test]
fn finish_reports_a_completed_worker_error() {
    let mut queue = PipelinedSubmissionQueue::new(|(), ready: &Sender<()>| {
        ready.send(()).expect("test worker should signal readiness");
        Err::<(), _>("submission failed")
    })
    .expect("test queue should create its worker");

    assert_eq!(queue.submit(()).unwrap(), None);
    assert_eq!(queue.finish().unwrap(), Some(Err("submission failed")));
}

#[test]
fn scheduler_profile_scopes_keep_submission_waits_distinct() {
    let source = include_str!("../queue.rs");

    for name in [
        "wait_previous_submission",
        "wait_worker_start",
        "wait_pending_submission",
        "pending_depth",
        "worker_utilization",
    ] {
        assert!(
            source.contains(name),
            "scheduler profiling must retain the `{name}` observation point"
        );
    }

    let worker_execution = source
        .split("move |submission, started| {")
        .nth(1)
        .expect("scheduler must retain a worker execution closure");
    let operation_lock = worker_execution
        .find("let _operation_guard = core.lock_operation();")
        .expect("worker must retain the RHI operation owner");
    let started = worker_execution
        .find("let _ = started.send(());")
        .expect("worker must retain its producer start signal");
    let busy = worker_execution
        .find("worker_utilization")
        .expect("worker occupancy must begin after the operation lock");
    let execute = worker_execution
        .find("let result = submission.execute(core.as_ref());")
        .expect("worker must execute its sealed submission");
    let idle = worker_execution[execute..]
        .find("worker_utilization")
        .map(|relative_index| execute + relative_index)
        .expect("worker occupancy must finish after submission execution");

    assert!(operation_lock < started && started < busy && busy < execute && execute < idle);
}

#[test]
fn frame_submission_checks_device_admission_before_dispatching_work() {
    let source = include_str!("../queue.rs");
    let execution = source
        .split("fn execute(self, core: &WgpuRenderFrameworkCore)")
        .nth(1)
        .expect("frame submission must retain its execution entry point");
    let admission = execution
        .find("core.ensure_device_admission()?;")
        .expect("frame submission must reject work for a faulted device");
    let dispatch = execution
        .find("match self.kind")
        .expect("frame submission must dispatch one sealed frame kind");

    assert!(
        admission < dispatch,
        "device admission must happen before either frame execution path"
    );
}
