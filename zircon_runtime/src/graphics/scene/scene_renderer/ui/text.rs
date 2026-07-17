use std::collections::HashMap;

#[cfg(test)]
use glyphon::FontSystem;
use glyphon::{Cache, Color, Resolution, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport};

#[cfg(test)]
use super::atlas_renderer::GlyphAtlasBitmapRendererPrepareReport;
use super::atlas_renderer::{GlyphAtlasBitmapRenderer, GlyphAtlasBitmapRendererStorageSubmission};
use super::render::ScreenSpaceUiTextBatch;
use crate::asset::{ProjectAssetManager, ProjectAssetManagerAccess};
use crate::core::math::UVec2;
use crate::core::CoreError;
use crate::text::atlas::render_gpu_plan::GlyphAtlasGpuDrawPlan;
use crate::text::atlas::GlyphAtlasStorageFormat;
#[cfg(test)]
use crate::text::font::FontDatabase;
#[cfg(test)]
use crate::text::font::MissingGlyphDiagnosticsReport;
#[cfg(test)]
use crate::text::font::{shared_font_database_snapshot, SystemFontPolicy};
use crate::text::{NativeTextAlign, NativeTextBufferRequest, NativeTextWrap, TextRenderState};
use zircon_runtime_interface::ui::layout::UiFrame;
#[cfg(test)]
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextRenderMode};
use zircon_runtime_interface::ui::surface::{UiTextAlign, UiTextDirection, UiTextWrap};

mod font_assets;
mod font_id_report;
mod prepare_report;
mod resolved_batches;
mod sdf_fallback;

use self::font_assets::{ensure_font_asset_record, load_font_asset_record, LoadedUiFontAsset};
use self::font_id_report::{accumulate_text_font_id_report, ScreenSpaceUiTextFontIdReport};
use self::prepare_report::text_prepare_report;
pub(crate) use self::prepare_report::ScreenSpaceUiTextPrepareReport;
#[cfg(test)]
use self::prepare_report::ScreenSpaceUiTextRasterUploadReport;
use self::resolved_batches::resolve_text_batches;
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
#[cfg(test)]
use crate::text::native_bitmap_atlas;
use crate::text::native_bitmap_atlas::{
    bitmap_atlas_page_size, native_bitmap_atlas_handoff_for_report, NativeBitmapAtlasFrame,
    NativeBitmapAtlasHandoff, NativeBitmapAtlasPrepareReport, NativeBitmapAtlasTextArea,
};

const DEFAULT_FONT_ASSET: &str = "res://fonts/default.font.toml";
const NATIVE_BITMAP_ATLAS_RASTER_WORKER_COUNT: usize = 1;

#[cfg(test)]
fn initialize_screen_space_ui_font_system(
    font_system: &mut FontSystem,
    font_database: &mut FontDatabase,
) -> usize {
    let discovered = font_database.apply_system_font_policy(SystemFontPolicy::Discover);
    font_database.sync_font_system(font_system);
    discovered
}

pub(super) struct ScreenSpaceUiTextSystem {
    // Keep only versioned identity across frames; each asset operation resolves a bounded Arc.
    asset_manager: ProjectAssetManagerAccess,
    text_state: TextRenderState,
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
    render_glyphon: bool,
}

#[derive(Clone, Debug, Default)]
struct ScreenSpaceUiNativePrepareReport {
    font_ids: ScreenSpaceUiTextFontIdReport,
    bitmap_atlas: NativeBitmapAtlasPrepareReport,
    font_faces_changed: bool,
}

