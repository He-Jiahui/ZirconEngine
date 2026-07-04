use std::collections::HashMap;
use std::sync::Arc;

use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, Style,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport, Weight, Wrap,
};

use super::atlas_renderer::{
    GlyphAtlasBitmapRenderer, GlyphAtlasBitmapRendererPrepareReport,
    GlyphAtlasBitmapRendererStorageSubmission,
};
use super::font_asset::{load_ui_font_manifest_with_asset_manager, LoadedUiFontManifest};
use super::render::ScreenSpaceUiTextBatch;
use crate::asset::ProjectAssetManager;
use crate::core::math::UVec2;
use crate::graphics::text::atlas::render_gpu_plan::GlyphAtlasGpuDrawPlan;
use crate::graphics::text::atlas::GlyphAtlasStorageFormat;
use crate::graphics::text::font::FontDatabase;
use glyphon::cosmic_text::Align;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiTextAlign, UiTextDirection, UiTextRenderMode, UiTextRunPaintStyle,
    UiTextWrap,
};

mod font_id_report;
mod native_bitmap_atlas;
mod sdf_fallback;

use self::font_id_report::{accumulate_text_font_id_report, ScreenSpaceUiTextFontIdReport};
use self::native_bitmap_atlas::{
    bitmap_atlas_page_size, native_bitmap_atlas_frame, NativeBitmapAtlasPrepareReport,
};
use self::sdf_fallback::{apply_sdf_atlas_fallbacks, ScreenSpaceUiTextSdfFallbackReport};
use super::sdf_atlas::{ScreenSpaceUiSdfAtlas, SdfAtlasCacheReport};
use super::sdf_render::{ScreenSpaceUiSdfPrepareReport, ScreenSpaceUiSdfRenderer};
#[cfg(test)]
use super::sdf_upload::{SdfAtlasUploadMode, SdfAtlasUploadReport};
use super::text_pixel_snap::text_origin_device_px;
use crate::ui::text::shaper::resolve_text_render_mode;

const DEFAULT_FONT_ASSET: &str = "res://fonts/default.font.toml";

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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ScreenSpaceUiTextPrepareReport {
    pub(super) input_auto_text_batch_count: usize,
    pub(super) input_native_text_batch_count: usize,
    pub(super) input_sdf_text_batch_count: usize,
    pub(super) resolved_native_text_batch_count: usize,
    pub(super) resolved_sdf_text_batch_count: usize,
    pub(super) sdf_fallback: ScreenSpaceUiTextSdfFallbackReport,
    pub(super) native_font_ids: ScreenSpaceUiTextFontIdReport,
    pub(super) native_bitmap_atlas: NativeBitmapAtlasPrepareReport,
    pub(super) bitmap_atlas_renderer: GlyphAtlasBitmapRendererPrepareReport,
    pub(super) sdf_atlas: SdfAtlasCacheReport,
    pub(super) sdf_renderer: ScreenSpaceUiSdfPrepareReport,
}

struct ScreenSpaceUiTextBackend {
    _cache: Cache,
    viewport: Viewport,
    atlas: TextAtlas,
    renderer: TextRenderer,
    render_glyphon: bool,
}

#[derive(Clone, Debug, Default)]
struct ScreenSpaceUiNativePrepareReport {
    font_ids: ScreenSpaceUiTextFontIdReport,
    bitmap_atlas: NativeBitmapAtlasPrepareReport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NativeBitmapAtlasHandoff {
    SingleStorageReplacement,
    MixedStorageReplacement,
    GlyphonFallback,
}

#[derive(Clone, Debug, Default)]
struct ResolvedScreenSpaceUiTextBatches {
    native_texts: Vec<ScreenSpaceUiTextBatch>,
    sdf_texts: Vec<ScreenSpaceUiTextBatch>,
}

impl ResolvedScreenSpaceUiTextBatches {
    fn from_explicit_batches(
        native_texts: &[ScreenSpaceUiTextBatch],
        sdf_texts: &[ScreenSpaceUiTextBatch],
    ) -> Self {
        Self {
            native_texts: native_texts.to_vec(),
            sdf_texts: sdf_texts.to_vec(),
        }
    }

    fn push_resolved_auto_text(
        &mut self,
        text: ScreenSpaceUiTextBatch,
        resolved_mode: UiTextRenderMode,
    ) {
        match resolved_mode {
            UiTextRenderMode::Auto | UiTextRenderMode::Native => self.native_texts.push(text),
            UiTextRenderMode::Sdf => self.sdf_texts.push(text),
        }
    }

    fn native_texts(&self) -> &[ScreenSpaceUiTextBatch] {
        &self.native_texts
    }

