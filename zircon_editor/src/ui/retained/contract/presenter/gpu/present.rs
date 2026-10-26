use zircon_runtime::rhi::{UiSurfaceDrawList, UiSurfacePresentStats, UiSurfacePresenter};

use super::super::super::chrome_command_stream::{
    build_chrome_command_stream_with_residency,
    ui_surface_draw_list_from_owned_stream_with_generation_and_residency,
    ui_surface_draw_list_from_owned_stream_with_residency,
    ui_surface_draw_list_from_stream_with_residency, ChromeCommandStream,
};
use super::super::super::data::{
    FrameRect, HostPresentationGenerationCursor, HostWindowPresentationData,
};
use super::super::super::diagnostics::{HostInvalidationDiagnostics, HostRefreshDiagnostics};
use super::geometry::{damage_pixel_count, full_surface_pixels};
use super::stats::record_present_stats;
use super::GpuChromePresenter;
use crate::ui::retained_host::host_contract::presenter::error::{
    HostPresenterError, HostPresenterResult,
};
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

impl<P: UiSurfacePresenter> GpuChromePresenter<P> {
    pub(in crate::ui::retained_host::host_contract) fn present(
        &mut self,
        presentation: &HostWindowPresentationData,
        presentation_cursor: HostPresentationGenerationCursor,
        damage: Option<FrameRect>,
        invalidation: HostInvalidationDiagnostics,
    ) -> HostPresenterResult<HostRefreshDiagnostics> {
        if self
            .normal_capture_controller
            .before_present(presentation_cursor, self.size)
        {
            return Ok(self.diagnostics.clone());
        }
        self.submitted_text_profile = None;
        self.surface.set_text_layout_observation(
            super::super::super::profiling_artifacts::profile_capture_enabled(),
        );
        self.native_resize_draw_list = None;
        self.native_resize_projection_size = self.size;
        // Explicit capture observes all current visible text by actually submitting a full stream.
        let stream_damage = observed_stream_damage(
            damage.as_ref(),
            self.surface_cache_initialized,
            super::super::super::profiling_artifacts::profile_capture_enabled(),
        );
        let capture_paint = super::super::super::profiling_artifacts::profile_capture_enabled()
            .then(|| super::super::super::paint_template_nodes::PaintEvidenceScope::begin_for_normal_present(
                &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")));
        let stream = build_chrome_command_stream_with_residency(
            presentation,
            self.size,
            stream_damage,
            true,
            |resource_key, generation| {
                self.surface
                    .is_image_resource_resident(resource_key, generation)
            },
        );
        let mut sources = super::super::super::profiling_artifacts::profile_capture_enabled()
            .then(|| super::super::super::profiling_artifacts::source_rows(&stream, presentation));
        let mut paint = capture_paint.map(|scope| scope.and_then(|scope| scope.finish()));
        if let (Some(sources), Some(Ok(paint))) = (sources.as_mut(), paint.as_mut()) {
            super::super::super::profiling_artifacts::native_hierarchy_draws::bind_native_hierarchy_draws(
                &stream,presentation,sources,paint);
        }
        let mut normal_evidence = sources.as_ref().zip(paint).map(|(sources, paint)| {
            super::super::super::profiling_artifacts::stream_observations(
                &stream,
                presentation,
                sources,
                paint,
            )
        });
        let command_stream_full_rebuild = stream.is_full_rebuild();
        let submitted_damage = stream.damage().cloned();
        let region_present = submitted_damage.is_some();
        let surface_size = stream.surface_size();
        let scene_image = sources
            .as_ref()
            .map(|sources| {
                super::super::super::profiling_artifacts::normal_present::scene_image_selection(
                    presentation,
                    &stream,
                    sources,
                )
            })
            .unwrap_or(Ok(None));
        // A damage stream contains only commands for the damaged region. It is not a complete
        // projection and must never share the full-projection cache key across presents.
        let draw_list = ui_surface_draw_list_from_owned_stream_with_residency(
            stream,
            |resource_key, generation| {
                self.surface
                    .is_image_resource_resident(resource_key, generation)
            },
        );
        // Freeze attribution before stream consumption, then verify the same actual RHI command.
        if let Some(request) = self.normal_capture_controller.prepare_frame_readback(
            presentation_cursor,
            &draw_list,
            scene_image,
        ) {
            if let Err(error) = self.surface.request_frame_readback(request) {
                self.normal_capture_controller.fail_frame_readback(&format!(
                    "normal GPU readback unsupported or refused: {error}"
                ));
            }
        }
        let result = self.present_draw_list_with_damage_diagnostics(
            draw_list,
            surface_size,
            region_present,
            submitted_damage.as_ref(),
            invalidation,
        );
        if result.is_ok() {
            if let Some(evidence) = normal_evidence.as_mut() {
                super::super::super::profiling_artifacts::certify_image_residency(
                    evidence,
                    |key, generation| self.surface.is_image_resource_resident(key, generation),
                );
            }
            if let (Some(snapshot), Some(sources)) =
                (self.surface.last_submitted_text_layout(), sources)
            {
                self.submitted_text_profile =
                    Some(super::super::super::profiling_artifacts::bind_submitted(
                        snapshot,
                        &sources,
                        command_stream_full_rebuild,
                    ));
                if let (Some(profile), Some(mut evidence)) =
                    (self.submitted_text_profile.as_mut(), normal_evidence)
                {
                    evidence["presentedFrameCount"] = profile["presentedFrameCount"].clone();
                    evidence["presentationCursor"] = serde_json::json!(presentation_cursor);
                    if let Some(mut inventory) = presentation
                        .normal_inventory_for_present(presentation_cursor, result.is_ok())
                    {
                        inventory["presentationCursor"] = serde_json::json!(presentation_cursor);
                        inventory["presentedFrameCount"] = profile["presentedFrameCount"].clone();
                        evidence["arrangedInventoryPublication"] = inventory;
                    }
                    if let Some((stamp, business)) = presentation.normal_workbench_business.as_ref()
                    {
                        if stamp.same_business_generation(presentation_cursor) {
                            if let Ok(mut publication) = serde_json::to_value(business.as_ref()) {
                                publication["authority"] =
                                    "normal-retained-workbench-projection".into();
                                publication["observed"] = true.into();
                                publication["presentationCursor"] =
                                    serde_json::json!(presentation_cursor);
                                publication["presentedFrameCount"] =
                                    profile["presentedFrameCount"].clone();
                                evidence["workbenchStatePublication"] = publication;
                            }
                        }
                    }
                    finalize_viewport_toolbar_success(
                        &mut evidence,
                        presentation
                            .normal_viewport_toolbar
                            .as_ref()
                            .map(|(stamp, _, _)| *stamp),
                        result.is_ok(),
                        presentation_cursor,
                        &profile["presentedFrameCount"],
                    );
                    profile["normalWindowEvidence"] = evidence;
                    self.normal_capture_controller.after_successful_present(
                        presentation_cursor,
                        self.size,
                        profile,
                    );
                }
            }
        }
        result
    }

    pub(in crate::ui::retained_host::host_contract) fn present_during_native_resize(
        &mut self,
        presentation: &HostWindowPresentationData,
        presentation_cursor: HostPresentationGenerationCursor,
        invalidation: HostInvalidationDiagnostics,
    ) -> HostPresenterResult<HostRefreshDiagnostics> {
        self.normal_capture_controller
            .invalidate("native-resize-present");
        let _ = presentation_cursor;
        self.submitted_text_profile = None;
        let reused_snapshot = self.native_resize_draw_list.is_some();
        if !reused_snapshot {
            self.native_resize_generation = self.native_resize_generation.saturating_add(1);
            let stream = build_chrome_command_stream_with_residency(
                presentation,
                self.native_resize_projection_size,
                None,
                true,
                |resource_key, generation| {
                    self.surface
                        .is_image_resource_resident(resource_key, generation)
                },
            );
            let mut draw_list =
                ui_surface_draw_list_from_owned_stream_with_generation_and_residency(
                    stream,
                    self.native_resize_generation,
                    |resource_key, generation| {
                        self.surface
                            .is_image_resource_resident(resource_key, generation)
                    },
                );
            draw_list.retarget_surface_size_preserving_projection(self.size);
            self.native_resize_draw_list = Some(draw_list);
            #[cfg(test)]
            {
                self.native_resize_snapshot_build_count =
                    self.native_resize_snapshot_build_count.saturating_add(1);
            }
            zircon_runtime::profile_counter!(
                "editor",
                "ui.window_resize.command_snapshot_build_count",
                1_u8
            );
        } else {
            #[cfg(test)]
            {
                self.native_resize_snapshot_reuse_count =
                    self.native_resize_snapshot_reuse_count.saturating_add(1);
            }
            zircon_runtime::profile_counter!(
                "editor",
                "ui.window_resize.command_snapshot_reuse_count",
                1_u8
            );
        }

        let mut draw_list = self
            .native_resize_draw_list
            .take()
            .expect("native resize draw list is initialized above");
        draw_list.retarget_surface_size_preserving_projection(self.size);
        draw_list.damage = None;
        let stats = self.surface.present(&draw_list);
        self.native_resize_draw_list = Some(draw_list);
        let stats = require_submitted_present(stats?)?;
        let diagnostics = self.finish_present(stats, self.size, false, None, invalidation)?;
        // The resize projection is a frozen transaction snapshot, not the ordinary retained
        // surface baseline required by a later region-only command stream.
        self.surface_cache_initialized = false;
        Ok(diagnostics)
    }

    pub(in crate::ui::retained_host::host_contract) fn present_stream(
        &mut self,
        stream: &ChromeCommandStream,
        invalidation: HostInvalidationDiagnostics,
    ) -> HostPresenterResult<HostRefreshDiagnostics> {
        self.submitted_text_profile = None;
        self.present_stream_with_damage_diagnostics(stream, stream.damage(), invalidation)
    }

    fn present_stream_with_damage_diagnostics(
        &mut self,
        stream: &ChromeCommandStream,
        diagnostic_damage: Option<&FrameRect>,
        invalidation: HostInvalidationDiagnostics,
    ) -> HostPresenterResult<HostRefreshDiagnostics> {
        let draw_list =
            ui_surface_draw_list_from_stream_with_residency(stream, |resource_key, generation| {
                self.surface
                    .is_image_resource_resident(resource_key, generation)
            });
        let region_present = diagnostic_damage.is_some() || !stream.is_full_rebuild();
        self.present_draw_list_with_damage_diagnostics(
            draw_list,
            stream.surface_size(),
            region_present,
            diagnostic_damage,
            invalidation,
        )
    }

    fn present_draw_list_with_damage_diagnostics(
        &mut self,
        draw_list: UiSurfaceDrawList,
        surface_size: (u32, u32),
        region_present: bool,
        diagnostic_damage: Option<&FrameRect>,
        invalidation: HostInvalidationDiagnostics,
    ) -> HostPresenterResult<HostRefreshDiagnostics> {
        let stats = require_submitted_present(self.surface.present_owned(draw_list)?)?;
        self.finish_present(
            stats,
            surface_size,
            region_present,
            diagnostic_damage,
            invalidation,
        )
    }

    fn finish_present(
        &mut self,
        stats: UiSurfacePresentStats,
        surface_size: (u32, u32),
        region_present: bool,
        diagnostic_damage: Option<&FrameRect>,
        invalidation: HostInvalidationDiagnostics,
    ) -> HostPresenterResult<HostRefreshDiagnostics> {
        record_present_stats(self, &stats, region_present);
        self.surface_cache_initialized = true;

        let painted_pixels = diagnostic_damage
            .map(|damage| damage_pixel_count(damage, surface_size))
            .unwrap_or_else(|| full_surface_pixels(surface_size));
        record_current_ui_perf_counter(UiPerfCounter::PaintedPixels, painted_pixels as f64);
        record_current_ui_perf_counter(
            UiPerfCounter::PresentedSurfacePixels,
            full_surface_pixels(surface_size) as f64,
        );
        if region_present {
            record_current_ui_perf_counter(UiPerfCounter::RegionPaintCount, 1.0);
        } else {
            record_current_ui_perf_counter(UiPerfCounter::FullPaintCount, 1.0);
        }
        self.diagnostics
            .record_present(painted_pixels, !region_present, region_present);
        Ok(self
            .diagnostics
            .clone()
            .with_invalidation_diagnostics(invalidation))
    }
}

fn require_submitted_present(
    stats: UiSurfacePresentStats,
) -> HostPresenterResult<UiSurfacePresentStats> {
    if stats.outcome.is_submitted() {
        Ok(stats)
    } else {
        Err(HostPresenterError::RetryableSurfacePresent)
    }
}

fn observed_stream_damage<'a>(
    damage: Option<&'a FrameRect>,
    cache_ready: bool,
    observe: bool,
) -> Option<&'a FrameRect> {
    damage.filter(|_| cache_ready && !observe)
}
#[cfg(test)]
#[path = "../../gpu/tests/present_text_observation.rs"]
mod text_observation_tests;

