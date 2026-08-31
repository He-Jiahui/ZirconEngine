use std::sync::mpsc;

use crate::core::framework::render::{
    IblBakeArtifactDescriptor, IblBakeArtifactReadbackSections,
    SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
};
use crate::graphics::debug_markers::{RENDERDOC_MARKER_READBACK, insert_marker};
use crate::graphics::types::GraphicsError;

use super::staging::{BufferReadback, CubeMipChainReadback};

/// Keeps a cache-miss artifact readback behind one GPU submission and one device wait.
/// The artifact payload remains sectioned so its on-disk format and cache contract do not change.
pub(super) struct IblBakeArtifactWgpuReadbackBatch {
    encoder: wgpu::CommandEncoder,
    pmrem: Option<CubeMipChainReadback>,
    sh9: Option<BufferReadback>,
    irradiance_cube: Option<CubeMipChainReadback>,
}

impl IblBakeArtifactWgpuReadbackBatch {
    pub(super) fn new(device: &wgpu::Device) -> Self {
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

    pub(super) fn add_pmrem(
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

    pub(super) fn add_sh9(
        &mut self,
        device: &wgpu::Device,
        source: &super::resources::IblBakeArtifactWgpuBufferReadback<'_>,
    ) {
        self.sh9 = Some(BufferReadback::encode(
            device,
            &mut self.encoder,
            source.buffer,
            source.offset,
            source.size,
            "zircon-readback-ibl-sh9",
        ));
    }

    pub(super) fn add_irradiance_cube(&mut self, device: &wgpu::Device, texture: &wgpu::Texture) {
        self.irradiance_cube = Some(CubeMipChainReadback::encode(
            device,
            &mut self.encoder,
            texture,
            SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
            1,
            "zircon-readback-ibl-irradiance-cube",
        ));
    }

    pub(super) fn finish(
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
