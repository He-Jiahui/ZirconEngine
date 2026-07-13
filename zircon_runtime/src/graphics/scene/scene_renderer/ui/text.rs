use std::collections::HashMap;
use std::sync::Arc;

use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, Style,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport, Weight, Wrap,
};

#[cfg(test)]
use super::atlas_renderer::GlyphAtlasBitmapRendererPrepareReport;
use super::atlas_renderer::{GlyphAtlasBitmapRenderer, GlyphAtlasBitmapRendererStorageSubmission};
use super::render::ScreenSpaceUiTextBatch;
use crate::asset::ProjectAssetManager;
use crate::core::framework::render::TextShapeRequest;
use crate::core::math::UVec2;
use crate::graphics::text::atlas::render_gpu_plan::GlyphAtlasGpuDrawPlan;
use crate::graphics::text::atlas::{GlyphAtlasBitmapRetryFrameState, GlyphAtlasStorageFormat};
#[cfg(test)]
use crate::graphics::text::font::MissingGlyphDiagnosticsReport;
use crate::graphics::text::font::{
    publish_shared_font_database, shared_font_database_snapshot, FontDatabase, SystemFontPolicy,
};
use crate::graphics::text::parallel::raster_pool::{
    TextRasterWorkerPool, TextRasterWorkerPoolOptions,
};
use crate::graphics::text::shaping::fallback_text_spans;
use glyphon::cosmic_text::Align;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiTextAlign, UiTextDirection, UiTextRenderMode, UiTextRunPaintStyle,
    UiTextWrap,
};

mod font_assets;
mod font_id_report;
mod native_bitmap_atlas;
mod prepare_report;
mod resolved_batches;
mod sdf_fallback;

use self::font_assets::{
    effective_text_render_mode, ensure_font_asset_record, load_font_asset_record,
    resolve_font_asset_record, LoadedUiFontAsset,
};
use self::font_id_report::{
    accumulate_text_font_id_report, resolved_style_for_text_batch, ScreenSpaceUiTextFontIdReport,
};
use self::native_bitmap_atlas::{
    bitmap_atlas_page_size, native_bitmap_atlas_frame, native_bitmap_atlas_handoff_for_report,
    native_bitmap_atlas_idle_prepare_report, NativeBitmapAtlasFrame, NativeBitmapAtlasHandoff,
    NativeBitmapAtlasPrepareReport, NativeBitmapAtlasSourceCache, NativeBitmapAtlasTextArea,
};
use self::prepare_report::text_prepare_report;
pub(crate) use self::prepare_report::ScreenSpaceUiTextPrepareReport;
#[cfg(test)]
use self::prepare_report::ScreenSpaceUiTextRasterUploadReport;
use self::resolved_batches::ResolvedScreenSpaceUiTextBatches;
use self::sdf_fallback::apply_sdf_atlas_fallbacks;
#[cfg(test)]
use self::sdf_fallback::ScreenSpaceUiTextSdfFallbackReport;
use super::sdf_atlas::ScreenSpaceUiSdfAtlas;
#[cfg(test)]
use super::sdf_atlas::SdfAtlasCacheReport;
#[cfg(test)]
use super::sdf_render::ScreenSpaceUiSdfPrepareReport;
use super::sdf_render::ScreenSpaceUiSdfRenderer;
#[cfg(test)]
use super::sdf_upload::{SdfAtlasUploadMode, SdfAtlasUploadReport};
use super::text_pixel_snap::text_origin_device_px;
use crate::graphics::text::atlas::GlyphAtlasSet;

const DEFAULT_FONT_ASSET: &str = "res://fonts/default.font.toml";
const NATIVE_BITMAP_ATLAS_RASTER_WORKER_COUNT: usize = 1;

