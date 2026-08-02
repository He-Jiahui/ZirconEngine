use super::super::super::*;
use std::time::{Duration, Instant};

mod runtime;
mod startup;

pub(super) struct AssetRefreshEvents {
    pub(super) asset_changes: Vec<AssetChange>,
    pub(super) editor_asset_changes: Vec<EditorAssetChange>,
    pub(super) resource_changes: Vec<ResourceEvent>,
    pub(super) resource_generation_lagged: bool,
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
    }
}
