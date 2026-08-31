use crate::core::framework::render::{IblBakeArtifactContents, IblBakeArtifactDescriptor};
use crate::graphics::types::GraphicsError;

#[derive(Clone, Copy)]
pub(crate) struct IblBakeArtifactWgpuReadbackResources<'a> {
    descriptor: IblBakeArtifactDescriptor,
    pmrem_texture: Option<&'a wgpu::Texture>,
    irradiance_sh9_buffer: Option<IblBakeArtifactWgpuBufferReadback<'a>>,
    irradiance_cube_texture: Option<&'a wgpu::Texture>,
}

#[derive(Clone, Copy)]
pub(crate) struct IblBakeArtifactWgpuBufferReadback<'a> {
    pub(crate) buffer: &'a wgpu::Buffer,
    pub(crate) offset: u64,
    pub(crate) size: u64,
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
        self.irradiance_sh9_buffer = Some(IblBakeArtifactWgpuBufferReadback {
            buffer,
            offset: 0,
            size: crate::core::framework::render::IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES as u64,
        });
        self
    }

    pub fn with_irradiance_sh9_buffer_range(
        mut self,
        buffer: &'a wgpu::Buffer,
        offset: u64,
        size: u64,
    ) -> Self {
        self.irradiance_sh9_buffer = Some(IblBakeArtifactWgpuBufferReadback {
            buffer,
            offset,
            size,
        });
        self
    }

    pub fn with_irradiance_cube_texture(mut self, texture: &'a wgpu::Texture) -> Self {
        self.irradiance_cube_texture = Some(texture);
        self
    }

    pub(super) const fn pmrem_texture(&self) -> Option<&'a wgpu::Texture> {
        self.pmrem_texture
    }

    pub(super) const fn irradiance_sh9_buffer(
        &self,
    ) -> Option<IblBakeArtifactWgpuBufferReadback<'a>> {
        self.irradiance_sh9_buffer
    }

    pub(super) const fn irradiance_cube_texture(&self) -> Option<&'a wgpu::Texture> {
        self.irradiance_cube_texture
    }
}

pub(super) fn required_wgpu_readback_resource<'a, T>(
    resource: Option<&'a T>,
    label: &'static str,
) -> Result<&'a T, GraphicsError> {
    resource.ok_or_else(|| {
        GraphicsError::BufferMap(format!(
            "missing required IBL bake readback resource: {label}"
        ))
    })
}

pub(super) fn required_irradiance_sh9_readback_resource<'a>(
    resources: &IblBakeArtifactWgpuReadbackResources<'a>,
) -> Result<IblBakeArtifactWgpuBufferReadback<'a>, GraphicsError> {
    let source = resources.irradiance_sh9_buffer().ok_or_else(|| {
        GraphicsError::BufferMap(
            "missing required IBL bake readback resource: SH9 buffer".to_string(),
        )
    })?;
    let expected_size = resources
        .descriptor()
        .expected_irradiance_sh9_size_bytes()
        .ok_or_else(|| {
            GraphicsError::BufferMap(
                "IBL bake SH9 readback has no descriptor byte length".to_string(),
            )
        })? as u64;
    if source.size != expected_size {
        return Err(GraphicsError::BufferMap(format!(
            "IBL bake SH9 readback window is {} bytes, expected {expected_size}",
            source.size
        )));
    }
    let end = source.offset.checked_add(source.size).ok_or_else(|| {
        GraphicsError::BufferMap("IBL bake SH9 readback window overflows u64".to_string())
    })?;
    if end > source.buffer.size() {
        return Err(GraphicsError::BufferMap(format!(
            "IBL bake SH9 readback window [{}..{}) exceeds physical buffer size {}",
            source.offset,
            end,
            source.buffer.size()
        )));
    }
    Ok(source)
}
