use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use zircon_runtime::core::framework::channel::ChannelWakeCallback;

use super::super::{HostPaintImagePixels, RasterTargetSize};
use super::cache::{
    begin_visual_asset_source_load, finish_visual_asset_source_load,
    store_visual_asset_pixels_if_snapshot, VisualAssetSourceSnapshot,
};
use super::pixels::load_visual_asset_pixels_uncached;
use crate::core::jobs::{
    EditorJob, EditorJobSpec, EditorJobSystem, JobCategory, JobContext, JobError, JobPriority,
};
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::frame_geometry::union_optional_frames;
use crate::ui::retained_host::ui_perf::{
    current_ui_perf_scenario, record_ui_perf_counter, UiPerfCounter, UiPerfScenario,
};

pub(super) enum VisualAssetLoadSchedule {
    Synchronous(Vec<PathBuf>),
    Deferred,
}

struct VisualAssetLoadRequest {
    key: String,
    base_key: String,
    candidates: Vec<PathBuf>,
    target: Option<RasterTargetSize>,
    tint: Option<[u8; 4]>,
    binding_epoch: u64,
    scenario: UiPerfScenario,
}

struct VisualAssetLoadJob {
    request: VisualAssetLoadRequest,
    source_snapshot: Option<VisualAssetSourceSnapshot>,
    completion_armed: bool,
}

struct VisualAssetPendingLoad {
    binding_epoch: u64,
    damage_frame: Option<FrameRect>,
}

pub(in crate::ui::retained_host) struct VisualAssetLoadCompletion {
    pub(in crate::ui::retained_host) scenario: UiPerfScenario,
    pub(in crate::ui::retained_host) damage_frame: Option<FrameRect>,
}

impl EditorJob for VisualAssetLoadJob {
    type Output = ();

    fn run(mut self, context: JobContext) -> Result<Self::Output, JobError> {
        context.check_cancelled()?;
        let source_paths = self.request.candidates.clone();
        self.source_snapshot = Some(begin_visual_asset_source_load(
            &self.request.base_key,
            &source_paths,
        ));
        let pixels = load_visual_asset_pixels_uncached(
            self.request.candidates.clone(),
            &self.request.base_key,
            self.request.target,
            self.request.tint,
        );
        context.check_cancelled()?;
        let stored = store_visual_asset_pixels_for_current_binding(
            self.request.binding_epoch,
            self.source_snapshot
                .take()
                .expect("visual asset source snapshot must precede background loading"),
            self.request.key.clone(),
            source_paths,
            pixels,
        );
        let counter = if stored {
            UiPerfCounter::VisualAssetAsyncCompletedCount
        } else {
            UiPerfCounter::VisualAssetAsyncStaleDiscardCount
        };
        record_ui_perf_counter(self.request.scenario, counter, 1.0);
        complete_visual_asset_load(
            &self.request.key,
            self.request.binding_epoch,
            self.request.scenario,
        );
        self.completion_armed = false;
        Ok(())
    }
}

impl Drop for VisualAssetLoadJob {
    fn drop(&mut self) {
        if let Some(snapshot) = self.source_snapshot.take() {
            finish_visual_asset_source_load(snapshot);
        }
        if self.completion_armed {
            release_pending_key(&self.request.key, self.request.binding_epoch);
        }
    }
}

#[derive(Default)]
struct VisualAssetLoadScheduler {
    jobs: Option<EditorJobSystem>,
    completion_wake: Option<ChannelWakeCallback>,
    pending_keys: BTreeMap<String, VisualAssetPendingLoad>,
    binding_epoch: u64,
    completed: Option<VisualAssetLoadCompletion>,
}

impl VisualAssetLoadScheduler {
    fn binding_is_current(&self, binding_epoch: u64) -> bool {
        self.binding_epoch == binding_epoch
    }

    fn reserve_pending_key(
        &mut self,
        key: &str,
        binding_epoch: u64,
        damage_frame: Option<FrameRect>,
    ) -> bool {
        if let Some(pending) = self.pending_keys.get_mut(key) {
            if pending.binding_epoch == binding_epoch {
                merge_pending_damage_frame(&mut pending.damage_frame, damage_frame);
                return false;
            }
        }
        self.pending_keys.insert(
            key.to_owned(),
            VisualAssetPendingLoad {
                binding_epoch,
                damage_frame,
            },
        );
        true
    }