pub(super) struct ScreenSpaceUiTextSystem {
    asset_manager: Arc<ProjectAssetManager>,
    font_system: FontSystem,
    font_database: FontDatabase,
    swash_cache: SwashCache,
    font_assets: HashMap<String, LoadedUiFontAsset>,
    native: ScreenSpaceUiTextBackend,
    bitmap_atlas_renderer: GlyphAtlasBitmapRenderer,
    sdf_atlas: ScreenSpaceUiSdfAtlas,
    sdf_renderer: ScreenSpaceUiSdfRenderer,
    last_prepare_report: ScreenSpaceUiTextPrepareReport,
}

struct ScreenSpaceUiTextBackend {
    _cache: Cache,
    viewport: Viewport,
    atlas: TextAtlas,
    renderer: TextRenderer,
    bitmap_source_cache: NativeBitmapAtlasSourceCache,
    bitmap_retry_state: GlyphAtlasBitmapRetryFrameState,
    bitmap_atlas: GlyphAtlasSet,
    bitmap_raster_worker_pool: Option<TextRasterWorkerPool>,
    bitmap_atlas_frame_index: u64,
    render_glyphon: bool,
}

#[derive(Clone, Debug, Default)]
struct ScreenSpaceUiNativePrepareReport {
    font_ids: ScreenSpaceUiTextFontIdReport,
    bitmap_atlas: NativeBitmapAtlasPrepareReport,
}

impl ScreenSpaceUiTextSystem {
    pub(super) fn new(
        asset_manager: Arc<ProjectAssetManager>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        let mut font_system = FontSystem::new();
        let (_, mut font_database) = shared_font_database_snapshot();
        initialize_screen_space_ui_font_system(&mut font_system, &mut font_database);
        let mut font_assets = HashMap::new();
        let default_font = load_font_asset_record(
            &mut font_system,
            &mut font_database,
            DEFAULT_FONT_ASSET,
            &asset_manager,
        );
        if let Some(record) = default_font.as_ref() {
            if let Some(family) = record.family.as_deref() {
                font_database.set_default_ui_family(family);
                font_system
                    .db_mut()
                    .set_sans_serif_family(family.to_string());
                font_system
                    .db_mut()
                    .set_monospace_family(family.to_string());
                publish_shared_font_database(&font_database);
            }
            font_assets.insert(DEFAULT_FONT_ASSET.to_string(), record.clone());
        }

        Self {
            asset_manager,
            font_system,
            font_database,
            swash_cache: SwashCache::new(),
            font_assets,
            native: ScreenSpaceUiTextBackend::new(device, queue, target_format),
            bitmap_atlas_renderer: GlyphAtlasBitmapRenderer::new(device, target_format),
            sdf_atlas: ScreenSpaceUiSdfAtlas::new(),
            sdf_renderer: ScreenSpaceUiSdfRenderer::new(device, target_format),
            last_prepare_report: ScreenSpaceUiTextPrepareReport::default(),
        }
    }

