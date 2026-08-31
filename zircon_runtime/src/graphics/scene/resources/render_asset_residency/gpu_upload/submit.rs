use std::ops::Range;
use std::sync::Arc;

use thiserror::Error;

use crate::asset::artifact::{RENDER_ARTIFACT_STATIC_MESH_FORMAT_V1, RenderArtifactMeshBounds};
use crate::asset::assets::{
    LIGHTMAP_RGBA16F_GPU_FORMAT, RGBA8_UNORM_FORMAT, RGBA8_UNORM_SRGB_FORMAT,
};
use crate::graphics::scene::resources::render_asset_residency::{
    RenderAssetDeviceEpoch, RenderAssetResidencyTicket,
};
use zr_rhi::{
    BufferDesc, BufferHandle, BufferUpload, BufferUploadBatch, BufferUsage, RenderDevice, RhiError,
    SubmissionStatus, SubmissionTicket, TextureDesc, TextureDimension, TextureFormat,
    TextureHandle, TextureUpload, TextureUploadBatch, TextureUsage, TextureViewDesc,
    TextureViewDimension, TextureViewHandle,
};

use super::contract::RenderAssetGpuUploadQuote;
use super::plan::{
    PreparedMeshUpload, PreparedRenderAssetGpuUpload, PreparedTextureUpload,
    RenderAssetGpuUploadPlan,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RenderAssetGpuArtifactKind {
    Texture,
    Mesh,
}

pub(crate) enum RenderAssetGpuArtifact {
    Texture(RenderAssetGpuTextureArtifact),
    Mesh(RenderAssetGpuMeshArtifact),
}

impl RenderAssetGpuArtifact {
    pub(crate) const fn kind(&self) -> RenderAssetGpuArtifactKind {
        match self {
            Self::Texture(_) => RenderAssetGpuArtifactKind::Texture,
            Self::Mesh(_) => RenderAssetGpuArtifactKind::Mesh,
        }
    }

    pub(crate) const fn allocation_bytes(&self) -> u64 {
        match self {
            Self::Texture(texture) => texture.allocation_bytes(),
            Self::Mesh(mesh) => mesh.allocation_bytes(),
        }
    }

    pub(crate) const fn texture(&self) -> Option<&RenderAssetGpuTextureArtifact> {
        match self {
            Self::Texture(texture) => Some(texture),
            Self::Mesh(_) => None,
        }
    }

    pub(crate) const fn mesh(&self) -> Option<&RenderAssetGpuMeshArtifact> {
        match self {
            Self::Texture(_) => None,
            Self::Mesh(mesh) => Some(mesh),
        }
    }

    pub(crate) fn retire(mut self, device: &dyn RenderDevice) -> Result<(), (Self, RhiError)> {
        let result = match &mut self {
            Self::Texture(texture) => texture.retire(device),
            Self::Mesh(mesh) => mesh.retire(device),
        };
        result.map_err(|error| (self, error))
    }
}

pub(crate) struct RenderAssetGpuTextureArtifact {
    texture: TextureHandle,
    view: TextureViewHandle,
    format: TextureFormat,
    source_mips: Range<u32>,
    layer_count: u32,
    allocation_bytes: u64,
    retirement_progress: u8,
}

impl RenderAssetGpuTextureArtifact {
    pub(crate) const fn texture(&self) -> TextureHandle {
        self.texture
    }

    pub(crate) const fn view(&self) -> TextureViewHandle {
        self.view
    }

    pub(crate) const fn format(&self) -> TextureFormat {
        self.format
    }

    pub(crate) fn source_mips(&self) -> Range<u32> {
        self.source_mips.clone()
    }

    pub(crate) const fn layer_count(&self) -> u32 {
        self.layer_count
    }

    pub(crate) const fn allocation_bytes(&self) -> u64 {
        self.allocation_bytes
    }

    fn retire(&mut self, device: &dyn RenderDevice) -> Result<(), RhiError> {
        const VIEW_RETIRED: u8 = 1 << 0;
        const TEXTURE_RETIRED: u8 = 1 << 1;

        if self.retirement_progress & VIEW_RETIRED == 0 {
            device.destroy_texture_view(self.view)?;
            self.retirement_progress |= VIEW_RETIRED;
        }
        if self.retirement_progress & TEXTURE_RETIRED == 0 {
            device.destroy_texture(self.texture)?;
            self.retirement_progress |= TEXTURE_RETIRED;
        }
        Ok(())
    }
}

pub(crate) struct RenderAssetGpuMeshArtifact {
    vertex_buffer: BufferHandle,
    index_buffer: BufferHandle,
    lods: Arc<[RenderAssetGpuMeshLod]>,
    allocation_bytes: u64,
    retirement_progress: u8,
}

impl RenderAssetGpuMeshArtifact {
    pub(crate) const fn vertex_buffer(&self) -> BufferHandle {
        self.vertex_buffer
    }

    pub(crate) const fn index_buffer(&self) -> BufferHandle {
        self.index_buffer
    }

    pub(crate) fn lods(&self) -> &[RenderAssetGpuMeshLod] {
        self.lods.as_ref()
    }

    pub(crate) const fn allocation_bytes(&self) -> u64 {
        self.allocation_bytes
    }

    fn retire(&mut self, device: &dyn RenderDevice) -> Result<(), RhiError> {
        const INDEX_RETIRED: u8 = 1 << 0;
        const VERTEX_RETIRED: u8 = 1 << 1;

        let mut first_error = None;
        if self.retirement_progress & INDEX_RETIRED == 0 {
            match device.destroy_buffer(self.index_buffer) {
                Ok(()) => self.retirement_progress |= INDEX_RETIRED,
                Err(error) => first_error = Some(error),
            }
        }
        if self.retirement_progress & VERTEX_RETIRED == 0 {
            match device.destroy_buffer(self.vertex_buffer) {
                Ok(()) => self.retirement_progress |= VERTEX_RETIRED,
                Err(error) => {
                    first_error.get_or_insert(error);
                }
            }
        }
        match first_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RenderAssetGpuMeshLod {
    lod: u16,
    vertex_count: u32,
    index_count: u32,
    vertex_offset: u64,
    index_offset: u64,
    bounds: RenderArtifactMeshBounds,
}

impl RenderAssetGpuMeshLod {
    pub(crate) const fn lod(self) -> u16 {
        self.lod
    }

    pub(crate) const fn vertex_count(self) -> u32 {
        self.vertex_count
    }

    pub(crate) const fn index_count(self) -> u32 {
        self.index_count
    }

    pub(crate) const fn vertex_offset(self) -> u64 {
        self.vertex_offset
    }

    pub(crate) const fn index_offset(self) -> u64 {
        self.index_offset
    }

    pub(crate) const fn bounds(self) -> RenderArtifactMeshBounds {
        self.bounds
    }
}

pub(crate) struct RenderAssetGpuUploadLease {
    cpu_lease: super::super::RenderAssetCpuArtifactLease,
    artifact: RenderAssetGpuArtifact,
    submission: SubmissionTicket,
    quote: RenderAssetGpuUploadQuote,
}

impl RenderAssetGpuUploadLease {
    pub(crate) const fn ticket(&self) -> RenderAssetResidencyTicket {
        self.cpu_lease.ticket()
    }

    pub(crate) const fn submission(&self) -> SubmissionTicket {
        self.submission
    }

    pub(crate) const fn quote(&self) -> RenderAssetGpuUploadQuote {
        self.quote
    }

    pub(crate) const fn artifact(&self) -> &RenderAssetGpuArtifact {
        &self.artifact
    }

    pub(in crate::graphics::scene::resources::render_asset_residency) fn finalize(
        self,
        status: SubmissionStatus,
    ) -> RenderAssetGpuUploadFinalize {
        match status {
            SubmissionStatus::Accepted | SubmissionStatus::Submitted => {
                RenderAssetGpuUploadFinalize::Pending(self)
            }
            SubmissionStatus::Completed => {
                let (artifact, submission, quote) = self.into_terminal_parts();
                RenderAssetGpuUploadFinalize::Resident {
                    artifact,
                    submission,
                    quote,
                }
            }
            SubmissionStatus::Failed
            | SubmissionStatus::Cancelled
            | SubmissionStatus::DeviceLost => {
                let (artifact, submission, quote) = self.into_terminal_parts();
                RenderAssetGpuUploadFinalize::Failed {
                    artifact,
                    submission,
                    quote,
                    status,
                }
            }
        }
    }

    fn into_terminal_parts(
        self,
    ) -> (
        RenderAssetGpuArtifact,
        SubmissionTicket,
        RenderAssetGpuUploadQuote,
    ) {
        let Self {
            cpu_lease,
            artifact,
            submission,
            quote,
        } = self;
        drop(cpu_lease);
        (artifact, submission, quote)
    }
}

pub(in crate::graphics::scene::resources::render_asset_residency) enum RenderAssetGpuUploadFinalize
{
    Pending(RenderAssetGpuUploadLease),
    Resident {
        artifact: RenderAssetGpuArtifact,
        submission: SubmissionTicket,
        quote: RenderAssetGpuUploadQuote,
    },
    Failed {
        artifact: RenderAssetGpuArtifact,
        submission: SubmissionTicket,
        quote: RenderAssetGpuUploadQuote,
        status: SubmissionStatus,
    },
}

#[derive(Debug, Error)]
pub(crate) enum RenderAssetGpuUploadSubmitError {
    #[error("render upload ticket targets device {expected:?}, but backend owns {actual:?}")]
    DeviceMismatch {
        expected: RenderAssetDeviceEpoch,
        actual: RenderAssetDeviceEpoch,
    },
    #[error("semantic texture format `{format}` has no unambiguous MVP WGPU mapping")]
    UnsupportedTextureFormat { format: Arc<str> },
    #[error("semantic mesh format `{format}` has no WGPU upload mapping")]
    UnsupportedMeshFormat { format: Arc<str> },
    #[error(transparent)]
    Rhi(#[from] RhiError),
    #[error(
        "render asset GPU upload failed: {operation}; resource rollback also failed: {cleanup}"
    )]
    ResourceRollback {
        operation: RhiError,
        cleanup: RhiError,
    },
}

impl RenderAssetGpuUploadPlan {
    pub(crate) fn submit(
        self,
        device: &dyn RenderDevice,
    ) -> Result<RenderAssetGpuUploadLease, RenderAssetGpuUploadSubmitError> {
        let expected = self.ticket().device();
        let actual = RenderAssetDeviceEpoch::new(device.device_id(), device.generation());
        if expected != actual {
            return Err(RenderAssetGpuUploadSubmitError::DeviceMismatch { expected, actual });
        }
        let (cpu_lease, prepared, quote) = self.into_parts();
        let (artifact, submission) = match prepared {
            PreparedRenderAssetGpuUpload::Texture(upload) => {
                submit_texture_upload(device, upload, quote.destination_bytes())?
            }
            PreparedRenderAssetGpuUpload::Mesh(upload) => {
                submit_mesh_upload(device, upload, quote.destination_bytes())?
            }
        };
        Ok(RenderAssetGpuUploadLease {
            cpu_lease,
            artifact,
            submission,
            quote,
        })
    }
}

fn submit_texture_upload(
    device: &dyn RenderDevice,
    upload: PreparedTextureUpload,
    allocation_bytes: u64,
) -> Result<(RenderAssetGpuArtifact, SubmissionTicket), RenderAssetGpuUploadSubmitError> {
    let format = texture_format(upload.platform_format.as_ref()).ok_or_else(|| {
        RenderAssetGpuUploadSubmitError::UnsupportedTextureFormat {
            format: Arc::clone(&upload.platform_format),
        }
    })?;
    let mip_level_count = upload.source_mips.end - upload.source_mips.start;
    let dimension = if upload.layer_count == 1 {
        TextureDimension::D2
    } else {
        TextureDimension::D2Array
    };
    let view_dimension = if upload.layer_count == 1 {
        TextureViewDimension::D2
    } else {
        TextureViewDimension::D2Array
    };
    let texture = device.create_texture(
        &TextureDesc::new(
            "zircon-render-asset-semantic-texture",
            upload.width,
            upload.height,
            format,
            TextureUsage::SAMPLED | TextureUsage::COPY_DST | TextureUsage::COPY_SRC,
        )
        .with_dimension(dimension)
        .with_array_layers(upload.layer_count)
        .with_mip_levels(mip_level_count),
    )?;
    let view = match device.create_texture_view(&TextureViewDesc::new(
        "zircon-render-asset-semantic-texture-view",
        texture,
        view_dimension,
    )) {
        Ok(view) => view,
        Err(operation) => {
            return Err(rollback_texture(device, None, texture, operation));
        }
    };
    let mut batch = TextureUploadBatch::new();
    for subresource in upload.uploads {
        batch.push(TextureUpload::from_payload(
            texture,
            subresource.region,
            u64::from(subresource.bytes_per_row),
            subresource.bytes,
        ));
    }
    let submission = match device.write_texture_batch(batch) {
        Ok(submission) => submission,
        Err(operation) => {
            return Err(rollback_texture(device, Some(view), texture, operation));
        }
    };
    Ok((
        RenderAssetGpuArtifact::Texture(RenderAssetGpuTextureArtifact {
            texture,
            view,
            format,
            source_mips: upload.source_mips,
            layer_count: upload.layer_count,
            allocation_bytes,
            retirement_progress: 0,
        }),
        submission,
    ))
}

fn submit_mesh_upload(
    device: &dyn RenderDevice,
    upload: PreparedMeshUpload,
    allocation_bytes: u64,
) -> Result<(RenderAssetGpuArtifact, SubmissionTicket), RenderAssetGpuUploadSubmitError> {
    if upload.platform_format.as_ref() != RENDER_ARTIFACT_STATIC_MESH_FORMAT_V1 {
        return Err(RenderAssetGpuUploadSubmitError::UnsupportedMeshFormat {
            format: upload.platform_format,
        });
    }
    let vertex_buffer = device.create_buffer(&BufferDesc::new(
        "zircon-render-asset-mesh-packed-vertex-buffer",
        upload.vertex_buffer_bytes,
        BufferUsage::VERTEX | BufferUsage::COPY_DST | BufferUsage::COPY_SRC,
    ))?;
    let index_buffer = match device.create_buffer(&BufferDesc::new(
        "zircon-render-asset-mesh-packed-index-buffer",
        upload.index_buffer_bytes,
        BufferUsage::INDEX | BufferUsage::COPY_DST | BufferUsage::COPY_SRC,
    )) {
        Ok(index_buffer) => index_buffer,
        Err(operation) => {
            return Err(rollback_mesh(device, vertex_buffer, None, operation));
        }
    };
    let mut batch = BufferUploadBatch::new();
    let mut resident_lods = Vec::with_capacity(upload.lods.len());
    for lod in upload.lods {
        let vertex_source_range = lod.vertex_source_range;
        let vertex = BufferUpload::new(
            vertex_buffer,
            lod.vertex_destination_offset,
            Arc::clone(&lod.bytes),
            vertex_source_range.clone(),
        );
        let Some(vertex) = vertex else {
            let operation = invalid_source_range(&vertex_source_range, lod.bytes.len());
            return Err(rollback_mesh(
                device,
                vertex_buffer,
                Some(index_buffer),
                operation,
            ));
        };
        let index_source_range = lod.index_source_range;
        let index = BufferUpload::new(
            index_buffer,
            lod.index_destination_offset,
            lod.bytes,
            index_source_range.clone(),
        );
        let Some(index) = index else {
            let operation = invalid_source_range(&index_source_range, vertex.payload_owner().len());
            return Err(rollback_mesh(
                device,
                vertex_buffer,
                Some(index_buffer),
                operation,
            ));
        };
        batch.push(vertex);
        batch.push(index);
        resident_lods.push(RenderAssetGpuMeshLod {
            lod: lod.lod,
            vertex_count: lod.vertex_count,
            index_count: lod.index_count,
            vertex_offset: lod.vertex_destination_offset,
            index_offset: lod.index_destination_offset,
            bounds: lod.bounds,
        });
    }
    let submission = match device.write_buffer_batch(batch) {
        Ok(submission) => submission,
        Err(operation) => {
            return Err(rollback_mesh(
                device,
                vertex_buffer,
                Some(index_buffer),
                operation,
            ));
        }
    };
    Ok((
        RenderAssetGpuArtifact::Mesh(RenderAssetGpuMeshArtifact {
            vertex_buffer,
            index_buffer,
            lods: resident_lods.into(),
            allocation_bytes,
            retirement_progress: 0,
        }),
        submission,
    ))
}

fn texture_format(format: &str) -> Option<TextureFormat> {
    match format.trim() {
        RGBA8_UNORM_FORMAT => Some(TextureFormat::Rgba8Unorm),
        RGBA8_UNORM_SRGB_FORMAT => Some(TextureFormat::Rgba8UnormSrgb),
        LIGHTMAP_RGBA16F_GPU_FORMAT => Some(TextureFormat::Rgba16Float),
        _ => None,
    }
}

fn rollback_texture(
    device: &dyn RenderDevice,
    view: Option<TextureViewHandle>,
    texture: TextureHandle,
    operation: RhiError,
) -> RenderAssetGpuUploadSubmitError {
    let mut cleanup_error = None;
    if let Some(view) = view {
        if let Err(error) = device.destroy_texture_view(view) {
            cleanup_error = Some(error);
        }
    }
    if let Err(error) = device.destroy_texture(texture) {
        cleanup_error.get_or_insert(error);
    }
    match cleanup_error {
        Some(cleanup) => RenderAssetGpuUploadSubmitError::ResourceRollback { operation, cleanup },
        None => RenderAssetGpuUploadSubmitError::Rhi(operation),
    }
}

fn rollback_mesh(
    device: &dyn RenderDevice,
    vertex_buffer: BufferHandle,
    index_buffer: Option<BufferHandle>,
    operation: RhiError,
) -> RenderAssetGpuUploadSubmitError {
    let mut cleanup_error = None;
    if let Some(index_buffer) = index_buffer {
        if let Err(error) = device.destroy_buffer(index_buffer) {
            cleanup_error = Some(error);
        }
    }
    if let Err(error) = device.destroy_buffer(vertex_buffer) {
        cleanup_error.get_or_insert(error);
    }
    match cleanup_error {
        Some(cleanup) => RenderAssetGpuUploadSubmitError::ResourceRollback { operation, cleanup },
        None => RenderAssetGpuUploadSubmitError::Rhi(operation),
    }
}

fn invalid_source_range(range: &Range<usize>, payload_bytes: usize) -> RhiError {
    RhiError::InvalidUploadSourceRange {
        start: range.start,
        end: range.end,
        payload_bytes,
    }
}
