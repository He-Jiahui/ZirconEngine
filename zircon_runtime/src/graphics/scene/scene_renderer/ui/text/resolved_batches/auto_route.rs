use std::collections::{HashMap, HashSet, VecDeque};

use zircon_runtime_interface::ui::surface::UiTextRenderMode;

use super::super::super::render::{ScreenSpaceUiTextBatch, ScreenSpaceUiTextRouteIdentity};
use super::super::font_assets::{LoadedUiFontAsset, effective_text_render_mode};
use super::auto_text_policy_request;
use crate::text::raster::{GlyphRasterPath, auto_raster_path_for_request};

const AUTO_TEXT_ROUTE_CAPACITY: usize = 2_048;
const AUTO_TEXT_ROUTE_MAX_IDLE_FRAMES: u64 = 300;
const AUTO_TEXT_ROUTE_RECENCY_COMPACTION_FACTOR: usize = 4;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer::ui::text) struct AutoTextRasterRouteFrameReport {
    pub(in crate::graphics::scene::scene_renderer::ui::text) capacity: usize,
    pub(in crate::graphics::scene::scene_renderer::ui::text) entry_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui::text) generation_cache_hit_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui::text) policy_evaluation_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui::text) retained_warm_route_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui::text) route_switch_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui::text) capacity_eviction_count: usize,
    pub(in crate::graphics::scene::scene_renderer::ui::text) idle_eviction_count: usize,
}

#[derive(Clone, Copy, Debug)]
struct AutoTextRasterRouteEntry {
    command_generation: u64,
    mode: UiTextRenderMode,
    last_seen_frame: u64,
    recency_token: u64,
}

#[derive(Clone, Debug)]
struct AutoTextRasterRouteRecency {
    identity: ScreenSpaceUiTextRouteIdentity,
    token: u64,
}

pub(in crate::graphics::scene::scene_renderer::ui::text) struct AutoTextRasterRouter {
    capacity: usize,
    frame_index: u64,
    next_recency_token: u64,
    entries: HashMap<ScreenSpaceUiTextRouteIdentity, AutoTextRasterRouteEntry>,
    active_routes: HashSet<ScreenSpaceUiTextRouteIdentity>,
    recency: VecDeque<AutoTextRasterRouteRecency>,
    frame_report: AutoTextRasterRouteFrameReport,
}

impl Default for AutoTextRasterRouter {
    fn default() -> Self {
        Self::with_capacity(AUTO_TEXT_ROUTE_CAPACITY)
    }
}

