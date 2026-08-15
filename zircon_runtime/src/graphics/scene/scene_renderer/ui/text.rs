use glyphon::{Cache, Color, Resolution, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport};

#[cfg(test)]
use super::atlas_renderer::GlyphAtlasBitmapRendererPrepareReport;
use super::atlas_renderer::{GlyphAtlasBitmapRenderer, GlyphAtlasBitmapRendererStorageSubmission};
use super::render::ScreenSpaceUiTextBatch;
use crate::asset::ProjectAssetManagerAccess;
use crate::core::CoreError;
use crate::text::atlas::GlyphAtlasBitmapPageShadowCommit;
#[cfg(test)]
use crate::text::font::MissingGlyphDiagnosticsReport;
use crate::text::{NativeTextAlign, NativeTextBufferRequest, NativeTextWrap, TextRenderState};
use zircon_runtime_interface::ui::layout::UiFrame;
#[cfg(test)]
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextRenderMode};
use zircon_runtime_interface::ui::surface::{UiTextAlign, UiTextDirection, UiTextWrap};

mod fallback_overlay;
mod font_assets;
mod font_id_report;
mod prepare_report;
mod resolved_batches;
mod sdf_cpu_frame;
mod sdf_fallback;

use self::font_assets::{ensure_font_asset_record, UiFontAssetCache};
use self::font_id_report::{accumulate_text_font_id_report, ScreenSpaceUiTextFontIdReport};
#[cfg(feature = "profiling")]
use self::prepare_report::record_text_prepare_profile;
use self::prepare_report::text_prepare_report;
pub(crate) use self::prepare_report::ScreenSpaceUiTextPrepareReport;
#[cfg(test)]
use self::prepare_report::ScreenSpaceUiTextRasterUploadReport;
use self::resolved_batches::{resolve_text_batches, AutoTextRasterRouter};
use self::sdf_cpu_frame::SdfTextCpuFrame;
use self::sdf_fallback::apply_sdf_atlas_fallbacks_with_cpu_runs;
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
    NativeBitmapAtlasHandoff, NativeBitmapAtlasPrepareReport, NativeBitmapAtlasStorageSubmission,
    NativeBitmapAtlasTextArea,
};

const DEFAULT_FONT_ASSET: &str = "res://fonts/default.font.toml";

pub(super) struct ScreenSpaceUiTextSystem {
    // Keep only versioned identity across frames; each asset operation resolves a bounded Arc.
    asset_manager: ProjectAssetManagerAccess,
    text_state: TextRenderState,
    font_assets: UiFontAssetCache,
    native: ScreenSpaceUiTextBackend,
    bitmap_atlas_renderer: GlyphAtlasBitmapRenderer,
    sdf_atlas: ScreenSpaceUiSdfAtlas,
    sdf_cpu_frame: SdfTextCpuFrame,
    sdf_renderer: ScreenSpaceUiSdfRenderer,
    auto_raster_router: AutoTextRasterRouter,
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
}

