// Staged Plan 11 acquisition helpers land before the runtime bake scheduler consumes them.

use std::sync::mpsc;

use crate::core::framework::render::{
    source_cubemap_mip_size, IblBakeArtifactContents, IblBakeArtifactDescriptor,
    IblBakeArtifactReadbackSections, IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES, SOURCE_CUBEMAP_FACE_COUNT,
    SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
};
use crate::graphics::debug_markers::{insert_marker, RENDERDOC_MARKER_READBACK};
use crate::graphics::types::GraphicsError;

const RGBA16FLOAT_BYTES_PER_TEXEL: u32 = 8;

#[derive(Clone, Copy)]
pub(crate) struct IblBakeArtifactWgpuReadbackResources<'a> {
    descriptor: IblBakeArtifactDescriptor,
    pmrem_texture: Option<&'a wgpu::Texture>,
    irradiance_sh9_buffer: Option<&'a wgpu::Buffer>,
    irradiance_cube_texture: Option<&'a wgpu::Texture>,
}

impl<'a> IblBakeArtifactWgpuReadbackResources<'a> {
    pub const fn new(descriptor: IblBakeArtifactDescriptor) -> Self {
        Self {
            descriptor,
            pmrem_texture: None,
            irradiance_sh9_buffer: None,
            irradiance_cube_texture: None,
        }
    }

    pub const fn descriptor(&self) -> IblBakeArtifactDescriptor {
        self.descriptor
    }

    pub const fn requires_pmrem_texture(&self) -> bool {
        self.descriptor
            .contents()
            .contains(IblBakeArtifactContents::PMREM)
    }

    pub const fn requires_irradiance_sh9_buffer(&self) -> bool {
        self.descriptor
            .contents()
            .contains(IblBakeArtifactContents::SH9)
    }

    pub const fn requires_irradiance_cube_texture(&self) -> bool {
        self.descriptor
            .contents()
            .contains(IblBakeArtifactContents::IEM)
    }

    pub fn with_pmrem_texture(mut self, texture: &'a wgpu::Texture) -> Self {
        self.pmrem_texture = Some(texture);
        self
    }

    pub fn with_irradiance_sh9_buffer(mut self, buffer: &'a wgpu::Buffer) -> Self {
        self.irradiance_sh9_buffer = Some(buffer);
        self
    }

    pub fn with_irradiance_cube_texture(mut self, texture: &'a wgpu::Texture) -> Self {
        self.irradiance_cube_texture = Some(texture);
        self
    }
}

pub(crate) fn read_ibl_bake_artifact_wgpu_sections(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    resources: IblBakeArtifactWgpuReadbackResources<'_>,
) -> Result<IblBakeArtifactReadbackSections, GraphicsError> {
    let descriptor = resources.descriptor;
    build_ibl_bake_artifact_wgpu_readback_batch(device, resources)?
        .finish(device, queue, descriptor)
}

pub(crate) fn prepare_ibl_bake_artifact_wgpu_readback(
    device: &wgpu::Device,
    resources: IblBakeArtifactWgpuReadbackResources<'_>,
) -> Result<IblBakeArtifactWgpuPendingReadback, GraphicsError> {
    let descriptor = resources.descriptor;
    Ok(build_ibl_bake_artifact_wgpu_readback_batch(device, resources)?.into_pending(descriptor))
}

fn build_ibl_bake_artifact_wgpu_readback_batch(
    device: &wgpu::Device,
    resources: IblBakeArtifactWgpuReadbackResources<'_>,
) -> Result<IblBakeArtifactWgpuReadbackBatch, GraphicsError> {
    let descriptor = resources.descriptor;
    let mut batch = IblBakeArtifactWgpuReadbackBatch::new(device);

    if resources.requires_pmrem_texture() {
        let texture = required_wgpu_readback_resource(resources.pmrem_texture, "PMREM texture")?;
        batch.add_pmrem(
            device,
            texture,
            descriptor.face_size(),
            descriptor.mip_count(),
        );
    }

    if resources.requires_irradiance_sh9_buffer() {
        let buffer =
            required_wgpu_readback_resource(resources.irradiance_sh9_buffer, "SH9 buffer")?;
        batch.add_sh9(device, buffer);
    }

    if resources.requires_irradiance_cube_texture() {
        let texture = required_wgpu_readback_resource(
            resources.irradiance_cube_texture,
            "irradiance cube texture",
        )?;
        batch.add_irradiance_cube(device, texture);
    }

    Ok(batch)
}

