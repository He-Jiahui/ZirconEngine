use std::collections::{HashMap, VecDeque};

use zircon_runtime_interface::ui::surface::UiTextRenderMode;

use super::super::super::render::{ScreenSpaceUiTextBatch, ScreenSpaceUiTextRouteIdentity};
use super::super::font_assets::{effective_text_render_mode, LoadedUiFontAsset};
use super::auto_text_policy_request;
use crate::text::raster::{auto_raster_path_for_request, GlyphRasterPath};

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
                break;
            };
            if self.entry_matches_recency(&recency) {
                self.entries.remove(&recency.identity);
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
            self.recency.pop_front();
            self.entries.remove(&recency.identity);
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
        let mut live = self
            .entries
            .iter()
            .map(|(identity, entry)| AutoTextRasterRouteRecency {
                identity: identity.clone(),
                token: entry.recency_token,
            })
            .collect::<Vec<_>>();
        live.sort_by_key(|recency| {
            self.entries
                .get(&recency.identity)
                .map(|entry| (entry.last_seen_frame, entry.recency_token))
                .unwrap_or_default()
        });
        self.recency = live.into();
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