impl ScreenSpaceUiTextSystem {
    pub(super) fn new(
        asset_manager: ProjectAssetManagerAccess,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
    ) -> Result<Self, CoreError> {
        let resolved_asset_manager = asset_manager.resolve()?;
        let mut text_state = TextRenderState::new(NATIVE_BITMAP_ATLAS_RASTER_WORKER_COUNT);
        let mut font_assets = HashMap::new();
        let default_font = load_font_asset_record(
            &mut text_state,
            DEFAULT_FONT_ASSET,
            resolved_asset_manager.as_ref(),
        );
        text_state.set_project_composite_font(
            default_font
                .as_ref()
                .and_then(|record| record.composite_font.clone()),
        );
        if let Some(record) = default_font.as_ref() {
            if let Some(family) = record.family.as_deref() {
                text_state.set_default_ui_family(family);
            }
            font_assets.insert(DEFAULT_FONT_ASSET.to_string(), record.clone());
        }
        text_state.publish_font_database();

        Ok(Self {
            asset_manager,
            text_state,
            font_assets,
            native: ScreenSpaceUiTextBackend::new(device, queue, target_format),
            bitmap_atlas_renderer: GlyphAtlasBitmapRenderer::new(device, target_format),
            sdf_atlas: ScreenSpaceUiSdfAtlas::new(),
            sdf_renderer: ScreenSpaceUiSdfRenderer::new(device, target_format),
            last_prepare_report: ScreenSpaceUiTextPrepareReport::default(),
        })
    }