    pub(super) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        viewport_size: crate::core::math::UVec2,
        auto_texts: &[ScreenSpaceUiTextBatch],
        native_texts: &[ScreenSpaceUiTextBatch],
        sdf_texts: &[ScreenSpaceUiTextBatch],
    ) {
        let font_asset_count_before_resolve = self.font_assets.len();
        let mut resolved_texts = resolve_text_batches(
            &mut self.font_system,
            &mut self.font_database,
            &mut self.font_assets,
            self.asset_manager.as_ref(),
            auto_texts,
            native_texts,
            sdf_texts,
        );
        let font_faces_changed_before_native =
            self.font_assets.len() != font_asset_count_before_resolve;
        self.sdf_atlas.prepare(resolved_texts.sdf_texts());
        let sdf_generation_failures = self.sdf_renderer.generation_failures_for_plan(
            self.sdf_atlas.plan(),
            &mut self.font_database,
            self.asset_manager.as_ref(),
        );
        self.sdf_atlas
            .record_generation_failures(&sdf_generation_failures);
        let sdf_fallback_glyph_advances =
            self.sdf_renderer.measure_text_glyph_advances_for_fallbacks(
                resolved_texts.sdf_texts(),
                &mut self.font_database,
                self.asset_manager.as_ref(),
            );
        let sdf_fallback_report = apply_sdf_atlas_fallbacks(
            &mut resolved_texts.native_texts,
            &mut resolved_texts.sdf_texts,
            &self.sdf_atlas.plan().runs,
            &sdf_fallback_glyph_advances,
        );
        if sdf_fallback_report.has_whole_batch_fallbacks() {
            self.sdf_atlas
                .discard_cached_slots_not_in_texts(resolved_texts.sdf_texts());
            self.sdf_atlas.prepare(resolved_texts.sdf_texts());
            let sdf_generation_failures = self.sdf_renderer.generation_failures_for_plan(
                self.sdf_atlas.plan(),
                &mut self.font_database,
                self.asset_manager.as_ref(),
            );
            self.sdf_atlas
                .record_generation_failures(&sdf_generation_failures);
        }
        let sdf_atlas_report = self.sdf_atlas.cache_report();
        self.sdf_renderer.prepare(
            device,
            queue,
            viewport_size,
            resolved_texts.sdf_texts(),
            resolved_texts.native_texts(),
            self.sdf_atlas.plan(),
            sdf_atlas_report.clone(),
            &mut self.font_database,
            self.asset_manager.as_ref(),
        );
        let sdf_renderer_report = self.sdf_renderer.prepare_report();
        let native_font_id_report = self.native.prepare(
            device,
            queue,
            viewport_size,
            resolved_texts.native_texts(),
            &mut self.bitmap_atlas_renderer,
            &mut self.font_system,
            &mut self.font_database,
            &mut self.swash_cache,
            &mut self.font_assets,
            self.asset_manager.as_ref(),
            font_faces_changed_before_native,
        );
        let bitmap_atlas_renderer_report = self.bitmap_atlas_renderer.prepare_report();
        let missing_glyphs = self.font_database.take_missing_glyph_diagnostics();
        self.last_prepare_report = text_prepare_report(
            auto_texts,
            native_texts,
            sdf_texts,
            &resolved_texts,
            sdf_fallback_report,
            native_font_id_report,
            missing_glyphs,
            bitmap_atlas_renderer_report,
            sdf_atlas_report,
            sdf_renderer_report,
        );
    }

    pub(super) fn render<'pass>(&'pass mut self, pass: &mut wgpu::RenderPass<'pass>) {
        self.native.render(pass);
        self.bitmap_atlas_renderer.render(pass);
        self.sdf_renderer.render(pass);
    }

    pub(super) fn prepare_report(&self) -> ScreenSpaceUiTextPrepareReport {
        self.last_prepare_report.clone()
    }
}

fn initialize_screen_space_ui_font_system(
    font_system: &mut FontSystem,
    font_database: &mut FontDatabase,
) -> usize {
    let discovered = font_database.apply_system_font_policy(SystemFontPolicy::Discover);
    font_database.sync_font_system(font_system);
    discovered
}

