use std::{sync::Arc, time::Instant};

use wgpu::util::DeviceExt;
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch, WgpuTextureUploadBatch};

use crate::core::framework::render::RenderReflectionProbeWorkloadReport;
use crate::core::resource::ResourceId;
#[cfg(test)]
use crate::graphics::backend::{
    read_buffer_bytes, read_texture_rgba16float_region, BufferByteReadback,
    Rgba16FloatTextureRegionReadback,
};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::types::ViewportRenderFrame;

use super::capacity::ReflectionProbeResourceCapacity;
pub(super) use super::capacity::{
    MAX_REFLECTION_PROBES, REFLECTION_PROBE_FACE_COUNT, REFLECTION_PROBE_FACE_SIZE,
    REFLECTION_PROBE_MIP_COUNT,
};
pub(in crate::graphics::scene::scene_renderer) use super::capacity::{
    PLANAR_REFLECTION_MIP_COUNT, PLANAR_REFLECTION_TEXTURE_SIZE,
};
use super::gpu_layout::{
    GpuPlanarReflection, GpuReflectionProbe, GpuReflectionProbeHeader, ReflectionProbeGpuBindings,
};
pub(in crate::graphics::scene::scene_renderer) use super::report::ReflectionProbeUploadReport;
use super::report::{record_probe_asset_rejection, PendingReflectionProbeUpload};
use super::selection::{
    probe_distance_to_influence, reflection_probe_candidate_order,
    selected_planar_reflection_params, ReflectionProbeCandidate,
};
use super::slot_allocator::{ProbeCubemapSlotAllocator, ProbeCubemapSlotReservation};
use super::upload::{
    append_probe_pmrem_texture_uploads, validate_probe_pmrem_texture, ReflectionProbeAssetError,
};

pub(in crate::graphics::scene::scene_renderer) struct SceneReflectionProbeResources {
    probe_buffer: Arc<wgpu::Buffer>,
    header_buffer: Arc<wgpu::Buffer>,
    capture_disabled_header_buffer: Arc<wgpu::Buffer>,
    cubemap_array: wgpu::Texture,
    cubemap_array_view: Arc<wgpu::TextureView>,
    planar_texture: Arc<wgpu::Texture>,
    planar_texture_view: Arc<wgpu::TextureView>,
    planar_params_buffer: Arc<wgpu::Buffer>,
    capture_disabled_planar_params_buffer: Arc<wgpu::Buffer>,
    slots: ProbeCubemapSlotAllocator,
    next_prepare_epoch: u64,
    pending_uploads: Vec<PendingReflectionProbeUpload>,
    last_report: ReflectionProbeUploadReport,
    environment_only_placeholder: bool,
    environment_only_provider_upgrade: bool,
    #[cfg(test)]
    probe_capacity: usize,
    #[cfg(test)]
    candidate_registry_resolution_count: usize,
}

impl SceneReflectionProbeResources {
    pub(in crate::graphics::scene::scene_renderer) fn new(device: &wgpu::Device) -> Self {
        Self::new_with_capacity(device, ReflectionProbeResourceCapacity::FULL, false)
    }

    /// Defers the large local-provider textures until an environment preview actually requests one.
    pub(in crate::graphics::scene::scene_renderer) fn new_environment_only_preview(
        device: &wgpu::Device,
    ) -> Self {
        Self::new_with_capacity(
            device,
            ReflectionProbeResourceCapacity::ENVIRONMENT_PREVIEW_PLACEHOLDER,
            true,
        )
    }

    /// The environment-only shader omits the local-provider ABI. Once the
    /// placeholder has expanded, subsequent Base variants must use the generic
    /// environment implementation that samples those providers.
    pub(in crate::graphics::scene::scene_renderer) const fn requires_generic_environment_pbr(
        &self,
    ) -> bool {
        self.environment_only_provider_upgrade
    }