    pub(super) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        viewport_size: crate::core::math::UVec2,
        auto_texts: &[ScreenSpaceUiTextBatch],
        native_texts: &[ScreenSpaceUiTextBatch],
        sdf_texts: &[ScreenSpaceUiTextBatch],
    ) -> Result<(), CoreError> {
        let asset_manager = self.asset_manager.resolve()?;
        let mut resolved_texts = resolve_text_batches(
            &mut self.text_state,
            &mut self.font_assets,
            asset_manager.as_ref(),
            auto_texts,
            native_texts,
            sdf_texts,
        );
        if resolved_texts.font_faces_changed() {
            self.invalidate_font_faces();
        }
        self.sdf_atlas.prepare(resolved_texts.sdf_texts());
        let sdf_generation_failures = self.sdf_renderer.generation_failures_for_plan(
            self.sdf_atlas.plan(),
            &mut self.text_state,
            asset_manager.as_ref(),
        );
        self.sdf_atlas
            .record_generation_failures(&sdf_generation_failures);
        let sdf_fallback_glyph_advances =
            self.sdf_renderer.measure_text_glyph_advances_for_fallbacks(
                resolved_texts.sdf_texts(),
                &mut self.text_state,
                asset_manager.as_ref(),
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
                &mut self.text_state,
                asset_manager.as_ref(),
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
            &mut self.text_state,
            asset_manager.as_ref(),
        );
        self.sdf_atlas.mark_prepared_pages_uploaded();
        let sdf_renderer_report = self.sdf_renderer.prepare_report();
        let native_font_id_report = self.native.prepare(
            device,
            queue,
            viewport_size,
            resolved_texts.native_texts(),
            &mut self.bitmap_atlas_renderer,
            &mut self.text_state,
            &mut self.font_assets,
            asset_manager.as_ref(),
        );
        if native_font_id_report.font_faces_changed {
            self.sdf_atlas.invalidate_font_faces();
        }
        let bitmap_atlas_renderer_report = self.bitmap_atlas_renderer.prepare_report();
        let missing_glyphs = self.text_state.take_missing_glyph_diagnostics();
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
        Ok(())
    }

    fn invalidate_font_faces(&mut self) {
        self.text_state.invalidate_font_faces();
        self.bitmap_atlas_renderer
            .discard_all_for_face_invalidation();
        self.sdf_atlas.invalidate_font_faces();
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
        text_state: &mut TextRenderState,
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

        let font_asset_count_at_entry = font_assets.len();

        if texts.is_empty() {
            self.atlas.trim();
            self.render_glyphon = false;
            let bitmap_atlas = text_state.prepare_idle_bitmap_atlas();
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
                text_state,
                font_assets,
                asset_manager,
                text.font.as_deref(),
                text.font_family.as_deref(),
                text.style.code,
            );
            let native_buffer = text_state.shape_native_buffer(NativeTextBufferRequest {
                text: &text.text,
                font_asset: text.font.as_deref(),
                family: family_name.as_deref(),
                language: text.language.as_deref(),
                font_weight: text.font_weight,
                font_size: text.font_size,
                line_height: text.line_height,
                width: text.frame.width,
                height: text.frame.height,
                direction: text.text_direction.into(),
                wrap: native_text_wrap(text.wrap),
                align: native_text_align(text.text_align, text.text_direction),
                strong: text.style.strong,
                emphasis: text.style.emphasis,
                code: text.style.code,
            });
            accumulate_text_font_id_report(
                &mut font_id_report,
                &native_buffer.buffer,
                native_buffer.primary_face,
                text_state.font_database(),
            );
            buffers.push(native_buffer.buffer);
        }
        let font_faces_changed = font_assets.len() != font_asset_count_at_entry;
        if font_faces_changed {
            text_state.invalidate_font_faces();
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

        let bitmap_frame =
            text_state.prepare_bitmap_atlas(viewport_size, bitmap_text_areas.as_slice());
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
                text_state.prepare_glyphon_fallback(
                    device,
                    queue,
                    &mut self.renderer,
                    &mut self.atlas,
                    &self.viewport,
                    text_areas,
                );
            }
        }

        ScreenSpaceUiNativePrepareReport {
            font_ids: font_id_report,
            bitmap_atlas: bitmap_atlas_report,
            font_faces_changed,
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

fn resolve_family_name(
    text_state: &mut TextRenderState,
    font_assets: &mut HashMap<String, LoadedUiFontAsset>,
    asset_manager: &ProjectAssetManager,
    font_asset: Option<&str>,
    preferred_family: Option<&str>,
    code: bool,
) -> Option<String> {
    if code {
        return ensure_font_asset_record(
            text_state,
            font_assets,
            asset_manager,
            DEFAULT_FONT_ASSET,
        )
        .record
        .and_then(|record| record.family.clone());
    }
    if let Some(family) = preferred_family.filter(|family| !family.trim().is_empty()) {
        if let Some(asset) = font_asset.filter(|asset| !asset.trim().is_empty()) {
            let _ = ensure_font_asset_record(text_state, font_assets, asset_manager, asset);
        }
        return Some(family.to_string());
    }

    let asset = font_asset
        .filter(|asset| !asset.trim().is_empty())
        .unwrap_or(DEFAULT_FONT_ASSET);
    ensure_font_asset_record(text_state, font_assets, asset_manager, asset)
        .record
        .and_then(|record| record.family.clone())
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

fn native_text_align(align: UiTextAlign, direction: UiTextDirection) -> NativeTextAlign {
    match align {
        UiTextAlign::Left => NativeTextAlign::Left,
        UiTextAlign::Center => NativeTextAlign::Center,
        UiTextAlign::Right => NativeTextAlign::Right,
        UiTextAlign::Start if matches!(direction, UiTextDirection::RightToLeft) => {
            NativeTextAlign::Right
        }
        UiTextAlign::Start => NativeTextAlign::Left,
        UiTextAlign::End if matches!(direction, UiTextDirection::RightToLeft) => {
            NativeTextAlign::Left
        }
        UiTextAlign::End => NativeTextAlign::Right,
        UiTextAlign::Justify => NativeTextAlign::Justified,
    }
}

fn native_text_wrap(wrap: UiTextWrap) -> NativeTextWrap {
    match wrap {
        UiTextWrap::None => NativeTextWrap::None,
        UiTextWrap::Word | UiTextWrap::WordSmart => NativeTextWrap::Word,
        UiTextWrap::Glyph => NativeTextWrap::Glyph,
    }
}

#[cfg(test)]
mod tests;