    fn release_pending_key(
        &mut self,
        key: &str,
        binding_epoch: u64,
    ) -> Option<VisualAssetPendingLoad> {
        if self
            .pending_keys
            .get(key)
            .is_some_and(|pending| pending.binding_epoch == binding_epoch)
        {
            return self.pending_keys.remove(key);
        }
        None
    }
}

fn merge_pending_damage_frame(current: &mut Option<FrameRect>, additional: Option<FrameRect>) {
    *current = match (current.take(), additional) {
        (Some(current), Some(additional)) => union_optional_frames(Some(current), Some(additional)),
        _ => None,
    };
}

pub(in crate::ui::retained_host) fn bind_visual_asset_loader(
    jobs: EditorJobSystem,
    completion_wake: ChannelWakeCallback,
) -> u64 {
    let mut scheduler = lock_scheduler();
    scheduler.binding_epoch = scheduler.binding_epoch.saturating_add(1).max(1);
    scheduler.jobs = Some(jobs);
    scheduler.completion_wake = Some(completion_wake);
    scheduler.pending_keys.clear();
    scheduler.completed = None;
    scheduler.binding_epoch
}

pub(in crate::ui::retained_host) fn unbind_visual_asset_loader(binding_epoch: u64) {
    let mut scheduler = lock_scheduler();
    if !scheduler.binding_is_current(binding_epoch) {
        return;
    }
    scheduler.binding_epoch = scheduler.binding_epoch.saturating_add(1).max(1);
    scheduler.jobs = None;
    scheduler.completion_wake = None;
    scheduler.pending_keys.clear();
    scheduler.completed = None;
}

pub(in crate::ui::retained_host) fn take_visual_asset_completion(
) -> Option<VisualAssetLoadCompletion> {
    lock_scheduler().completed.take()
}

pub(super) fn schedule_visual_asset_load(
    key: &str,
    base_key: &str,
    candidates: impl FnOnce() -> Vec<PathBuf>,
    target: Option<RasterTargetSize>,
    tint: Option<[u8; 4]>,
    damage_frame: Option<FrameRect>,
) -> VisualAssetLoadSchedule {
    let scenario = current_ui_perf_scenario();
    let (jobs, binding_epoch) = {
        let mut scheduler = lock_scheduler();
        let Some(jobs) = scheduler.jobs.clone() else {
            return VisualAssetLoadSchedule::Synchronous(candidates());
        };
        let binding_epoch = scheduler.binding_epoch;
        if !scheduler.reserve_pending_key(key, binding_epoch, damage_frame) {
            record_ui_perf_counter(
                scenario,
                UiPerfCounter::VisualAssetAsyncDeduplicatedCount,
                1.0,
            );
            return VisualAssetLoadSchedule::Deferred;
        }
        (jobs, binding_epoch)
    };

    zircon_runtime::profile_counter!("editor", "ui.visual_asset_cache.candidate_build_count", 1);
    record_ui_perf_counter(
        scenario,
        UiPerfCounter::VisualAssetCacheCandidateBuildCount,
        1.0,
    );
    let candidates = candidates();
    let estimated_bytes = key
        .len()
        .saturating_add(base_key.len())
        .saturating_add(
            candidates
                .iter()
                .map(|path| path.as_os_str().len())
                .sum::<usize>(),
        )
        .max(1);
    let request = VisualAssetLoadRequest {
        key: key.to_owned(),
        base_key: base_key.to_owned(),
        candidates,
        target,
        tint,
        binding_epoch,
        scenario,
    };
    let spec = EditorJobSpec::new(
        format!("Materialize UI visual asset {base_key}"),
        JobCategory::Thumbnail,
    )
    .with_priority(JobPriority::Background)
    .with_estimated_bytes(estimated_bytes);
    let submitted = jobs.submit(
        spec,
        VisualAssetLoadJob {
            request,
            source_snapshot: None,
            completion_armed: true,
        },
    );
    if submitted.is_err() {
        release_pending_key(key, binding_epoch);
        record_ui_perf_counter(
            scenario,
            UiPerfCounter::VisualAssetAsyncSubmissionRejectedCount,
            1.0,
        );
    } else {
        record_ui_perf_counter(scenario, UiPerfCounter::VisualAssetAsyncEnqueuedCount, 1.0);
    }
    VisualAssetLoadSchedule::Deferred
}

