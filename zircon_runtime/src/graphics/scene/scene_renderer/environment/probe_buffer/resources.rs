use std::sync::Arc;

use wgpu::util::DeviceExt;

use crate::core::framework::render::{
    ProbeInfluenceShape, ReflectionProbeData, RenderCameraTarget, derive_planar_reflection_camera,
};
use crate::core::math::{Vec3, view_matrix};
#[cfg(test)]
use crate::graphics::backend::{
    BufferByteReadback, Rgba16FloatTextureRegionReadback, read_buffer_bytes,
    read_texture_rgba16float_region,
};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::types::ViewportRenderFrame;

use super::gpu_layout::{
    GpuPlanarReflection, GpuReflectionProbe, GpuReflectionProbeHeader, ReflectionProbeGpuBindings,
};
use super::slot_allocator::ProbeCubemapSlotAllocator;
use super::upload::{
    ReflectionProbeAssetError, ReflectionProbeAssetRejection, upload_probe_pmrem_texture,
    validate_probe_pmrem_texture,
};

pub(super) const MAX_REFLECTION_PROBES: usize = 64;
pub(super) const REFLECTION_PROBE_FACE_SIZE: u32 = 128;
pub(super) const REFLECTION_PROBE_MIP_COUNT: u32 = 8;
pub(in crate::graphics::scene::scene_renderer) const PLANAR_REFLECTION_TEXTURE_SIZE: u32 = 1024;
pub(in crate::graphics::scene::scene_renderer) const PLANAR_REFLECTION_MIP_COUNT: u32 = 11;
const ENVIRONMENT_PREVIEW_PLACEHOLDER_PROBE_COUNT: usize = 1;
const ENVIRONMENT_PREVIEW_PLACEHOLDER_FACE_SIZE: u32 = 1;
const ENVIRONMENT_PREVIEW_PLACEHOLDER_MIP_COUNT: u32 = 1;
const ENVIRONMENT_PREVIEW_PLACEHOLDER_PLANAR_TEXTURE_SIZE: u32 = 1;
const ENVIRONMENT_PREVIEW_PLACEHOLDER_PLANAR_MIP_COUNT: u32 = 1;

#[derive(Clone, Copy)]
struct ReflectionProbeResourceCapacity {
    probe_count: usize,
    cubemap_face_size: u32,
    cubemap_mip_count: u32,
    planar_texture_size: u32,
    planar_mip_count: u32,
}

impl ReflectionProbeResourceCapacity {
    const FULL: Self = Self {
        probe_count: MAX_REFLECTION_PROBES,
        cubemap_face_size: REFLECTION_PROBE_FACE_SIZE,
        cubemap_mip_count: REFLECTION_PROBE_MIP_COUNT,
        planar_texture_size: PLANAR_REFLECTION_TEXTURE_SIZE,
        planar_mip_count: PLANAR_REFLECTION_MIP_COUNT,
    };

    const ENVIRONMENT_PREVIEW_PLACEHOLDER: Self = Self {
        probe_count: ENVIRONMENT_PREVIEW_PLACEHOLDER_PROBE_COUNT,
        cubemap_face_size: ENVIRONMENT_PREVIEW_PLACEHOLDER_FACE_SIZE,
        cubemap_mip_count: ENVIRONMENT_PREVIEW_PLACEHOLDER_MIP_COUNT,
        planar_texture_size: ENVIRONMENT_PREVIEW_PLACEHOLDER_PLANAR_TEXTURE_SIZE,
        planar_mip_count: ENVIRONMENT_PREVIEW_PLACEHOLDER_PLANAR_MIP_COUNT,
    };
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::graphics::scene::scene_renderer) struct ReflectionProbeUploadReport {
    pub(in crate::graphics::scene::scene_renderer) extracted_probe_count: usize,
    pub(in crate::graphics::scene::scene_renderer) active_probe_count: usize,
    pub(in crate::graphics::scene::scene_renderer) uploaded_cubemap_count: usize,
    pub(in crate::graphics::scene::scene_renderer) rejected_cubemap_count: usize,
    pub(in crate::graphics::scene::scene_renderer) first_rejection:
        Option<ReflectionProbeAssetRejection>,
}

