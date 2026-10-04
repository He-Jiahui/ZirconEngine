use super::super::super::*;
use std::time::{Duration, Instant};
use zircon_runtime::resource::ResourceEvent;

mod runtime;
mod startup;

pub(super) const MAX_ACCUMULATED_ASSET_REFRESH_EVENTS: usize = 4096;
pub(super) const ASSET_REFRESH_QUIET_PERIOD: Duration = Duration::from_millis(32);
pub(super) const MAX_ASSET_REFRESH_DEFERRAL: Duration = Duration::from_millis(250);

#[derive(Default)]
pub(super) struct AssetRefreshEvents {
    pub(super) asset_changes: Vec<AssetChange>,
    pub(super) editor_asset_changes: Vec<EditorAssetChange>,
    pub(super) resource_changes: Vec<ResourceEvent>,
    pub(super) resource_generation_lagged: bool,
    pub(super) active_scene_reload_requested: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AssetRefreshCommitReason {
    QueueQuiesced,
    Capacity,
    MaxDeferral,
}

#[derive(Default)]
pub(in crate::ui::retained_host::app) struct AssetRefreshAccumulator {
    events: AssetRefreshEvents,
    deferred_since: Option<Instant>,
    last_event_at: Option<Instant>,
    resource_sequence_exhausted: bool,
}

#[derive(Default)]
pub(in crate::ui::retained_host::app) struct AssetRefreshQueueAgeState {
    asset_backlog_since: Option<Instant>,
    editor_backlog_since: Option<Instant>,
    resource_backlog_since: Option<Instant>,
}

impl AssetRefreshQueueAgeState {
    pub(super) fn observe(
        &mut self,
        now: Instant,
        asset_pending: bool,
        editor_pending: bool,
        resource_pending: bool,
    ) -> [Duration; 3] {
        [
            observe_backlog_age(&mut self.asset_backlog_since, now, asset_pending),
            observe_backlog_age(&mut self.editor_backlog_since, now, editor_pending),
            observe_backlog_age(&mut self.resource_backlog_since, now, resource_pending),
        ]
    }
}

fn observe_backlog_age(
    backlog_since: &mut Option<Instant>,
    now: Instant,
    pending: bool,
) -> Duration {
    if !pending {
        *backlog_since = None;
        return Duration::ZERO;
    }
    let since = backlog_since.get_or_insert(now);
    now.saturating_duration_since(*since)
}

impl AssetRefreshEvents {
    pub(super) fn is_empty(&self) -> bool {
        self.asset_changes.is_empty()
            && self.editor_asset_changes.is_empty()
            && self.resource_changes.is_empty()
            && !self.resource_generation_lagged
            && !self.active_scene_reload_requested
    }

    fn event_count(&self) -> usize {
        self.asset_changes.len()
            + self.editor_asset_changes.len()
            + self.resource_changes.len()
            + usize::from(self.active_scene_reload_requested)
    }

    fn append(&mut self, mut next: Self) {
        self.asset_changes.append(&mut next.asset_changes);
        self.editor_asset_changes
            .append(&mut next.editor_asset_changes);
        self.resource_changes.append(&mut next.resource_changes);
        self.resource_generation_lagged |= next.resource_generation_lagged;
        self.active_scene_reload_requested |= next.active_scene_reload_requested;
    }
}

impl AssetRefreshAccumulator {
    pub(super) fn resource_sequence_exhausted(&self) -> bool {
        self.resource_sequence_exhausted
    }

    pub(super) fn latch_resource_sequence_exhaustion(&mut self) -> bool {
        if self.resource_sequence_exhausted {
            return false;
        }
        self.resource_sequence_exhausted = true;
        true
    }

    pub(in crate::ui::retained_host::app) fn request_active_scene_reload(
        &mut self,
        now: Instant,
    ) -> Instant {
        self.events.active_scene_reload_requested = true;
        self.deferred_since.get_or_insert(now);
        self.last_event_at = Some(now);
        self.next_commit_deadline()
            .expect("a requested active-scene reload has a commit deadline")
    }

    pub(super) fn accumulate(
        &mut self,
        events: AssetRefreshEvents,
        backlog_pending: bool,
        now: Instant,
    ) -> Option<AssetRefreshEvents> {
        if !events.is_empty() {
            self.last_event_at = Some(now);
        }
        self.events.append(events);
        if self.events.is_empty() {
            self.deferred_since = None;
            self.last_event_at = None;
            return None;
        }

        let deferred_since = self.deferred_since.get_or_insert(now);
        let deferred_for = now.saturating_duration_since(*deferred_since);
        let quiet_for =
            now.saturating_duration_since(self.last_event_at.unwrap_or(*deferred_since));
        let reason = asset_refresh_commit_reason(
            self.events.event_count(),
            backlog_pending,
            deferred_for,
            quiet_for,
        );
        let Some(reason) = reason else {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.asset_refresh.accumulated_event_count",
                self.events.event_count()
            );
            return None;
        };

        record_commit_reason(reason);
        if self.events.resource_generation_lagged {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.asset_refresh.commit_contains_resource_lag",
                1
            );
        }
        self.deferred_since = None;
        self.last_event_at = None;
        Some(std::mem::take(&mut self.events))
    }

    pub(super) fn next_commit_deadline(&self) -> Option<Instant> {
        if self.events.is_empty() {
            return None;
        }
        let deferred_since = self.deferred_since?;
        let last_event_at = self.last_event_at.unwrap_or(deferred_since);
        Some(
            (last_event_at + ASSET_REFRESH_QUIET_PERIOD)
                .min(deferred_since + MAX_ASSET_REFRESH_DEFERRAL),
        )
    }
}

pub(super) fn asset_refresh_commit_reason(
    event_count: usize,
    backlog_pending: bool,
    deferred_for: Duration,
    quiet_for: Duration,
) -> Option<AssetRefreshCommitReason> {
    if event_count >= MAX_ACCUMULATED_ASSET_REFRESH_EVENTS {
        Some(AssetRefreshCommitReason::Capacity)
    } else if !backlog_pending && deferred_for >= MAX_ASSET_REFRESH_DEFERRAL {
        Some(AssetRefreshCommitReason::MaxDeferral)
    } else if !backlog_pending && quiet_for >= ASSET_REFRESH_QUIET_PERIOD {
        Some(AssetRefreshCommitReason::QueueQuiesced)
    } else {
        None
    }
}

fn record_commit_reason(reason: AssetRefreshCommitReason) {
    match reason {
        AssetRefreshCommitReason::QueueQuiesced => {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.asset_refresh.commit_reason_queue_quiesced",
                1
            );
        }
        AssetRefreshCommitReason::Capacity => {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.asset_refresh.commit_reason_capacity",
                1
            );
        }
        AssetRefreshCommitReason::MaxDeferral => {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.asset_refresh.commit_reason_max_deferral",
                1
            );
        }
    }
}

#[cfg(test)]
#[path = "tests/events_accumulation_policy_tests.rs"]
mod accumulation_policy_tests;
