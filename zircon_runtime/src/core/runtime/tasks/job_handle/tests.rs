use std::panic::{self, AssertUnwindSafe};
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use super::super::{JobScheduler, TaskPool};
use super::JobHandle;
use crate::core::runtime::tasks::TaskPoolDescriptor;

#[test]
fn job_handle_accessors_recover_poisoned_state_lock() {
    let handle = JobHandle::pending_with_dependencies(1);

    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
        let _guard = handle.state.inner.lock().unwrap();
        panic!("poison job handle state");
    }));

    assert!(!handle.is_complete());
    let (dependent_ran_tx, dependent_ran_rx) = std::sync::mpsc::sync_channel(1);
    assert!(handle.add_dependent(Box::new(move || {
        dependent_ran_tx
            .send(())
            .expect("dependent completion should be delivered");
    })));
    assert!(handle.dependency_completed());
    handle.mark_complete();

    assert!(handle.is_complete());
    assert!(handle.panic_message().is_none());
    dependent_ran_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("dependent completion should run asynchronously");
}

#[test]
fn cancelled_handle_reports_typed_terminal_state_without_panicking_wait() {
    let handle = JobHandle::pending_with_dependencies(0);

    handle.mark_cancelled();

    assert!(handle.is_complete());
    assert!(handle.is_cancelled());
    assert_eq!(
        handle.terminal_state(),
        Some(crate::core::runtime::tasks::TaskState::Cancelled)
    );
    assert!(panic::catch_unwind(AssertUnwindSafe(|| handle.wait())).is_ok());
}

#[test]
fn job_handle_wait_recovers_poisoned_state_lock() {
    let handle = JobHandle::pending_with_dependencies(0);

    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
        let _guard = handle.state.inner.lock().unwrap();
        panic!("poison job handle wait state");
    }));

    let completer = handle.clone();
    let completion_thread = thread::spawn(move || {
        thread::sleep(Duration::from_millis(1));
        completer.mark_complete();
    });

    handle.wait();
    completion_thread.join().unwrap();
    assert!(handle.is_complete());
}

#[test]
fn combined_dependency_terminal_path_uses_one_state_lock() {
    let source = include_str!("../job_handle.rs");
    let start = source
        .find("fn combined_dependency_completed")
        .expect("combined dependency implementation");
    let end = source[start..]
        .find("impl JobState")
        .map(|offset| start + offset)
        .expect("job state implementation");
    let implementation = &source[start..end];

    assert!(implementation.contains("inner.lifecycle = if inner.panic_message.is_some()"));
    assert!(implementation.contains("std::mem::take(&mut inner.dependents)"));
    assert!(!implementation.contains("self.panic_message()"));
    assert!(!implementation.contains("self.mark_panicked"));
    assert!(!implementation.contains("self.mark_complete"));
}

#[test]
fn job_terminal_observer_runs_once_when_dependency_continuation_unwinds() {
    let handle = JobHandle::pending_with_dependencies(0);
    let sibling = JobHandle::pending_with_dependencies(0);
    let (observer_tx, observer_rx) = std::sync::mpsc::sync_channel(1);
    assert!(handle.add_dependent(Box::new(|| {
        panic!("dependency continuation failure");
    })));
    let combined = JobHandle::combine(&[handle.clone(), sibling.clone()]);
    handle.on_terminal(move || {
        observer_tx
            .send(())
            .expect("observer completion should be delivered");
    });

    handle.mark_complete();
    assert!(handle.is_complete());
    sibling.mark_complete();
    combined.wait();
    handle.wait();
    handle.mark_complete();
    observer_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("observer should run after continuation delivery");
    assert!(
        observer_rx.try_recv().is_err(),
        "observer should run exactly once"
    );
}

#[test]
fn late_terminal_observer_waits_until_dependency_continuations_finish() {
    let handle = JobHandle::pending_with_dependencies(0);
    let continuation_entered = Arc::new(std::sync::Barrier::new(2));
    let release_continuation = Arc::new(std::sync::Barrier::new(2));
    let observer_ran = Arc::new(AtomicBool::new(false));

    let continuation_entered_for_callback = Arc::clone(&continuation_entered);
    let release_continuation_for_callback = Arc::clone(&release_continuation);
    assert!(handle.add_dependent(Box::new(move || {
        continuation_entered_for_callback.wait();
        release_continuation_for_callback.wait();
    })));

    let handle_for_completion = handle.clone();
    let completion = thread::spawn(move || handle_for_completion.mark_complete());
    continuation_entered.wait();

    let observer_ran_for_callback = Arc::clone(&observer_ran);
    handle.on_terminal(move || {
        observer_ran_for_callback.store(true, Ordering::SeqCst);
    });
    assert!(
        !observer_ran.load(Ordering::SeqCst),
        "a late observer must not overtake an in-flight dependency continuation"
    );

    release_continuation.wait();
    completion.join().unwrap();
    let observer_deadline = std::time::Instant::now() + Duration::from_secs(1);
    while !observer_ran.load(Ordering::SeqCst) {
        assert!(
            std::time::Instant::now() < observer_deadline,
            "observer should run after dependency continuation delivery"
        );
        thread::yield_now();
    }
}

