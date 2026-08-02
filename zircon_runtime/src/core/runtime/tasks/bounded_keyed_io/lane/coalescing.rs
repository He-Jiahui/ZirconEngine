use std::collections::VecDeque;

use super::{release_reservation, LaneState, TerminalNotification, WorkEntry};
use crate::core::runtime::tasks::bounded_keyed_io::BoundedKeyedIoTerminal;

pub(super) fn insert_ordered(queue: &mut VecDeque<WorkEntry>, entry: WorkEntry) {
    let insertion = queue
        .iter()
        .position(|queued| {
            queued.epoch > entry.epoch
                || (queued.epoch == entry.epoch
                    && (queued.fence || queued.ticket.id() > entry.ticket.id()))
        })
        .unwrap_or(queue.len());
    queue.insert(insertion, entry);
}

pub(super) fn coalesce_queued_generation(
    state: &mut LaneState,
    successor: &WorkEntry,
    notifications: &mut Vec<TerminalNotification>,
) -> bool {
    let Some(key) = successor.key.as_deref() else {
        return true;
    };
    let active_successor = state.active.as_ref().is_some_and(|active| {
        active.epoch == successor.epoch
            && active.key.as_deref() == Some(key)
            && active.generation > successor.generation
    });
    let queued_successor = state.queue.iter().any(|queued| {
        !queued.fence
            && queued.epoch == successor.epoch
            && queued.key.as_deref() == Some(key)
            && queued.generation > successor.generation
    });
    if active_successor || queued_successor {
        let successor_generation = state
            .active
            .iter()
            .filter(|active| active.epoch == successor.epoch && active.key.as_deref() == Some(key))
            .map(|active| active.generation)
            .chain(
                state
                    .queue
                    .iter()
                    .filter(|queued| {
                        !queued.fence
                            && queued.epoch == successor.epoch
                            && queued.key.as_deref() == Some(key)
                    })
                    .map(|queued| queued.generation),
            )
            .max()
            .unwrap_or(successor.generation);
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

    let mut index = 0;
    while index < state.queue.len() {
        let matches = {
            let queued = &state.queue[index];
            !queued.fence && queued.epoch == successor.epoch && queued.key.as_deref() == Some(key)
        };
        if !matches {
            index += 1;
            continue;
        }
        let queued = state
            .queue
            .remove(index)
            .expect("matched queued entry must exist");
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
