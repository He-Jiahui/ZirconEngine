use std::sync::Arc;

use zircon_runtime_interface::ui::surface::UiTextRenderMode;

use super::super::render::{
    text_advances::refresh_renderer_fallback_text_batch_glyphs, ScreenSpaceUiTextBatch,
};
use super::font_assets::{effective_text_render_mode, UiFontAssetCache};
use crate::text::font::{FontCollectionRevision, FontCollectionService};
use crate::text::raster::{
    raster_path_for_request, GlyphRasterEffects, GlyphRasterPath, GlyphRasterPolicyRequest,
};
use crate::text::TextLayoutFallbackReport;

#[path = "resolved_batches/auto_route.rs"]
mod auto_route;

pub(super) use auto_route::{AutoTextRasterRouteFrameReport, AutoTextRasterRouter};

#[derive(Clone, Debug, Default)]
pub(super) struct ResolvedScreenSpaceUiTextBatches {
    pub(super) native_texts: Vec<ScreenSpaceUiTextBatch>,
    pub(super) sdf_texts: Vec<ScreenSpaceUiTextBatch>,
    font_faces_changed: bool,
    auto_route: AutoTextRasterRouteFrameReport,
    post_layout_stale_artifact_batch_rejection_count: usize,
}

impl ResolvedScreenSpaceUiTextBatches {
    pub(super) fn from_explicit_batches(
        native_texts: &[ScreenSpaceUiTextBatch],
        sdf_texts: &[ScreenSpaceUiTextBatch],
    ) -> Self {
        Self {
            native_texts: native_texts.to_vec(),
            sdf_texts: sdf_texts.to_vec(),
            font_faces_changed: false,
            auto_route: AutoTextRasterRouteFrameReport::default(),
            post_layout_stale_artifact_batch_rejection_count: 0,
        }
    }

    pub(super) fn push_resolved_auto_text(
        &mut self,
        text: ScreenSpaceUiTextBatch,
        resolved_mode: UiTextRenderMode,
    ) {
        match resolved_mode {
            UiTextRenderMode::Auto | UiTextRenderMode::Native => self.native_texts.push(text),
            UiTextRenderMode::Sdf | UiTextRenderMode::Msdf | UiTextRenderMode::Mtsdf => {
                self.sdf_texts.push(text)
            }
        }
    }

    pub(super) fn native_texts(&self) -> &[ScreenSpaceUiTextBatch] {
        &self.native_texts
    }

    pub(super) fn sdf_texts(&self) -> &[ScreenSpaceUiTextBatch] {
        &self.sdf_texts
    }

    pub(super) fn font_faces_changed(&self) -> bool {
        self.font_faces_changed
    }

    pub(super) fn auto_route_report(&self) -> AutoTextRasterRouteFrameReport {
        self.auto_route
    }

    pub(super) fn layout_fallback_report(&self) -> TextLayoutFallbackReport {
        let mut report = TextLayoutFallbackReport::default();
        for error in self
            .native_texts
            .iter()
            .chain(self.sdf_texts.iter())
            .filter_map(|text| text.layout_error.as_ref())
        {
            report.record(error);
        }
        report
    }

    pub(super) fn post_layout_stale_artifact_batch_rejection_count(&self) -> usize {
        self.post_layout_stale_artifact_batch_rejection_count
    }

    fn reconcile_after_font_load(
        &mut self,
        shaping_changed: bool,
        font_revision: FontCollectionRevision,
        font_collection: &Arc<FontCollectionService>,
    ) {
        let native_rejections = reconcile_batch_set_after_font_load(
            &mut self.native_texts,
            shaping_changed,
            font_revision,
            font_collection,
        );
        let sdf_rejections = reconcile_batch_set_after_font_load(
            &mut self.sdf_texts,
            shaping_changed,
            font_revision,
            font_collection,
        );
        self.post_layout_stale_artifact_batch_rejection_count = self
            .post_layout_stale_artifact_batch_rejection_count
            .saturating_add(native_rejections)
            .saturating_add(sdf_rejections);
    }
}