fn complete_visual_asset_load(key: &str, binding_epoch: u64, scenario: UiPerfScenario) {
    let completion_wake = {
        let mut scheduler = lock_scheduler();
        let Some(pending) = scheduler.release_pending_key(key, binding_epoch) else {
            return;
        };
        if scheduler.binding_epoch != binding_epoch {
            return;
        }
        let mut damage_frame = pending.damage_frame;
        if let Some(completed) = scheduler.completed.take() {
            merge_pending_damage_frame(&mut damage_frame, completed.damage_frame);
        }
        scheduler.completed = Some(VisualAssetLoadCompletion {
            scenario,
            damage_frame,
        });
        scheduler.completion_wake.clone()
    };
    if let Some(completion_wake) = completion_wake {
        completion_wake();
    }
}

fn store_visual_asset_pixels_for_current_binding(
    binding_epoch: u64,
    source_snapshot: VisualAssetSourceSnapshot,
    key: String,
    source_paths: Vec<PathBuf>,
    pixels: Option<HostPaintImagePixels>,
) -> bool {
    let scheduler = lock_scheduler();
    if !scheduler.binding_is_current(binding_epoch) {
        drop(scheduler);
        finish_visual_asset_source_load(source_snapshot);
        return false;
    }
    store_visual_asset_pixels_if_snapshot(source_snapshot, key, source_paths, pixels)
}

fn release_pending_key(key: &str, binding_epoch: u64) {
    let _ = lock_scheduler().release_pending_key(key, binding_epoch);
}

fn lock_scheduler() -> std::sync::MutexGuard<'static, VisualAssetLoadScheduler> {
    VISUAL_ASSET_LOAD_SCHEDULER
        .get_or_init(|| Mutex::new(VisualAssetLoadScheduler::default()))
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
}

static VISUAL_ASSET_LOAD_SCHEDULER: OnceLock<Mutex<VisualAssetLoadScheduler>> = OnceLock::new();

#[cfg(test)]
mod tests {
    use super::VisualAssetLoadScheduler;
    use crate::ui::retained_host::host_contract::data::FrameRect;

    #[test]
    fn pending_loads_deduplicate_same_key() {
        let mut scheduler = VisualAssetLoadScheduler::default();

        assert!(scheduler.reserve_pending_key("icon:save@24x24", 7, None));
        assert!(!scheduler.reserve_pending_key("icon:save@24x24", 7, None));
        let _ = scheduler.release_pending_key("icon:save@24x24", 7);
        assert!(scheduler.reserve_pending_key("icon:save@24x24", 7, None));
    }

    #[test]
    fn duplicate_pending_loads_union_damage_frames() {
        let mut scheduler = VisualAssetLoadScheduler::default();
        let first = FrameRect {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        };
        let second = FrameRect {
            x: 50.0,
            y: 10.0,
            width: 20.0,
            height: 25.0,
        };

        assert!(scheduler.reserve_pending_key("icon:save@24x24", 7, Some(first)));
        assert!(!scheduler.reserve_pending_key("icon:save@24x24", 7, Some(second)));

        let damage = scheduler
            .pending_keys
            .get("icon:save@24x24")
            .and_then(|pending| pending.damage_frame.as_ref())
            .expect("duplicate visible users should retain a union damage frame");
        assert_eq!(
            damage,
            &FrameRect {
                x: 10.0,
                y: 10.0,
                width: 60.0,
                height: 50.0,
            }
        );
    }

    #[test]
    fn unknown_damage_keeps_the_full_frame_completion_fallback() {
        let mut damage = Some(FrameRect {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        });

        super::merge_pending_damage_frame(&mut damage, None);

        assert!(damage.is_none());
    }

    #[test]
    fn stale_binding_cannot_publish_visual_pixels() {
        let mut scheduler = VisualAssetLoadScheduler::default();
        scheduler.binding_epoch = 9;

        assert!(!scheduler.binding_is_current(8));
        assert!(scheduler.binding_is_current(9));
    }
}