impl ScreenSpaceUiTextBackend {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, target_format: wgpu::TextureFormat) -> Self {
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, target_format);
        let renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);

        Self {
            _cache: cache,
            viewport,
            atlas,
            renderer,
            bitmap_source_cache: NativeBitmapAtlasSourceCache::default(),
            bitmap_retry_state: GlyphAtlasBitmapRetryFrameState::new(),
            bitmap_atlas: GlyphAtlasSet::default(),
            bitmap_raster_worker_pool: TextRasterWorkerPool::new(TextRasterWorkerPoolOptions::new(
                NATIVE_BITMAP_ATLAS_RASTER_WORKER_COUNT,
            ))
            .ok(),
            bitmap_atlas_frame_index: 0,
            render_glyphon: true,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        viewport_size: crate::core::math::UVec2,
        texts: &[ScreenSpaceUiTextBatch],
        bitmap_atlas_renderer: &mut GlyphAtlasBitmapRenderer,
        font_system: &mut FontSystem,
        font_database: &mut FontDatabase,
        swash_cache: &mut SwashCache,
        font_assets: &mut HashMap<String, LoadedUiFontAsset>,
        asset_manager: &ProjectAssetManager,
        font_faces_changed: bool,
    ) -> ScreenSpaceUiNativePrepareReport {
        self.viewport.update(
            queue,
            Resolution {
                width: viewport_size.x.max(1),
                height: viewport_size.y.max(1),
            },
        );

        let font_asset_count_at_entry = font_assets.len();
        if font_faces_changed {
            self.bitmap_source_cache.discard_all_for_face_invalidation();
            self.bitmap_retry_state.discard_all_for_face_invalidation();
            self.bitmap_atlas = GlyphAtlasSet::default();
            bitmap_atlas_renderer.discard_all_for_face_invalidation();
        }

        if texts.is_empty() {
            self.atlas.trim();
            self.render_glyphon = false;
            self.bitmap_atlas = GlyphAtlasSet::default();
            self.bitmap_retry_state
                .replace_blocked_glyphs(std::iter::empty());
            let bitmap_atlas = native_bitmap_atlas_idle_prepare_report(
                &mut self.bitmap_source_cache,
                &mut self.bitmap_retry_state,
            );
            bitmap_atlas_renderer.prepare_plan(
                device,
                &GlyphAtlasGpuDrawPlan::default(),
                UVec2::new(1, 1),
                1,
                GlyphAtlasStorageFormat::R8Unorm,
            );
            return ScreenSpaceUiNativePrepareReport {
                bitmap_atlas,
                ..ScreenSpaceUiNativePrepareReport::default()
            };
        }

        let mut buffers = Vec::with_capacity(texts.len());
        let mut font_id_report = ScreenSpaceUiTextFontIdReport::default();
        for text in texts {
            let family_name = resolve_family_name(
                font_system,
                font_database,
                font_assets,
                asset_manager,
                text.font.as_deref(),
                text.font_family.as_deref(),
            );
            let attrs = text_attrs(family_name.as_deref(), text.font_weight, text.style);
            let resolved_style = resolved_style_for_text_batch(text, family_name.as_deref());
            let source_range = zircon_runtime_interface::ui::surface::UiTextRange {
                start: 0,
                end: text.text.len(),
            };
            let fallback_spans = fallback_text_spans(
                &text.text,
                TextShapeRequest::horizontal(
                    &text.text,
                    &resolved_style,
                    text.text_direction,
                    source_range,
                ),
                font_database,
            );
            let mut buffer =
                Buffer::new(font_system, Metrics::new(text.font_size, text.line_height));
            buffer.set_size(
                font_system,
                Some(text.frame.width.max(1.0)),
                Some(text.frame.height.max(1.0)),
            );
            buffer.set_wrap(
                font_system,
                match text.wrap {
                    UiTextWrap::None => Wrap::None,
                    UiTextWrap::Word | UiTextWrap::WordSmart => Wrap::Word,
                    UiTextWrap::Glyph => Wrap::Glyph,
                },
            );
            let alignment = Some(native_text_align(text.text_align, text.text_direction));
            if fallback_spans.is_empty() {
                buffer.set_text(
                    font_system,
                    &text.text,
                    &attrs,
                    Shaping::Advanced,
                    alignment,
                );
            } else {
                buffer.set_rich_text(
                    font_system,
                    fallback_spans.iter().map(|span| {
                        let span_attrs = span
                            .family
                            .as_deref()
                            .map(|family| attrs.clone().family(Family::Name(family)))
                            .unwrap_or_else(|| attrs.clone());
                        (&text.text[span.range.clone()], span_attrs)
                    }),
                    &attrs,
                    Shaping::Advanced,
                    alignment,
                );
            }
            buffer.shape_until_scroll(font_system, false);
            accumulate_text_font_id_report(
                &mut font_id_report,
                &resolved_style,
                &buffer,
                font_database,
            );
            buffers.push(buffer);
        }
        if font_assets.len() != font_asset_count_at_entry {
            self.bitmap_source_cache.discard_all_for_face_invalidation();
            self.bitmap_retry_state.discard_all_for_face_invalidation();
            self.bitmap_atlas = GlyphAtlasSet::default();
            bitmap_atlas_renderer.discard_all_for_face_invalidation();
        }

        let text_areas = texts
            .iter()
            .zip(buffers.iter())
            .map(|(text, buffer)| {
                let placement = native_text_area_placement(viewport_size, text);
                TextArea {
                    buffer,
                    left: placement.left,
                    top: placement.top,
                    scale: 1.0,
                    bounds: placement.bounds,
                    default_color: pack_color(text.color),
                    custom_glyphs: &[],
                }
            })
            .collect::<Vec<_>>();
        let bitmap_text_areas = texts
            .iter()
            .zip(text_areas.iter())
            .map(|(text, text_area)| {
                NativeBitmapAtlasTextArea::new(text_area, text.background_color)
            })
            .collect::<Vec<_>>();

        let bitmap_frame = native_bitmap_atlas_frame(
            font_system,
            font_database,
            self.bitmap_raster_worker_pool.as_ref(),
            &mut self.bitmap_source_cache,
            &mut self.bitmap_retry_state,
            std::mem::take(&mut self.bitmap_atlas),
            viewport_size,
            next_native_bitmap_atlas_frame_index(&mut self.bitmap_atlas_frame_index),
            bitmap_text_areas.as_slice(),
        );
        self.bitmap_atlas = bitmap_frame.submission.run.atlas.clone();
        let bitmap_atlas_report = bitmap_frame.prepare_report();
        drop(bitmap_text_areas);
        match native_bitmap_atlas_handoff_for_report(&bitmap_atlas_report) {
            NativeBitmapAtlasHandoff::SingleStorageReplacement => {
                bitmap_atlas_renderer.prepare_submission_with_face_validity(
                    device,
                    queue,
                    &bitmap_frame.submission,
                    bitmap_frame.source_bytes(),
                    bitmap_atlas_page_size(),
                    bitmap_frame.atlas_layer_count(),
                    bitmap_frame
                        .atlas_storage_format()
                        .unwrap_or(GlyphAtlasStorageFormat::R8Unorm),
                    bitmap_frame.face_validity(),
                );
                self.render_glyphon = false;
                self.atlas.trim();
            }
            NativeBitmapAtlasHandoff::MixedStorageReplacement => {
                let storage_submissions = bitmap_frame.storage_submissions();
                let renderer_submissions = storage_submissions
                    .iter()
                    .map(|submission| {
                        GlyphAtlasBitmapRendererStorageSubmission::new_with_face_validity(
                            &submission.submission,
                            submission.source_bytes(),
                            submission.atlas_layer_count(),
                            submission.storage_format,
                            submission.face_validity(),
                        )
                    })
                    .collect::<Vec<_>>();
                bitmap_atlas_renderer.prepare_storage_submissions(
                    device,
                    queue,
                    renderer_submissions.as_slice(),
                    bitmap_atlas_page_size(),
                );
                self.render_glyphon = false;
                self.atlas.trim();
            }
            NativeBitmapAtlasHandoff::TransparentPlaceholder => {
                prepare_native_bitmap_atlas_transparent_placeholder(
                    device,
                    queue,
                    bitmap_atlas_renderer,
                    &bitmap_frame,
                );
                self.render_glyphon = false;
                self.atlas.trim();
            }
            NativeBitmapAtlasHandoff::GlyphonFallback => {
                bitmap_atlas_renderer.prepare_plan(
                    device,
                    &GlyphAtlasGpuDrawPlan::default(),
                    UVec2::new(1, 1),
                    1,
                    GlyphAtlasStorageFormat::R8Unorm,
                );
                self.render_glyphon = true;
                let _ = self.renderer.prepare(
                    device,
                    queue,
                    font_system,
                    &mut self.atlas,
                    &self.viewport,
                    text_areas,
                    swash_cache,
                );
            }
        }

        ScreenSpaceUiNativePrepareReport {
            font_ids: font_id_report,
            bitmap_atlas: bitmap_atlas_report,
        }
    }

    fn render<'pass>(&'pass mut self, pass: &mut wgpu::RenderPass<'pass>) {
        if self.render_glyphon {
            let _ = self.renderer.render(&self.atlas, &self.viewport, pass);
        }
    }
}

