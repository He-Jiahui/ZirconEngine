use super::atlas_renderer::GlyphAtlasBitmapRenderer;
#[cfg(test)]
use super::atlas_renderer::GlyphAtlasBitmapRendererPrepareReport;
use super::render::{
    PlannedScreenSpaceUi, ScreenSpaceUiResolvedGlyphArtifactRouteReport, ScreenSpaceUiTextBatch,
};
use crate::asset::ProjectAssetManagerAccess;
use crate::core::CoreError;
use crate::graphics::types::GraphicsError;
use crate::text::TextRenderState;
use crate::text::atlas::{GlyphAtlasBitmapPageShadowCommit, GlyphAtlasFormat};
#[cfg(test)]
use crate::text::font::MissingGlyphDiagnosticsReport;
use crate::text::font::{
    DEFAULT_UI_FONT_ASSET as DEFAULT_FONT_ASSET, FontCollectionRevision, FontCollectionService,
    RuntimeFontAssetClaimScope, SystemFontPolicy,
};
#[cfg(test)]
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextRenderMode};

mod fallback_overlay;
mod font_assets;
mod font_id_report;
mod native_glyph_run;
mod prepare_report;
mod resolved_batches;
mod sdf_cpu_frame;
mod sdf_fallback;
mod segment_cache;

pub(crate) use self::font_assets::UiFontAssetCacheReport;
use self::font_assets::{UiFontAssetCache, font_asset_cache_report, refresh_font_asset_records};
use self::font_id_report::ScreenSpaceUiTextFontIdReport;
pub(in crate::graphics::scene::scene_renderer::ui) use self::native_glyph_run::native_bitmap_atlas_glyph_runs;
pub(crate) use self::prepare_report::ScreenSpaceUiTextPrepareReport;
#[cfg(test)]
use self::prepare_report::ScreenSpaceUiTextRasterUploadReport;
#[cfg(feature = "profiling")]
use self::prepare_report::record_text_prepare_profile;
use self::prepare_report::{ScreenSpaceUiResolvedTextReport, text_prepare_report};
use self::resolved_batches::AutoTextRasterRouter;
use self::sdf_cpu_frame::SdfTextCpuFrame;
#[cfg(test)]
use self::sdf_fallback::ScreenSpaceUiTextSdfFallbackReport;
use self::sdf_fallback::apply_sdf_atlas_fallbacks_with_cpu_runs;
use self::segment_cache::{ScreenSpaceUiTextSegmentCache, ScreenSpaceUiTextSegmentProduct};
use super::sdf_atlas::ScreenSpaceUiSdfAtlas;
#[cfg(test)]
use super::sdf_atlas::SdfAtlasCacheReport;
use super::sdf_render::ScreenSpaceUiSdfPrepareReport;
use super::sdf_render::ScreenSpaceUiSdfRenderer;
#[cfg(test)]
use super::sdf_upload::SdfAtlasUploadReport;
use super::text_pixel_snap::text_origin_device_px;
#[cfg(test)]
use crate::text::native_bitmap_atlas;
use crate::text::native_bitmap_atlas::{
    NativeBitmapAtlasFrame, NativeBitmapAtlasHandoff, NativeBitmapAtlasPrepareReport,
    bitmap_atlas_page_size, native_bitmap_atlas_handoff_for_report,
};
use std::sync::Arc;
use zr_rhi_wgpu::{WgpuBufferUploadBatch, WgpuTextureUploadBatch};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ScreenSpaceUiTextFrameProductGeneration(u64);

impl ScreenSpaceUiTextFrameProductGeneration {
    pub(super) fn next(counter: &mut u64) -> Self {
        *counter = counter.saturating_add(1).max(1);
        Self(*counter)
    }
}

#[derive(Default)]
struct ScreenSpaceUiTextAtlasRecoveryState {
    bitmap_full_replay_required: bool,
    sdf_full_replay_required: bool,
}

impl ScreenSpaceUiTextAtlasRecoveryState {
    fn bitmap_force_full_upload(&self, transaction_force_full_upload: bool) -> bool {
        transaction_force_full_upload || self.bitmap_full_replay_required
    }

    fn sdf_force_full_upload(&self, transaction_force_full_upload: bool) -> bool {
        transaction_force_full_upload || self.sdf_full_replay_required
    }

