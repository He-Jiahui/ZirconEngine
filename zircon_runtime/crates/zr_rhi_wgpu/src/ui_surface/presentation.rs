use crate::GpuPassTimer;
use zr_rhi::{RenderDevice, RhiError, UiSurfaceDrawList, UiSurfacePresentOutcome, UiSurfaceRect};

use super::batching::BatchDrawPlan;
use super::geometry::damage_with_analytic_coverage;
use super::render_pass::{record_draw_plan_to_view, TargetLoad};
use super::text::WgpuUiTextPrepareStats;
use super::{
    WgpuUiSurfacePresentation, WgpuUiSurfaceRenderStats, WgpuUiSurfaceRenderer,
    UI_GPU_TIMER_PASS_NAME,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RetainedCacheCommit {
    OrdinaryBaseline,
    ResizeProjection(Option<u64>),
}

impl WgpuUiSurfaceRenderer {
    pub(super) fn present(
        &mut self,
        draw_list: &UiSurfaceDrawList,
    ) -> Result<WgpuUiSurfacePresentation, RhiError> {
        self.poll_local_completion_timeline()?;
        self.resize_if_needed(draw_list.surface_size)?;
        let Some(surface_texture) = self.acquire_surface_texture()? else {
            return Ok(retryable_surface_presentation(draw_list.surface_size));
        };
        self.present_index = self.present_index.saturating_add(1);
        let retained_cache_size = if draw_list.is_target_only_resize() {
            draw_list.projection_size()
        } else {
            draw_list.surface_size
        };
        if let Some(retained_cache) = &mut self.retained_cache {
            if !retained_cache.matches(self.config.format, retained_cache_size) {
                retained_cache.resize(&self.device, self.config.format, retained_cache_size);
            }
        }
        let cache_ready = self.retained_cache.as_ref().is_some_and(|retained_cache| {
            retained_cache.matches(self.config.format, retained_cache_size)
                && if draw_list.is_target_only_resize() {
                    retained_cache.is_projection_ready(draw_list.generation())
                } else {
                    retained_cache.ordinary_baseline_ready()
                }
        });
        let mode = surface_render_mode(draw_list, cache_ready);
        let damage = render_damage(draw_list, mode);
        let resolved_draw_plan = self
            .compiled_batch_plan
            .resolve(draw_list, mode == SurfaceRenderMode::FullRedraw);
        let mut draw_list_stats = resolved_draw_plan
            .draw_list_stats
            .unwrap_or_else(|| draw_list.stats());
        draw_list_stats.surface_size = draw_list.surface_size;
        let draw_plan = resolved_draw_plan.plan;
        let (mut image_resource_stats, text_stats) =
            if mode == SurfaceRenderMode::RetainedProjectionCopy {
                (
                    self.image_cache
                        .residency_stats(&self.shared_image_registry),
                    WgpuUiTextPrepareStats::default(),
                )
            } else {
                (
                    self.image_cache.prepare(
                        &self.device,
                        &self.queue,
                        &self.image_bind_group_layout,
                        &self.image_sampler,
                        self.present_index,
                        draw_list,
                        &draw_plan.image_upload_sources,
                        self.external_images.as_deref(),
                        self.shared_image_registry.as_ref(),
                        &mut self.pending_image_resources,
                    ),
                    self.text.prepare(
                        &self.device,
                        &self.queue,
                        draw_list.projection_size(),
                        draw_list,
                        &draw_plan.ops,
                    ),
                )
            };
        let render_stats =
            self.render_draw_list_to_surface(draw_list, &draw_plan, mode, damage, surface_texture)?;
        let residency = self
            .image_cache
            .residency_stats(&self.shared_image_registry);
        image_resource_stats.cache_resident_bytes = residency.cache_resident_bytes;
        image_resource_stats.cpu_resident_bytes = residency.cpu_resident_bytes;
        image_resource_stats.shared_resident_bytes = residency.shared_resident_bytes;
        image_resource_stats.device_allocation_count = residency.device_allocation_count;
        image_resource_stats.device_allocation_bytes = residency.device_allocation_bytes;
        image_resource_stats.registry_evicted_pinned_bytes =
            residency.registry_evicted_pinned_bytes;
        image_resource_stats.surface_pin_count = residency.surface_pin_count;
        image_resource_stats.in_flight_present_pin_count = residency.in_flight_present_pin_count;
        image_resource_stats.eviction_completion_count = residency.eviction_completion_count;
        if mode != SurfaceRenderMode::RetainedProjectionCopy {
            if let Some(provider) = self.external_images.as_deref() {
                for source_index in self.image_cache.resolved_external_source_indices() {
                    let Some(source) = draw_plan.image_upload_sources.get(*source_index) else {
                        continue;
                    };
                    // Submission has accepted the product, so future renderer frames may skip
                    // only this viewport's CPU capture fallback.
                    provider.confirm_resident(&source.resource_key, source.resource_generation);
                }
            }
        }
        let mut batch_stats = draw_plan.stats;
        batch_stats.batch_plan_build_count = resolved_draw_plan.batch_plan_build_count;
        batch_stats.batch_plan_cache_hit_count = resolved_draw_plan.batch_plan_cache_hit_count;
        if resolved_draw_plan.batch_plan_build_count == 0 {
            batch_stats.overlap_candidate_count = 0;
        }
        batch_stats.vertex_buffer_create_count =
            render_stats.draw_buffers.vertex_buffer_create_count;
        batch_stats.vertex_upload_bytes = render_stats.draw_buffers.vertex_upload_bytes;
        batch_stats.retained_cache_copy_bytes = render_stats.retained_cache_copy_bytes;
        Ok(WgpuUiSurfacePresentation {
            outcome: render_stats.outcome,
            submission: render_stats.submission,
            draw_list_stats,
            batch_stats,
            text_stats,
            image_resource_stats: Some(image_resource_stats),
            recorded_stats: Some(render_stats.recorded),
            gpu_timestamp_supported: render_stats.gpu_timestamp_supported,
            gpu_time_us: render_stats.gpu_time_us,
            gpu_profile_latency_frames: render_stats.gpu_profile_latency_frames,
        })
    }

    fn resize_if_needed(&mut self, size: (u32, u32)) -> Result<(), RhiError> {
        if size != (self.config.width, self.config.height) {
            self.resize(size)?;
        }
        Ok(())
    }

    fn poll_local_completion_timeline(&mut self) -> Result<(), RhiError> {
        if !self.completion_owner.is_local() {
            return Ok(());
        }
        self.render_device.poll_submissions()?;
        if let Some(readback_queue) = self.gpu_readback_queue.as_mut() {
            let _ = readback_queue.collect_completed_after_device_poll();
        }
        Ok(())
    }

    fn acquire_surface_texture(&mut self) -> Result<Option<wgpu::SurfaceTexture>, RhiError> {
        let acquisition = self.surface.get_current_texture();
        if retryable_surface_outcome(&acquisition).is_some() {
            if matches!(
                &acquisition,
                wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost
            ) {
                self.surface.configure(&self.device, &self.config);
            }
            return Ok(None);
        }
        match acquisition {
            wgpu::CurrentSurfaceTexture::Success(surface_texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => Ok(Some(surface_texture)),
            wgpu::CurrentSurfaceTexture::Validation => Err(RhiError::SurfaceUnavailable(
                "surface validation error".to_string(),
            )),
            wgpu::CurrentSurfaceTexture::Outdated
            | wgpu::CurrentSurfaceTexture::Lost
            | wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded => {
                unreachable!("retryable surface acquisitions return above")
            }
        }
    }

    fn render_draw_list_to_surface(
        &mut self,
        draw_list: &UiSurfaceDrawList,
        draw_plan: &BatchDrawPlan,
        mode: SurfaceRenderMode,
        damage: Option<UiSurfaceRect>,
        surface_texture: wgpu::SurfaceTexture,
    ) -> Result<WgpuUiSurfaceRenderStats, RhiError> {
        let completed_timing = if self.gpu_readback_queue.is_some() {
            self.gpu_pass_timer
                .as_mut()
                .and_then(GpuPassTimer::try_collect)
        } else {
            None
        };
        let gpu_time_us = completed_timing.as_ref().and_then(|result| {
            result
                .pass_timings
                .iter()
                .find(|timing| timing.pass_name == UI_GPU_TIMER_PASS_NAME)
                .map(|timing| timing.gpu_time_us)
        });
        let gpu_profile_latency_frames = completed_timing.map_or(0, |result| {
            self.present_index
                .saturating_sub(result.frame_generation)
                .min(u64::from(u32::MAX)) as u32
        });
        let mut render_stats = WgpuUiSurfaceRenderStats {
            gpu_timestamp_supported: self.gpu_pass_timer.is_some()
                && self.gpu_readback_queue.is_some(),
            gpu_time_us,
            gpu_profile_latency_frames,
            ..WgpuUiSurfaceRenderStats::default()
        };
        let target_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let resolved_buffers = (mode != SurfaceRenderMode::RetainedProjectionCopy).then(|| {
            self.compiled_draw_buffers
                .resolve(&self.device, &self.queue, draw_list, draw_plan)
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("zircon-ui-surface-encoder"),
            });
        encoder.push_debug_group("zircon::UI");
        let readback_ready = self.gpu_pass_timer.is_some()
            && self
                .gpu_readback_queue
                .as_mut()
                .is_some_and(|readback_queue| {
                    readback_queue.prepare_frame(self.present_index).is_ok()
                });
        let timestamp_scope = if readback_ready {
            self.gpu_pass_timer.as_mut().and_then(|timer| {
                timer.begin_frame(self.present_index);
                timer.begin_pass(&mut encoder, UI_GPU_TIMER_PASS_NAME)
            })
        } else {
            None
        };
        let mut retained_cache_commit = None;

        match mode {
            SurfaceRenderMode::FullRedraw => {
                let buffers = &resolved_buffers
                    .as_ref()
                    .expect("full redraw resolves draw buffers")
                    .buffers;
                if let Some(retained_cache) = &mut self.retained_cache {
                    render_stats.add_recorded(record_draw_plan_to_view(
                        &mut encoder,
                        retained_cache.view(),
                        TargetLoad::ClearBlack,
                        retained_cache.size(),
                        draw_list.projection_size(),
                        damage,
                        draw_plan,
                        buffers,
                        &mut self.damage_draw_op_candidates,
                        &self.damage_clear_pipeline,
                        &self.solid_pipeline,
                        &self.solid_instance_pipeline,
                        &self.image_pipeline,
                        &self.image_cache,
                        &mut self.text,
                    ));
                    render_stats.retained_cache_copy_bytes = render_stats
                        .retained_cache_copy_bytes
                        .saturating_add(retained_cache.record_copy_to_surface(
                            &mut encoder,
                            &surface_texture.texture,
                            &target_view,
                            draw_list.surface_size,
                        ));
                    if retained_cache.copy_requires_target_clear(draw_list.surface_size) {
                        render_stats.recorded.render_pass_count =
                            render_stats.recorded.render_pass_count.saturating_add(1);
                    }
                    retained_cache_commit = Some(if draw_list.is_target_only_resize() {
                        RetainedCacheCommit::ResizeProjection(draw_list.generation())
                    } else {
                        RetainedCacheCommit::OrdinaryBaseline
                    });
                } else {
                    render_stats.add_recorded(record_draw_plan_to_view(
                        &mut encoder,
                        &target_view,
                        TargetLoad::ClearBlack,
                        draw_list.surface_size,
                        draw_list.projection_size(),
                        damage,
                        draw_plan,
                        buffers,
                        &mut self.damage_draw_op_candidates,
                        &self.damage_clear_pipeline,
                        &self.solid_pipeline,
                        &self.solid_instance_pipeline,
                        &self.image_pipeline,
                        &self.image_cache,
                        &mut self.text,
                    ));
                }
            }
            SurfaceRenderMode::DamagePatch => {
                let retained_cache = self.retained_cache.as_mut().ok_or_else(|| {
                    RhiError::SurfaceUnavailable(
                        "damage patch requested without a retained surface cache".to_string(),
                    )
                })?;
                let buffers = &resolved_buffers
                    .as_ref()
                    .expect("damage patch resolves draw buffers")
                    .buffers;
                render_stats.add_recorded(record_draw_plan_to_view(
                    &mut encoder,
                    retained_cache.view(),
                    TargetLoad::Load,
                    draw_list.surface_size,
                    draw_list.projection_size(),
                    damage,
                    draw_plan,
                    buffers,
                    &mut self.damage_draw_op_candidates,
                    &self.damage_clear_pipeline,
                    &self.solid_pipeline,
                    &self.solid_instance_pipeline,
                    &self.image_pipeline,
                    &self.image_cache,
                    &mut self.text,
                ));
                render_stats.retained_cache_copy_bytes = render_stats
                    .retained_cache_copy_bytes
                    .saturating_add(retained_cache.record_copy_to_surface(
                        &mut encoder,
                        &surface_texture.texture,
                        &target_view,
                        draw_list.surface_size,
                    ));
                retained_cache_commit = Some(RetainedCacheCommit::OrdinaryBaseline);
            }
            SurfaceRenderMode::RetainedProjectionCopy => {
                let retained_cache = self.retained_cache.as_ref().ok_or_else(|| {
                    RhiError::SurfaceUnavailable(
                        "retained projection copy requested without a retained surface cache"
                            .to_string(),
                    )
                })?;
                if retained_cache.copy_requires_target_clear(draw_list.surface_size) {
                    render_stats.recorded.render_pass_count =
                        render_stats.recorded.render_pass_count.saturating_add(1);
                }
                render_stats.retained_cache_copy_bytes = retained_cache.record_copy_to_surface(
                    &mut encoder,
                    &surface_texture.texture,
                    &target_view,
                    draw_list.surface_size,
                );
            }
        }

        if let (Some(timer), Some(scope)) = (&self.gpu_pass_timer, timestamp_scope) {
            timer.end_pass(&mut encoder, scope);
        }
        if readback_ready {
            if let (Some(timer), Some(readback_queue)) =
                (&mut self.gpu_pass_timer, &mut self.gpu_readback_queue)
            {
                let _ = timer.resolve_and_request(&mut encoder, readback_queue);
                if let Err(error) = readback_queue.encode_copies(&mut encoder, self.present_index) {
                    readback_queue.abort_frame(self.present_index);
                    return Err(RhiError::SurfaceUnavailable(error.to_string()));
                }
            }
        }
        encoder.pop_debug_group();
        let image_allocation_pins = (mode != SurfaceRenderMode::RetainedProjectionCopy)
            .then(|| self.image_cache.pin_prepared_allocations_for_submission())
            .flatten();
        let submission =
            Some(self.submit_present_command_buffer(encoder.finish(), image_allocation_pins)?);
        if let (Some(retained_cache), Some(commit)) =
            (&mut self.retained_cache, retained_cache_commit)
        {
            match commit {
                RetainedCacheCommit::OrdinaryBaseline => {
                    retained_cache.mark_ordinary_baseline_ready();
                }
                RetainedCacheCommit::ResizeProjection(generation) => {
                    retained_cache.mark_projection_ready(generation);
                }
            }
        }
        if readback_ready {
            if let Some(readback_queue) = &mut self.gpu_readback_queue {
                if readback_queue.begin_map(self.present_index).is_err() {
                    readback_queue.abort_frame(self.present_index);
                }
            }
        }
        surface_texture.present();
        Ok(WgpuUiSurfaceRenderStats {
            submission,
            draw_buffers: resolved_buffers
                .map(|resolved| resolved.stats)
                .unwrap_or_default(),
            ..render_stats
        })
    }
}

pub(super) fn retryable_surface_presentation(
    surface_size: (u32, u32),
) -> WgpuUiSurfacePresentation {
    let mut draw_list_stats = zr_rhi::UiSurfacePresentStats::default();
    draw_list_stats.outcome = UiSurfacePresentOutcome::RetryableNoSubmit;
    draw_list_stats.surface_size = surface_size;

    WgpuUiSurfacePresentation {
        outcome: UiSurfacePresentOutcome::RetryableNoSubmit,
        submission: None,
        draw_list_stats,
        batch_stats: Default::default(),
        text_stats: Default::default(),
        image_resource_stats: None,
        recorded_stats: None,
        gpu_timestamp_supported: false,
        gpu_time_us: None,
        gpu_profile_latency_frames: 0,
    }
}

pub(super) fn retryable_surface_outcome(
    acquisition: &wgpu::CurrentSurfaceTexture,
) -> Option<UiSurfacePresentOutcome> {
    matches!(
        acquisition,
        wgpu::CurrentSurfaceTexture::Outdated
            | wgpu::CurrentSurfaceTexture::Lost
            | wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
    )
    .then_some(UiSurfacePresentOutcome::RetryableNoSubmit)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SurfaceRenderMode {
    FullRedraw,
    DamagePatch,
    RetainedProjectionCopy,
}

pub(super) fn surface_render_mode(
    draw_list: &UiSurfaceDrawList,
    cache_ready: bool,
) -> SurfaceRenderMode {
    if draw_list.is_target_only_resize() && cache_ready {
        SurfaceRenderMode::RetainedProjectionCopy
    } else if draw_list.damage.is_some() && cache_ready {
        SurfaceRenderMode::DamagePatch
    } else {
        SurfaceRenderMode::FullRedraw
    }
}

pub(super) fn render_damage(
    draw_list: &UiSurfaceDrawList,
    mode: SurfaceRenderMode,
) -> Option<UiSurfaceRect> {
    (mode == SurfaceRenderMode::DamagePatch)
        .then(|| damage_with_analytic_coverage(draw_list.damage, draw_list.projection_size()))
        .flatten()
}