    /// Ensures a capture target never aliases the tiny environment-only placeholder array.
    ///
    /// Capture publication copies the fixed-size PMREM chain into the local-provider array. The
    /// placeholder is deliberately too small for that copy, so expand it before reserving a slot.
    /// The placeholder is only used before any local-provider upload is accepted. Upgrading it
    /// here therefore cannot discard an in-flight upload and makes the copy destination valid for
    /// the fixed PMREM chain before the reservation is created.
    pub(in crate::graphics::scene::scene_renderer) fn ensure_environment_capture_provider(
        &mut self,
        device: &wgpu::Device,
    ) {
        if !self.environment_only_placeholder {
            return;
        }
        debug_assert!(
            self.pending_uploads.is_empty(),
            "environment-only placeholder cannot own a local-provider upload"
        );
        self.upgrade_environment_only_provider(device);
    }

    fn new_with_capacity(
        device: &wgpu::Device,
        capacity: ReflectionProbeResourceCapacity,
        environment_only_placeholder: bool,
    ) -> Self {
        let probe_buffer_usage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        #[cfg(test)]
        let probe_buffer_usage = probe_buffer_usage | wgpu::BufferUsages::COPY_SRC;
        let probe_buffer = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-reflection-probe-storage"),
            size: (std::mem::size_of::<GpuReflectionProbe>() * capacity.probe_count) as u64,
            usage: probe_buffer_usage,
            mapped_at_creation: false,
        }));
        let header_buffer_usage = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
        #[cfg(test)]
        let header_buffer_usage = header_buffer_usage | wgpu::BufferUsages::COPY_SRC;
        let header_buffer = Arc::new(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("zircon-reflection-probe-header"),
                contents: bytemuck::bytes_of(&GpuReflectionProbeHeader::default()),
                usage: header_buffer_usage,
            },
        ));
        let capture_disabled_header_buffer = Arc::new(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("zircon-environment-capture-disabled-reflection-probe-header"),
                contents: bytemuck::bytes_of(&GpuReflectionProbeHeader::default()),
                usage: wgpu::BufferUsages::UNIFORM,
            },
        ));
        let cubemap_array_usage =
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
        #[cfg(test)]
        let cubemap_array_usage = cubemap_array_usage | wgpu::TextureUsages::COPY_SRC;
        let cubemap_array = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-reflection-probe-pmrem-cube-array"),
            size: wgpu::Extent3d {
                width: capacity.cubemap_face_size,
                height: capacity.cubemap_face_size,
                depth_or_array_layers: capacity.cubemap_slot_count as u32
                    * REFLECTION_PROBE_FACE_COUNT,
            },
            mip_level_count: capacity.cubemap_mip_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: cubemap_array_usage,
            view_formats: &[],
        });
        let cubemap_array_view =
            Arc::new(cubemap_array.create_view(&wgpu::TextureViewDescriptor {
                label: Some("zircon-reflection-probe-pmrem-cube-array-view"),
                format: Some(wgpu::TextureFormat::Rgba16Float),
                dimension: Some(wgpu::TextureViewDimension::CubeArray),
                usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: Some(capacity.cubemap_mip_count),
                base_array_layer: 0,
                array_layer_count: Some(
                    capacity.cubemap_slot_count as u32 * REFLECTION_PROBE_FACE_COUNT,
                ),
            }));
        let planar_texture = Arc::new(device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-planar-reflection-mip-chain"),
            size: wgpu::Extent3d {
                width: capacity.planar_texture_size,
                height: capacity.planar_texture_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: capacity.planar_mip_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        }));
        let planar_texture_view =
            Arc::new(planar_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("zircon-planar-reflection-mip-chain-view"),
                format: Some(wgpu::TextureFormat::Rgba16Float),
                dimension: Some(wgpu::TextureViewDimension::D2),
                usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: Some(capacity.planar_mip_count),
                base_array_layer: 0,
                array_layer_count: Some(1),
            }));
        let planar_params_buffer = Arc::new(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("zircon-planar-reflection-params"),
                contents: bytemuck::bytes_of(&GpuPlanarReflection::default()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        ));
        let capture_disabled_planar_params_buffer = Arc::new(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("zircon-environment-capture-disabled-planar-reflection-params"),
                contents: bytemuck::bytes_of(&GpuPlanarReflection::default()),
                usage: wgpu::BufferUsages::UNIFORM,
            },
        ));
        Self {
            probe_buffer,
            header_buffer,
            capture_disabled_header_buffer,
            cubemap_array,
            cubemap_array_view,
            planar_texture,
            planar_texture_view,
            planar_params_buffer,
            capture_disabled_planar_params_buffer,
            slots: ProbeCubemapSlotAllocator::new(capacity.probe_count),
            next_prepare_epoch: 0,
            pending_uploads: Vec::new(),
            last_report: ReflectionProbeUploadReport::default(),
            environment_only_placeholder,
            environment_only_provider_upgrade: false,
            #[cfg(test)]
            probe_capacity: capacity.probe_count,
            #[cfg(test)]
            candidate_registry_resolution_count: 0,
        }
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer) const fn is_environment_only_placeholder_for_tests(
        &self,
    ) -> bool {
        self.environment_only_placeholder
    }

    pub(in crate::graphics::scene::scene_renderer) fn bindings(
        &self,
    ) -> ReflectionProbeGpuBindings {
        ReflectionProbeGpuBindings::new(
            Arc::clone(&self.probe_buffer),
            Arc::clone(&self.header_buffer),
            Arc::clone(&self.cubemap_array_view),
            Arc::clone(&self.planar_params_buffer),
            Arc::clone(&self.planar_texture_view),
        )
    }

    /// Retains the generic forward ABI while preventing recursive reflection capture feedback.
    ///
    /// The probe and planar textures remain bound because the layout requires them, but the
    /// immutable zeroed metadata makes both providers unreachable in the shader. The global
    /// environment stays in scene group zero, so capture still records the authored sky.
    pub(in crate::graphics::scene::scene_renderer) fn environment_capture_bindings(
        &self,
    ) -> ReflectionProbeGpuBindings {
        ReflectionProbeGpuBindings::new(
            Arc::clone(&self.probe_buffer),
            Arc::clone(&self.capture_disabled_header_buffer),
            Arc::clone(&self.cubemap_array_view),
            Arc::clone(&self.capture_disabled_planar_params_buffer),
            Arc::clone(&self.planar_texture_view),
        )
    }

    pub(in crate::graphics::scene::scene_renderer) fn planar_texture(&self) -> Arc<wgpu::Texture> {
        Arc::clone(&self.planar_texture)
    }

    pub(in crate::graphics::scene::scene_renderer) fn reserve_environment_capture_target(
        &mut self,
        cubemap: ResourceId,
        revision: u64,
    ) -> Option<ProbeCubemapSlotReservation> {
        let prepare_epoch = self.begin_prepare_epoch();
        self.slots
            .reserve_for_capture(cubemap, revision, prepare_epoch)
    }

    pub(in crate::graphics::scene::scene_renderer) fn copy_environment_capture_probe(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::Texture,
        slot: u32,
    ) {
        let destination_layer = slot.saturating_mul(REFLECTION_PROBE_FACE_COUNT);
        for mip_level in 0..REFLECTION_PROBE_MIP_COUNT {
            let extent = (REFLECTION_PROBE_FACE_SIZE >> mip_level).max(1);
            encoder.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: source,
                    mip_level,
                    origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyTextureInfo {
                    texture: &self.cubemap_array,
                    mip_level,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: destination_layer,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: extent,
                    height: extent,
                    depth_or_array_layers: REFLECTION_PROBE_FACE_COUNT,
                },
            );
        }
    }

    pub(in crate::graphics::scene::scene_renderer) fn commit_environment_capture_target(
        &mut self,
        reservation: ProbeCubemapSlotReservation,
    ) {
        self.slots.commit(
            reservation.cubemap(),
            reservation.revision(),
            reservation.slot(),
            reservation.prepare_epoch(),
        );
    }

    pub(in crate::graphics::scene::scene_renderer) fn cancel_environment_capture_target(
        &mut self,
        reservation: ProbeCubemapSlotReservation,
    ) {
        self.slots.cancel(reservation);
    }

    pub(in crate::graphics::scene::scene_renderer) fn last_workload_report(
        &self,
    ) -> RenderReflectionProbeWorkloadReport {
        RenderReflectionProbeWorkloadReport {
            extracted_probe_count: self.last_report.extracted_probe_count,
            camera_layer_candidate_count: self.last_report.camera_layer_candidate_count,
            attempted_candidate_count: self.last_report.attempted_candidate_count,
            active_probe_count: self.last_report.active_probe_count,
            capacity_dropped_candidate_count: self.last_report.capacity_dropped_candidate_count,
            scheduled_cubemap_upload_count: self.last_report.scheduled_cubemap_upload_count,
            scheduled_cubemap_upload_bytes: self.last_report.scheduled_cubemap_upload_bytes,
            scheduled_texture_write_count: self.last_report.scheduled_texture_write_count,
            asset_load_call_count: self.last_report.asset_load_call_count,
            asset_load_cpu_time_us: self.last_report.asset_load_cpu_time_us,
            rejected_cubemap_count: self.last_report.rejected_cubemap_count,
            full_resolution_fragment_probe_visit_upper_bound: 0,
        }
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer) fn last_report_diagnostics(
        &self,
    ) -> (usize, usize, usize, usize, Option<String>) {
        (
            self.last_report.extracted_probe_count,
            self.last_report.active_probe_count,
            self.last_report.scheduled_cubemap_upload_count,
            self.last_report.rejected_cubemap_count,
            self.last_report
                .first_rejection
                .map(|rejection| format!("{rejection:?}")),
        )
    }

    #[cfg(test)]
    pub(super) const fn candidate_registry_resolution_count_for_tests(&self) -> usize {
        self.candidate_registry_resolution_count
    }

    #[cfg(test)]
    pub(super) fn gpu_planar_params_for_tests(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<GpuPlanarReflection, crate::graphics::types::GraphicsError> {
        let bytes = read_buffer_bytes(
            device,
            queue,
            &self.planar_params_buffer,
            BufferByteReadback {
                source_offset: 0,
                byte_len: std::mem::size_of::<GpuPlanarReflection>() as u64,
                label: "zircon-planar-reflection-params-readback",
            },
        )?;
        Ok(bytemuck::pod_read_unaligned::<GpuPlanarReflection>(&bytes))
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer) fn gpu_upload_diagnostics(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(u32, [[f32; 4]; 2], [[u16; 4]; 2]), crate::graphics::types::GraphicsError> {
        let diagnostic_probe_count = self.probe_capacity.min(2);
        let header_bytes = read_buffer_bytes(
            device,
            queue,
            &self.header_buffer,
            BufferByteReadback {
                source_offset: 0,
                byte_len: std::mem::size_of::<GpuReflectionProbeHeader>() as u64,
                label: "zircon-reflection-probe-header-readback",
            },
        )?;
        let probe_bytes = read_buffer_bytes(
            device,
            queue,
            &self.probe_buffer,
            BufferByteReadback {
                source_offset: 0,
                byte_len: (std::mem::size_of::<GpuReflectionProbe>() * diagnostic_probe_count)
                    as u64,
                label: "zircon-reflection-probe-storage-readback",
            },
        )?;
        let header = bytemuck::pod_read_unaligned::<GpuReflectionProbeHeader>(&header_bytes);
        let mut probes = [GpuReflectionProbe::default(); 2];
        for (index, probe) in probes.iter_mut().enumerate().take(diagnostic_probe_count) {
            let byte_offset = index * std::mem::size_of::<GpuReflectionProbe>();
            *probe = bytemuck::pod_read_unaligned::<GpuReflectionProbe>(
                &probe_bytes[byte_offset..byte_offset + std::mem::size_of::<GpuReflectionProbe>()],
            );
        }
        let mut first_texels = [[0_u16; 4]; 2];
        for (slot, texel) in first_texels
            .iter_mut()
            .enumerate()
            .take(diagnostic_probe_count)
        {
            let bytes = read_texture_rgba16float_region(
                device,
                queue,
                &self.cubemap_array,
                Rgba16FloatTextureRegionReadback {
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: slot as u32 * REFLECTION_PROBE_FACE_COUNT,
                    },
                    size: wgpu::Extent3d {
                        width: 1,
                        height: 1,
                        depth_or_array_layers: 1,
                    },
                    label: "zircon-reflection-probe-texel-readback",
                },
            )?;
            for channel in 0..4 {
                texel[channel] = u16::from_le_bytes([bytes[channel * 2], bytes[channel * 2 + 1]]);
            }
        }
        Ok((
            header.probe_count,
            [probes[0].position_blend, probes[1].position_blend],
            first_texels,
        ))
    }

    pub(in crate::graphics::scene::scene_renderer) fn prepare(
        &mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
        enabled: bool,
        frame_buffer_uploads: &mut WgpuBufferUploadBatch,
        frame_texture_uploads: &mut WgpuTextureUploadBatch,
    ) -> ReflectionProbeUploadReport {
        self.discard_pending_uploads();
        let mut report = ReflectionProbeUploadReport {
            extracted_probe_count: frame.environment().probes.len(),
            ..ReflectionProbeUploadReport::default()
        };
        let planar_params = selected_planar_reflection_params(frame);
        if self.environment_only_placeholder && planar_params.is_some() {
            self.upgrade_environment_only_provider(device);
        }
        if !enabled {
            self.append_buffer_uploads(
                frame_buffer_uploads,
                planar_params,
                &[],
                frame
                    .extract
                    .view
                    .selected_camera_layers()
                    .to_scene_schema_v1_mask_lossy(),
            );
            self.last_report = report;
            return report;
        }

        let camera_position = frame.effective_camera().transform.translation;
        let camera_layers = frame.extract.view.selected_camera_layers();
        let mut candidates = frame
            .environment()
            .probes
            .iter()
            .enumerate()
            .filter_map(|(extraction_order, probe)| {
                if !probe.layer_mask().intersects(camera_layers) {
                    return None;
                }
                report.camera_layer_candidate_count += 1;
                let cubemap = probe.baked_cubemap()?;
                (probe.intensity() > 0.0).then(|| ReflectionProbeCandidate {
                    probe,
                    cubemap,
                    revision: None,
                    distance: probe_distance_to_influence(probe, camera_position),
                    extraction_order,
                })
            })
            .collect::<Vec<_>>();
        let eligible_candidate_count = candidates.len();
        let mut overflow_candidates = if candidates.len() > MAX_REFLECTION_PROBES {
            candidates
                .select_nth_unstable_by(MAX_REFLECTION_PROBES, reflection_probe_candidate_order);
            candidates.split_off(MAX_REFLECTION_PROBES)
        } else {
            Vec::new()
        };
        candidates.sort_by(reflection_probe_candidate_order);
        let asset_manager = match streamer.asset_manager() {
            Ok(asset_manager) => asset_manager,
            Err(_) => {
                self.append_buffer_uploads(
                    frame_buffer_uploads,
                    planar_params,
                    &[],
                    camera_layers.to_scene_schema_v1_mask_lossy(),
                );
                self.last_report = report;
                return report;
            }
        };
        let resource_manager = asset_manager.resource_manager();
        let prepare_epoch = self.begin_prepare_epoch();
        #[cfg(test)]
        let mut candidate_registry_resolution_count = 0;
        {
            let registry = resource_manager.registry();
            for candidate in &mut candidates {
                #[cfg(test)]
                {
                    candidate_registry_resolution_count += 1;
                }
                candidate.revision = registry
                    .get(candidate.cubemap)
                    .map(|record| record.revision);
            }
        }
        let mut gpu_probes = Vec::with_capacity(candidates.len());
        let mut upload_candidate =
            |candidate: ReflectionProbeCandidate<'_>, gpu_probes: &mut Vec<GpuReflectionProbe>| {
                report.attempted_candidate_count += 1;
                let ReflectionProbeCandidate {
                    probe,
                    cubemap,
                    revision,
                    ..
                } = candidate;
                let Some(revision) = revision else {
                    record_probe_asset_rejection(
                        &mut report,
                        ReflectionProbeAssetError::MissingResource { cubemap },
                    );
                    return;
                };
                let slot = if self.slots.capture_pending(cubemap) {
                    let Some(allocation) = self.slots.acquire(cubemap, revision, prepare_epoch)
                    else {
                        return;
                    };
                    debug_assert!(!allocation.requires_upload);
                    allocation.slot
                } else {
                    match self.slots.available(cubemap, revision, prepare_epoch) {
                        Some(_) => {
                            let allocation = self
                                .slots
                                .acquire(cubemap, revision, prepare_epoch)
                                .expect("available probe slot must remain admissible");
                            debug_assert!(!allocation.requires_upload);
                            allocation.slot
                        }
                        _ => {
                            report.asset_load_call_count += 1;
                            let asset_load_started = Instant::now();
                            let texture_result = asset_manager.load_texture_asset(cubemap);
                            report.asset_load_cpu_time_us =
                                report.asset_load_cpu_time_us.saturating_add(
                                    u64::try_from(asset_load_started.elapsed().as_micros())
                                        .unwrap_or(u64::MAX),
                                );
                            let texture = match texture_result {
                                Ok(texture) => texture,
                                Err(source) => {
                                    record_probe_asset_rejection(
                                        &mut report,
                                        ReflectionProbeAssetError::Load { cubemap, source },
                                    );
                                    return;
                                }
                            };
                            let bytes = match validate_probe_pmrem_texture(cubemap, &texture) {
                                Ok(bytes) => bytes,
                                Err(error) => {
                                    record_probe_asset_rejection(&mut report, error);
                                    return;
                                }
                            };
                            if self.environment_only_placeholder {
                                self.upgrade_environment_only_provider(device);
                            }
                            let Some(allocation) =
                                self.slots.acquire(cubemap, revision, prepare_epoch)
                            else {
                                return;
                            };
                            debug_assert!(allocation.requires_upload);
                            append_probe_pmrem_texture_uploads(
                                frame_texture_uploads,
                                &self.cubemap_array,
                                allocation.slot,
                                bytes,
                            );
                            self.pending_uploads.push(PendingReflectionProbeUpload {
                                cubemap,
                                revision,
                                slot: allocation.slot,
                                prepare_epoch,
                            });
                            report.scheduled_cubemap_upload_count += 1;
                            report.scheduled_cubemap_upload_bytes = report
                                .scheduled_cubemap_upload_bytes
                                .saturating_add(u64::try_from(bytes.len()).unwrap_or(u64::MAX));
                            report.scheduled_texture_write_count = report
                                .scheduled_texture_write_count
                                .saturating_add(REFLECTION_PROBE_MIP_COUNT as usize);
                            allocation.slot
                        }
                    }
                };
                gpu_probes.push(GpuReflectionProbe::from_probe(
                    probe,
                    slot,
                    REFLECTION_PROBE_MIP_COUNT,
                ));
            };
        for candidate in candidates {
            upload_candidate(candidate, &mut gpu_probes);
        }
        if gpu_probes.len() < MAX_REFLECTION_PROBES && !overflow_candidates.is_empty() {
            overflow_candidates.sort_by(reflection_probe_candidate_order);
            for mut candidate in overflow_candidates {
                if gpu_probes.len() == MAX_REFLECTION_PROBES {
                    break;
                }
                {
                    let registry = resource_manager.registry();
                    #[cfg(test)]
                    {
                        candidate_registry_resolution_count += 1;
                    }
                    candidate.revision = registry
                        .get(candidate.cubemap)
                        .map(|record| record.revision);
                }
                upload_candidate(candidate, &mut gpu_probes);
            }
        }
        drop(upload_candidate);
        report.capacity_dropped_candidate_count =
            eligible_candidate_count.saturating_sub(report.attempted_candidate_count);
        #[cfg(test)]
        {
            self.candidate_registry_resolution_count += candidate_registry_resolution_count;
        }

        report.active_probe_count = gpu_probes.len();
        self.append_buffer_uploads(
            frame_buffer_uploads,
            planar_params,
            &gpu_probes,
            camera_layers.to_scene_schema_v1_mask_lossy(),
        );
        self.last_report = report;
        report
    }

    pub(in crate::graphics::scene::scene_renderer) fn commit_pending_uploads(&mut self) {
        for pending in std::mem::take(&mut self.pending_uploads) {
            self.slots.commit(
                pending.cubemap,
                pending.revision,
                pending.slot,
                pending.prepare_epoch,
            );
        }
    }

    pub(in crate::graphics::scene::scene_renderer) fn discard_pending_uploads(&mut self) {
        self.pending_uploads.clear();
    }

    fn begin_prepare_epoch(&mut self) -> u64 {
        self.next_prepare_epoch = self.next_prepare_epoch.wrapping_add(1);
        if self.next_prepare_epoch == 0 {
            self.slots.invalidate_pending_epochs();
            self.next_prepare_epoch = 1;
        }
        self.next_prepare_epoch
    }

    fn upgrade_environment_only_provider(&mut self, device: &wgpu::Device) {
        debug_assert!(self.pending_uploads.is_empty());
        let next_prepare_epoch = self.next_prepare_epoch;
        *self = Self::new(device);
        self.next_prepare_epoch = next_prepare_epoch;
        self.environment_only_provider_upgrade = true;
    }

    fn append_buffer_uploads(
        &self,
        frame_batch: &mut WgpuBufferUploadBatch,
        planar_params: Option<GpuPlanarReflection>,
        gpu_probes: &[GpuReflectionProbe],
        camera_layer_mask: u32,
    ) {
        let planar_params = planar_params.unwrap_or_default();
        let probe_header = GpuReflectionProbeHeader::with_probe_count_and_camera_layer_mask(
            u32::try_from(gpu_probes.len()).expect("reflection probe count exceeded u32"),
            camera_layer_mask,
        );
        let mut payload = Vec::with_capacity(
            std::mem::size_of::<GpuPlanarReflection>()
                + std::mem::size_of_val(gpu_probes)
                + std::mem::size_of::<GpuReflectionProbeHeader>(),
        );
        let planar_start = payload.len();
        payload.extend_from_slice(bytemuck::bytes_of(&planar_params));
        let planar_range = planar_start..payload.len();
        let probes_start = payload.len();
        payload.extend_from_slice(bytemuck::cast_slice(gpu_probes));
        let probes_range = probes_start..payload.len();
        let header_start = payload.len();
        payload.extend_from_slice(bytemuck::bytes_of(&probe_header));
        let header_range = header_start..payload.len();

        let payload: Arc<[u8]> = Arc::from(payload);
        frame_batch.push(
            WgpuBufferUpload::new(
                self.planar_params_buffer.as_ref().clone(),
                0,
                Arc::clone(&payload),
                planar_range,
            )
            .expect("planar reflection upload range must reference its packed payload"),
        );
        if !probes_range.is_empty() {
            frame_batch.push(
                WgpuBufferUpload::new(
                    self.probe_buffer.as_ref().clone(),
                    0,
                    Arc::clone(&payload),
                    probes_range,
                )
                .expect("reflection probe upload range must reference its packed payload"),
            );
        }
        frame_batch.push(
            WgpuBufferUpload::new(
                self.header_buffer.as_ref().clone(),
                0,
                payload,
                header_range,
            )
            .expect("reflection probe header range must reference its packed payload"),
        );
    }
}