fn prepare_native_bitmap_atlas_transparent_placeholder(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bitmap_atlas_renderer: &mut GlyphAtlasBitmapRenderer,
    bitmap_frame: &NativeBitmapAtlasFrame,
) {
    if let Some(atlas_storage_format) = bitmap_frame.atlas_storage_format() {
        bitmap_atlas_renderer.prepare_submission_with_face_validity(
            device,
            queue,
            &bitmap_frame.submission,
            bitmap_frame.source_bytes(),
            bitmap_atlas_page_size(),
            bitmap_frame.atlas_layer_count(),
            atlas_storage_format,
            bitmap_frame.face_validity(),
        );
        return;
    }

    let storage_submissions = bitmap_frame.storage_submissions();
    if storage_submissions.is_empty() {
        bitmap_atlas_renderer.prepare_plan(
            device,
            &GlyphAtlasGpuDrawPlan::default(),
            UVec2::new(1, 1),
            1,
            GlyphAtlasStorageFormat::R8Unorm,
        );
        return;
    }

    let renderer_submissions = storage_submissions
        .iter()
        .map(|submission| {
            GlyphAtlasBitmapRendererStorageSubmission::new_with_face_validity(
                &submission.submission,
                submission.source_bytes(),
                submission.atlas_layer_count(),
                submission.storage_format,
                submission.face_validity(),
            )
        })
        .collect::<Vec<_>>();
    bitmap_atlas_renderer.prepare_storage_submissions(
        device,
        queue,
        renderer_submissions.as_slice(),
        bitmap_atlas_page_size(),
    );
}