    fn sdf_texts(&self) -> &[ScreenSpaceUiTextBatch] {
        &self.sdf_texts
    }

    fn sdf_atlas_texts(&self) -> &[ScreenSpaceUiTextBatch] {
        &self.sdf_texts
    }
}

#[derive(Clone, Debug, Default)]
struct LoadedUiFontAsset {
    family: Option<String>,
    render_mode: Option<UiTextRenderMode>,
}

impl ScreenSpaceUiTextSystem {
    pub(super) fn new(
        asset_manager: Arc<ProjectAssetManager>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        let mut font_system = FontSystem::new();
        let mut font_database = FontDatabase::with_default_fallbacks();
        font_database.load_system_fonts();
        let mut font_assets = HashMap::new();
        let default_font = load_font_asset_record(
            &mut font_system,
            &mut font_database,
            DEFAULT_FONT_ASSET,
            &asset_manager,
        );
        if let Some(record) = default_font.as_ref() {
            if let Some(family) = record.family.as_deref() {
                font_system
                    .db_mut()
                    .set_sans_serif_family(family.to_string());
                font_system
                    .db_mut()
                    .set_monospace_family(family.to_string());
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
        let mut resolved_texts = resolve_text_batches(
            &mut self.font_system,
            &mut self.font_database,
            &mut self.font_assets,
            self.asset_manager.as_ref(),
            auto_texts,
            native_texts,
            sdf_texts,
        );
        self.sdf_atlas.prepare(resolved_texts.sdf_atlas_texts());
        let sdf_fallback_glyph_advances =
            self.sdf_renderer.measure_text_glyph_advances_for_fallbacks(
                resolved_texts.sdf_atlas_texts(),
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
                .discard_cached_slots_not_in_texts(resolved_texts.sdf_atlas_texts());
            self.sdf_atlas.prepare(resolved_texts.sdf_atlas_texts());
        }
        let sdf_atlas_report = self.sdf_atlas.cache_report();
        self.sdf_renderer.prepare(
            device,
            queue,
            viewport_size,
            resolved_texts.sdf_texts(),
            self.sdf_atlas.plan(),
            sdf_atlas_report.clone(),
            &mut self.font_database,
            self.asset_manager.as_ref(),
        );
        let sdf_renderer_report = self.sdf_renderer.prepare_report();
        let native_prepare_report = self.native.prepare(
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
        );
        let bitmap_atlas_renderer_report = self.bitmap_atlas_renderer.prepare_report();
        self.last_prepare_report = text_prepare_report(
            auto_texts,
            native_texts,
            sdf_texts,
            &resolved_texts,
            sdf_fallback_report,
            native_prepare_report,
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
    ) -> ScreenSpaceUiNativePrepareReport {
        self.viewport.update(
            queue,
            Resolution {
                width: viewport_size.x.max(1),
                height: viewport_size.y.max(1),
            },
        );

        if texts.is_empty() {
            self.atlas.trim();
            self.render_glyphon = false;
            bitmap_atlas_renderer.prepare_plan(
                device,
                &GlyphAtlasGpuDrawPlan::default(),
                UVec2::new(1, 1),
                1,
                GlyphAtlasStorageFormat::R8Unorm,
            );
            return ScreenSpaceUiNativePrepareReport::default();
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
            accumulate_text_font_id_report(
                &mut font_id_report,
                text,
                family_name.as_deref(),
                font_database,
            );
            let attrs = text_attrs(family_name.as_deref(), text.font_weight, text.style);
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
            buffer.set_text(
                font_system,
                &text.text,
                &attrs,
                Shaping::Advanced,
                Some(native_text_align(text.text_align, text.text_direction)),
            );
            buffer.shape_until_scroll(font_system, false);
            buffers.push(buffer);
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

        let bitmap_frame = native_bitmap_atlas_frame(
            font_system,
            swash_cache,
            viewport_size,
            text_areas.as_slice(),
        );
        let bitmap_atlas_report = bitmap_frame.prepare_report();
        match native_bitmap_atlas_handoff_for_report(&bitmap_atlas_report) {
            NativeBitmapAtlasHandoff::SingleStorageReplacement => {
                bitmap_atlas_renderer.prepare_submission(
                    device,
                    queue,
                    &bitmap_frame.submission,
                    bitmap_frame.source_bytes(),
                    bitmap_atlas_page_size(),
                    bitmap_frame.atlas_layer_count(),
                    bitmap_frame
                        .atlas_storage_format()
                        .unwrap_or(GlyphAtlasStorageFormat::R8Unorm),
                );
                self.render_glyphon = false;
                self.atlas.trim();
            }
            NativeBitmapAtlasHandoff::MixedStorageReplacement => {
                let storage_submissions = bitmap_frame.storage_submissions();
                let renderer_submissions = storage_submissions
                    .iter()
                    .map(|submission| {
                        GlyphAtlasBitmapRendererStorageSubmission::new(
                            &submission.submission,
                            submission.source_bytes(),
                            submission.atlas_layer_count(),
                            submission.storage_format,
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

fn native_bitmap_atlas_handoff_for_report(
    report: &NativeBitmapAtlasPrepareReport,
) -> NativeBitmapAtlasHandoff {
    if report.replaces_glyphon {
        NativeBitmapAtlasHandoff::SingleStorageReplacement
    } else if report.mixed_storage_replacement_ready {
        NativeBitmapAtlasHandoff::MixedStorageReplacement
    } else {
        NativeBitmapAtlasHandoff::GlyphonFallback
    }
}

fn text_prepare_report(
    auto_texts: &[ScreenSpaceUiTextBatch],
    native_texts: &[ScreenSpaceUiTextBatch],
    sdf_texts: &[ScreenSpaceUiTextBatch],
    resolved_texts: &ResolvedScreenSpaceUiTextBatches,
    sdf_fallback: ScreenSpaceUiTextSdfFallbackReport,
    native_prepare: ScreenSpaceUiNativePrepareReport,
    bitmap_atlas_renderer: GlyphAtlasBitmapRendererPrepareReport,
    sdf_atlas: SdfAtlasCacheReport,
    sdf_renderer: ScreenSpaceUiSdfPrepareReport,
) -> ScreenSpaceUiTextPrepareReport {
    ScreenSpaceUiTextPrepareReport {
        input_auto_text_batch_count: auto_texts.len(),
        input_native_text_batch_count: native_texts.len(),
        input_sdf_text_batch_count: sdf_texts.len(),
        resolved_native_text_batch_count: resolved_texts.native_texts().len(),
        resolved_sdf_text_batch_count: resolved_texts.sdf_texts().len(),
        sdf_fallback,
        native_font_ids: native_prepare.font_ids,
        native_bitmap_atlas: native_prepare.bitmap_atlas,
        bitmap_atlas_renderer,
        sdf_atlas,
        sdf_renderer,
    }
}

fn resolve_font_asset_record<'a>(
    font_system: &mut FontSystem,
    font_database: &mut FontDatabase,
    font_assets: &'a mut HashMap<String, LoadedUiFontAsset>,
    asset_manager: &ProjectAssetManager,
    font_asset: Option<&str>,
) -> Option<&'a LoadedUiFontAsset> {
    let asset = font_asset
        .filter(|asset| !asset.trim().is_empty())
        .unwrap_or(DEFAULT_FONT_ASSET);
    Some(ensure_font_asset_record(
        font_system,
        font_database,
        font_assets,
        asset_manager,
        asset,
    ))
}

fn effective_text_render_mode(
    requested_mode: UiTextRenderMode,
    font_asset: Option<&LoadedUiFontAsset>,
) -> UiTextRenderMode {
    resolve_text_render_mode(
        requested_mode,
        font_asset.and_then(|asset| asset.render_mode),
    )
}

fn load_font_asset_record(
    font_system: &mut FontSystem,
    font_database: &mut FontDatabase,
    asset_ref: &str,
    asset_manager: &ProjectAssetManager,
) -> Option<LoadedUiFontAsset> {
    let manifest = load_ui_font_manifest_with_asset_manager(asset_ref, Some(asset_manager))?;
    let face = register_loaded_font_manifest(font_database, &manifest)?;
    let _ = font_database.load_face_into_font_system(face, font_system);
    Some(LoadedUiFontAsset {
        family: manifest.family,
        render_mode: manifest.render_mode,
    })
}

fn register_loaded_font_manifest(
    font_database: &mut FontDatabase,
    manifest: &LoadedUiFontManifest,
) -> Option<crate::core::framework::render::FontFaceId> {
    if let Some(asset) = &manifest.asset {
        return font_database
            .register_font_asset(asset, &manifest.source_path)
            .ok()
            .and_then(|faces| faces.first().copied());
    }

    font_database
        .register_font_file(
            &manifest.source_path,
            manifest.family.as_deref(),
            manifest.face_index,
        )
        .ok()
}

fn ensure_font_asset_record<'a>(
    font_system: &mut FontSystem,
    font_database: &mut FontDatabase,
    font_assets: &'a mut HashMap<String, LoadedUiFontAsset>,
    asset_manager: &ProjectAssetManager,
    asset_ref: &str,
) -> &'a LoadedUiFontAsset {
    font_assets.entry(asset_ref.to_string()).or_insert_with(|| {
        load_font_asset_record(font_system, font_database, asset_ref, asset_manager)
            .unwrap_or_default()
    })
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
