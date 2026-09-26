use std::collections::VecDeque;
use std::panic::{catch_unwind, AssertUnwindSafe};

use super::{
    BoundedKeyedIoAdmissionError, BoundedKeyedIoTerminal, LaneInner, LaneState,
    TerminalNotification, TerminalObserver, WorkEntry,
};

// 接纳与字节计费必须在通道锁内先于持有工作体；拒绝路径不接管调用方资源。
pub(super) fn reserve(
    lane: &LaneInner,
    state: &mut LaneState,
    retained_bytes: usize,
) -> Result<(), BoundedKeyedIoAdmissionError> {
    if !state.accepting {
        return Err(BoundedKeyedIoAdmissionError::Closed);
    }
    if state.reserved_entries >= lane.limits.max_entries {
        return Err(BoundedKeyedIoAdmissionError::EntryCapacityExceeded);
    }
    let next_bytes = state
        .retained_bytes
        .checked_add(retained_bytes)
        .ok_or(BoundedKeyedIoAdmissionError::RetainedBytesOverflow)?;
    if next_bytes > lane.limits.max_retained_bytes {
        return Err(BoundedKeyedIoAdmissionError::RetainedBytesCapacityExceeded);
    }
    state.reserved_entries += 1;
    state.retained_bytes = next_bytes;
    state.submitted = state.submitted.saturating_add(1);
    Ok(())
}

pub(super) fn release_reservation(state: &mut LaneState, retained_bytes: usize) {
    state.reserved_entries = state.reserved_entries.saturating_sub(1);
    state.retained_bytes = state.retained_bytes.saturating_sub(retained_bytes);
}

pub(super) fn take_ticket_id(state: &mut LaneState) -> u64 {
    let id = state.next_ticket_id;
    state.next_ticket_id = state.next_ticket_id.saturating_add(1);
    id
}

pub(super) fn mark_pump_needed(state: &mut LaneState) -> bool {
    if state.pump_active || !front_is_runnable(state) {
        false
    } else {
        state.pump_active = true;
        true
    }
}

// 更早的挂起接纳仍占顺序位置；泵不得越过它启动队首或提前完成栅栏。
pub(super) fn front_is_runnable(state: &LaneState) -> bool {
    let Some(front) = state.queue.front() else {
        return false;
    };
    let Some((suspended_epoch, suspended_ticket_id)) = state.suspended_order.first().copied()
    else {
        return true;
    };
    suspended_epoch > front.epoch
        || (suspended_epoch == front.epoch
            && !front.fence
            && suspended_ticket_id >= front.ticket.id())
}

pub(super) fn remove_suspended_entry(state: &mut LaneState, ticket_id: u64) -> Option<WorkEntry> {
    let entry = state.suspended.remove(&ticket_id)?;
    let removed = state.suspended_order.remove(&(entry.epoch, ticket_id));
    debug_assert!(removed, "suspended order index must mirror ticket storage");
    Some(entry)
}

pub(super) fn merge_ordered_queue(
    queue: &mut VecDeque<WorkEntry>,
    retained_suspended: VecDeque<WorkEntry>,
) {
    merge_ordered(queue, retained_suspended, |left, right| {
        left.epoch < right.epoch
            || (left.epoch == right.epoch
                && ((!left.fence && right.fence)
                    || (left.fence == right.fence && left.ticket.id() <= right.ticket.id())))
    });
}

pub(in crate::core::runtime::tasks::bounded_keyed_io) fn merge_ordered<T>(
    queue: &mut VecDeque<T>,
    mut incoming: VecDeque<T>,
    precedes_or_equals: impl Fn(&T, &T) -> bool,
) {
    if incoming.is_empty() {
        return;
    }
    let mut existing = std::mem::take(queue);
    queue.reserve(existing.len().saturating_add(incoming.len()));
    while let (Some(existing_front), Some(incoming_front)) = (existing.front(), incoming.front()) {
        if precedes_or_equals(existing_front, incoming_front) {
            queue.push_back(
                existing
                    .pop_front()
                    .expect("existing front must remain present"),
            );
        } else {
            queue.push_back(
                incoming
                    .pop_front()
                    .expect("incoming front must remain present"),
            );
        }
    }
    queue.append(&mut existing);
    queue.append(&mut incoming);
}

// 执行前终结必须同时归还容量并保留票据已有的终态；观察者留待锁外投递。
pub(super) fn finish_pre_start_entry(
    state: &mut LaneState,
    entry: WorkEntry,
    requested_terminal: BoundedKeyedIoTerminal,
    notifications: &mut Vec<TerminalNotification>,
) {
    let terminal = if entry.ticket.mark_terminal_before_start(requested_terminal) {
        requested_terminal
    } else {
        entry.ticket.terminal().unwrap_or(requested_terminal)
    };
    release_reservation(state, entry.retained_bytes);
    state.cancelled = state.cancelled.saturating_add(1);
    notifications.push(TerminalNotification {
        observer: entry.terminal_observer,
        terminal,
    });
}

pub(super) fn notify_observers(notifications: Vec<TerminalNotification>) {
    for notification in notifications {
        notify_observer(notification.observer, notification.terminal);
    }
}

pub(super) fn notify_observer(
    observer: Option<TerminalObserver>,
    terminal: BoundedKeyedIoTerminal,
) {
    if let Some(observer) = observer {
        let _ = catch_unwind(AssertUnwindSafe(|| observer(terminal)));
    }
}