pub(crate) struct IblBakeArtifactWgpuPendingReadback {
    descriptor: IblBakeArtifactDescriptor,
    command_buffer: Option<wgpu::CommandBuffer>,
    pmrem: Option<CubeMipChainReadback>,
    sh9: Option<BufferReadback>,
    irradiance_cube: Option<CubeMipChainReadback>,
    completion: Option<mpsc::Receiver<Result<(), String>>>,
    remaining_map_count: usize,
}

impl IblBakeArtifactWgpuPendingReadback {
    pub(crate) fn take_command_buffer(&mut self) -> Option<wgpu::CommandBuffer> {
        self.command_buffer.take()
    }

    pub(crate) fn begin_map(&mut self) {
        if self.remaining_map_count == 0 || self.completion.is_some() {
            return;
        }
        let (sender, receiver) = mpsc::channel();
        if let Some(readback) = self.pmrem.as_ref() {
            readback.map_async(sender.clone());
        }
        if let Some(readback) = self.sh9.as_ref() {
            readback.map_async(sender.clone());
        }
        if let Some(readback) = self.irradiance_cube.as_ref() {
            readback.map_async(sender.clone());
        }
        drop(sender);
        self.completion = Some(receiver);
    }

    pub(crate) fn poll_ready(&mut self) -> Result<bool, GraphicsError> {
        if self.remaining_map_count == 0 {
            return Ok(true);
        }
        let Some(completion) = self.completion.as_ref() else {
            return Ok(false);
        };
        loop {
            match completion.try_recv() {
                Ok(Ok(())) => {
                    self.remaining_map_count = self.remaining_map_count.saturating_sub(1);
                    if self.remaining_map_count == 0 {
                        return Ok(true);
                    }
                }
                Ok(Err(error)) => {
                    self.unmap_all();
                    return Err(GraphicsError::BufferMap(error));
                }
                Err(mpsc::TryRecvError::Empty) => return Ok(false),
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.unmap_all();
                    return Err(GraphicsError::BufferMap(
                        "IBL bake readback map callbacks disconnected before completion"
                            .to_string(),
                    ));
                }
            }
        }
    }

    pub(crate) fn finish(self) -> Result<IblBakeArtifactReadbackSections, GraphicsError> {
        if self.remaining_map_count != 0 {
            return Err(GraphicsError::BufferMap(
                "IBL bake readback was consumed before all map callbacks completed".to_string(),
            ));
        }
        Ok(readback_sections_from_mapped_buffers(
            self.descriptor,
            self.pmrem,
            self.sh9,
            self.irradiance_cube,
        ))
    }

    fn unmap_all(&self) {
        IblBakeArtifactWgpuReadbackBatch::unmap_all(&self.pmrem, &self.sh9, &self.irradiance_cube);
    }
}

/// Keeps a cache-miss artifact readback behind one GPU submission and one device wait.
/// The artifact payload remains sectioned so its on-disk format and cache contract do not change.
struct IblBakeArtifactWgpuReadbackBatch {
    encoder: wgpu::CommandEncoder,
    pmrem: Option<CubeMipChainReadback>,
    sh9: Option<BufferReadback>,
    irradiance_cube: Option<CubeMipChainReadback>,
}