impl ScreenSpaceUiTextSystem {
    pub(super) fn new(
        asset_manager: ProjectAssetManagerAccess,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
    ) -> Result<Self, CoreError> {
        let resolved_asset_manager = asset_manager.resolve()?;
        let mut text_state = TextRenderState::new_with_process_raster_worker_budget();
        let mut font_assets = UiFontAssetCache::new();
        let _ = ensure_font_asset_record(
            &mut text_state,
            &mut font_assets,
            resolved_asset_manager.as_ref(),
            DEFAULT_FONT_ASSET,
        );

        Ok(Self {
            asset_manager,
            text_state,
            font_assets,
            native: ScreenSpaceUiTextBackend::new(device, queue, target_format),
            bitmap_atlas_renderer: GlyphAtlasBitmapRenderer::new(device, target_format),
            sdf_atlas: ScreenSpaceUiSdfAtlas::new(),
            sdf_cpu_frame: SdfTextCpuFrame::default(),
            sdf_renderer: ScreenSpaceUiSdfRenderer::new(device, target_format),
            auto_raster_router: AutoTextRasterRouter::default(),
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
        let prepare_report = {
            crate::profile_scope!("runtime", "ui_text.prepare", "screen_space_ui_text");
            let asset_manager = self.asset_manager.resolve()?;
            self.text_state.begin_sdf_generation_frame();
            let mut resolved_texts = resolve_text_batches(
                &mut self.text_state,
                &mut self.font_assets,
                asset_manager.as_ref(),
                &mut self.auto_raster_router,
                auto_texts,
                native_texts,
                sdf_texts,
            );
            if resolved_texts.font_faces_changed() {
                self.invalidate_font_faces();
            }
            self.sdf_atlas.prepare(resolved_texts.sdf_texts());
            let mut sdf_atlas_bake = self.text_state.build_sdf_atlas(
                self.sdf_atlas.plan().atlas_size,
                &self.sdf_atlas.plan().slots,
                asset_manager.as_ref(),
            );
            self.sdf_atlas
                .record_generation_failures(&sdf_atlas_bake.generation_failures);
            let cpu_plan_reused = self.sdf_cpu_frame.prepare(
                resolved_texts.sdf_texts(),
                resolved_texts.native_texts(),
                &mut self.text_state,
                asset_manager.as_ref(),
            );
            let sdf_fallback_report = {
                let (sdf_cpu_runs, native_decoration_metrics) = self.sdf_cpu_frame.outputs_mut();
                apply_sdf_atlas_fallbacks_with_cpu_runs(
                    &mut resolved_texts.native_texts,
                    &mut resolved_texts.sdf_texts,
                    &self.sdf_atlas.plan().runs,
                    sdf_cpu_runs,
                    native_decoration_metrics,
                )
            };
            if sdf_fallback_report.needs_sdf_cpu_rebuild() {
                self.sdf_cpu_frame.invalidate();
            }
            if sdf_fallback_report.has_whole_batch_fallbacks() {
                self.sdf_atlas
                    .discard_cached_slots_not_in_texts(resolved_texts.sdf_texts());
                self.sdf_atlas.prepare(resolved_texts.sdf_texts());
                sdf_atlas_bake = self.text_state.build_sdf_atlas(
                    self.sdf_atlas.plan().atlas_size,
                    &self.sdf_atlas.plan().slots,
                    asset_manager.as_ref(),
                );
                self.sdf_atlas
                    .record_generation_failures(&sdf_atlas_bake.generation_failures);
            }
            let sdf_atlas_report = self.sdf_atlas.cache_report();
            let (sdf_cpu_runs, native_decoration_metrics) = self.sdf_cpu_frame.outputs();
            self.sdf_renderer.prepare(
                device,
                queue,
                viewport_size,
                resolved_texts.sdf_texts(),
                sdf_cpu_runs,
                resolved_texts.native_texts(),
                native_decoration_metrics,
                self.sdf_atlas.plan(),
                &sdf_atlas_bake,
                sdf_atlas_report.clone(),
                cpu_plan_reused,
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
            );
            let bitmap_atlas_renderer_report = self.bitmap_atlas_renderer.prepare_report();
            let missing_glyphs = self.text_state.take_missing_glyph_diagnostics();
            text_prepare_report(
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
            )
        };
        self.last_prepare_report = prepare_report;
        #[cfg(feature = "profiling")]
        record_text_prepare_profile(&self.last_prepare_report);
        Ok(())
    }

    fn invalidate_font_faces(&mut self) {
        self.text_state.invalidate_font_faces();
        self.bitmap_atlas_renderer
            .discard_all_for_face_invalidation();
        self.sdf_atlas.invalidate_font_faces();
        self.sdf_cpu_frame.invalidate();
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
        font_assets: &UiFontAssetCache,
    ) -> ScreenSpaceUiNativePrepareReport {
        crate::profile_scope!(
            "runtime",
            "ui_text.native_raster_plan",
            "native_text_prepare"
        );
        self.viewport.update(
            queue,
            Resolution {
                width: viewport_size.x.max(1),
                height: viewport_size.y.max(1),
            },
        );

        if texts.is_empty() {
            self.disable_glyphon();
            let bitmap_atlas = text_state.prepare_idle_bitmap_atlas();
            bitmap_atlas_renderer.prepare_idle();
            return ScreenSpaceUiNativePrepareReport {
                bitmap_atlas,
                ..ScreenSpaceUiNativePrepareReport::default()
            };
        }

        let mut buffers = Vec::with_capacity(texts.len());
        let mut font_id_report = ScreenSpaceUiTextFontIdReport::default();
        for text in texts {
            let family_name = resolve_family_name(
                font_assets,
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
            // Resolve through TextRenderState; text_state.font_database() remains its canonical source.
            accumulate_text_font_id_report(
                &mut font_id_report,
                &native_buffer.buffer,
                native_buffer.primary_face,
                |backend| text_state.font_face_id(backend),
            );
            buffers.push(native_buffer.buffer);
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
                    scale: placement.raster_scale,
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
        let (bitmap_atlas_report, storage_submissions) =
            bitmap_frame.prepare_report_with_storage_submissions();
        drop(bitmap_text_areas);
        let shadow_commit = match native_bitmap_atlas_handoff_for_report(&bitmap_atlas_report) {
            NativeBitmapAtlasHandoff::SingleStorageReplacement => {
                if let Some(atlas_format) = bitmap_frame.atlas_format() {
                    let shadow_commit = bitmap_atlas_renderer
                        .prepare_submission_with_face_validity(
                            device,
                            queue,
                            &bitmap_frame.submission,
                            bitmap_frame.source_bytes(),
                            bitmap_atlas_page_size(),
                            bitmap_frame.atlas_layer_count(),
                            atlas_format,
                            bitmap_frame.face_validity(),
                        );
                    self.disable_glyphon();
                    shadow_commit
                } else {
                    bitmap_atlas_renderer.prepare_idle();
                    self.render_glyphon = true;
                    text_state.prepare_glyphon_fallback(
                        device,
                        queue,
                        &mut self.renderer,
                        &mut self.atlas,
                        &self.viewport,
                        text_areas,
                    );
                    GlyphAtlasBitmapPageShadowCommit::default()
                }
            }
            NativeBitmapAtlasHandoff::MixedStorageReplacement => {
                let renderer_submissions = native_bitmap_atlas_renderer_storage_submissions(
                    storage_submissions.as_slice(),
                );
                let shadow_commit = bitmap_atlas_renderer.prepare_storage_submissions(
                    device,
                    queue,
                    renderer_submissions.as_slice(),
                    bitmap_atlas_page_size(),
                );
                self.disable_glyphon();
                shadow_commit
            }
            NativeBitmapAtlasHandoff::NoVisibleGlyphs => {
                bitmap_atlas_renderer.prepare_idle();
                self.disable_glyphon();
                GlyphAtlasBitmapPageShadowCommit::default()
            }
            NativeBitmapAtlasHandoff::TransparentPlaceholder => {
                let shadow_commit = prepare_native_bitmap_atlas_transparent_placeholder(
                    device,
                    queue,
                    bitmap_atlas_renderer,
                    &bitmap_frame,
                    storage_submissions.as_slice(),
                );
                self.disable_glyphon();
                shadow_commit
            }
            NativeBitmapAtlasHandoff::GlyphonFallback => {
                bitmap_atlas_renderer.prepare_idle();
                self.render_glyphon = true;
                text_state.prepare_glyphon_fallback(
                    device,
                    queue,
                    &mut self.renderer,
                    &mut self.atlas,
                    &self.viewport,
                    text_areas,
                );
                GlyphAtlasBitmapPageShadowCommit::default()
            }
        };
        let accept_frame_atlas = bitmap_frame.submission.run.upload_copies.is_empty()
            || bitmap_atlas_renderer
                .prepare_report()
                .upload_ready_to_write_texture;
        drop(storage_submissions);
        text_state.finish_bitmap_atlas_frame(bitmap_frame, shadow_commit, accept_frame_atlas);

        ScreenSpaceUiNativePrepareReport {
            font_ids: font_id_report,
            bitmap_atlas: bitmap_atlas_report,
        }
    }

    fn disable_glyphon(&mut self) {
        // Keep the idle atlas reusable during stable native frames; trim only on a real handoff.
        if take_glyphon_atlas_trim_transition(&mut self.render_glyphon) {
            self.atlas.trim();
        }
    }

    fn render<'pass>(&'pass mut self, pass: &mut wgpu::RenderPass<'pass>) {
        if self.render_glyphon {
            let _ = self.renderer.render(&self.atlas, &self.viewport, pass);
        }
    }
}

fn take_glyphon_atlas_trim_transition(render_glyphon: &mut bool) -> bool {
    std::mem::replace(render_glyphon, false)
}

fn prepare_native_bitmap_atlas_transparent_placeholder(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bitmap_atlas_renderer: &mut GlyphAtlasBitmapRenderer,
    bitmap_frame: &NativeBitmapAtlasFrame,
    storage_submissions: &[NativeBitmapAtlasStorageSubmission],
) -> GlyphAtlasBitmapPageShadowCommit {
    if let Some(atlas_format) = bitmap_frame.atlas_format() {
        return bitmap_atlas_renderer.prepare_submission_with_face_validity(
            device,
            queue,
            &bitmap_frame.submission,
            bitmap_frame.source_bytes(),
            bitmap_atlas_page_size(),
            bitmap_frame.atlas_layer_count(),
            atlas_format,
            bitmap_frame.face_validity(),
        );
    }

    if storage_submissions.is_empty() {
        bitmap_atlas_renderer.prepare_idle();
        return GlyphAtlasBitmapPageShadowCommit::default();
    }

    let renderer_submissions =
        native_bitmap_atlas_renderer_storage_submissions(storage_submissions);
    bitmap_atlas_renderer.prepare_storage_submissions(
        device,
        queue,
        renderer_submissions.as_slice(),
        bitmap_atlas_page_size(),
    )
}

fn native_bitmap_atlas_renderer_storage_submissions<'a>(
    storage_submissions: &'a [NativeBitmapAtlasStorageSubmission],
) -> Vec<GlyphAtlasBitmapRendererStorageSubmission<'a>> {
    storage_submissions
        .iter()
        .map(|submission| {
            let source_bytes = (!submission.submission.upload_commands().is_empty())
                .then(|| submission.source_bytes())
                .unwrap_or_default();
            GlyphAtlasBitmapRendererStorageSubmission::new_with_face_validity(
                &submission.submission,
                source_bytes,
                submission.atlas_layer_count(),
                submission.atlas_format,
                submission.face_validity(),
            )
        })
        .collect()
}

fn resolve_family_name(
    font_assets: &UiFontAssetCache,
    font_asset: Option<&str>,
    preferred_family: Option<&str>,
    code: bool,
) -> Option<String> {
    if code {
        return font_assets
            .get(DEFAULT_FONT_ASSET)
            .and_then(|entry| entry.loaded_asset())
            .and_then(|record| record.family.clone());
    }
    if let Some(family) = preferred_family.filter(|family| !family.trim().is_empty()) {
        return Some(family.to_string());
    }

    let asset = font_asset
        .filter(|asset| !asset.trim().is_empty())
        .unwrap_or(DEFAULT_FONT_ASSET);
    font_assets
        .get(asset)
        .and_then(|entry| entry.loaded_asset())
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
    raster_scale: f32,
    bounds: TextBounds,
}

fn native_text_area_placement(
    viewport_size: crate::core::math::UVec2,
    text: &ScreenSpaceUiTextBatch,
) -> NativeTextAreaPlacement {
    NativeTextAreaPlacement {
        left: text_origin_device_px(text.frame.x),
        top: text_origin_device_px(text.frame.y),
        raster_scale: native_text_raster_scale(text.raster_scale),
        bounds: text_bounds(viewport_size, text),
    }
}

fn native_text_raster_scale(scale: f32) -> f32 {
    if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
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