fn reconcile_batch_set_after_font_load(
    texts: &mut Vec<ScreenSpaceUiTextBatch>,
    shaping_changed: bool,
    font_revision: FontCollectionRevision,
    font_collection: &Arc<FontCollectionService>,
) -> usize {
    let mut stale_artifact_rejection_count = 0usize;
    texts.retain_mut(|text| {
        if text.glyph_artifact_line.as_ref().is_some_and(|line| {
            line.artifact.font_lease.revision() != font_revision
                || line.font_generation != font_revision.generation()
                || line.artifact.font_generation != font_revision.generation()
        }) {
            stale_artifact_rejection_count = stale_artifact_rejection_count.saturating_add(1);
            return false;
        }
        if text.glyph_artifact_line.is_none() && (shaping_changed || text.shaped_glyphs.is_empty())
        {
            refresh_renderer_fallback_text_batch_glyphs(text, font_collection);
        }
        true
    });
    stale_artifact_rejection_count
}

pub(super) fn resolved_auto_text_render_mode(
    text: &ScreenSpaceUiTextBatch,
    font_asset: Option<&super::font_assets::LoadedUiFontAsset>,
) -> UiTextRenderMode {
    let resolved_font_mode = effective_text_render_mode(UiTextRenderMode::Auto, font_asset);
    if font_asset
        .and_then(|asset| asset.render_mode)
        .is_some_and(|mode| !matches!(mode, UiTextRenderMode::Auto))
    {
        return resolved_font_mode;
    }

    match raster_path_for_request(auto_text_policy_request(text)) {
        GlyphRasterPath::Bitmap => UiTextRenderMode::Native,
        GlyphRasterPath::Sdf => UiTextRenderMode::Sdf,
        GlyphRasterPath::Msdf => UiTextRenderMode::Msdf,
        GlyphRasterPath::Mtsdf => UiTextRenderMode::Mtsdf,
    }
}

fn auto_text_policy_request(text: &ScreenSpaceUiTextBatch) -> GlyphRasterPolicyRequest {
    let mut request = GlyphRasterPolicyRequest::new(text.font_size, false);
    request.effects = GlyphRasterEffects {
        outline: text.text_effects.outline.is_some(),
        shadow: text.text_effects.shadow.is_some(),
        glow: text.text_effects.glow.is_some(),
        true_distance_effects: text.text_effects.glow.is_some(),
    };
    request
}

/// 保留显式 native/SDF 批次并解析 Auto 路由后，按当前字体 revision 和 generation 淘汰陈旧 artifact 批次。
/// 只有无 artifact 的回退批次会在字体变化或缺少 shaped glyphs 时使用当前字体集合重塑。
pub(super) fn resolve_text_batches_after_font_dependencies(
    font_assets: &UiFontAssetCache,
    auto_router: &mut AutoTextRasterRouter,
    auto_texts: &[ScreenSpaceUiTextBatch],
    native_texts: &[ScreenSpaceUiTextBatch],
    sdf_texts: &[ScreenSpaceUiTextBatch],
    shaping_changed: bool,
    font_faces_changed: bool,
    font_revision: FontCollectionRevision,
    font_collection: &Arc<FontCollectionService>,
) -> ResolvedScreenSpaceUiTextBatches {
    let mut resolved =
        ResolvedScreenSpaceUiTextBatches::from_explicit_batches(native_texts, sdf_texts);
    resolved.font_faces_changed = font_faces_changed;
    for text in auto_texts {
        let asset = text
            .font
            .as_deref()
            .filter(|asset| !asset.trim().is_empty())
            .unwrap_or(super::DEFAULT_FONT_ASSET);
        let font_asset = font_assets
            .get(asset)
            .and_then(|entry| entry.loaded_asset());
        resolved.push_resolved_auto_text(text.clone(), auto_router.resolve(text, font_asset));
    }
    resolved.auto_route = auto_router.frame_report();

    resolved.reconcile_after_font_load(shaping_changed, font_revision, font_collection);
    resolved
}

impl ResolvedScreenSpaceUiTextBatches {
    pub(super) fn append_segment_cloned(&mut self, segment: &Self) {
        self.native_texts
            .extend(segment.native_texts.iter().cloned());
        self.sdf_texts.extend(segment.sdf_texts.iter().cloned());
        self.font_faces_changed |= segment.font_faces_changed;
        self.post_layout_stale_artifact_batch_rejection_count = self
            .post_layout_stale_artifact_batch_rejection_count
            .saturating_add(segment.post_layout_stale_artifact_batch_rejection_count);
    }
}

#[cfg(test)]
#[path = "tests/resolved_batches.rs"]
mod tests;