impl IblBakeArtifactWgpuReadbackBatch {
    fn new(device: &wgpu::Device) -> Self {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-readback-ibl-artifact-encoder"),
        });
        insert_marker(&mut encoder, RENDERDOC_MARKER_READBACK);
        Self {
            encoder,
            pmrem: None,
            sh9: None,
            irradiance_cube: None,
        }
    }

    fn add_pmrem(
        &mut self,
        device: &wgpu::Device,
        texture: &wgpu::Texture,
        face_size: u32,
        mip_count: u32,
    ) {
        self.pmrem = Some(CubeMipChainReadback::encode(
            device,
            &mut self.encoder,
            texture,
            face_size,
            mip_count,
            "zircon-readback-ibl-pmrem",
        ));
    }

    fn add_sh9(&mut self, device: &wgpu::Device, source: &wgpu::Buffer) {
        self.sh9 = Some(BufferReadback::encode(
            device,
            &mut self.encoder,
            source,
            IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES as u64,
            "zircon-readback-ibl-sh9",
        ));
    }

    fn add_irradiance_cube(&mut self, device: &wgpu::Device, texture: &wgpu::Texture) {
        self.irradiance_cube = Some(CubeMipChainReadback::encode(
            device,
            &mut self.encoder,
            texture,
            SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
            1,
            "zircon-readback-ibl-irradiance-cube",
        ));
    }

    fn into_pending(
        self,
        descriptor: IblBakeArtifactDescriptor,
    ) -> IblBakeArtifactWgpuPendingReadback {
        let readback_count = self.pmrem.is_some() as usize
            + self.sh9.is_some() as usize
            + self.irradiance_cube.is_some() as usize;
        let Self {
            encoder,
            pmrem,
            sh9,
            irradiance_cube,
        } = self;
        IblBakeArtifactWgpuPendingReadback {
            descriptor,
            command_buffer: (readback_count > 0).then(|| encoder.finish()),
            pmrem,
            sh9,
            irradiance_cube,
            completion: None,
            remaining_map_count: readback_count,
        }
    }

    fn finish(
        self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        descriptor: IblBakeArtifactDescriptor,
    ) -> Result<IblBakeArtifactReadbackSections, GraphicsError> {
        let readback_count = self.pmrem.is_some() as usize
            + self.sh9.is_some() as usize
            + self.irradiance_cube.is_some() as usize;
        if readback_count == 0 {
            return Ok(IblBakeArtifactReadbackSections::new(descriptor));
        }

        let Self {
            encoder,
            pmrem,
            sh9,
            irradiance_cube,
        } = self;
        queue.submit([encoder.finish()]);
        let (sender, receiver) = mpsc::channel();
        if let Some(readback) = pmrem.as_ref() {
            readback.map_async(sender.clone());
        }
        if let Some(readback) = sh9.as_ref() {
            readback.map_async(sender.clone());
        }
        if let Some(readback) = irradiance_cube.as_ref() {
            readback.map_async(sender.clone());
        }
        drop(sender);
        if let Err(error) = device.poll(wgpu::PollType::wait_indefinitely()) {
            Self::unmap_all(&pmrem, &sh9, &irradiance_cube);
            return Err(GraphicsError::BufferMap(error.to_string()));
        }
        let mut mapping_error = None;
        for _ in 0..readback_count {
            match receiver.recv() {
                Ok(Ok(())) => {}
                Ok(Err(error)) => {
                    mapping_error.get_or_insert(GraphicsError::BufferMap(error));
                }
                Err(error) => {
                    mapping_error.get_or_insert(GraphicsError::BufferMap(error.to_string()));
                }
            }
        }
        if let Some(error) = mapping_error {
            Self::unmap_all(&pmrem, &sh9, &irradiance_cube);
            return Err(error);
        }

        Ok(readback_sections_from_mapped_buffers(
            descriptor,
            pmrem,
            sh9,
            irradiance_cube,
        ))
    }

    fn unmap_all(
        pmrem: &Option<CubeMipChainReadback>,
        sh9: &Option<BufferReadback>,
        irradiance_cube: &Option<CubeMipChainReadback>,
    ) {
        if let Some(readback) = pmrem.as_ref() {
            readback.unmap();
        }
        if let Some(readback) = sh9.as_ref() {
            readback.unmap();
        }
        if let Some(readback) = irradiance_cube.as_ref() {
            readback.unmap();
        }
    }
}

fn readback_sections_from_mapped_buffers(
    descriptor: IblBakeArtifactDescriptor,
    pmrem: Option<CubeMipChainReadback>,
    sh9: Option<BufferReadback>,
    irradiance_cube: Option<CubeMipChainReadback>,
) -> IblBakeArtifactReadbackSections {
    let mut sections = IblBakeArtifactReadbackSections::new(descriptor);
    if let Some(readback) = pmrem {
        sections = sections.with_pmrem_rgba16f_bytes(readback.into_bytes());
    }
    if let Some(readback) = sh9 {
        sections = sections.with_irradiance_sh9_bytes(readback.into_bytes());
    }
    if let Some(readback) = irradiance_cube {
        sections = sections.with_irradiance_cube_rgba16f_bytes(readback.into_bytes());
    }
    sections
}