pub(in crate::graphics::scene::scene_renderer) struct SceneReflectionProbeResources {
    probe_buffer: Arc<wgpu::Buffer>,
    header_buffer: Arc<wgpu::Buffer>,
    cubemap_array: wgpu::Texture,
    cubemap_array_view: Arc<wgpu::TextureView>,
    planar_texture: Arc<wgpu::Texture>,
    planar_texture_view: Arc<wgpu::TextureView>,
    planar_params_buffer: Arc<wgpu::Buffer>,
    slots: ProbeCubemapSlotAllocator,
    last_report: ReflectionProbeUploadReport,
    environment_only_placeholder: bool,
    environment_only_provider_upgrade: bool,
    #[cfg(test)]
    probe_capacity: usize,
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
        let cubemap_array_usage =
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
        #[cfg(test)]
        let cubemap_array_usage = cubemap_array_usage | wgpu::TextureUsages::COPY_SRC;
        let cubemap_array = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-reflection-probe-pmrem-cube-array"),
            size: wgpu::Extent3d {
                width: capacity.cubemap_face_size,
                height: capacity.cubemap_face_size,
                depth_or_array_layers: capacity.probe_count as u32 * 6,
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
                array_layer_count: Some(capacity.probe_count as u32 * 6),
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
        Self {
            probe_buffer,
            header_buffer,
            cubemap_array,
            cubemap_array_view,
            planar_texture,
            planar_texture_view,
            planar_params_buffer,
            slots: ProbeCubemapSlotAllocator::new(capacity.probe_count),
            last_report: ReflectionProbeUploadReport::default(),
            environment_only_placeholder,
            environment_only_provider_upgrade: false,
            #[cfg(test)]
            probe_capacity: capacity.probe_count,
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

    pub(in crate::graphics::scene::scene_renderer) fn planar_texture(&self) -> Arc<wgpu::Texture> {
        Arc::clone(&self.planar_texture)
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer) fn last_report_diagnostics(
        &self,
    ) -> (usize, usize, usize, usize, Option<String>) {
        (
            self.last_report.extracted_probe_count,
            self.last_report.active_probe_count,
            self.last_report.uploaded_cubemap_count,
            self.last_report.rejected_cubemap_count,
            self.last_report
                .first_rejection
                .map(|rejection| format!("{rejection:?}")),
        )
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
                        z: slot as u32 * 6,
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
        queue: &wgpu::Queue,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
        enabled: bool,
    ) -> ReflectionProbeUploadReport {
        if self.environment_only_placeholder
            && reflection_provider_resources_requested(frame, enabled)
        {
            *self = Self::new(device);
            self.environment_only_provider_upgrade = true;
        }
        self.prepare_planar_reflection(queue, frame);
        let mut report = ReflectionProbeUploadReport {
            extracted_probe_count: frame.environment().probes.len(),
            ..ReflectionProbeUploadReport::default()
        };
        if !enabled {
            self.write_probe_header(queue, 0);
            self.last_report = report;
            return report;
        }

        let camera_position = frame.effective_camera().transform.translation;
        let camera_layers = frame.extract.view.selected_camera_layers();
        let mut candidates = frame
            .environment()
            .probes
            .iter()
            .filter_map(|probe| {
                let cubemap = probe.baked_cubemap()?;
                (probe.intensity() > 0.0 && probe.layer_mask().intersects(camera_layers))
                    .then(|| {
                        (
                            probe,
                            cubemap,
                            None,
                            probe_distance_to_influence(probe, camera_position),
                        )
                    })
            })
            .collect::<Vec<_>>();
        let candidate_order = |
            left: &(&ReflectionProbeData, crate::core::resource::ResourceId, Option<u64>, f32),
            right: &(&ReflectionProbeData, crate::core::resource::ResourceId, Option<u64>, f32),
        | {
            left.3
                .total_cmp(&right.3)
                .then_with(|| right.0.priority().cmp(&left.0.priority()))
                .then_with(|| left.0.probe_id().cmp(&right.0.probe_id()))
        };
        if candidates.len() > MAX_REFLECTION_PROBES {
            candidates.select_nth_unstable_by(MAX_REFLECTION_PROBES, candidate_order);
            candidates.truncate(MAX_REFLECTION_PROBES);
        }
        candidates.sort_by(candidate_order);

        let asset_manager = match streamer.asset_manager() {
            Ok(asset_manager) => asset_manager,
            Err(_) => return report,
        };
        let resource_manager = asset_manager.resource_manager();
        {
            let registry = resource_manager.registry();
            for (_, cubemap, revision, _) in &mut candidates {
                *revision = registry.get(*cubemap).map(|record| record.revision);
            }
        }
        let mut gpu_probes = Vec::with_capacity(candidates.len());
        for (probe, cubemap, revision, _) in candidates {
            let Some(revision) = revision else {
                record_probe_asset_rejection(
                    &mut report,
                    ReflectionProbeAssetError::MissingResource { cubemap },
                );
                continue;
            };
            let slot = match self.slots.get(cubemap) {
                Some(slot) if slot.revision == revision => {
                    self.slots.acquire(cubemap, revision).slot
                }
                _ => {
                    let texture = match asset_manager.load_texture_asset(cubemap) {
                        Ok(texture) => texture,
                        Err(source) => {
                            record_probe_asset_rejection(
                                &mut report,
                                ReflectionProbeAssetError::Load { cubemap, source },
                            );
                            continue;
                        }
                    };
                    let bytes = match validate_probe_pmrem_texture(cubemap, &texture) {
                        Ok(bytes) => bytes,
                        Err(error) => {
                            record_probe_asset_rejection(&mut report, error);
                            continue;
                        }
                    };
                    let allocation = self.slots.acquire(cubemap, revision);
                    debug_assert!(allocation.requires_upload);
                    upload_probe_pmrem_texture(queue, &self.cubemap_array, allocation.slot, bytes);
                    report.uploaded_cubemap_count += 1;
                    allocation.slot
                }
            };
            gpu_probes.push(GpuReflectionProbe::from_probe(
                probe,
                slot,
                REFLECTION_PROBE_MIP_COUNT,
            ));
        }

        if !gpu_probes.is_empty() {
            queue.write_buffer(&self.probe_buffer, 0, bytemuck::cast_slice(&gpu_probes));
        }
        report.active_probe_count = gpu_probes.len();
        self.write_probe_header(queue, gpu_probes.len() as u32);
        self.last_report = report;
        report
    }

    fn write_probe_header(&self, queue: &wgpu::Queue, probe_count: u32) {
        queue.write_buffer(
            &self.header_buffer,
            0,
            bytemuck::bytes_of(&GpuReflectionProbeHeader::with_probe_count(probe_count)),
        );
    }

    fn prepare_planar_reflection(&self, queue: &wgpu::Queue, frame: &ViewportRenderFrame) {
        let selected_target = frame.extract.view.selected_camera_target();
        let is_capture_camera = matches!(selected_target, RenderCameraTarget::Texture(target) if frame
            .extract
            .lighting
            .advanced_lighting
            .planar_probes
            .iter()
            .any(|probe| probe.capture_target() == Some(*target)));
        let params = if is_capture_camera {
            GpuPlanarReflection::default()
        } else {
            frame
                .extract
                .lighting
                .advanced_lighting
                .planar_probes
                .iter()
                .filter(|probe| {
                    probe.capture_target().is_some()
                        && probe
                            .layer_mask
                            .intersects(frame.extract.view.selected_camera_layers())
                })
                .min_by_key(|probe| probe.probe_id)
                .and_then(|probe| planar_gpu_params(frame, probe))
                .unwrap_or_default()
        };
        queue.write_buffer(&self.planar_params_buffer, 0, bytemuck::bytes_of(&params));
    }
}

fn reflection_provider_resources_requested(frame: &ViewportRenderFrame, enabled: bool) -> bool {
    let camera_layers = frame.extract.view.selected_camera_layers();
    let has_visible_baked_probe = enabled
        && frame.environment().probes.iter().any(|probe| {
            probe.baked_cubemap().is_some()
                && probe.intensity() > 0.0
                && probe.layer_mask().intersects(camera_layers)
        });
    let planar_probes = &frame.extract.lighting.advanced_lighting.planar_probes;
    let is_planar_capture_camera = matches!(
        frame.extract.view.selected_camera_target(),
        RenderCameraTarget::Texture(target) if planar_probes
            .iter()
            .any(|probe| probe.capture_target() == Some(*target))
    );
    let has_visible_planar_provider = planar_probes.iter().any(|probe| {
        probe.capture_target().is_some() && probe.layer_mask.intersects(camera_layers)
    });
    has_visible_baked_probe || is_planar_capture_camera || has_visible_planar_provider
}

fn planar_gpu_params(
    frame: &ViewportRenderFrame,
    probe: &crate::core::framework::render::PlanarReflectionProbeData,
) -> Option<GpuPlanarReflection> {
    let target = probe.capture_target()?;
    let main_camera = frame.extract.view.selected_camera_descriptor()?;
    let reflected = derive_planar_reflection_camera(main_camera, probe, target)?;
    let projection = reflected.camera.projection_override?;
    let clip_from_world = projection * view_matrix(reflected.camera.transform);
    let determinant = probe.plane_transform.determinant();
    if !determinant.is_finite() || determinant.abs() <= 1.0e-6 {
        return None;
    }
    let local_from_world = probe.plane_transform.inverse();
    let resolution = probe.resolution.clamp(1, PLANAR_REFLECTION_TEXTURE_SIZE);
    let mip_count = u32::BITS - resolution.leading_zeros();
    let scale = resolution as f32 / PLANAR_REFLECTION_TEXTURE_SIZE as f32;
    Some(GpuPlanarReflection {
        clip_from_world: clip_from_world.to_cols_array_2d(),
        local_from_world: local_from_world.to_cols_array_2d(),
        bounds_min: probe.bounds_min.extend(0.0).to_array(),
        bounds_max: probe.bounds_max.extend(0.0).to_array(),
        sample_params: [scale, scale, mip_count as f32, 1.0],
    })
}

fn probe_distance_to_influence(probe: &ReflectionProbeData, world_position: Vec3) -> f32 {
    let position_delta = world_position - probe.position();
    match probe.shape() {
        ProbeInfluenceShape::Box { half_extents, .. } => {
            let local = probe.rotation().conjugate() * position_delta;
            (local.abs() - half_extents).max(Vec3::ZERO).length()
        }
        ProbeInfluenceShape::Sphere { radius, .. } => (position_delta.length() - radius).max(0.0),
    }
}

fn record_probe_asset_rejection(
    report: &mut ReflectionProbeUploadReport,
    error: ReflectionProbeAssetError,
) {
    report.rejected_cubemap_count += 1;
    if report.first_rejection.is_none() {
        report.first_rejection = Some(error.rejection());
    }
}
