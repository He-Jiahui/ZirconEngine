use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use super::super::{BoundedKeyedIoLane, BoundedKeyedIoLimits, BoundedKeyedIoWorkDeadline};
use super::*;
use crate::core::runtime::tasks::{JobScheduler, TaskPool, TaskPoolDescriptor};

#[test]
fn astra_m25_foreign_lane_authority_cannot_cancel_a_colliding_ticket() {
    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::io().with_worker_threads(1),
    ));
    let left = BoundedKeyedIoLane::new(BoundedKeyedIoLimits::new(4, 4), scheduler.clone());
    let right = BoundedKeyedIoLane::new(BoundedKeyedIoLimits::new(4, 4), scheduler);
    let source = left
        .try_admit(
            "key",
            7,
            1,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(|| Ok(())),
        )
        .unwrap();
    let ran = Arc::new(AtomicUsize::new(0));
    let ran_work = Arc::clone(&ran);
    let target = right
        .try_admit(
            "key",
            7,
            1,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(move || {
                ran_work.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }),
        )
        .unwrap();
    let source_ticket = source.ticket();
    let target_ticket = target.ticket();
    assert_eq!(source_ticket.id(), target_ticket.id());
    assert_eq!(source_ticket.generation(), target_ticket.generation());
    let authority = source.cancel_authority();
    assert_eq!(
        target_ticket.cancel_before_start(&authority),
        Err(BoundedKeyedIoCancelError::WrongAuthority)
    );
    assert_eq!(target_ticket.terminal(), None);
    assert_eq!(
        source_ticket
            .clone()
            .cancel_before_start(&authority.clone()),
        Ok(())
    );
    assert_eq!(source_ticket.cancel_before_start(&authority), Ok(()));
    let target_ticket = target.activate();
    assert_eq!(
        target_ticket.wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    );
    assert_eq!(ran.load(Ordering::SeqCst), 1);
}

#[test]
fn astra_m25_reused_numeric_identity_rejects_old_authority() {
    let old = BoundedKeyedIoTicket::pending(42, 9, false);
    let authority = BoundedKeyedIoCancelAuthority::new(&old);
    assert!(old.mark_terminal(BoundedKeyedIoTerminal::Succeeded));
    drop(old);
    for generation in [9, 10] {
        let replacement = BoundedKeyedIoTicket::pending(42, generation, false);
        assert_eq!(
            replacement.cancel_before_start(&authority),
            Err(BoundedKeyedIoCancelError::WrongAuthority)
        );
        assert_eq!(replacement.terminal(), None);
        let current = BoundedKeyedIoCancelAuthority::new(&replacement);
        assert_eq!(replacement.cancel_before_start(&current), Ok(()));
    }
}

#[test]
fn astra_m25_owner_checks_preserve_fence_and_start_guards() {
    let pinned = BoundedKeyedIoTicket::pending(1, 1, true);
    let pinned_authority = BoundedKeyedIoCancelAuthority::new(&pinned);
    assert_eq!(
        pinned.cancel_before_start(&pinned_authority),
        Err(BoundedKeyedIoCancelError::FencePinned)
    );
    assert_eq!(pinned.terminal(), None);
    pinned.unpin_from_fence();
    assert_eq!(pinned.cancel_before_start(&pinned_authority), Ok(()));

    let running = BoundedKeyedIoTicket::pending(2, 1, false);
    let running_authority = BoundedKeyedIoCancelAuthority::new(&running);
    assert!(running.mark_started());
    assert_eq!(
        running.cancel_before_start(&running_authority),
        Err(BoundedKeyedIoCancelError::AlreadyStarted)
    );
    assert_eq!(running.terminal(), None);
}