struct BufferReadback {
    buffer: wgpu::Buffer,
}

impl BufferReadback {
    fn encode(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        source: &wgpu::Buffer,
        byte_len: u64,
        label: &'static str,
    ) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: byte_len,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_buffer_to_buffer(source, 0, &buffer, 0, byte_len);
        Self { buffer }
    }

    fn map_async(&self, sender: mpsc::Sender<Result<(), String>>) {
        self.buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = sender.send(result.map_err(|error| error.to_string()));
            });
    }

    fn into_bytes(self) -> Vec<u8> {
        let slice = self.buffer.slice(..);
        let mapped = slice.get_mapped_range();
        let bytes = mapped.to_vec();
        drop(mapped);
        self.buffer.unmap();
        bytes
    }

    fn unmap(&self) {
        self.buffer.unmap();
    }
}

struct CubeMipChainReadback {
    buffer: wgpu::Buffer,
    face_size: u32,
    mip_count: u32,
}

impl CubeMipChainReadback {
    fn encode(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        texture: &wgpu::Texture,
        face_size: u32,
        mip_count: u32,
        label: &'static str,
    ) -> Self {
        let face_size = face_size.max(1);
        let mip_count = mip_count.max(1);
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: cube_mip_staging_size_bytes(face_size, mip_count),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut staging_offset = 0;
        for face in 0..SOURCE_CUBEMAP_FACE_COUNT as u32 {
            for mip_level in 0..mip_count {
                let mip_size = source_cubemap_mip_size(face_size, mip_level);
                let unpadded_bytes_per_row = mip_size * RGBA16FLOAT_BYTES_PER_TEXEL;
                let padded_bytes_per_row = unpadded_bytes_per_row
                    .div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
                    * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
                encoder.copy_texture_to_buffer(
                    wgpu::TexelCopyTextureInfo {
                        texture,
                        mip_level,
                        origin: wgpu::Origin3d {
                            x: 0,
                            y: 0,
                            z: face,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyBufferInfo {
                        buffer: &buffer,
                        layout: wgpu::TexelCopyBufferLayout {
                            offset: staging_offset,
                            bytes_per_row: Some(padded_bytes_per_row),
                            rows_per_image: Some(mip_size),
                        },
                    },
                    wgpu::Extent3d {
                        width: mip_size,
                        height: mip_size,
                        depth_or_array_layers: 1,
                    },
                );
                staging_offset += padded_bytes_per_row as u64 * mip_size as u64;
            }
        }
        Self {
            buffer,
            face_size,
            mip_count,
        }
    }

    fn map_async(&self, sender: mpsc::Sender<Result<(), String>>) {
        self.buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = sender.send(result.map_err(|error| error.to_string()));
            });
    }

    fn into_bytes(self) -> Vec<u8> {
        let slice = self.buffer.slice(..);
        let mapped = slice.get_mapped_range();
        let bytes = strip_padded_cube_mip_chain(&mapped, self.face_size, self.mip_count);
        drop(mapped);
        self.buffer.unmap();
        bytes
    }

    fn unmap(&self) {
        self.buffer.unmap();
    }
}

fn cube_mip_staging_size_bytes(face_size: u32, mip_count: u32) -> u64 {
    let mut total = 0;
    for _face in 0..SOURCE_CUBEMAP_FACE_COUNT {
        for mip_level in 0..mip_count {
            let mip_size = source_cubemap_mip_size(face_size, mip_level);
            let unpadded_bytes_per_row = mip_size * RGBA16FLOAT_BYTES_PER_TEXEL;
            let padded_bytes_per_row = unpadded_bytes_per_row
                .div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
                * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            total += padded_bytes_per_row as u64 * mip_size as u64;
        }
    }
    total
}

