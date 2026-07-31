use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

const MAX_TRANSIENT_RETRY_ATTEMPTS: u8 = 6;
const BASE_TRANSIENT_RETRY_DELAY: Duration = Duration::from_millis(50);
const MAX_TRANSIENT_RETRY_DELAY: Duration = Duration::from_secs(2);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::ui::host::asset_editor_sessions) struct UiAssetRefreshRequest {
    pub(super) generation: u64,
    pub(super) changed_asset_ids: BTreeSet<String>,
    pub(super) retry_attempt: u8,
}

struct DeferredUiAssetRefreshRetry {
    generation: u64,
    retry_attempt: u8,
    not_before: Instant,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::ui::host::asset_editor_sessions) struct UiAssetRefreshQueueSnapshot {
    pub(in crate::ui::host::asset_editor_sessions) latest_generation: u64,
    pub(in crate::ui::host::asset_editor_sessions) pending_asset_count: usize,
    pub(in crate::ui::host::asset_editor_sessions) active: bool,
    pub(in crate::ui::host::asset_editor_sessions) deferred_retry_count: usize,
    pub(in crate::ui::host::asset_editor_sessions) exhausted_retry_count: u64,
    pub(in crate::ui::host::asset_editor_sessions) superseded_count: u64,
}

#[derive(Default)]
pub(super) struct UiAssetRefreshQueue {
    latest_generation: u64,
    pending_asset_ids: BTreeSet<String>,
    active_generation: Option<u64>,
    deferred_retries: BTreeMap<String, DeferredUiAssetRefreshRetry>,
    exhausted_retry_count: u64,
    superseded_count: u64,
}

impl UiAssetRefreshQueue {
    pub(super) fn enqueue<I>(&mut self, changed_asset_ids: I) -> bool
    where
        I: IntoIterator<Item = String>,
    {
        let changed_asset_ids = changed_asset_ids.into_iter().collect::<BTreeSet<_>>();
        if changed_asset_ids.is_empty() {
            return false;
        }
        for asset_id in &changed_asset_ids {
            self.deferred_retries.remove(asset_id);
        }
        self.latest_generation = self.latest_generation.saturating_add(1);
        self.pending_asset_ids.extend(changed_asset_ids);
        true
    }

    pub(super) fn start_next(&mut self) -> Option<UiAssetRefreshRequest> {
        self.start_next_at(Instant::now())
    }

    pub(super) fn start_next_at(&mut self, now: Instant) -> Option<UiAssetRefreshRequest> {
        if self.active_generation.is_some() {
            return None;
        }
        let (changed_asset_ids, retry_attempt) = if self.pending_asset_ids.is_empty() {
            let retry_attempt = self
                .deferred_retries
                .values()
                .filter(|retry| retry.not_before <= now)
                .map(|retry| (retry.not_before, retry.retry_attempt))
                .min()
                .map(|(_, retry_attempt)| retry_attempt)?;
            let changed_asset_ids = self
                .deferred_retries
                .iter()
                .filter(|(_, retry)| {
                    retry.not_before <= now && retry.retry_attempt == retry_attempt
                })
                .map(|(asset_id, _)| asset_id.clone())
                .collect::<BTreeSet<_>>();
            for asset_id in &changed_asset_ids {
                self.deferred_retries.remove(asset_id);
            }
            (changed_asset_ids, retry_attempt)
        } else {
            (std::mem::take(&mut self.pending_asset_ids), 0)
        };
        let generation = self.latest_generation;
        self.active_generation = Some(generation);
        Some(UiAssetRefreshRequest {
            generation,
            changed_asset_ids,
            retry_attempt,
        })
    }

    pub(super) fn finish(&mut self, request: &UiAssetRefreshRequest) -> bool {
        debug_assert_eq!(self.active_generation, Some(request.generation));
        self.active_generation = None;
        let superseded = request.generation < self.latest_generation;
        if superseded {
            self.superseded_count = self.superseded_count.saturating_add(1);
            self.pending_asset_ids
                .extend(request.changed_asset_ids.iter().cloned());
        }
        superseded
    }

    pub(super) fn defer_retry_at(
        &mut self,
        changed_asset_ids: BTreeSet<String>,
        retry_attempt: u8,
        generation: u64,
        now: Instant,
    ) -> bool {
        let next_attempt = retry_attempt.saturating_add(1);
        if changed_asset_ids.is_empty() {
            return false;
        }
        for asset_id in &changed_asset_ids {
            self.pending_asset_ids.remove(asset_id);
        }
        if next_attempt > MAX_TRANSIENT_RETRY_ATTEMPTS {
            let mut exhausted = 0_u64;
            for asset_id in changed_asset_ids {
                let remove = self
                    .deferred_retries
                    .get(&asset_id)
                    .is_none_or(|retry| generation >= retry.generation);
                if remove {
                    self.deferred_retries.remove(&asset_id);
                    exhausted = exhausted.saturating_add(1);
                }
            }
            self.exhausted_retry_count = self.exhausted_retry_count.saturating_add(exhausted);
            return false;
        }
        let exponent = u32::from(next_attempt.saturating_sub(1));
        let delay = BASE_TRANSIENT_RETRY_DELAY
            .saturating_mul(2_u32.saturating_pow(exponent))
            .min(MAX_TRANSIENT_RETRY_DELAY);
        let not_before = now + delay;
        let mut scheduled = false;
        for asset_id in changed_asset_ids {
            let replace = self.deferred_retries.get(&asset_id).is_none_or(|existing| {
                generation > existing.generation
                    || (generation == existing.generation && next_attempt >= existing.retry_attempt)
            });
            if replace {
                self.deferred_retries.insert(
                    asset_id,
                    DeferredUiAssetRefreshRetry {
                        generation,
                        retry_attempt: next_attempt,
                        not_before,
                    },
                );
                scheduled = true;
            }
        }
        scheduled
    }

    pub(super) fn reset_project_epoch(&mut self) {
        self.latest_generation = self.latest_generation.saturating_add(1);
        self.pending_asset_ids.clear();
        self.active_generation = None;
        self.deferred_retries.clear();
        self.exhausted_retry_count = 0;
    }

    pub(super) fn complete_without_work(&mut self, generation: u64) {
        debug_assert_eq!(self.active_generation, Some(generation));
        self.active_generation = None;
    }

    pub(super) fn snapshot(&self) -> UiAssetRefreshQueueSnapshot {
        UiAssetRefreshQueueSnapshot {
            latest_generation: self.latest_generation,
            pending_asset_count: self.pending_asset_ids.len() + self.deferred_retries.len(),
            active: self.active_generation.is_some(),
            deferred_retry_count: self.deferred_retries.len(),
            exhausted_retry_count: self.exhausted_retry_count,
            superseded_count: self.superseded_count,
        }
    }
}
