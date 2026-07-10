use std::sync::Arc;

use wgpu::util::DeviceExt;

use crate::core::framework::render::{ProbeInfluenceShape, ReflectionProbeData};
use crate::core::math::Vec3;
#[cfg(test)]
use crate::graphics::backend::{
    read_buffer_bytes, read_texture_rgba16float_region, BufferByteReadback,
    Rgba16FloatTextureRegionReadback,
};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::types::ViewportRenderFrame;

use super::gpu_layout::{GpuReflectionProbe, GpuReflectionProbeHeader, ReflectionProbeGpuBindings};
use super::slot_allocator::ProbeCubemapSlotAllocator;
use super::upload::{
    upload_probe_pmrem_texture, validate_probe_pmrem_texture, ReflectionProbeAssetError,
    ReflectionProbeAssetRejection,
};

pub(super) const MAX_REFLECTION_PROBES: usize = 64;
pub(super) const REFLECTION_PROBE_FACE_SIZE: u32 = 128;
pub(super) const REFLECTION_PROBE_MIP_COUNT: u32 = 8;
const REFLECTION_PROBE_CUBE_ARRAY_LAYER_COUNT: u32 = MAX_REFLECTION_PROBES as u32 * 6;

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
    slots: ProbeCubemapSlotAllocator,
    last_report: ReflectionProbeUploadReport,
}

impl SceneReflectionProbeResources {
    pub(in crate::graphics::scene::scene_renderer) fn new(device: &wgpu::Device) -> Self {
        let probe_buffer_usage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        #[cfg(test)]
        let probe_buffer_usage = probe_buffer_usage | wgpu::BufferUsages::COPY_SRC;
        let probe_buffer = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-reflection-probe-storage"),
            size: (std::mem::size_of::<GpuReflectionProbe>() * MAX_REFLECTION_PROBES) as u64,
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
                width: REFLECTION_PROBE_FACE_SIZE,
                height: REFLECTION_PROBE_FACE_SIZE,
                depth_or_array_layers: REFLECTION_PROBE_CUBE_ARRAY_LAYER_COUNT,
            },
            mip_level_count: REFLECTION_PROBE_MIP_COUNT,
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
                mip_level_count: Some(REFLECTION_PROBE_MIP_COUNT),
                base_array_layer: 0,
                array_layer_count: Some(REFLECTION_PROBE_CUBE_ARRAY_LAYER_COUNT),
            }));
        Self {
            probe_buffer,
            header_buffer,
            cubemap_array,
            cubemap_array_view,
            slots: ProbeCubemapSlotAllocator::new(MAX_REFLECTION_PROBES),
            last_report: ReflectionProbeUploadReport::default(),
        }
    }

    pub(in crate::graphics::scene::scene_renderer) fn bindings(
        &self,
    ) -> ReflectionProbeGpuBindings {
        ReflectionProbeGpuBindings::new(
            Arc::clone(&self.probe_buffer),
            Arc::clone(&self.header_buffer),
            Arc::clone(&self.cubemap_array_view),
        )
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
                byte_len: (std::mem::size_of::<GpuReflectionProbe>() * 2) as u64,
                label: "zircon-reflection-probe-storage-readback",
            },
        )?;
        let header = bytemuck::pod_read_unaligned::<GpuReflectionProbeHeader>(&header_bytes);
        let probes = [
            bytemuck::pod_read_unaligned::<GpuReflectionProbe>(
                &probe_bytes[..std::mem::size_of::<GpuReflectionProbe>()],
            ),
            bytemuck::pod_read_unaligned::<GpuReflectionProbe>(
                &probe_bytes[std::mem::size_of::<GpuReflectionProbe>()..],
            ),
        ];
        let mut first_texels = [[0_u16; 4]; 2];
        for (slot, texel) in first_texels.iter_mut().enumerate() {
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
        queue: &wgpu::Queue,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
        enabled: bool,
    ) -> ReflectionProbeUploadReport {
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
            .filter(|probe| {
                probe.baked_cubemap().is_some()
                    && probe.intensity() > 0.0
                    && probe.layer_mask().intersects(camera_layers)
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            probe_distance_to_influence(left, camera_position)
                .total_cmp(&probe_distance_to_influence(right, camera_position))
                .then_with(|| right.priority().cmp(&left.priority()))
                .then_with(|| left.probe_id().cmp(&right.probe_id()))
        });
        candidates.truncate(MAX_REFLECTION_PROBES);

        let asset_manager = streamer.asset_manager();
        let resource_manager = asset_manager.resource_manager();
        let mut gpu_probes = Vec::with_capacity(candidates.len());
        for probe in candidates {
            let Some(cubemap) = probe.baked_cubemap() else {
                continue;
            };
            let registry = resource_manager.registry();
            let Some(record) = registry.get(cubemap) else {
                record_probe_asset_rejection(
                    &mut report,
                    ReflectionProbeAssetError::MissingResource { cubemap },
                );
                continue;
            };
            let revision = record.revision;
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
}

fn probe_distance_to_influence(probe: &ReflectionProbeData, world_position: Vec3) -> f32 {
    let local = probe.rotation().conjugate() * (world_position - probe.position());
    match probe.shape() {
        ProbeInfluenceShape::Box { half_extents, .. } => {
            (local.abs() - half_extents).max(Vec3::ZERO).length()
        }
        ProbeInfluenceShape::Sphere { radius, .. } => (local.length() - radius).max(0.0),
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