fn strip_padded_cube_mip_chain(mapped: &[u8], face_size: u32, mip_count: u32) -> Vec<u8> {
    let byte_len = (0..SOURCE_CUBEMAP_FACE_COUNT)
        .flat_map(|_| 0..mip_count)
        .map(|mip_level| {
            let mip_size = source_cubemap_mip_size(face_size, mip_level) as usize;
            mip_size * mip_size * RGBA16FLOAT_BYTES_PER_TEXEL as usize
        })
        .sum();
    let mut bytes = vec![0; byte_len];
    let mut staging_offset = 0;
    let mut output_offset = 0;
    for _face in 0..SOURCE_CUBEMAP_FACE_COUNT {
        for mip_level in 0..mip_count {
            let mip_size = source_cubemap_mip_size(face_size, mip_level);
            let unpadded_bytes_per_row = mip_size * RGBA16FLOAT_BYTES_PER_TEXEL;
            let padded_bytes_per_row = unpadded_bytes_per_row
                .div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
                * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            for row in 0..mip_size as usize {
                let source_offset = staging_offset + row * padded_bytes_per_row as usize;
                let target_offset = output_offset + row * unpadded_bytes_per_row as usize;
                bytes[target_offset..target_offset + unpadded_bytes_per_row as usize]
                    .copy_from_slice(
                        &mapped[source_offset..source_offset + unpadded_bytes_per_row as usize],
                    );
            }
            staging_offset += padded_bytes_per_row as usize * mip_size as usize;
            output_offset += unpadded_bytes_per_row as usize * mip_size as usize;
        }
    }
    bytes
}

