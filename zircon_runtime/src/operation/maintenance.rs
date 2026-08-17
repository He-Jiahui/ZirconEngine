use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use zircon_runtime_interface::{ZrRuntimeOperationDetailKindV2, ZrRuntimeOperationPhase};

use crate::core::runtime::tasks::TaskTimer;

use super::service::RuntimeOperationTaskState;

pub(super) fn expire_due_deadlines_in_state(state: &mut RuntimeOperationTaskState, now: Instant) {
    let expired: Vec<_> = state
        .tasks
        .iter()
        .filter_map(|(handle, task)| {
            let deadline = task.deadline?;
            (matches!(
                task.phase,
                ZrRuntimeOperationPhase::Queued
                    | ZrRuntimeOperationPhase::Preparing
                    | ZrRuntimeOperationPhase::ReadyToApply
            ) && !task.apply_claimed
                && now >= deadline)
                .then_some((*handle, deadline))
        })
        .collect();
    for (handle, deadline) in expired {
        let elapsed = duration_millis_u64(now.saturating_duration_since(deadline));
        let released_bytes = {
            let Some(task) = state.tasks.get_mut(&handle) else {
                continue;
            };
            if task.apply_claimed
                || !matches!(
                    task.phase,
                    ZrRuntimeOperationPhase::Queued
                        | ZrRuntimeOperationPhase::Preparing
                        | ZrRuntimeOperationPhase::ReadyToApply
                )
            {
                continue;
            }
            let released_bytes = std::mem::replace(&mut task.retained_bytes, 0);
            task.payload = None;
            task.prepared_command = None;
            task.prepared_result = None;
            task.prepared_command_bytes = 0;
            task.prepared_result_bytes = 0;
            task.result = None;
            task.deadline_armed = false;
            task.snapshot_claimed = false;
            task.phase = ZrRuntimeOperationPhase::Expired;
            task.detail_kind = ZrRuntimeOperationDetailKindV2::DeadlineElapsed;
            task.detail_value = elapsed;
            task.terminal_at = Some(now);
            released_bytes
        };
        state.retained_bytes = state
            .retained_bytes
            .checked_sub(released_bytes)
            .expect("expired operation bytes must remain accounted");
    }
}

pub(super) fn expire_terminal_results_in_state(
    state: &mut RuntimeOperationTaskState,
    terminal_result_ttl: Duration,
    now: Instant,
) {
    let expired: Vec<_> = state
        .tasks
        .iter()
        .filter_map(|(handle, task)| {
            let terminal_at = task.terminal_at?;
            (matches!(
                task.phase,
                ZrRuntimeOperationPhase::Completed | ZrRuntimeOperationPhase::Failed
            ) && !task.harvest_in_flight
                && now.saturating_duration_since(terminal_at) >= terminal_result_ttl)
                .then_some((*handle, terminal_at))
        })
        .collect();
    for (handle, terminal_at) in expired {
        let elapsed = duration_millis_u64(now.saturating_duration_since(terminal_at));
        let released_bytes = {
            let Some(task) = state.tasks.get_mut(&handle) else {
                continue;
            };
            if !matches!(
                task.phase,
                ZrRuntimeOperationPhase::Completed | ZrRuntimeOperationPhase::Failed
            ) {
                continue;
            }
            let released_bytes = std::mem::replace(&mut task.retained_bytes, 0);
            task.payload = None;
            task.prepared_command = None;
            task.prepared_result = None;
            task.prepared_command_bytes = 0;
            task.prepared_result_bytes = 0;
            task.result = None;
            task.phase = ZrRuntimeOperationPhase::Expired;
            task.detail_kind = ZrRuntimeOperationDetailKindV2::TerminalTtlElapsed;
            task.detail_value = elapsed;
            released_bytes
        };
        state.retained_bytes = state
            .retained_bytes
            .checked_sub(released_bytes)
            .expect("expired terminal result bytes must remain accounted");
    }
}

pub(super) fn refresh_operation_maintenance_alarm(
    state: &Arc<Mutex<RuntimeOperationTaskState>>,
    refresh: &Arc<Mutex<()>>,
    terminal_result_ttl: Duration,
) -> Result<(), ()> {
    let _refresh = refresh
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let (deadline, generation, previous_subscription) = {
        let mut state_guard = lock_operation_state(state);
        let deadline = next_maintenance_deadline(&state_guard, terminal_result_ttl);
        if state_guard.maintenance_deadline == deadline {
            return Ok(());
        }
        state_guard.maintenance_generation = state_guard.maintenance_generation.wrapping_add(1);
        state_guard.maintenance_deadline = None;
        (
            deadline,
            state_guard.maintenance_generation,
            state_guard.maintenance_subscription.take(),
        )
    };
    drop(previous_subscription);

    let Some(deadline) = deadline else {
        return Ok(());
    };
    let timer = TaskTimer::process_default().map_err(|_| ())?;
    let weak_state = Arc::downgrade(state);
    let refresh_for_callback = Arc::clone(refresh);
    let subscription = timer
        .schedule_at(deadline, move || {
            let Some(state) = weak_state.upgrade() else {
                return;
            };
            let previous_subscription = {
                let mut state_guard = lock_operation_state(&state);
                if state_guard.maintenance_generation != generation {
                    return;
                }
                state_guard.maintenance_deadline = None;
                let previous_subscription = state_guard.maintenance_subscription.take();
                let now = Instant::now();
                expire_due_deadlines_in_state(&mut state_guard, now);
                expire_terminal_results_in_state(&mut state_guard, terminal_result_ttl, now);
                previous_subscription
            };
            drop(previous_subscription);
            let _ = refresh_operation_maintenance_alarm(
                &state,
                &refresh_for_callback,
                terminal_result_ttl,
            );
        })
        .map_err(|_| ())?;
    let mut state_guard = lock_operation_state(state);
    if state_guard.maintenance_generation == generation {
        state_guard.maintenance_deadline = Some(deadline);
        state_guard.maintenance_subscription = Some(subscription);
    }
    Ok(())
}

pub(super) fn lock_operation_state(
    state: &Arc<Mutex<RuntimeOperationTaskState>>,
) -> MutexGuard<'_, RuntimeOperationTaskState> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn duration_millis_u64(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn next_maintenance_deadline(
    state: &RuntimeOperationTaskState,
    terminal_result_ttl: Duration,
) -> Option<Instant> {
    state
        .tasks
        .values()
        .filter_map(|task| {
            (matches!(
                task.phase,
                ZrRuntimeOperationPhase::Queued
                    | ZrRuntimeOperationPhase::Preparing
                    | ZrRuntimeOperationPhase::ReadyToApply
            ) && !task.apply_claimed)
                .then_some(task.deadline)
                .flatten()
        })
        .chain(state.tasks.values().filter_map(|task| {
            (matches!(
                task.phase,
                ZrRuntimeOperationPhase::Completed | ZrRuntimeOperationPhase::Failed
            ))
            .then(|| {
                task.terminal_at
                    .and_then(|terminal_at| terminal_at.checked_add(terminal_result_ttl))
                    .or(task.terminal_at)
            })
            .flatten()
        }))
        .min()
}