#[test]
fn terminal_observers_keep_registration_order_when_late_observer_arrives() {
    let handle = JobHandle::pending_with_dependencies(0);
    let delivered = Arc::new(std::sync::Mutex::new(Vec::new()));
    let (first_started_tx, first_started_rx) = std::sync::mpsc::sync_channel(1);
    let release_first = Arc::new(std::sync::Barrier::new(2));
    let (late_delivered_tx, late_delivered_rx) = std::sync::mpsc::sync_channel(1);

    let delivered_for_first = Arc::clone(&delivered);
    let release_first_for_callback = Arc::clone(&release_first);
    handle.on_terminal(move || {
        first_started_tx
            .send(())
            .expect("first observer should start");
        release_first_for_callback.wait();
        delivered_for_first
            .lock()
            .expect("observer order lock")
            .push(1);
    });
    let delivered_for_second = Arc::clone(&delivered);
    handle.on_terminal(move || {
        delivered_for_second
            .lock()
            .expect("observer order lock")
            .push(2);
    });

    handle.mark_complete();
    first_started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("first observer should begin");

    let delivered_for_late = Arc::clone(&delivered);
    handle.on_terminal(move || {
        delivered_for_late
            .lock()
            .expect("observer order lock")
            .push(3);
        late_delivered_tx
            .send(())
            .expect("late observer completion should be reported");
    });
    assert!(
        late_delivered_rx
            .recv_timeout(Duration::from_millis(100))
            .is_err(),
        "a late observer must not overtake the blocked origin observer batch"
    );

    release_first.wait();
    late_delivered_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("late observer should eventually run");
    assert_eq!(
        *delivered.lock().expect("observer order lock"),
        vec![1, 2, 3],
        "terminal observers must preserve one handle's registration order"
    );
}

#[test]
fn panic_dependency_chain_uses_bounded_terminal_delivery() {
    const CHILD_ENV: &str = "ZIRCON_PANIC_DEPENDENCY_CHAIN_CHILD";
    const CHILD_COMPLETE: &str = "zircon panic dependency chain completed";
    const DEPTH: usize = 100_000;

    if std::env::var_os(CHILD_ENV).is_some() {
        let scheduler = JobScheduler::from_pool(TaskPool::new(
            TaskPoolDescriptor::compute().with_worker_threads(1),
        ));
        let (release_root_tx, release_root_rx) = std::sync::mpsc::sync_channel(1);
        let root = scheduler.schedule(move || {
            release_root_rx
                .recv()
                .expect("parent should release the root task");
            panic!("root task failure");
        });
        let mut tail = root;
        for _ in 0..DEPTH {
            tail = scheduler.schedule_after(&[tail], || {});
        }

        release_root_tx
            .send(())
            .expect("root task should still be waiting");
        assert!(
            panic::catch_unwind(AssertUnwindSafe(|| tail.wait())).is_err(),
            "the propagated root panic must reach the chain tail"
        );
        eprintln!("{CHILD_COMPLETE}");
        return;
    }

    let test_executable = std::env::current_exe().expect("current lib-test executable");
    let listed = Command::new(&test_executable)
        .arg("--list")
        .output()
        .expect("list current lib-test names");
    assert!(
        listed.status.success(),
        "current lib-test list must succeed"
    );
    let test_suffix = "::panic_dependency_chain_uses_bounded_terminal_delivery";
    let listed_stdout = String::from_utf8_lossy(&listed.stdout);
    let test_name = listed_stdout
        .lines()
        .filter_map(|line| line.strip_suffix(": test"))
        .find(|name| name.ends_with(test_suffix))
        .unwrap_or_else(|| panic!("lib-test list should contain `{test_suffix}`"))
        .to_owned();

    let output = Command::new(test_executable)
        .args(["--exact", test_name.as_str(), "--nocapture"])
        .env(CHILD_ENV, "1")
        .output()
        .expect("launch isolated deep panic-dependency test");

    assert!(
        output.status.success(),
        "terminal delivery must not overflow the child stack: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(CHILD_COMPLETE),
        "child should prove the full dependency chain reached its terminal state"
    );
}
