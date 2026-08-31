use thiserror::Error;

use crate::asset::artifact::RenderSubresourceId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RenderAssetGpuUploadLimits {
    max_subresources: usize,
    max_staging_bytes: u64,
    max_destination_bytes: u64,
}

impl RenderAssetGpuUploadLimits {
    pub(crate) const fn new(
        max_subresources: usize,
        max_staging_bytes: u64,
        max_destination_bytes: u64,
    ) -> Self {
        Self {
            max_subresources,
            max_staging_bytes,
            max_destination_bytes,
        }
    }

    pub(crate) const fn max_subresources(self) -> usize {
        self.max_subresources
    }

    pub(crate) const fn max_staging_bytes(self) -> u64 {
        self.max_staging_bytes
    }

    pub(crate) const fn max_destination_bytes(self) -> u64 {
        self.max_destination_bytes
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RenderAssetGpuUploadQuote {
    subresource_count: usize,
    staging_bytes: u64,
    destination_bytes: u64,
}

impl RenderAssetGpuUploadQuote {
    pub(super) const fn new(
        subresource_count: usize,
        staging_bytes: u64,
        destination_bytes: u64,
    ) -> Self {
        Self {
            subresource_count,
            staging_bytes,
            destination_bytes,
        }
    }

    pub(crate) const fn subresource_count(self) -> usize {
        self.subresource_count
    }

    pub(crate) const fn staging_bytes(self) -> u64 {
        self.staging_bytes
    }

    pub(crate) const fn destination_bytes(self) -> u64 {
        self.destination_bytes
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RenderAssetGpuUploadBudgetClass {
    Subresources,
    Staging,
    Destination,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum RenderAssetGpuUploadPlanError {
    #[error("render asset GPU upload requires at least one decoded semantic block")]
    Empty,
    #[error("render asset GPU upload contains duplicate subresource {subresource:?}")]
    DuplicateSubresource { subresource: RenderSubresourceId },
    #[error("decoded subresource {subresource:?} is not owned by the manifest")]
    UnknownManifestBlock { subresource: RenderSubresourceId },
    #[error("decoded subresource {subresource:?} descriptor differs from the manifest")]
    ManifestBlockMismatch { subresource: RenderSubresourceId },
    #[error(
        "decoded subresource {subresource:?} has {actual} bytes but the manifest requires {expected}"
    )]
    DecodedByteCountMismatch {
        subresource: RenderSubresourceId,
        expected: u64,
        actual: usize,
    },
    #[error("texture upload received non-texture subresource {subresource:?}")]
    UnexpectedTextureSubresource { subresource: RenderSubresourceId },
    #[error("mesh upload received non-LOD subresource {subresource:?}")]
    UnexpectedMeshSubresource { subresource: RenderSubresourceId },
    #[error("texture upload frontier is missing mip {mip} layer {layer}")]
    IncompleteTextureFrontier { mip: u32, layer: u32 },
    #[error("mesh upload frontier is missing LOD {lod}")]
    IncompleteMeshFrontier { lod: u16 },
    #[error("render artifact layout is missing subresource {subresource:?}")]
    MissingSubresourceLayout { subresource: RenderSubresourceId },
    #[error("render asset upload byte total overflows")]
    ByteTotalOverflow,
    #[error("render asset upload byte range does not fit this address space")]
    AddressSpaceOverflow,
    #[error(
        "render asset GPU upload {class:?} budget requires {requested} but the limit is {limit}"
    )]
    BudgetExceeded {
        class: RenderAssetGpuUploadBudgetClass,
        requested: u64,
        limit: u64,
    },
}
