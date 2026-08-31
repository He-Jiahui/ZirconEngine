use std::collections::{BTreeMap, HashSet};
use std::ops::Range;
use std::sync::Arc;

use crate::asset::artifact::{
    RenderArtifactBlockDescriptor, RenderArtifactLayout, RenderArtifactManifest,
    RenderArtifactMeshBounds, RenderArtifactMeshLodUploadLayout, RenderArtifactTextureLayout,
    RenderArtifactTextureSubresourceLayout, RenderSubresourceId,
};
use crate::graphics::scene::resources::render_asset_residency::RenderAssetCpuArtifactLease;
use zr_rhi::TextureCopyRegion;

use super::contract::{
    RenderAssetGpuUploadBudgetClass, RenderAssetGpuUploadLimits, RenderAssetGpuUploadPlanError,
    RenderAssetGpuUploadQuote,
};

const BUFFER_COPY_ALIGNMENT: u64 = 4;

#[derive(Clone)]
pub(super) struct UploadSourceBlock {
    descriptor: RenderArtifactBlockDescriptor,
    bytes: Arc<[u8]>,
}

impl UploadSourceBlock {
    pub(super) fn new(descriptor: RenderArtifactBlockDescriptor, bytes: Arc<[u8]>) -> Self {
        Self { descriptor, bytes }
    }
}

pub(crate) struct RenderAssetGpuUploadPlan {
    cpu_lease: RenderAssetCpuArtifactLease,
    prepared: PreparedRenderAssetGpuUpload,
    quote: RenderAssetGpuUploadQuote,
}

impl RenderAssetGpuUploadPlan {
    pub(crate) fn prepare(
        cpu_lease: RenderAssetCpuArtifactLease,
        limits: RenderAssetGpuUploadLimits,
    ) -> Result<Self, RenderAssetGpuUploadPlanError> {
        let blocks = cpu_lease
            .blocks()
            .iter()
            .map(|block| {
                UploadSourceBlock::new(block.descriptor().clone(), Arc::clone(block.bytes()))
            })
            .collect::<Vec<_>>();
        let (prepared, quote) = prepare_upload(cpu_lease.manifest(), blocks, limits)?;
        Ok(Self {
            cpu_lease,
            prepared,
            quote,
        })
    }

    pub(crate) const fn ticket(&self) -> super::super::RenderAssetResidencyTicket {
        self.cpu_lease.ticket()
    }

    pub(crate) const fn quote(&self) -> RenderAssetGpuUploadQuote {
        self.quote
    }