fn next_native_bitmap_atlas_frame_index(frame_index: &mut u64) -> u64 {
    *frame_index = frame_index.saturating_add(1).max(1);
    *frame_index
}

fn text_attrs<'a>(
    family_name: Option<&'a str>,
    font_weight: u16,
    style: UiTextRunPaintStyle,
) -> Attrs<'a> {
    let mut attrs = if style.code {
        Attrs::new().family(Family::Monospace)
    } else {
        family_name
            .map(|family| Attrs::new().family(Family::Name(family)))
            .unwrap_or_else(Attrs::new)
    };
    let resolved_weight = UiResolvedStyle::normalized_font_weight(font_weight);
    let resolved_weight = if style.strong {
        resolved_weight.max(Weight::BOLD.0)
    } else {
        resolved_weight
    };
    attrs = attrs.weight(Weight(resolved_weight));
    if style.strong {
        debug_assert!(attrs.weight.0 >= Weight::BOLD.0);
    }
    if style.emphasis {
        attrs = attrs.style(Style::Italic);
    }
    attrs
}

fn resolve_family_name(
    font_system: &mut FontSystem,
    font_database: &mut FontDatabase,
    font_assets: &mut HashMap<String, LoadedUiFontAsset>,
    asset_manager: &ProjectAssetManager,
    font_asset: Option<&str>,
    preferred_family: Option<&str>,
) -> Option<String> {
    if let Some(family) = preferred_family.filter(|family| !family.trim().is_empty()) {
        if let Some(asset) = font_asset.filter(|asset| !asset.trim().is_empty()) {
            ensure_font_asset_record(
                font_system,
                font_database,
                font_assets,
                asset_manager,
                asset,
            );
        }
        return Some(family.to_string());
    }

    let asset = font_asset
        .filter(|asset| !asset.trim().is_empty())
        .unwrap_or(DEFAULT_FONT_ASSET);
    ensure_font_asset_record(
        font_system,
        font_database,
        font_assets,
        asset_manager,
        asset,
    )
    .family
    .clone()
}