impl AutoTextRasterRouter {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity,
            frame_index: 0,
            next_recency_token: 0,
            entries: HashMap::new(),
            active_routes: HashSet::new(),
            recency: VecDeque::new(),
            frame_report: AutoTextRasterRouteFrameReport::default(),
        }
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::ui::text) fn with_capacity_for_test(
        capacity: usize,
    ) -> Self {
        Self::with_capacity(capacity)
    }

    pub(in crate::graphics::scene::scene_renderer::ui::text) fn begin_frame(&mut self) {
        self.frame_index = self.frame_index.saturating_add(1).max(1);
        self.frame_report = AutoTextRasterRouteFrameReport {
            capacity: self.capacity,
            ..AutoTextRasterRouteFrameReport::default()
        };
        self.evict_idle_routes();
        self.compact_recency_if_needed();
        self.frame_report.entry_count = self.entries.len();
    }

    pub(in crate::graphics::scene::scene_renderer::ui::text) fn replace_active_routes(
        &mut self,
        routes: impl IntoIterator<Item = ScreenSpaceUiTextRouteIdentity>,
    ) {
        let mut active_routes = HashSet::with_capacity(self.entries.len().min(self.capacity));
        active_routes.extend(
            routes
                .into_iter()
                .filter(|identity| self.entries.contains_key(identity)),
        );
        self.active_routes = active_routes;
    }

    pub(in crate::graphics::scene::scene_renderer::ui::text) fn clear_active_routes(&mut self) {
        self.active_routes.clear();
    }

    pub(in crate::graphics::scene::scene_renderer::ui::text) fn resolve(
        &mut self,
        text: &ScreenSpaceUiTextBatch,
        font_asset: Option<&LoadedUiFontAsset>,
    ) -> UiTextRenderMode {
        let font_mode = effective_text_render_mode(UiTextRenderMode::Auto, font_asset);
        if font_asset
            .and_then(|asset| asset.render_mode)
            .is_some_and(|mode| !matches!(mode, UiTextRenderMode::Auto))
        {
            return font_mode;
        }

        if let Some(entry) = self.entries.get(&text.route_identity).copied() {
            if entry.command_generation == text.command_generation {
                self.frame_report.generation_cache_hit_count = self
                    .frame_report
                    .generation_cache_hit_count
                    .saturating_add(1);
                self.touch(text.route_identity.clone(), entry);
                return entry.mode;
            }
        }

        let previous_mode = self
            .entries
            .get(&text.route_identity)
            .map(|entry| entry.mode);
        let decision = auto_raster_path_for_request(
            auto_text_policy_request(text),
            previous_mode.and_then(glyph_raster_path_for_mode),
        );
        let mode = text_render_mode_for_path(decision.path);
        self.frame_report.policy_evaluation_count =
            self.frame_report.policy_evaluation_count.saturating_add(1);
        self.frame_report.retained_warm_route_count = self
            .frame_report
            .retained_warm_route_count
            .saturating_add(usize::from(decision.retained_warm_path));
        self.frame_report.route_switch_count =
            self.frame_report
                .route_switch_count
                .saturating_add(usize::from(
                    previous_mode.is_some_and(|previous| previous != mode),
                ));

        if self.capacity == 0 {
            return mode;
        }
        if !self.entries.contains_key(&text.route_identity) {
            self.evict_for_capacity();
        }
        let entry = AutoTextRasterRouteEntry {
            command_generation: text.command_generation,
            mode,
            last_seen_frame: self.frame_index,
            recency_token: self.next_token(),
        };
        self.entries.insert(text.route_identity.clone(), entry);
        self.recency.push_back(AutoTextRasterRouteRecency {
            identity: text.route_identity.clone(),
            token: entry.recency_token,
        });
        self.frame_report.entry_count = self.entries.len();
        mode
    }

    pub(in crate::graphics::scene::scene_renderer::ui::text) fn frame_report(
        &self,
    ) -> AutoTextRasterRouteFrameReport {
        AutoTextRasterRouteFrameReport {
            entry_count: self.entries.len(),
            ..self.frame_report
        }
    }

    fn touch(
        &mut self,
        identity: ScreenSpaceUiTextRouteIdentity,
        mut entry: AutoTextRasterRouteEntry,
    ) {
        entry.last_seen_frame = self.frame_index;
        entry.recency_token = self.next_token();
        self.entries.insert(identity.clone(), entry);
        self.recency.push_back(AutoTextRasterRouteRecency {
            identity,
            token: entry.recency_token,
        });
    }

    fn evict_for_capacity(&mut self) {
        while self.entries.len() >= self.capacity {
            let Some(recency) = self.recency.pop_front() else {
                self.entries.clear();
                self.active_routes.clear();
                break;
            };
            if self.entry_matches_recency(&recency) {
                self.entries.remove(&recency.identity);
                self.active_routes.remove(&recency.identity);
                self.frame_report.capacity_eviction_count =
                    self.frame_report.capacity_eviction_count.saturating_add(1);
            }
        }
    }

    fn evict_idle_routes(&mut self) {
        while let Some(recency) = self.recency.front().cloned() {
            if !self.entry_matches_recency(&recency) {
                self.recency.pop_front();
                continue;
            }
            let Some(entry) = self.entries.get(&recency.identity) else {
                self.recency.pop_front();
                continue;
            };
            if self.frame_index.saturating_sub(entry.last_seen_frame)
                <= AUTO_TEXT_ROUTE_MAX_IDLE_FRAMES
            {
                break;
            }
            if self.active_routes.contains(&recency.identity) {
                break;
            }
            self.recency.pop_front();
            self.entries.remove(&recency.identity);
            self.active_routes.remove(&recency.identity);
            self.frame_report.idle_eviction_count =
                self.frame_report.idle_eviction_count.saturating_add(1);
        }
    }

    fn compact_recency_if_needed(&mut self) {
        let max_recency_len = self
            .capacity
            .saturating_mul(AUTO_TEXT_ROUTE_RECENCY_COMPACTION_FACTOR)
            .max(1);
        if self.recency.len() <= max_recency_len {
            return;
        }
        self.recency = compact_live_recency(&self.entries);
    }

    fn entry_matches_recency(&self, recency: &AutoTextRasterRouteRecency) -> bool {
        self.entries
            .get(&recency.identity)
            .is_some_and(|entry| entry.recency_token == recency.token)
    }

    fn next_token(&mut self) -> u64 {
        self.next_recency_token = self.next_recency_token.saturating_add(1).max(1);
        self.next_recency_token
    }
}

