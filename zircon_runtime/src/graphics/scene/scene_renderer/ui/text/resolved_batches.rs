use std::sync::Arc;

use zircon_runtime_interface::ui::surface::UiTextRenderMode;

use super::super::render::{
    ScreenSpaceUiTextBatch, text_advances::refresh_renderer_fallback_text_batch_glyphs,
};
use super::font_assets::{UiFontAssetCache, effective_text_render_mode};
use crate::text::TextLayoutFallbackReport;
use crate::text::font::{FontCollectionRevision, FontCollectionService};
use crate::text::raster::{
    GlyphRasterEffects, GlyphRasterPath, GlyphRasterPolicyRequest, raster_path_for_request,
};

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
mod tests {
    use std::sync::Arc;

    use zircon_runtime_interface::ui::event_ui::UiNodeId;
    use zircon_runtime_interface::ui::layout::UiFrame;
    use zircon_runtime_interface::ui::surface::{
        UiResolvedStyle, UiTextAlign, UiTextDirection, UiTextRange, UiTextWrap, UiTextWritingMode,
    };

    use super::ResolvedScreenSpaceUiTextBatches;
    use crate::graphics::scene::scene_renderer::ui::render::{
        ScreenSpaceUiGlyphArtifactLine, ScreenSpaceUiTextBatch, ScreenSpaceUiTextRouteIdentity,
    };
    use crate::text::ResolvedTextGlyphArtifact;
    use crate::text::sdf::SdfMode;

    #[test]
    fn stale_font_generation_artifact_batch_is_rejected_without_a_layout_session() {
        let current_collection = crate::text::font::FontCollectionService::from_database(
            crate::text::font::runtime_default_font_database_for_test(),
        );
        let current_revision = current_collection.revision();
        let stale_generation = current_revision.generation().wrapping_add(1);
        let mut resolved = ResolvedScreenSpaceUiTextBatches::from_explicit_batches(
            &[artifact_batch_with_font_lease(
                stale_generation,
                crate::text::ResolvedTextGlyphArtifactFontLease::capture(
                    current_collection.collection_snapshot(),
                ),
            )],
            &[],
        );
        let constructions_before =
            crate::text::current_thread_text_layout_session_construction_count();

        resolved.reconcile_after_font_load(false, current_revision, &current_collection);

        let constructions_after =
            crate::text::current_thread_text_layout_session_construction_count();
        assert!(resolved.native_texts().is_empty());
        assert_eq!(
            resolved.post_layout_stale_artifact_batch_rejection_count(),
            1
        );
        assert_eq!(constructions_after, constructions_before);
    }

    #[test]
    fn same_generation_artifact_from_foreign_collection_is_rejected() {
        let current_collection = crate::text::font::FontCollectionService::from_database(
            crate::text::font::runtime_default_font_database_for_test(),
        );
        let foreign_collection = crate::text::font::FontCollectionService::from_database(
            crate::text::font::runtime_default_font_database_for_test(),
        );
        assert_eq!(
            current_collection.generation(),
            foreign_collection.generation()
        );
        let generation = current_collection.generation();
        let mut resolved = ResolvedScreenSpaceUiTextBatches::from_explicit_batches(
            &[artifact_batch_with_font_lease(
                generation,
                crate::text::ResolvedTextGlyphArtifactFontLease::capture(
                    foreign_collection.collection_snapshot(),
                ),
            )],
            &[],
        );

        resolved.reconcile_after_font_load(
            false,
            current_collection.revision(),
            &current_collection,
        );

        assert!(resolved.native_texts().is_empty());
        assert_eq!(
            resolved.post_layout_stale_artifact_batch_rejection_count(),
            1
        );
    }

    #[test]
    fn renderer_fallback_shapes_with_the_owned_font_collection() {
        let font_collection = crate::text::font::FontCollectionService::from_database(
            crate::text::font::runtime_default_font_database_for_test(),
        );
        assert_ne!(
            font_collection.collection_id(),
            crate::text::font::shared_font_collection_service().collection_id()
        );
        let mut batch = artifact_batch_with_font_lease(
            font_collection.generation(),
            crate::text::ResolvedTextGlyphArtifactFontLease::capture(
                font_collection.collection_snapshot(),
            ),
        );
        batch.glyph_artifact_line = None;
        batch.source_range = None;
        batch.glyph_advances.clear();
        batch.preserve_shaped_glyphs = false;
        let mut resolved = ResolvedScreenSpaceUiTextBatches::from_explicit_batches(&[batch], &[]);

        resolved.reconcile_after_font_load(false, font_collection.revision(), &font_collection);

        let shaped = &resolved.native_texts()[0].shaped_glyphs;
        assert!(!shaped.is_empty());
        assert!(shaped.iter().all(|glyph| {
            glyph
                .font_id
                .is_some_and(|font| font.collection == font_collection.collection_id())
        }));
    }

    fn artifact_batch_with_font_lease(
        font_generation: u64,
        font_lease: crate::text::ResolvedTextGlyphArtifactFontLease,
    ) -> ScreenSpaceUiTextBatch {
        let frame = UiFrame::new(0.0, 0.0, 24.0, 24.0);
        ScreenSpaceUiTextBatch {
            route_identity: ScreenSpaceUiTextRouteIdentity::new(
                "runtime.ui.text.stale-artifact-rejection.test",
                UiNodeId::new(1),
                None,
            ),
            command_generation: 1,
            raster_scale: 1.0,
            text: "stale".to_string(),
            frame,
            clip_frame: None,
            source_range: Some(UiTextRange { start: 0, end: 5 }),
            is_source_isomorphic_layout_line: false,
            glyph_advances: vec![24.0],
            shaped_glyphs: Vec::new(),
            preserve_shaped_glyphs: true,
            glyph_artifact_line: Some(ScreenSpaceUiGlyphArtifactLine {
                artifact: Arc::new(ResolvedTextGlyphArtifact {
                    source_text: Arc::from("stale"),
                    source_text_origin: 0,
                    font_generation,
                    font_lease,
                    style: UiResolvedStyle::default(),
                    writing_mode: UiTextWritingMode::HorizontalTb,
                    lines: vec![None],
                    logical_virtual_line_sequences: None,
                }),
                line_index: 0,
                font_generation,
                glyph_range: 0..0,
            }),
            layout_error: None,
            color: [1.0; 4],
            background_color: None,
            font: None,
            font_family: None,
            language: None,
            font_weight: UiResolvedStyle::DEFAULT_FONT_WEIGHT,
            font_size: 16.0,
            line_height: 20.0,
            text_align: UiTextAlign::Left,
            text_direction: UiTextDirection::LeftToRight,
            writing_mode: UiTextWritingMode::HorizontalTb,
            wrap: UiTextWrap::None,
            style: Default::default(),
            distance_field_mode: SdfMode::Sdf,
            text_effects: Default::default(),
            text_decorations: Default::default(),
            text_decoration_baseline: None,
            clip_transform: None,
        }
    }
}