fn finalize_viewport_toolbar_success(
    evidence: &mut serde_json::Value,
    toolbar_cursor: Option<HostPresentationGenerationCursor>,
    successful: bool,
    cursor: HostPresentationGenerationCursor,
    presented_frame_count: &serde_json::Value,
) {
    if successful && toolbar_cursor == Some(cursor) {
        if let Some(toolbar) = evidence.get_mut("viewportToolbarPublication") {
            toolbar["presentationCursor"] = serde_json::json!(cursor);
            toolbar["presentedFrameCount"] = presented_frame_count.clone();
        }
    } else if let Some(evidence) = evidence.as_object_mut() {
        evidence.remove("viewportToolbarPublication");
    }
}
#[cfg(test)]
mod viewport_toolbar_success_tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn toolbar_success_requires_each_of_the_seven_actual_cursor_fields() {
        let toolbar_cursor = Some(HostPresentationGenerationCursor::default());
        let mut exact = json!({"viewportToolbarPublication":{"observed":true}});
        finalize_viewport_toolbar_success(
            &mut exact,
            toolbar_cursor,
            true,
            Default::default(),
            &json!(7),
        );
        assert_eq!(
            exact["viewportToolbarPublication"]["presentedFrameCount"],
            7
        );
        assert_eq!(
            exact["viewportToolbarPublication"]["presentationCursor"],
            json!(HostPresentationGenerationCursor::default())
        );
        for cursor in [
            HostPresentationGenerationCursor::new_with_geometry(1, 0, 0, 0, 0, 0, 0),
            HostPresentationGenerationCursor::new_with_geometry(0, 1, 0, 0, 0, 0, 0),
            HostPresentationGenerationCursor::new_with_geometry(0, 0, 1, 0, 0, 0, 0),
            HostPresentationGenerationCursor::new_with_geometry(0, 0, 0, 1, 0, 0, 0),
            HostPresentationGenerationCursor::new_with_geometry(0, 0, 0, 0, 1, 0, 0),
            HostPresentationGenerationCursor::new_with_geometry(0, 0, 0, 0, 0, 1, 0),
            HostPresentationGenerationCursor::new_with_geometry(0, 0, 0, 0, 0, 0, 1),
        ] {
            let mut stale = json!({"viewportToolbarPublication":{"observed":true}});
            finalize_viewport_toolbar_success(&mut stale, toolbar_cursor, true, cursor, &json!(8));
            assert!(stale.get("viewportToolbarPublication").is_none());
        }
    }
    #[test]
    fn failed_present_and_missing_prepared_metadata_cannot_publish_toolbar_evidence() {
        for (successful, toolbar_cursor) in [
            (false, Some(HostPresentationGenerationCursor::default())),
            (true, None),
        ] {
            let mut evidence = json!({"viewportToolbarPublication":{"observed":true}});
            finalize_viewport_toolbar_success(
                &mut evidence,
                toolbar_cursor,
                successful,
                Default::default(),
                &json!(8),
            );
            assert!(evidence.get("viewportToolbarPublication").is_none());
        }
    }
}