fn compact_live_recency(
    entries: &HashMap<ScreenSpaceUiTextRouteIdentity, AutoTextRasterRouteEntry>,
) -> VecDeque<AutoTextRasterRouteRecency> {
    let mut live = entries
        .iter()
        .map(|(identity, entry)| ((entry.last_seen_frame, entry.recency_token), identity))
        .collect::<Vec<_>>();
    live.sort_by_key(|(recency_key, _)| *recency_key);
    live.into_iter()
        .map(|((_, token), identity)| AutoTextRasterRouteRecency {
            identity: identity.clone(),
            token,
        })
        .collect()
}

fn glyph_raster_path_for_mode(mode: UiTextRenderMode) -> Option<GlyphRasterPath> {
    match mode {
        UiTextRenderMode::Auto => None,
        UiTextRenderMode::Native => Some(GlyphRasterPath::Bitmap),
        UiTextRenderMode::Sdf => Some(GlyphRasterPath::Sdf),
        UiTextRenderMode::Msdf => Some(GlyphRasterPath::Msdf),
        UiTextRenderMode::Mtsdf => Some(GlyphRasterPath::Mtsdf),
    }
}

fn text_render_mode_for_path(path: GlyphRasterPath) -> UiTextRenderMode {
    match path {
        GlyphRasterPath::Bitmap => UiTextRenderMode::Native,
        GlyphRasterPath::Sdf => UiTextRenderMode::Sdf,
        GlyphRasterPath::Msdf => UiTextRenderMode::Msdf,
        GlyphRasterPath::Mtsdf => UiTextRenderMode::Mtsdf,
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use zircon_runtime_interface::ui::event_ui::UiNodeId;

    use super::*;

    fn route_identity(index: u64) -> ScreenSpaceUiTextRouteIdentity {
        ScreenSpaceUiTextRouteIdentity::new(
            format!("runtime.ui.auto-route.{index:05}"),
            UiNodeId::new(index),
            None,
        )
    }

    fn route_entry(last_seen_frame: u64, recency_token: u64) -> AutoTextRasterRouteEntry {
        AutoTextRasterRouteEntry {
            command_generation: recency_token,
            mode: UiTextRenderMode::Native,
            last_seen_frame,
            recency_token,
        }
    }

    #[test]
    fn optimization_batch_20260826h_runtime11c_recency_projection_preserves_frame_token_order() {
        let oldest = route_identity(7);
        let same_frame_earlier = route_identity(2);
        let same_frame_later = route_identity(9);
        let entries = [
            (same_frame_later.clone(), route_entry(11, 9)),
            (oldest.clone(), route_entry(3, 7)),
            (same_frame_earlier.clone(), route_entry(11, 4)),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();

        let compacted = compact_live_recency(&entries)
            .into_iter()
            .map(|recency| (recency.identity, recency.token))
            .collect::<Vec<_>>();

        assert_eq!(
            compacted,
            vec![(oldest, 7), (same_frame_earlier, 4), (same_frame_later, 9)]
        );
    }

    #[test]
    fn optimization_batch_20260826h_runtime11c_recency_sort_uses_cached_projection() {
        let source = include_str!("auto_route.rs");
        let compaction = source
            .split("fn compact_recency_if_needed")
            .nth(1)
            .expect("recency compaction method")
            .split("fn entry_matches_recency")
            .next()
            .expect("bounded recency compaction method");
        let projection = source
            .split("fn compact_live_recency")
            .nth(1)
            .expect("cached recency projection")
            .split("fn glyph_raster_path_for_mode")
            .next()
            .expect("bounded cached recency projection");

        assert!(compaction.contains("compact_live_recency(&self.entries)"));
        assert!(!compaction.contains(".get(&recency.identity)"));
        assert!(projection.contains("entry.last_seen_frame, entry.recency_token"));
        assert!(projection.contains("identity: identity.clone()"));
    }

    #[test]
    fn active_cached_frame_route_survives_idle_eviction_without_per_frame_touch() {
        let identity = route_identity(17);
        let absent_identity = route_identity(18);
        let mut router = AutoTextRasterRouter::with_capacity_for_test(2);
        router.entries.insert(identity.clone(), route_entry(0, 1));
        router.recency.push_back(AutoTextRasterRouteRecency {
            identity: identity.clone(),
            token: 1,
        });
        router.replace_active_routes([identity.clone(), absent_identity]);
        assert_eq!(router.active_routes.len(), 1);

        for _ in 0..=AUTO_TEXT_ROUTE_MAX_IDLE_FRAMES {
            router.begin_frame();
        }

        assert!(router.entries.contains_key(&identity));
        assert_eq!(router.frame_report().idle_eviction_count, 0);

        router.clear_active_routes();
        router.begin_frame();

        assert!(!router.entries.contains_key(&identity));
        assert_eq!(router.frame_report().idle_eviction_count, 1);
    }

    #[test]
    #[ignore = "release performance evidence; run through the validation coordinator"]
    fn optimization_batch_20260826h_runtime11c_recency_sort_projection_performance_evidence() {
        fn legacy_compact(
            entries: &HashMap<ScreenSpaceUiTextRouteIdentity, AutoTextRasterRouteEntry>,
        ) -> VecDeque<AutoTextRasterRouteRecency> {
            let mut live = entries
                .iter()
                .map(|(identity, entry)| AutoTextRasterRouteRecency {
                    identity: identity.clone(),
                    token: entry.recency_token,
                })
                .collect::<Vec<_>>();
            live.sort_by_key(|recency| {
                entries
                    .get(&recency.identity)
                    .map(|entry| (entry.last_seen_frame, entry.recency_token))
                    .unwrap_or_default()
            });
            live.into()
        }

        let entries = (0..4_096_u64)
            .rev()
            .map(|index| {
                (
                    route_identity(index),
                    route_entry(index % 257, index.saturating_add(1)),
                )
            })
            .collect::<HashMap<_, _>>();
        let mut legacy_samples = Vec::with_capacity(17);
        let mut projected_samples = Vec::with_capacity(17);
        for _ in 0..17 {
            let started = Instant::now();
            black_box(legacy_compact(black_box(&entries)));
            legacy_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(compact_live_recency(black_box(&entries)));
            projected_samples.push(started.elapsed().as_nanos());
        }

        legacy_samples.sort_unstable();
        projected_samples.sort_unstable();
        let legacy_p95 = legacy_samples[16];
        let projected_p95 = projected_samples[16];
        println!(
            "RUNTIME11C_AUTO_TEXT_RECENCY_SORT_PROJECTION_BENCH_V1 entries={} legacy_p95_ns={} projected_p95_ns={} legacy_sort_map_lookup_path=1 projected_sort_map_lookup_path=0 identity_clones_before={} identity_clones_after={} target_ratio_bp=6000",
            entries.len(),
            legacy_p95,
            projected_p95,
            entries.len(),
            entries.len(),
        );
        assert!(
            projected_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(6_000),
            "projected recency sort P95 {projected_p95} ns exceeded 60% of legacy {legacy_p95} ns"
        );
    }
}