    pub(crate) const fn kind(&self) -> RenderAssetGpuUploadPlanKind {
        match &self.prepared {
            PreparedRenderAssetGpuUpload::Texture(_) => RenderAssetGpuUploadPlanKind::Texture,
            PreparedRenderAssetGpuUpload::Mesh(_) => RenderAssetGpuUploadPlanKind::Mesh,
        }
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        RenderAssetCpuArtifactLease,
        PreparedRenderAssetGpuUpload,
        RenderAssetGpuUploadQuote,
    ) {
        (self.cpu_lease, self.prepared, self.quote)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RenderAssetGpuUploadPlanKind {
    Texture,
    Mesh,
}

pub(super) enum PreparedRenderAssetGpuUpload {
    Texture(PreparedTextureUpload),
    Mesh(PreparedMeshUpload),
}

pub(super) struct PreparedTextureUpload {
    pub(super) platform_format: Arc<str>,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) layer_count: u32,
    pub(super) source_mips: Range<u32>,
    pub(super) uploads: Vec<PreparedTextureSubresourceUpload>,
}

pub(super) struct PreparedTextureSubresourceUpload {
    pub(super) region: TextureCopyRegion,
    pub(super) bytes_per_row: u32,
    pub(super) bytes: Arc<[u8]>,
}

pub(super) struct PreparedMeshUpload {
    pub(super) platform_format: Arc<str>,
    pub(super) vertex_buffer_bytes: u64,
    pub(super) index_buffer_bytes: u64,
    pub(super) lods: Vec<PreparedMeshLodUpload>,
}

pub(super) struct PreparedMeshLodUpload {
    pub(super) lod: u16,
    pub(super) vertex_count: u32,
    pub(super) index_count: u32,
    pub(super) vertex_destination_offset: u64,
    pub(super) index_destination_offset: u64,
    pub(super) vertex_source_range: Range<usize>,
    pub(super) index_source_range: Range<usize>,
    pub(super) bounds: RenderArtifactMeshBounds,
    pub(super) bytes: Arc<[u8]>,
}

pub(super) fn prepare_upload(
    manifest: &RenderArtifactManifest,
    blocks: Vec<UploadSourceBlock>,
    limits: RenderAssetGpuUploadLimits,
) -> Result<(PreparedRenderAssetGpuUpload, RenderAssetGpuUploadQuote), RenderAssetGpuUploadPlanError>
{
    validate_source_blocks(manifest, &blocks)?;
    let subresource_count = blocks.len();
    ensure_budget(
        RenderAssetGpuUploadBudgetClass::Subresources,
        subresource_count as u64,
        limits.max_subresources() as u64,
    )?;
    let staging_bytes = blocks.iter().try_fold(0_u64, |total, block| {
        total
            .checked_add(block.descriptor.decoded_bytes())
            .ok_or(RenderAssetGpuUploadPlanError::ByteTotalOverflow)
    })?;
    ensure_budget(
        RenderAssetGpuUploadBudgetClass::Staging,
        staging_bytes,
        limits.max_staging_bytes(),
    )?;
    let (prepared, destination_bytes) = match manifest.layout() {
        RenderArtifactLayout::Texture { layout } => {
            let (upload, bytes) = prepare_texture_upload(layout, blocks)?;
            (PreparedRenderAssetGpuUpload::Texture(upload), bytes)
        }
        RenderArtifactLayout::Mesh { layout } => {
            let (upload, bytes) = prepare_mesh_upload(layout, blocks)?;
            (PreparedRenderAssetGpuUpload::Mesh(upload), bytes)
        }
    };
    ensure_budget(
        RenderAssetGpuUploadBudgetClass::Destination,
        destination_bytes,
        limits.max_destination_bytes(),
    )?;
    Ok((
        prepared,
        RenderAssetGpuUploadQuote::new(subresource_count, staging_bytes, destination_bytes),
    ))
}

fn validate_source_blocks(
    manifest: &RenderArtifactManifest,
    blocks: &[UploadSourceBlock],
) -> Result<(), RenderAssetGpuUploadPlanError> {
    if blocks.is_empty() {
        return Err(RenderAssetGpuUploadPlanError::Empty);
    }
    let mut seen = HashSet::with_capacity(blocks.len());
    for block in blocks {
        let subresource = block.descriptor.subresource();
        if !seen.insert(subresource) {
            return Err(RenderAssetGpuUploadPlanError::DuplicateSubresource { subresource });
        }
        let Some(expected) = manifest.block(subresource) else {
            return Err(RenderAssetGpuUploadPlanError::UnknownManifestBlock { subresource });
        };
        if expected != &block.descriptor {
            return Err(RenderAssetGpuUploadPlanError::ManifestBlockMismatch { subresource });
        }
        if block.bytes.len() as u64 != block.descriptor.decoded_bytes() {
            return Err(RenderAssetGpuUploadPlanError::DecodedByteCountMismatch {
                subresource,
                expected: block.descriptor.decoded_bytes(),
                actual: block.bytes.len(),
            });
        }
    }
    Ok(())
}

fn prepare_texture_upload(
    layout: &RenderArtifactTextureLayout,
    blocks: Vec<UploadSourceBlock>,
) -> Result<(PreparedTextureUpload, u64), RenderAssetGpuUploadPlanError> {
    let mut selected = BTreeMap::new();
    for block in blocks {
        let RenderSubresourceId::TextureMipLayer { mip, layer } = block.descriptor.subresource()
        else {
            return Err(
                RenderAssetGpuUploadPlanError::UnexpectedTextureSubresource {
                    subresource: block.descriptor.subresource(),
                },
            );
        };
        selected.insert((mip, layer), block);
    }
    let Some(&(first_mip, _)) = selected.keys().next() else {
        return Err(RenderAssetGpuUploadPlanError::Empty);
    };
    let Some(&(last_mip, _)) = selected.keys().next_back() else {
        return Err(RenderAssetGpuUploadPlanError::Empty);
    };
    let source_mip_end = last_mip
        .checked_add(1)
        .ok_or(RenderAssetGpuUploadPlanError::ByteTotalOverflow)?;
    let physical = texture_subresource_layout(layout, first_mip, 0)?;
    let mut uploads = Vec::with_capacity(selected.len());
    let mut destination_bytes = 0_u64;
    for mip in first_mip..source_mip_end {
        for layer in 0..layout.array_layer_count() {
            let Some(block) = selected.remove(&(mip, layer)) else {
                return Err(RenderAssetGpuUploadPlanError::IncompleteTextureFrontier {
                    mip,
                    layer,
                });
            };
            let upload_layout = texture_subresource_layout(layout, mip, layer)?;
            let bytes_per_row = u32::try_from(upload_layout.bytes_per_row())
                .map_err(|_| RenderAssetGpuUploadPlanError::AddressSpaceOverflow)?;
            destination_bytes = destination_bytes
                .checked_add(upload_layout.decoded_bytes())
                .ok_or(RenderAssetGpuUploadPlanError::ByteTotalOverflow)?;
            uploads.push(PreparedTextureSubresourceUpload {
                region: TextureCopyRegion::new(upload_layout.width(), upload_layout.height())
                    .with_mip_level(mip - first_mip)
                    .with_origin(0, 0, layer),
                bytes_per_row,
                bytes: block.bytes,
            });
        }
    }
    Ok((
        PreparedTextureUpload {
            platform_format: Arc::from(layout.block_format().platform_format()),
            width: physical.width(),
            height: physical.height(),
            layer_count: layout.array_layer_count(),
            source_mips: first_mip..source_mip_end,
            uploads,
        },
        destination_bytes,
    ))
}

fn prepare_mesh_upload(
    layout: &crate::asset::artifact::RenderArtifactMeshLayout,
    blocks: Vec<UploadSourceBlock>,
) -> Result<(PreparedMeshUpload, u64), RenderAssetGpuUploadPlanError> {
    let mut selected = BTreeMap::new();
    for block in blocks {
        let RenderSubresourceId::MeshLod { lod } = block.descriptor.subresource() else {
            return Err(RenderAssetGpuUploadPlanError::UnexpectedMeshSubresource {
                subresource: block.descriptor.subresource(),
            });
        };
        selected.insert(lod, block);
    }
    let Some(&first_lod) = selected.keys().next() else {
        return Err(RenderAssetGpuUploadPlanError::Empty);
    };
    let Some(&last_lod) = selected.keys().next_back() else {
        return Err(RenderAssetGpuUploadPlanError::Empty);
    };
    let mut vertex_buffer_bytes = 0_u64;
    let mut index_buffer_bytes = 0_u64;
    let mut lods = Vec::with_capacity(selected.len());
    for lod in first_lod..=last_lod {
        let Some(block) = selected.remove(&lod) else {
            return Err(RenderAssetGpuUploadPlanError::IncompleteMeshFrontier { lod });
        };
        let upload = mesh_subresource_layout(layout, lod)?;
        vertex_buffer_bytes = align_up(vertex_buffer_bytes, BUFFER_COPY_ALIGNMENT)?;
        index_buffer_bytes = align_up(index_buffer_bytes, BUFFER_COPY_ALIGNMENT)?;
        let vertex_destination_offset = vertex_buffer_bytes;
        let index_destination_offset = index_buffer_bytes;
        let vertex_source_range = usize_range(upload.vertex_range())?;
        let index_source_range = usize_range(upload.index_range())?;
        let vertex_range = upload.vertex_range();
        let index_range = upload.index_range();
        vertex_buffer_bytes = vertex_buffer_bytes
            .checked_add(vertex_range.end - vertex_range.start)
            .ok_or(RenderAssetGpuUploadPlanError::ByteTotalOverflow)?;
        index_buffer_bytes = index_buffer_bytes
            .checked_add(index_range.end - index_range.start)
            .ok_or(RenderAssetGpuUploadPlanError::ByteTotalOverflow)?;
        lods.push(PreparedMeshLodUpload {
            lod,
            vertex_count: upload.vertex_count(),
            index_count: upload.index_count(),
            vertex_destination_offset,
            index_destination_offset,
            vertex_source_range,
            index_source_range,
            bounds: upload.bounds(),
            bytes: block.bytes,
        });
    }
    let destination_bytes = vertex_buffer_bytes
        .checked_add(index_buffer_bytes)
        .ok_or(RenderAssetGpuUploadPlanError::ByteTotalOverflow)?;
    Ok((
        PreparedMeshUpload {
            platform_format: Arc::from(layout.platform_format()),
            vertex_buffer_bytes,
            index_buffer_bytes,
            lods,
        },
        destination_bytes,
    ))
}

fn texture_subresource_layout(
    layout: &RenderArtifactTextureLayout,
    mip: u32,
    layer: u32,
) -> Result<RenderArtifactTextureSubresourceLayout, RenderAssetGpuUploadPlanError> {
    layout.subresource_layout(mip, layer).ok_or(
        RenderAssetGpuUploadPlanError::MissingSubresourceLayout {
            subresource: RenderSubresourceId::TextureMipLayer { mip, layer },
        },
    )
}

fn mesh_subresource_layout(
    layout: &crate::asset::artifact::RenderArtifactMeshLayout,
    lod: u16,
) -> Result<RenderArtifactMeshLodUploadLayout, RenderAssetGpuUploadPlanError> {
    layout
        .subresource_layout(lod)
        .ok_or(RenderAssetGpuUploadPlanError::MissingSubresourceLayout {
            subresource: RenderSubresourceId::MeshLod { lod },
        })
}

fn usize_range(range: Range<u64>) -> Result<Range<usize>, RenderAssetGpuUploadPlanError> {
    let start = usize::try_from(range.start)
        .map_err(|_| RenderAssetGpuUploadPlanError::AddressSpaceOverflow)?;
    let end = usize::try_from(range.end)
        .map_err(|_| RenderAssetGpuUploadPlanError::AddressSpaceOverflow)?;
    Ok(start..end)
}

fn align_up(value: u64, alignment: u64) -> Result<u64, RenderAssetGpuUploadPlanError> {
    let remainder = value % alignment;
    if remainder == 0 {
        return Ok(value);
    }
    value
        .checked_add(alignment - remainder)
        .ok_or(RenderAssetGpuUploadPlanError::ByteTotalOverflow)
}

fn ensure_budget(
    class: RenderAssetGpuUploadBudgetClass,
    requested: u64,
    limit: u64,
) -> Result<(), RenderAssetGpuUploadPlanError> {
    if requested > limit {
        return Err(RenderAssetGpuUploadPlanError::BudgetExceeded {
            class,
            requested,
            limit,
        });
    }
    Ok(())
}