fn required_wgpu_readback_resource<'a, T>(
    resource: Option<&'a T>,
    label: &'static str,
) -> Result<&'a T, GraphicsError> {
    resource.ok_or_else(|| {
        GraphicsError::BufferMap(format!(
            "missing required IBL bake readback resource: {label}"
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::{
        required_wgpu_readback_resource, strip_padded_cube_mip_chain, SOURCE_CUBEMAP_FACE_COUNT,
    };
    use crate::core::framework::render::{
        build_source_cubemap_from_equirect, cubemap_direction_from_scaled_uv,
        cubemap_face_scaled_uv_from_direction, cubemap_scaled_uv_for_texel,
        source_cubemap_face_mip_offset, source_cubemap_mip_chain_with_bake_artifact,
        source_cubemap_mip_size, source_cubemap_pmrem_mip_from_roughness, CubemapFace,
        IblBakeArtifactContents, IblBakeArtifactDescriptor, IblBakeArtifactPayload,
        IblBakeArtifactRequest, ProceduralSkyParams, SourceCubemapIrradianceCube,
        SourceCubemapMipChain, SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
    };
    use crate::graphics::backend::RenderBackend;
    use crate::graphics::types::GraphicsError;
    use wgpu::util::DeviceExt;

    const RGBA16F_BYTES_PER_TEXEL: usize = 8;

    #[test]
    fn readback_resources_preserve_descriptor() {
        let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
        let descriptor =
            IblBakeArtifactDescriptor::current(key, 128, 8, IblBakeArtifactContents::PMREM_SH9_IEM);

        let resources = super::IblBakeArtifactWgpuReadbackResources::new(descriptor);

        assert_eq!(resources.descriptor(), descriptor);
    }

    #[test]
    fn readback_resources_report_required_wgpu_inputs_from_descriptor_contents() {
        let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
        let pmrem_sh9 =
            IblBakeArtifactDescriptor::current(key, 128, 8, IblBakeArtifactContents::PMREM_SH9);
        let pmrem_sh9_iem =
            IblBakeArtifactDescriptor::current(key, 128, 8, IblBakeArtifactContents::PMREM_SH9_IEM);

        let resources = super::IblBakeArtifactWgpuReadbackResources::new(pmrem_sh9);

        assert!(resources.requires_pmrem_texture());
        assert!(resources.requires_irradiance_sh9_buffer());
        assert!(!resources.requires_irradiance_cube_texture());

        let resources = super::IblBakeArtifactWgpuReadbackResources::new(pmrem_sh9_iem);

        assert!(resources.requires_pmrem_texture());
        assert!(resources.requires_irradiance_sh9_buffer());
        assert!(resources.requires_irradiance_cube_texture());
    }

    #[test]
    fn required_readback_resource_reports_missing_label() {
        let error = required_wgpu_readback_resource::<u32>(None, "SH9 buffer")
            .expect_err("missing resource should fail");

        assert!(matches!(
            error,
            GraphicsError::BufferMap(message) if message.contains("SH9 buffer")
        ));
    }

    #[test]
    fn batched_cube_readback_strips_padding_in_face_then_mip_order() {
        let bytes_per_face = 768;
        let mut mapped = vec![0_u8; bytes_per_face * SOURCE_CUBEMAP_FACE_COUNT as usize];
        let mut staging_offset = 0;
        let mut value = 1_u8;
        for _face in 0..SOURCE_CUBEMAP_FACE_COUNT {
            for mip_size in [2_usize, 1] {
                for row in 0..mip_size {
                    let row_offset = staging_offset + row * 256;
                    mapped[row_offset..row_offset + mip_size * 8].fill(value);
                }
                staging_offset += mip_size * 256;
                value = value.wrapping_add(1);
            }
        }

        let bytes = strip_padded_cube_mip_chain(&mapped, 2, 2);

        assert_eq!(bytes.len(), 240);
        assert_eq!(&bytes[..32], &[1_u8; 32]);
        assert_eq!(&bytes[32..40], &[2_u8; 8]);
        assert_eq!(&bytes[200..232], &[11_u8; 32]);
        assert_eq!(&bytes[232..], &[12_u8; 8]);
    }

    #[test]
    fn batched_readback_preserves_pmrem_sh9_and_iem_payload_bytes() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let key = ProceduralSkyParams::default_gradient().ibl_bake_key();
        let pmrem = build_source_cubemap_from_equirect(32, synthetic_seam_stress_environment);
        let request =
            IblBakeArtifactRequest::new(key, pmrem.source_face_size(), pmrem.source_mip_count())
                .with_required_contents(IblBakeArtifactContents::PMREM_SH9_IEM);
        let descriptor = IblBakeArtifactDescriptor::current_for_request(&request);
        let irradiance_cube = SourceCubemapIrradianceCube::new(
            SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
            vec![
                [0.125, 0.25, 0.5];
                CubemapFace::ALL.len()
                    * SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE as usize
                    * SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE as usize
            ],
        );
        let payload =
            IblBakeArtifactPayload::from_source_cubemap(descriptor, &pmrem, Some(&irradiance_cube))
                .expect("PMREM/SH9/IEM payload should encode");
        let pmrem_range = payload.pmrem_rgba16f_byte_range().expect("pmrem range");
        let sh9_range = payload.irradiance_sh9_byte_range().expect("sh9 range");
        let iem_range = payload
            .irradiance_cube_rgba16f_byte_range()
            .expect("irradiance cube range");
        let pmrem_texture = create_pmrem_texture(&backend.device, descriptor);
        upload_cube_payload_to_texture(
            &backend.queue,
            &pmrem_texture,
            descriptor.face_size(),
            descriptor.mip_count(),
            &payload.bytes()[pmrem_range],
        );
        let irradiance_cube_texture = create_cube_texture(
            &backend.device,
            "ibl-readback-irradiance-cube-texture",
            SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
            1,
        );
        upload_cube_payload_to_texture(
            &backend.queue,
            &irradiance_cube_texture,
            SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
            1,
            &payload.bytes()[iem_range],
        );
        let sh9_buffer = backend
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ibl-readback-seam-sh9-buffer"),
                contents: &payload.bytes()[sh9_range],
                usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
            });

        let sections = super::read_ibl_bake_artifact_wgpu_sections(
            &backend.device,
            &backend.queue,
            super::IblBakeArtifactWgpuReadbackResources::new(descriptor)
                .with_pmrem_texture(&pmrem_texture)
                .with_irradiance_sh9_buffer(&sh9_buffer)
                .with_irradiance_cube_texture(&irradiance_cube_texture),
        )
        .expect("WGPU PMREM/SH9/IEM batch readback should produce artifact sections");
        let readback_payload = sections
            .into_payload()
            .expect("readback sections should assemble into a current payload");
        assert_eq!(readback_payload.bytes(), payload.bytes());

        let applied = source_cubemap_mip_chain_with_bake_artifact(&pmrem, &readback_payload)
            .expect("readback payload should apply to the matching source cubemap");
        let mid_mip =
            source_cubemap_pmrem_mip_from_roughness(0.5, applied.pmrem_mip_count()).round() as u32;
        let rough_mip =
            source_cubemap_pmrem_mip_from_roughness(1.0, applied.pmrem_mip_count()).round() as u32;
        let expected_mid = pmrem_seam_luma_stats(&pmrem, mid_mip);
        let expected_rough = pmrem_seam_luma_stats(&pmrem, rough_mip);
        let applied_base = pmrem_seam_luma_stats(&applied, 0);
        let applied_mid = pmrem_seam_luma_stats(&applied, mid_mip);
        let applied_rough = pmrem_seam_luma_stats(&applied, rough_mip);

        assert_stats_close(expected_mid, applied_mid, 0.003);
        assert_stats_close(expected_rough, applied_rough, 0.003);
        assert!(
            applied_mid.mean < applied_base.mean * 0.9,
            "WGPU-readback PMREM mid mip should still reduce seam energy, base={applied_base:?} mid={applied_mid:?} rough={applied_rough:?}"
        );
        assert!(
            applied_rough.max < applied_base.max * 0.75,
            "WGPU-readback PMREM rough mip should reduce worst seam delta, base={applied_base:?} mid={applied_mid:?} rough={applied_rough:?}"
        );
    }

    fn create_pmrem_texture(
        device: &wgpu::Device,
        descriptor: IblBakeArtifactDescriptor,
    ) -> wgpu::Texture {
        create_cube_texture(
            device,
            "ibl-readback-seam-pmrem-texture",
            descriptor.face_size(),
            descriptor.mip_count(),
        )
    }

    fn create_cube_texture(
        device: &wgpu::Device,
        label: &'static str,
        face_size: u32,
        mip_count: u32,
    ) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: face_size,
                height: face_size,
                depth_or_array_layers: 6,
            },
            mip_level_count: mip_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        })
    }

    fn upload_cube_payload_to_texture(
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        face_size: u32,
        mip_count: u32,
        cube_bytes: &[u8],
    ) {
        for face in CubemapFace::ALL {
            for mip_level in 0..mip_count {
                let mip_size = source_cubemap_mip_size(face_size, mip_level);
                let unpadded_bytes_per_row = mip_size as usize * RGBA16F_BYTES_PER_TEXEL;
                let padded_bytes_per_row = unpadded_bytes_per_row
                    .next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize);
                let mut padded = vec![0; padded_bytes_per_row * mip_size as usize];
                let source_offset =
                    source_cubemap_face_mip_offset(face_size, mip_count, face, mip_level)
                        * RGBA16F_BYTES_PER_TEXEL;
                for row in 0..mip_size as usize {
                    let source_row =
                        source_offset + row * mip_size as usize * RGBA16F_BYTES_PER_TEXEL;
                    let target_row = row * padded_bytes_per_row;
                    padded[target_row..target_row + unpadded_bytes_per_row].copy_from_slice(
                        &cube_bytes[source_row..source_row + unpadded_bytes_per_row],
                    );
                }
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture,
                        mip_level,
                        origin: wgpu::Origin3d {
                            x: 0,
                            y: 0,
                            z: face.index() as u32,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &padded,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(padded_bytes_per_row as u32),
                        rows_per_image: Some(mip_size),
                    },
                    wgpu::Extent3d {
                        width: mip_size,
                        height: mip_size,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }
    }

    fn synthetic_seam_stress_environment(u: f32, v: f32) -> [f32; 4] {
        let wave_a = (std::f32::consts::TAU * u * 17.0).sin();
        let wave_b = (std::f32::consts::TAU * (u * 11.0 + v * 7.0)).cos();
        let wave_c = (std::f32::consts::PI * v * 9.0).sin();
        let luma = 0.55 + wave_a * 0.22 + wave_b * 0.16 + wave_c * 0.12;
        [luma, luma * 0.85, luma * 0.7, 1.0]
    }

    #[derive(Clone, Copy, Debug)]
    struct SeamLumaStats {
        mean: f32,
        max: f32,
    }

    fn pmrem_seam_luma_stats(cubemap: &SourceCubemapMipChain, mip_level: u32) -> SeamLumaStats {
        let mip_size = source_cubemap_mip_size(cubemap.pmrem_face_size(), mip_level);
        let mut sum = 0.0;
        let mut max = 0.0_f32;
        let mut count = 0.0;

        for face in CubemapFace::ALL {
            for side in CubeEdgeSide::ALL {
                let sample_start = if mip_size > 2 { 1 } else { 0 };
                let sample_end = if mip_size > 2 {
                    mip_size.saturating_sub(1)
                } else {
                    mip_size
                };
                for index in sample_start..sample_end {
                    let (x, y) = side.edge_texel(index, mip_size);
                    let current = cubemap.pmrem_texel(face, mip_level, x, y);
                    let (neighbor_face, neighbor_x, neighbor_y) =
                        side.neighbor_texel(face, index, mip_size);
                    let neighbor =
                        cubemap.pmrem_texel(neighbor_face, mip_level, neighbor_x, neighbor_y);
                    let delta = (luma(current) - luma(neighbor)).abs();
                    sum += delta;
                    max = max.max(delta);
                    count += 1.0;
                }
            }
        }

        SeamLumaStats {
            mean: sum / count,
            max,
        }
    }

    #[derive(Clone, Copy, Debug)]
    enum CubeEdgeSide {
        Left,
        Right,
        Top,
        Bottom,
    }

    impl CubeEdgeSide {
        const ALL: [Self; 4] = [Self::Left, Self::Right, Self::Top, Self::Bottom];

        fn edge_texel(self, index: u32, size: u32) -> (u32, u32) {
            match self {
                Self::Left => (0, index),
                Self::Right => (size.saturating_sub(1), index),
                Self::Top => (index, 0),
                Self::Bottom => (index, size.saturating_sub(1)),
            }
        }

        fn neighbor_texel(
            self,
            face: CubemapFace,
            index: u32,
            size: u32,
        ) -> (CubemapFace, u32, u32) {
            let edge_uv = match self {
                Self::Left => [
                    -1.0 - 1.0 / size as f32,
                    cubemap_scaled_uv_for_texel(0, index, size)[1],
                ],
                Self::Right => [
                    1.0 + 1.0 / size as f32,
                    cubemap_scaled_uv_for_texel(size.saturating_sub(1), index, size)[1],
                ],
                Self::Top => [
                    cubemap_scaled_uv_for_texel(index, 0, size)[0],
                    -1.0 - 1.0 / size as f32,
                ],
                Self::Bottom => [
                    cubemap_scaled_uv_for_texel(index, size.saturating_sub(1), size)[0],
                    1.0 + 1.0 / size as f32,
                ],
            };
            let direction = cubemap_direction_from_scaled_uv(face, edge_uv);
            let (neighbor_face, neighbor_uv) = cubemap_face_scaled_uv_from_direction(direction);
            (
                neighbor_face,
                texel_coord_from_scaled_axis(neighbor_uv[0], size),
                texel_coord_from_scaled_axis(neighbor_uv[1], size),
            )
        }
    }

    fn texel_coord_from_scaled_axis(scaled_axis: f32, size: u32) -> u32 {
        (((scaled_axis * 0.5 + 0.5) * size as f32 - 0.5).round() as i32)
            .clamp(0, size.saturating_sub(1) as i32) as u32
    }

    fn luma(texel: [f32; 4]) -> f32 {
        0.2126 * texel[0] + 0.7152 * texel[1] + 0.0722 * texel[2]
    }

    fn assert_stats_close(expected: SeamLumaStats, actual: SeamLumaStats, tolerance: f32) {
        assert!(
            (expected.mean - actual.mean).abs() <= tolerance,
            "mean seam delta changed across WGPU readback: expected={expected:?} actual={actual:?}"
        );
        assert!(
            (expected.max - actual.max).abs() <= tolerance,
            "max seam delta changed across WGPU readback: expected={expected:?} actual={actual:?}"
        );
    }
}