    fn note_bitmap_abort(&mut self, owner_contents_changed: bool) {
        self.bitmap_full_replay_required |= owner_contents_changed;
    }

    fn note_sdf_abort(&mut self, owner_contents_changed: bool) {
        self.sdf_full_replay_required |= owner_contents_changed;
    }

    fn commit_bitmap_recovery(&mut self, full_replay_complete: bool) {
        if full_replay_complete {
            self.bitmap_full_replay_required = false;
        }
    }

    fn commit_sdf_recovery(&mut self, recovery_complete: bool) {
        if recovery_complete {
            self.sdf_full_replay_required = false;
        }
    }
}

fn sdf_owner_has_render_contents(report: &ScreenSpaceUiSdfPrepareReport) -> bool {
    report.atlas_slot_count > 0 || report.vertex_count > 0 || report.draw_count > 0
}

pub(super) struct ScreenSpaceUiTextSystem {
    // Keep only versioned identity across frames; each asset operation resolves a bounded Arc.
    asset_manager: ProjectAssetManagerAccess,
    text_state: TextRenderState,
    font_assets: UiFontAssetCache,
    font_claim_scope: RuntimeFontAssetClaimScope,
    native: ScreenSpaceUiTextBackend,
    bitmap_atlas_renderer: GlyphAtlasBitmapRenderer,
    sdf_atlas: ScreenSpaceUiSdfAtlas,
    sdf_cpu_frame: SdfTextCpuFrame,
    sdf_renderer: ScreenSpaceUiSdfRenderer,
    auto_raster_router: AutoTextRasterRouter,
    segment_cache: ScreenSpaceUiTextSegmentCache,
    atlas_recovery: ScreenSpaceUiTextAtlasRecoveryState,
    pending_bitmap_atlas_frame: Option<PendingNativeBitmapAtlasFrame>,
    pending_bitmap_recovery: Option<PendingBitmapRecovery>,
    pending_sdf_atlas_upload: Option<PendingSdfAtlasUpload>,
    last_prepare_report: ScreenSpaceUiTextPrepareReport,
}

#[derive(Default)]
struct ScreenSpaceUiTextBackend;

#[derive(Clone, Debug, Default)]
struct ScreenSpaceUiNativePrepareReport {
    font_ids: ScreenSpaceUiTextFontIdReport,
    bitmap_atlas: NativeBitmapAtlasPrepareReport,
}

struct ScreenSpaceUiNativePreparation {
    report: ScreenSpaceUiNativePrepareReport,
    pending_bitmap_atlas_frame: Option<PendingNativeBitmapAtlasFrame>,
}

struct PendingNativeBitmapAtlasFrame {
    frame: NativeBitmapAtlasFrame,
    shadow_commit: GlyphAtlasBitmapPageShadowCommit,
    accept_frame_atlas: bool,
}

struct PendingBitmapRecovery {
    owner_contents_changed: bool,
    recovery_complete: bool,
}

struct PendingSdfAtlasUpload {
    owner_contents_changed: bool,
    recovery_complete: bool,
}

impl ScreenSpaceUiTextSystem {
    pub(super) fn new_with_font_collection(
        asset_manager: ProjectAssetManagerAccess,
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        font_collection: Arc<FontCollectionService>,
    ) -> Result<Self, CoreError> {
        let resolved_asset_manager = asset_manager.resolve()?;
        // Screen-space rendering is the explicit platform-font consumer; bind discovery before
        // TextRenderState captures its immutable database snapshot.
        let _ = font_collection.mutate_published_snapshot(|database| {
            database.apply_system_font_policy(SystemFontPolicy::Discover)
        });
        let mut text_state =
            TextRenderState::new_with_font_collection_and_process_raster_worker_budget(Arc::clone(
                &font_collection,
            ));
        let mut font_claim_scope = font_collection.runtime_font_asset_claim_scope();
        let mut font_assets = UiFontAssetCache::new();
        let default_font_dependencies = [Arc::<str>::from(DEFAULT_FONT_ASSET)];
        let _ = refresh_font_asset_records(
            &mut text_state,
            &mut font_assets,
            resolved_asset_manager.as_ref(),
            &default_font_dependencies,
            &mut font_claim_scope,
        );

        Ok(Self {
            asset_manager,
            text_state,
            font_assets,
            font_claim_scope,
            native: ScreenSpaceUiTextBackend,
            bitmap_atlas_renderer: GlyphAtlasBitmapRenderer::new(device, target_format),
            sdf_atlas: ScreenSpaceUiSdfAtlas::new(),
            sdf_cpu_frame: SdfTextCpuFrame::default(),
            sdf_renderer: ScreenSpaceUiSdfRenderer::new(device, target_format),
            auto_raster_router: AutoTextRasterRouter::default(),
            segment_cache: ScreenSpaceUiTextSegmentCache::default(),
            atlas_recovery: ScreenSpaceUiTextAtlasRecoveryState::default(),
            pending_bitmap_atlas_frame: None,
            pending_bitmap_recovery: None,
            pending_sdf_atlas_upload: None,
            last_prepare_report: ScreenSpaceUiTextPrepareReport::default(),
        })
    }

