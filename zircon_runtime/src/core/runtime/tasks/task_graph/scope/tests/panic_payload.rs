use std::panic::{catch_unwind, panic_any, AssertUnwindSafe};

use super::*;
use crate::core::runtime::tasks::{
    EngineTaskGraph, EngineTaskGraphOptions, TaskPoolKind, TaskState,
};

#[test]
fn astra_m1_owned_panic_message_moves_its_original_buffer() {
    let message = "owned failure".repeat(4096);
    let pointer = message.as_ptr();
    let recovered = panic_payload_message(Box::new(message));
    assert_eq!(recovered.as_ptr(), pointer);
    assert_eq!(recovered, "owned failure".repeat(4096));
    assert_eq!(
        panic_payload_message(Box::new("static failure")),
        "static failure"
    );
    assert_eq!(
        panic_payload_message(Box::new(42_u32)),
        "non-string panic payload"
    );
}

#[test]
fn astra_m1_scoped_and_detached_tasks_preserve_panic_messages() {
    let graph = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(1)).unwrap();
    let scope = graph
        .create_scope(TaskGraphScopeDescriptor::new("panic-payload"))
        .unwrap();
    let scheduler = JobScheduler::from_pool(graph.worker_pool().clone());
    for detached in [false, true] {
        for (id, expected) in [
            (1, "static failure"),
            (2, "owned failure"),
            (3, "non-string panic payload"),
        ] {
            let descriptor =
                TaskDescriptor::new(TaskId::new(id), TaskPoolKind::Compute, "panic-payload")
                    .with_cancellation_policy(TaskCancellationPolicy::FinishOnShutdown);
            let fail = move |_: TaskCancellationToken| match id {
                1 => panic!("static failure"),
                2 => panic_any(String::from("owned failure")),
                _ => panic_any(42_u32),
            };
            let task = if detached {
                TaskHandle::schedule_detached(&scheduler, descriptor, fail)
            } else {
                scope.submit(descriptor, fail).unwrap()
            };
            assert!(catch_unwind(AssertUnwindSafe(|| task.wait())).is_err());
            assert_eq!(
                task.status().state,
                crate::core::runtime::tasks::TaskState::Failed
            );
            assert_eq!(task.completion.panic_message().as_deref(), Some(expected));
            assert!(scope.wait_until_quiescent(Duration::from_secs(2)));
        }
    }
    assert_eq!(scope.census().failed, 3);
    graph.shutdown(Duration::from_secs(2)).unwrap();
}

#[test]
fn astra_m1_failed_dependency_preserves_original_cause_without_running_dependent() {
    let graph = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(1)).unwrap();
    let scope = graph
        .create_scope(TaskGraphScopeDescriptor::new("dependency-panic"))
        .unwrap();
    let descriptor = |id| {
        TaskDescriptor::new(TaskId::new(id), TaskPoolKind::Compute, "dependency-panic")
            .with_cancellation_policy(TaskCancellationPolicy::FinishOnShutdown)
    };
    let prerequisite = scope
        .submit(descriptor(1), |_| panic!("original dependency failure"))
        .unwrap();
    let dependent = scope
        .submit_after(&[prerequisite], descriptor(2), |_| panic!("must not run"))
        .unwrap();
    assert!(catch_unwind(AssertUnwindSafe(|| dependent.wait())).is_err());
    assert_eq!(dependent.status().state, TaskState::Failed);
    assert_eq!(
        dependent.completion.panic_message().as_deref(),
        Some("original dependency failure")
    );
    assert!(scope.wait_until_quiescent(Duration::from_secs(2)));
    assert_eq!(scope.census().failed, 2);
    graph.shutdown(Duration::from_secs(2)).unwrap();
}