fn resolve_text_batches(
    font_system: &mut FontSystem,
    font_database: &mut FontDatabase,
    font_assets: &mut HashMap<String, LoadedUiFontAsset>,
    asset_manager: &ProjectAssetManager,
    auto_texts: &[ScreenSpaceUiTextBatch],
    native_texts: &[ScreenSpaceUiTextBatch],
    sdf_texts: &[ScreenSpaceUiTextBatch],
) -> ResolvedScreenSpaceUiTextBatches {
    let mut resolved =
        ResolvedScreenSpaceUiTextBatches::from_explicit_batches(native_texts, sdf_texts);

    for text in auto_texts {
        let font_asset = resolve_font_asset_record(
            font_system,
            font_database,
            font_assets,
            asset_manager,
            text.font.as_deref(),
        );
        resolved.push_resolved_auto_text(
            text.clone(),
            effective_text_render_mode(UiTextRenderMode::Auto, font_asset),
        );
    }

    resolved
}

fn text_bounds(
    viewport_size: crate::core::math::UVec2,
    text: &ScreenSpaceUiTextBatch,
) -> TextBounds {
    let clip = text
        .clip_frame
        .unwrap_or_else(|| UiFrame::new(0.0, 0.0, viewport_size.x as f32, viewport_size.y as f32));
    let clip = clip
        .intersection(UiFrame::new(
            0.0,
            0.0,
            viewport_size.x as f32,
            viewport_size.y as f32,
        ))
        .unwrap_or_default();
    TextBounds {
        left: clip.x.max(0.0).floor() as i32,
        top: clip.y.max(0.0).floor() as i32,
        right: clip.right().max(0.0).ceil() as i32,
        bottom: clip.bottom().max(0.0).ceil() as i32,
    }
}

struct NativeTextAreaPlacement {
    left: f32,
    top: f32,
    bounds: TextBounds,
}

fn native_text_area_placement(
    viewport_size: crate::core::math::UVec2,
    text: &ScreenSpaceUiTextBatch,
) -> NativeTextAreaPlacement {
    NativeTextAreaPlacement {
        left: text_origin_device_px(text.frame.x),
        top: text_origin_device_px(text.frame.y),
        bounds: text_bounds(viewport_size, text),
    }
}

fn pack_color(color: [f32; 4]) -> Color {
    Color::rgba(
        (color[0].clamp(0.0, 1.0) * 255.0) as u8,
        (color[1].clamp(0.0, 1.0) * 255.0) as u8,
        (color[2].clamp(0.0, 1.0) * 255.0) as u8,
        (color[3].clamp(0.0, 1.0) * 255.0) as u8,
    )
}

fn native_text_align(align: UiTextAlign, direction: UiTextDirection) -> Align {
    match align {
        UiTextAlign::Left => Align::Left,
        UiTextAlign::Center => Align::Center,
        UiTextAlign::Right => Align::Right,
        UiTextAlign::Start if matches!(direction, UiTextDirection::RightToLeft) => Align::Right,
        UiTextAlign::Start => Align::Left,
        UiTextAlign::End if matches!(direction, UiTextDirection::RightToLeft) => Align::Left,
        UiTextAlign::End => Align::Right,
        UiTextAlign::Justify => Align::Justified,
    }
}

#[cfg(test)]
mod tests;
