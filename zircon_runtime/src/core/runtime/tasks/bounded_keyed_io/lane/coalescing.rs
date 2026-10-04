use std::collections::VecDeque;

use super::{release_reservation, LaneState, TerminalNotification, WorkEntry};
use crate::core::runtime::tasks::bounded_keyed_io::BoundedKeyedIoTerminal;

pub(super) fn insert_ordered(queue: &mut VecDeque<WorkEntry>, entry: WorkEntry) {
    insert_ordered_by(queue, entry, |queued, entry| {
        queued.epoch > entry.epoch
            || (queued.epoch == entry.epoch
                && (queued.fence || queued.ticket.id() > entry.ticket.id()))
    });
}

fn insert_ordered_by<T>(
    queue: &mut VecDeque<T>,
    entry: T,
    mut should_insert_before: impl FnMut(&T, &T) -> bool,
) {
    let append_to_tail = match queue.back() {
        Some(queued) => !should_insert_before(queued, &entry),
        None => true,
    };
    if append_to_tail {
        queue.push_back(entry);
        return;
    }

    let insertion = queue
        .iter()
        .position(|queued| should_insert_before(queued, &entry))
        .expect("queue tail must follow an entry that cannot append");
    queue.insert(insertion, entry);
}

fn latest_generation_above(
    successor_generation: u64,
    generations: impl Iterator<Item = u64>,
) -> Option<u64> {
    generations
        .filter(|generation| *generation > successor_generation)
        .max()
}

fn take_matching_entries<T>(
    queue: &mut VecDeque<T>,
    mut matches: impl FnMut(&T) -> bool,
) -> Vec<T> {
    let mut pending = std::mem::take(queue);
    queue.reserve(pending.len());
    let mut matching = Vec::with_capacity(pending.len());
    while let Some(entry) = pending.pop_front() {
        if matches(&entry) {
            matching.push(entry);
        } else {
            queue.push_back(entry);
        }
    }
    matching
}

#[cfg(test)]
#[path = "coalescing/tests/optimization_batch_ji_runtime648_tests.rs"]
mod optimization_batch_ji_runtime648_tests;

// 若 active/queued 中已有更高代，当前项直接终结为 Superseded；否则线性分区移除同键同 epoch 的旧队列项并保留其余顺序。
pub(super) fn coalesce_queued_generation(
    state: &mut LaneState,
    successor: &WorkEntry,
    notifications: &mut Vec<TerminalNotification>,
) -> bool {
    let Some(key) = successor.key.as_ref() else {
        return true;
    };
    let successor_generation = latest_generation_above(
        successor.generation,
        state
            .active
            .iter()
            .filter(|active| active.epoch == successor.epoch && active.key.as_ref() == Some(key))
            .map(|active| active.generation)
            .chain(
                state
                    .queue
                    .iter()
                    .filter(|queued| {
                        !queued.fence
                            && queued.epoch == successor.epoch
                            && queued.key.as_ref() == Some(key)
                    })
                    .map(|queued| queued.generation),
            ),
    );
    if let Some(successor_generation) = successor_generation {
        let terminal = BoundedKeyedIoTerminal::Superseded {
            successor: successor_generation,
        };
        successor.ticket.mark_terminal(terminal);
        notifications.push(TerminalNotification {
            observer: successor.terminal_observer.clone(),
            terminal,
        });
        release_reservation(state, successor.retained_bytes);
        state.superseded = state.superseded.saturating_add(1);
        state.coalesced = state.coalesced.saturating_add(1);
        return false;
    }

    let superseded_entries = take_matching_entries(&mut state.queue, |queued| {
        !queued.fence && queued.epoch == successor.epoch && queued.key.as_ref() == Some(key)
    });
    for queued in superseded_entries {
        let terminal = BoundedKeyedIoTerminal::Superseded {
            successor: successor.generation,
        };
        queued.ticket.mark_terminal(terminal);
        notifications.push(TerminalNotification {
            observer: queued.terminal_observer,
            terminal,
        });
        release_reservation(state, queued.retained_bytes);
        state.superseded = state.superseded.saturating_add(1);
        state.coalesced = state.coalesced.saturating_add(1);
    }
    true
}

#[cfg(test)]
#[path = "tests/coalescing.rs"]
mod tests;
