use std::sync::Arc;
use std::time::{Duration, Instant};
use zircon_runtime::core::resource::ResourceEventTryRecvError;

use super::{AssetRefreshEvents, RetainedEditorHost};

const MAX_ASSET_REFRESH_EVENTS_PER_STREAM: usize = 256;
const ASSET_REFRESH_DRAIN_TIME_BUDGET: Duration = Duration::from_millis(2);
const ASSET_REFRESH_STREAM_TIME_BUDGET: Duration = Duration::from_micros(600);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResourceEventDrainAction {
    ReconcileAndContinue,
    ReconcileAndStop,
    Stop,
}

fn asset_refresh_stream_capacity(pending_event_count: usize) -> usize {
    pending_event_count.min(MAX_ASSET_REFRESH_EVENTS_PER_STREAM)
}

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app::assets::refresh) fn drain_asset_refresh_events(
        &mut self,
    ) -> (AssetRefreshEvents, bool) {
        let drain_started = Instant::now();
        debug_assert!(
            ASSET_REFRESH_STREAM_TIME_BUDGET.saturating_mul(3) <= ASSET_REFRESH_DRAIN_TIME_BUDGET
        );

        let stream_started = Instant::now();
        let mut asset_changes = Vec::with_capacity(asset_refresh_stream_capacity(
            self.asset_change_events.len(),
        ));
        while can_drain_more(stream_started, asset_changes.len()) {
            let Ok(change) = self.asset_change_events.try_recv() else {
                break;
            };
            asset_changes.push(change);
        }

        let stream_started = Instant::now();
        let mut editor_asset_changes = Vec::with_capacity(asset_refresh_stream_capacity(
            self.editor_asset_change_events.pending_len(),
        ));
        let mut editor_delivery_queue_age = Duration::ZERO;
        while can_drain_more(stream_started, editor_asset_changes.len()) {
            let Some(delivery) = self.editor_asset_change_events.try_recv() else {
                break;
            };
            editor_delivery_queue_age = editor_delivery_queue_age.max(delivery.queue_age);
            editor_asset_changes.push(Arc::unwrap_or_clone(delivery.change));
        }

        let stream_started = Instant::now();
        let resource_stream_exhausted =
            self.asset_refresh_accumulator.resource_sequence_exhausted();
        let resource_pending_at_start = if resource_stream_exhausted {
            0
        } else {
            self.resource_change_events.len()
        };
        let mut resource_changes =
            Vec::with_capacity(asset_refresh_stream_capacity(resource_pending_at_start));
        let mut resource_generation_lagged = false;
        while !resource_stream_exhausted && can_drain_more(stream_started, resource_changes.len()) {
            match self.resource_change_events.try_recv() {
                Ok(change) => resource_changes.push(change),
                Err(error) => match resource_event_drain_action(error) {
                    ResourceEventDrainAction::ReconcileAndContinue => {
                        resource_generation_lagged = true;
                    }
                    ResourceEventDrainAction::ReconcileAndStop => {
                        if self
                            .asset_refresh_accumulator
                            .latch_resource_sequence_exhaustion()
                        {
                            resource_generation_lagged = true;
                            zircon_runtime::profile_counter!(
                                "editor",
                                "ui.asset_refresh.resource_sequence_exhausted_count",
                                1
                            );
                        }
                        break;
                    }
                    ResourceEventDrainAction::Stop => break,
                },
            }
        }

        let asset_pending = self.asset_change_events.len();
        let editor_pending = self.editor_asset_change_events.pending_len();
        let resource_pending = if self.asset_refresh_accumulator.resource_sequence_exhausted() {
            0
        } else {
            self.resource_change_events.len()
        };
        let queue_age = self.asset_refresh_queue_age.observe(
            Instant::now(),
            asset_pending > 0,
            editor_pending > 0,
            resource_pending > 0,
        );
        let editor_queue_age = queue_age[1].max(editor_delivery_queue_age);

        record_drain_metrics(
            asset_changes.len(),
            editor_asset_changes.len(),
            resource_changes.len(),
            [asset_pending, editor_pending, resource_pending],
            [queue_age[0], editor_queue_age, queue_age[2]],
            drain_started.elapsed(),
        );

        (
            AssetRefreshEvents {
                asset_changes,
                editor_asset_changes,
                resource_changes,
                resource_generation_lagged,
                active_scene_reload_requested: false,
            },
            asset_pending > 0 || editor_pending > 0 || resource_pending > 0,
        )
    }
}

fn can_drain_more(stream_started: Instant, drained_count: usize) -> bool {
    drained_count < MAX_ASSET_REFRESH_EVENTS_PER_STREAM
        && stream_started.elapsed() < ASSET_REFRESH_STREAM_TIME_BUDGET
}

fn resource_event_drain_action(error: ResourceEventTryRecvError) -> ResourceEventDrainAction {
    match error {
        ResourceEventTryRecvError::Lagged(_) => ResourceEventDrainAction::ReconcileAndContinue,
        ResourceEventTryRecvError::SequenceExhausted => ResourceEventDrainAction::ReconcileAndStop,
        ResourceEventTryRecvError::Empty | ResourceEventTryRecvError::Disconnected => {
            ResourceEventDrainAction::Stop
        }
    }
}

fn record_drain_metrics(
    asset_count: usize,
    editor_count: usize,
    resource_count: usize,
    pending: [usize; 3],
    queue_age: [Duration; 3],
    elapsed: Duration,
) {
    zircon_runtime::profile_counter!("editor", "ui.asset_refresh.asset_change_count", asset_count);
    zircon_runtime::profile_counter!(
        "editor",
        "ui.asset_refresh.editor_change_count",
        editor_count
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.asset_refresh.resource_change_count",
        resource_count
    );
    zircon_runtime::profile_counter!("editor", "ui.asset_refresh.asset_pending_count", pending[0]);
    zircon_runtime::profile_counter!(
        "editor",
        "ui.asset_refresh.editor_pending_count",
        pending[1]
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.asset_refresh.resource_pending_count",
        pending[2]
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.asset_refresh.asset_queue_age_us",
        duration_micros(queue_age[0])
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.asset_refresh.editor_queue_age_us",
        duration_micros(queue_age[1])
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.asset_refresh.resource_queue_age_us",
        duration_micros(queue_age[2])
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.asset_refresh.drain_elapsed_us",
        duration_micros(elapsed)
    );
}

fn duration_micros(duration: Duration) -> usize {
    duration.as_micros().min(usize::MAX as u128) as usize
}

#[cfg(test)]
#[path = "tests/runtime.rs"]
mod tests;

#[cfg(test)]
#[path = "runtime/tests/capacity_tests.rs"]
mod capacity_tests;