    pub(super) fn published_font_collection_revision(&self) -> FontCollectionRevision {
        self.text_state.published_font_collection_revision()
    }

    pub(super) fn prepare(
        &mut self,
        device: &wgpu::Device,
        viewport_size: crate::core::math::UVec2,
        render_segments: &[Arc<PlannedScreenSpaceUi>],
        resolved_glyph_artifact_routes: ScreenSpaceUiResolvedGlyphArtifactRouteReport,
        buffer_uploads: &mut WgpuBufferUploadBatch,
        texture_uploads: &mut WgpuTextureUploadBatch,
        force_full_upload: bool,
    ) -> Result<(), GraphicsError> {
        self.abort_pending_uploads();
        let sdf_force_full_upload = self.atlas_recovery.sdf_force_full_upload(force_full_upload);
        let bitmap_force_full_upload = self
            .atlas_recovery
            .bitmap_force_full_upload(force_full_upload);
        let prepare_report = {
            crate::profile_scope!("runtime", "ui_text.prepare", "screen_space_ui_text");
            let asset_manager = self
                .asset_manager
                .resolve()
                .map_err(|error| GraphicsError::Asset(error.to_string()))?;
            self.text_state.begin_sdf_generation_frame();
            self.auto_raster_router.begin_frame();
            self.segment_cache
                .refresh_font_dependencies(render_segments);
            let active_font_dependencies = self.segment_cache.active_font_dependencies();
            let font_refresh = refresh_font_asset_records(
                &mut self.text_state,
                &mut self.font_assets,
                asset_manager.as_ref(),
                active_font_dependencies,
                &mut self.font_claim_scope,
            );
            let claim_report = font_refresh.claims;
            crate::profile_counter!(
                "runtime",
                "ui_text.font_asset_claim_added_count",
                claim_report.added_claim_count
            );
            crate::profile_counter!(
                "runtime",
                "ui_text.font_asset_claim_released_count",
                claim_report.released_claim_count
            );
            crate::profile_counter!(
                "runtime",
                "ui_text.font_asset_claim_unclaimed_count",
                claim_report.unclaimed_asset_count
            );
            crate::profile_counter!(
                "runtime",
                "ui_text.font_asset_claim_font_inputs_changed",
                u8::from(claim_report.font_inputs_changed)
            );
            let shaping_changed = font_refresh.font_collection_changed;
            let font_faces_changed = shaping_changed;
            let font_records_reloaded = font_refresh.font_records_reloaded;
            if font_faces_changed {
                self.invalidate_font_faces();
            } else if font_records_reloaded {
                self.segment_cache.invalidate_frame_product();
            }
            let frame_product = self.segment_cache.prepare_frame_product(
                render_segments,
                viewport_size,
                self.text_state.font_collection_revision(),
                &self.font_assets,
                &mut self.auto_raster_router,
                shaping_changed,
                &self.text_state.font_collection(),
            );
            self.sdf_atlas.prepare_retained_segments(
                frame_product.sdf_text_segments(),
                frame_product.generation(),
            );
            let mut sdf_atlas_bake = self.text_state.build_sdf_atlas(
                self.sdf_atlas.plan().atlas_size,
                &self.sdf_atlas.plan().slots,
                asset_manager.as_ref(),
            );
            self.sdf_atlas
                .record_generation_failures(&sdf_atlas_bake.generation_failures);
            let cpu_plan_reused = self.sdf_cpu_frame.prepare_retained_segments(
                frame_product.sdf_text_segments(),
                frame_product.native_text_segments(),
                &mut self.text_state,
                asset_manager.as_ref(),
                frame_product.generation(),
            );
            debug_assert_eq!(
                frame_product.sdf_run_count(),
                self.sdf_atlas.plan().runs.len()
            );
            let (sdf_cpu_runs, native_decoration_metrics) = self.sdf_cpu_frame.outputs();
            debug_assert_eq!(frame_product.sdf_run_count(), sdf_cpu_runs.len());
            debug_assert_eq!(
                frame_product.native_run_count(),
                native_decoration_metrics.len()
            );
            let needs_sdf_fallback = frame_product.sdf_run_count()
                != self.sdf_atlas.plan().runs.len()
                || self
                    .sdf_atlas
                    .plan()
                    .runs
                    .iter()
                    .any(super::sdf_atlas::SdfAtlasRun::has_failures);
            let mut fallback_resolved_texts = None;
            let sdf_fallback_report = if needs_sdf_fallback {
                let mut resolved_texts = frame_product.materialize_resolved_texts();
                let (sdf_cpu_runs, native_decoration_metrics) = self.sdf_cpu_frame.outputs_mut();
                let report = apply_sdf_atlas_fallbacks_with_cpu_runs(
                    &mut resolved_texts.native_texts,
                    &mut resolved_texts.sdf_texts,
                    &self.sdf_atlas.plan().runs,
                    sdf_cpu_runs,
                    native_decoration_metrics,
                );
                fallback_resolved_texts = Some(resolved_texts);
                report
            } else {
                let (_, native_decoration_metrics) = self.sdf_cpu_frame.outputs_mut();
                native_decoration_metrics.truncate(frame_product.native_run_count());
                Default::default()
            };
            if sdf_fallback_report.needs_sdf_cpu_rebuild() {
                self.sdf_cpu_frame.invalidate();
            }
            if sdf_fallback_report.has_whole_batch_fallbacks() {
                let Some(resolved_texts) = fallback_resolved_texts.as_ref() else {
                    self.abort_pending_uploads();
                    return Err(GraphicsError::Asset(
                        "screen-space UI SDF fallback lost its resolved text artifact".to_string(),
                    ));
                };
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
            if let Some(resolved_texts) = fallback_resolved_texts.as_ref() {
                self.sdf_renderer.prepare(
                    device,
                    viewport_size,
                    resolved_texts.sdf_texts(),
                    sdf_cpu_runs,
                    resolved_texts.native_texts(),
                    native_decoration_metrics,
                    self.sdf_atlas.plan(),
                    &sdf_atlas_bake,
                    sdf_atlas_report.clone(),
                    cpu_plan_reused,
                    buffer_uploads,
                    texture_uploads,
                    sdf_force_full_upload,
                );
            } else {
                self.sdf_renderer.prepare_retained_segments(
                    device,
                    viewport_size,
                    frame_product.sdf_text_segments(),
                    frame_product.sdf_run_count(),
                    sdf_cpu_runs,
                    frame_product.native_text_segments(),
                    native_decoration_metrics,
                    self.sdf_atlas.plan(),
                    &sdf_atlas_bake,
                    sdf_atlas_report.clone(),
                    cpu_plan_reused,
                    frame_product.generation(),
                    buffer_uploads,
                    texture_uploads,
                    sdf_force_full_upload,
                );
            }
            let sdf_renderer_report = self.sdf_renderer.prepare_report();
            let sdf_owner_active = sdf_owner_has_render_contents(&sdf_renderer_report);
            self.pending_sdf_atlas_upload = Some(PendingSdfAtlasUpload {
                owner_contents_changed: sdf_owner_active,
                recovery_complete: sdf_force_full_upload
                    && (!sdf_owner_active
                        || ((sdf_renderer_report.atlas_slot_count == 0
                            || sdf_renderer_report.atlas_upload_full_texture)
                            && !sdf_renderer_report.atlas_upload_preparation_failed)),
            });
            if sdf_renderer_report.atlas_upload_preparation_failed {
                self.abort_pending_uploads();
                return Err(GraphicsError::Asset(
                    "screen-space UI SDF atlas upload preparation was incomplete".to_string(),
                ));
            }
            let native_preparation = if let Some(resolved_texts) = fallback_resolved_texts.as_ref()
            {
                self.native.prepare(
                    device,
                    viewport_size,
                    resolved_texts.native_texts(),
                    &mut self.bitmap_atlas_renderer,
                    &mut self.text_state,
                    buffer_uploads,
                    texture_uploads,
                    bitmap_force_full_upload,
                )
            } else {
                self.native.prepare_retained(
                    device,
                    viewport_size,
                    frame_product.native_run_count(),
                    frame_product.segment_products(),
                    frame_product.native_font_ids(),
                    &mut self.bitmap_atlas_renderer,
                    &mut self.text_state,
                    buffer_uploads,
                    texture_uploads,
                    bitmap_force_full_upload,
                )
            };
            self.pending_bitmap_atlas_frame = native_preparation.pending_bitmap_atlas_frame;
            let bitmap_atlas_renderer_report = self.bitmap_atlas_renderer.prepare_report();
            let bitmap_owner_active = native_preparation
                .report
                .bitmap_atlas
                .visible_raster_glyph_count
                > 0
                || bitmap_atlas_renderer_report.storage_pass_visible_glyph_count > 0;
            self.pending_bitmap_recovery = Some(PendingBitmapRecovery {
                owner_contents_changed: bitmap_owner_active,
                recovery_complete: bitmap_force_full_upload
                    && (!bitmap_owner_active
                        || (bitmap_atlas_renderer_report.upload_plan_build_count > 0
                            && bitmap_atlas_renderer_report.upload_ready_to_write_texture)),
            });
            if self
                .pending_bitmap_atlas_frame
                .as_ref()
                .is_some_and(|pending| !pending.accept_frame_atlas)
            {
                self.abort_pending_uploads();
                return Err(GraphicsError::Asset(
                    "screen-space UI bitmap atlas upload preparation was incomplete".to_string(),
                ));
            }
            let native_font_id_report = native_preparation.report;
            let missing_glyphs = self.text_state.take_missing_glyph_diagnostics();
            let font_assets = font_asset_cache_report(&self.font_assets);
            let resolved_report = fallback_resolved_texts
                .as_ref()
                .map(ScreenSpaceUiResolvedTextReport::from_resolved_texts)
                .unwrap_or_else(|| frame_product.resolved_report());
            text_prepare_report(
                frame_product.input_batch_counts(),
                self.auto_raster_router.frame_report(),
                resolved_glyph_artifact_routes,
                resolved_report,
                sdf_fallback_report,
                native_font_id_report,
                font_assets,
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

    pub(super) fn clear_frame_state(&mut self) {
        self.abort_pending_uploads();
        self.segment_cache.invalidate_frame_product();
        self.auto_raster_router.clear_active_routes();
        self.last_prepare_report = ScreenSpaceUiTextPrepareReport::default();
    }

    fn invalidate_font_faces(&mut self) {
        self.text_state.invalidate_font_faces();
        self.bitmap_atlas_renderer
            .discard_all_for_face_invalidation();
        self.sdf_atlas.invalidate_font_faces();
        self.sdf_cpu_frame.invalidate();
        self.segment_cache.invalidate_frame_product();
    }

    pub(super) fn render<'pass>(&'pass mut self, pass: &mut wgpu::RenderPass<'pass>) {
        self.bitmap_atlas_renderer.render(pass);
        self.sdf_renderer.render(pass);
    }

    pub(super) fn prepare_report(&self) -> ScreenSpaceUiTextPrepareReport {
        self.last_prepare_report.clone()
    }

    pub(super) fn commit_prepared_uploads(&mut self) {
        if let Some(pending) = self.pending_sdf_atlas_upload.take() {
            if pending.owner_contents_changed {
                self.sdf_atlas.mark_prepared_pages_uploaded();
            }
            self.atlas_recovery
                .commit_sdf_recovery(pending.recovery_complete);
        }
        if let Some(pending) = self.pending_bitmap_recovery.take() {
            self.atlas_recovery
                .commit_bitmap_recovery(pending.recovery_complete);
        }
        if let Some(pending) = self.pending_bitmap_atlas_frame.take() {
            self.text_state.finish_bitmap_atlas_frame(
                pending.frame,
                pending.shadow_commit,
                pending.accept_frame_atlas,
            );
        }
    }

    fn abort_pending_uploads(&mut self) {
        if let Some(pending) = self.pending_sdf_atlas_upload.take() {
            self.atlas_recovery
                .note_sdf_abort(pending.owner_contents_changed);
        }
        if let Some(pending) = self.pending_bitmap_recovery.take() {
            self.atlas_recovery
                .note_bitmap_abort(pending.owner_contents_changed);
        }
        if let Some(pending) = self.pending_bitmap_atlas_frame.take() {
            self.text_state
                .finish_bitmap_atlas_frame(pending.frame, pending.shadow_commit, false);
        }
    }
}

impl ScreenSpaceUiTextBackend {
    #[allow(clippy::too_many_arguments)]
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        viewport_size: crate::core::math::UVec2,
        texts: &[ScreenSpaceUiTextBatch],
        bitmap_atlas_renderer: &mut GlyphAtlasBitmapRenderer,
        text_state: &mut TextRenderState,
        buffer_uploads: &mut WgpuBufferUploadBatch,
        texture_uploads: &mut WgpuTextureUploadBatch,
        force_full_upload: bool,
    ) -> ScreenSpaceUiNativePreparation {
        crate::profile_scope!(
            "runtime",
            "ui_text.native_raster_plan",
            "native_text_prepare"
        );
        if texts.is_empty() {
            let bitmap_atlas = text_state.prepare_idle_bitmap_atlas();
            bitmap_atlas_renderer.prepare_idle();
            return ScreenSpaceUiNativePreparation {
                report: ScreenSpaceUiNativePrepareReport {
                    bitmap_atlas,
                    ..ScreenSpaceUiNativePrepareReport::default()
                },
                pending_bitmap_atlas_frame: None,
            };
        }

        let generated_glyph_run_projection = native_bitmap_atlas_glyph_runs(viewport_size, texts);
        let bitmap_frame = text_state.prepare_bitmap_atlas(
            viewport_size,
            generated_glyph_run_projection.glyph_runs.iter(),
        );
        prepare_native_bitmap_atlas_frame(
            device,
            generated_glyph_run_projection.font_ids,
            bitmap_atlas_renderer,
            bitmap_frame,
            buffer_uploads,
            texture_uploads,
            force_full_upload,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn prepare_retained(
        &mut self,
        device: &wgpu::Device,
        viewport_size: crate::core::math::UVec2,
        native_text_batch_count: usize,
        segment_products: &[Arc<ScreenSpaceUiTextSegmentProduct>],
        font_ids: ScreenSpaceUiTextFontIdReport,
        bitmap_atlas_renderer: &mut GlyphAtlasBitmapRenderer,
        text_state: &mut TextRenderState,
        buffer_uploads: &mut WgpuBufferUploadBatch,
        texture_uploads: &mut WgpuTextureUploadBatch,
        force_full_upload: bool,
    ) -> ScreenSpaceUiNativePreparation {
        crate::profile_scope!(
            "runtime",
            "ui_text.native_raster_plan",
            "native_text_prepare"
        );
        if native_text_batch_count == 0 {
            let bitmap_atlas = text_state.prepare_idle_bitmap_atlas();
            bitmap_atlas_renderer.prepare_idle();
            return ScreenSpaceUiNativePreparation {
                report: ScreenSpaceUiNativePrepareReport {
                    bitmap_atlas,
                    ..ScreenSpaceUiNativePrepareReport::default()
                },
                pending_bitmap_atlas_frame: None,
            };
        }

        let bitmap_frame = text_state.prepare_bitmap_atlas(
            viewport_size,
            segment_products
                .iter()
                .flat_map(|product| product.native_glyph_runs()),
        );
        prepare_native_bitmap_atlas_frame(
            device,
            font_ids,
            bitmap_atlas_renderer,
            bitmap_frame,
            buffer_uploads,
            texture_uploads,
            force_full_upload,
        )
    }
}

fn prepare_native_bitmap_atlas_frame(
    device: &wgpu::Device,
    font_ids: ScreenSpaceUiTextFontIdReport,
    bitmap_atlas_renderer: &mut GlyphAtlasBitmapRenderer,
    bitmap_frame: NativeBitmapAtlasFrame,
    buffer_uploads: &mut WgpuBufferUploadBatch,
    texture_uploads: &mut WgpuTextureUploadBatch,
    force_full_upload: bool,
) -> ScreenSpaceUiNativePreparation {
    let bitmap_atlas_report = bitmap_frame.prepare_report();
    let shadow_commit = match native_bitmap_atlas_handoff_for_report(&bitmap_atlas_report) {
        NativeBitmapAtlasHandoff::SingleStorageReplacement
        | NativeBitmapAtlasHandoff::MixedStorageReplacement => {
            let shadow_commit = prepare_native_bitmap_atlas_frame_submission(
                device,
                bitmap_atlas_renderer,
                &bitmap_frame,
                buffer_uploads,
                texture_uploads,
                force_full_upload,
            );
            shadow_commit
        }
        NativeBitmapAtlasHandoff::NoVisibleGlyphs => {
            bitmap_atlas_renderer.prepare_idle();
            GlyphAtlasBitmapPageShadowCommit::default()
        }
        NativeBitmapAtlasHandoff::TransparentPlaceholder => {
            let shadow_commit = prepare_native_bitmap_atlas_transparent_placeholder(
                device,
                bitmap_atlas_renderer,
                &bitmap_frame,
                buffer_uploads,
                texture_uploads,
                force_full_upload,
            );
            shadow_commit
        }
        NativeBitmapAtlasHandoff::Degraded => prepare_native_bitmap_atlas_transparent_placeholder(
            device,
            bitmap_atlas_renderer,
            &bitmap_frame,
            buffer_uploads,
            texture_uploads,
            force_full_upload,
        ),
    };
    let renderer_report = bitmap_atlas_renderer.prepare_report();
    let accept_frame_atlas = native_bitmap_atlas_frame_acceptance(
        renderer_report.upload_plan_build_count > 0,
        renderer_report.upload_ready_to_write_texture,
    );
    ScreenSpaceUiNativePreparation {
        report: ScreenSpaceUiNativePrepareReport {
            font_ids,
            bitmap_atlas: bitmap_atlas_report,
        },
        pending_bitmap_atlas_frame: Some(PendingNativeBitmapAtlasFrame {
            frame: bitmap_frame,
            shadow_commit,
            accept_frame_atlas,
        }),
    }
}

fn native_bitmap_atlas_frame_acceptance(
    upload_plan_built: bool,
    upload_ready_to_write_texture: bool,
) -> bool {
    !upload_plan_built || upload_ready_to_write_texture
}

fn prepare_native_bitmap_atlas_frame_submission(
    device: &wgpu::Device,
    bitmap_atlas_renderer: &mut GlyphAtlasBitmapRenderer,
    bitmap_frame: &NativeBitmapAtlasFrame,
    buffer_uploads: &mut WgpuBufferUploadBatch,
    texture_uploads: &mut WgpuTextureUploadBatch,
    force_full_upload: bool,
) -> GlyphAtlasBitmapPageShadowCommit {
    bitmap_atlas_renderer.prepare_submission_with_face_validity(
        device,
        &bitmap_frame.submission,
        bitmap_frame.source_bytes(),
        bitmap_atlas_page_size(),
        bitmap_frame.atlas_layer_count(),
        bitmap_frame
            .atlas_format()
            .unwrap_or(GlyphAtlasFormat::AlphaMask),
        bitmap_frame.face_validity(),
        buffer_uploads,
        texture_uploads,
        force_full_upload,
    )
}

fn prepare_native_bitmap_atlas_transparent_placeholder(
    device: &wgpu::Device,
    bitmap_atlas_renderer: &mut GlyphAtlasBitmapRenderer,
    bitmap_frame: &NativeBitmapAtlasFrame,
    buffer_uploads: &mut WgpuBufferUploadBatch,
    texture_uploads: &mut WgpuTextureUploadBatch,
    force_full_upload: bool,
) -> GlyphAtlasBitmapPageShadowCommit {
    prepare_native_bitmap_atlas_frame_submission(
        device,
        bitmap_atlas_renderer,
        bitmap_frame,
        buffer_uploads,
        texture_uploads,
        force_full_upload,
    )
}

#[cfg(test)]
mod tests;
