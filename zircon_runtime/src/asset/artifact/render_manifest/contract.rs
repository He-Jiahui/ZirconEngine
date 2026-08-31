use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::core::resource::UntypedResourceHandle;

use super::RenderArtifactManifestError;

mod mesh;

pub use mesh::{
    RenderArtifactMeshBounds, RenderArtifactMeshIndexFormat, RenderArtifactMeshLayout,
    RenderArtifactMeshLodLayout, RenderArtifactMeshLodUploadLayout, RenderArtifactMeshVertexFormat,
};

pub const RENDER_ARTIFACT_MANIFEST_SCHEMA_VERSION: u32 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RenderArtifactContentId([u8; 32]);

impl RenderArtifactContentId {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub(super) fn is_zero(self) -> bool {
        self.0.iter().all(|byte| *byte == 0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderArtifactBlockCodec {
    Raw,
    Zstd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderArtifactResidencyClass {
    Bootstrap,
    Streamable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RenderSubresourceId {
    TextureMipLayer { mip: u32, layer: u32 },
    MeshLod { lod: u16 },
    MeshClusterPage { lod: u16, page: u32 },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderArtifactBlockDescriptor {
    subresource: RenderSubresourceId,
    content_id: RenderArtifactContentId,
    codec: RenderArtifactBlockCodec,
    encoded_bytes: u64,
    decoded_bytes: u64,
    alignment: u32,
    platform_format: Arc<str>,
    residency: RenderArtifactResidencyClass,
    dependencies: Arc<[RenderSubresourceId]>,
}

impl RenderArtifactBlockDescriptor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        subresource: RenderSubresourceId,
        content_id: RenderArtifactContentId,
        codec: RenderArtifactBlockCodec,
        encoded_bytes: u64,
        decoded_bytes: u64,
        alignment: u32,
        platform_format: Arc<str>,
        residency: RenderArtifactResidencyClass,
        mut dependencies: Vec<RenderSubresourceId>,
    ) -> Self {
        dependencies.sort_unstable();
        dependencies.dedup();
        Self {
            subresource,
            content_id,
            codec,
            encoded_bytes,
            decoded_bytes,
            alignment,
            platform_format,
            residency,
            dependencies: dependencies.into(),
        }
    }

    pub const fn subresource(&self) -> RenderSubresourceId {
        self.subresource
    }

    pub const fn content_id(&self) -> RenderArtifactContentId {
        self.content_id
    }

    pub const fn codec(&self) -> RenderArtifactBlockCodec {
        self.codec
    }

    pub const fn encoded_bytes(&self) -> u64 {
        self.encoded_bytes
    }

    pub const fn decoded_bytes(&self) -> u64 {
        self.decoded_bytes
    }

    pub const fn alignment(&self) -> u32 {
        self.alignment
    }

    pub fn platform_format(&self) -> &str {
        self.platform_format.as_ref()
    }

    pub const fn residency(&self) -> RenderArtifactResidencyClass {
        self.residency
    }

    pub fn dependencies(&self) -> &[RenderSubresourceId] {
        self.dependencies.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderArtifactTextureBlockFormat {
    platform_format: Arc<str>,
    block_width: u32,
    block_height: u32,
    bytes_per_block: u32,
}

impl RenderArtifactTextureBlockFormat {
    pub fn new(
        platform_format: Arc<str>,
        block_width: u32,
        block_height: u32,
        bytes_per_block: u32,
    ) -> Self {
        Self {
            platform_format,
            block_width,
            block_height,
            bytes_per_block,
        }
    }

    pub fn platform_format(&self) -> &str {
        self.platform_format.as_ref()
    }

    pub const fn block_width(&self) -> u32 {
        self.block_width
    }

    pub const fn block_height(&self) -> u32 {
        self.block_height
    }

    pub const fn bytes_per_block(&self) -> u32 {
        self.bytes_per_block
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderArtifactTextureLayout {
    block_format: RenderArtifactTextureBlockFormat,
    width: u32,
    height: u32,
    mip_count: u32,
    array_layer_count: u32,
    bootstrap_first_mip: u32,
}

impl RenderArtifactTextureLayout {
    pub fn new(
        block_format: RenderArtifactTextureBlockFormat,
        width: u32,
        height: u32,
        mip_count: u32,
        array_layer_count: u32,
        bootstrap_first_mip: u32,
    ) -> Self {
        Self {
            block_format,
            width,
            height,
            mip_count,
            array_layer_count,
            bootstrap_first_mip,
        }
    }

    pub const fn block_format(&self) -> &RenderArtifactTextureBlockFormat {
        &self.block_format
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub const fn mip_count(&self) -> u32 {
        self.mip_count
    }

    pub const fn array_layer_count(&self) -> u32 {
        self.array_layer_count
    }

    pub const fn bootstrap_first_mip(&self) -> u32 {
        self.bootstrap_first_mip
    }

    pub fn subresource_layout(
        &self,
        mip: u32,
        layer: u32,
    ) -> Option<RenderArtifactTextureSubresourceLayout> {
        if mip >= self.mip_count || layer >= self.array_layer_count {
            return None;
        }
        let block_width = self.block_format.block_width;
        let block_height = self.block_format.block_height;
        let bytes_per_block = self.block_format.bytes_per_block;
        if block_width == 0 || block_height == 0 || bytes_per_block == 0 {
            return None;
        }
        let width = self.width.checked_shr(mip).unwrap_or(0).max(1);
        let height = self.height.checked_shr(mip).unwrap_or(0).max(1);
        let blocks_per_row = width.div_ceil(block_width);
        let block_rows = height.div_ceil(block_height);
        let bytes_per_row = u64::from(blocks_per_row).checked_mul(u64::from(bytes_per_block))?;
        let decoded_bytes = bytes_per_row.checked_mul(u64::from(block_rows))?;
        Some(RenderArtifactTextureSubresourceLayout {
            width,
            height,
            block_width,
            block_height,
            bytes_per_block,
            bytes_per_row,
            block_rows,
            decoded_bytes,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderArtifactTextureSubresourceLayout {
    width: u32,
    height: u32,
    block_width: u32,
    block_height: u32,
    bytes_per_block: u32,
    bytes_per_row: u64,
    block_rows: u32,
    decoded_bytes: u64,
}

impl RenderArtifactTextureSubresourceLayout {
    pub const fn width(self) -> u32 {
        self.width
    }

    pub const fn height(self) -> u32 {
        self.height
    }

    pub const fn block_width(self) -> u32 {
        self.block_width
    }

    pub const fn block_height(self) -> u32 {
        self.block_height
    }

    pub const fn bytes_per_block(self) -> u32 {
        self.bytes_per_block
    }

    pub const fn bytes_per_row(self) -> u64 {
        self.bytes_per_row
    }

    pub const fn block_rows(self) -> u32 {
        self.block_rows
    }

    pub const fn decoded_bytes(self) -> u64 {
        self.decoded_bytes
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RenderArtifactLayout {
    Texture { layout: RenderArtifactTextureLayout },
    Mesh { layout: RenderArtifactMeshLayout },
}

impl RenderArtifactLayout {
    pub fn texture(layout: RenderArtifactTextureLayout) -> Self {
        Self::Texture { layout }
    }

    pub fn mesh(layout: RenderArtifactMeshLayout) -> Self {
        Self::Mesh { layout }
    }

    pub fn platform_format(&self) -> &str {
        match self {
            Self::Texture { layout } => layout.block_format().platform_format(),
            Self::Mesh { layout } => layout.platform_format(),
        }
    }

    pub const fn texture_layout(&self) -> Option<&RenderArtifactTextureLayout> {
        match self {
            Self::Texture { layout } => Some(layout),
            Self::Mesh { .. } => None,
        }
    }

    pub const fn mesh_layout(&self) -> Option<&RenderArtifactMeshLayout> {
        match self {
            Self::Texture { .. } => None,
            Self::Mesh { layout } => Some(layout),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderArtifactManifest {
    schema_version: u32,
    resource: UntypedResourceHandle,
    asset_revision: u64,
    target_platform: Arc<str>,
    layout: RenderArtifactLayout,
    asset_dependencies: Arc<[UntypedResourceHandle]>,
    blocks: Arc<[RenderArtifactBlockDescriptor]>,
}

impl RenderArtifactManifest {
    pub fn new(
        resource: UntypedResourceHandle,
        asset_revision: u64,
        target_platform: Arc<str>,
        layout: RenderArtifactLayout,
        mut asset_dependencies: Vec<UntypedResourceHandle>,
        mut blocks: Vec<RenderArtifactBlockDescriptor>,
    ) -> Result<Self, RenderArtifactManifestError> {
        asset_dependencies.sort_by(super::validation::compare_resource_handles);
        asset_dependencies.dedup();
        blocks.sort_unstable_by_key(RenderArtifactBlockDescriptor::subresource);
        let manifest = Self {
            schema_version: RENDER_ARTIFACT_MANIFEST_SCHEMA_VERSION,
            resource,
            asset_revision,
            target_platform,
            layout,
            asset_dependencies: asset_dependencies.into(),
            blocks: blocks.into(),
        };
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn validate(&self) -> Result<(), RenderArtifactManifestError> {
        super::validation::validate_render_artifact_manifest(self)
    }

    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub const fn resource(&self) -> UntypedResourceHandle {
        self.resource
    }

    pub const fn asset_revision(&self) -> u64 {
        self.asset_revision
    }

    pub fn target_platform(&self) -> &str {
        self.target_platform.as_ref()
    }

    pub const fn layout(&self) -> &RenderArtifactLayout {
        &self.layout
    }

    pub fn asset_dependencies(&self) -> &[UntypedResourceHandle] {
        self.asset_dependencies.as_ref()
    }

    pub fn blocks(&self) -> &[RenderArtifactBlockDescriptor] {
        self.blocks.as_ref()
    }

    pub fn block(
        &self,
        subresource: RenderSubresourceId,
    ) -> Option<&RenderArtifactBlockDescriptor> {
        self.blocks
            .binary_search_by_key(&subresource, RenderArtifactBlockDescriptor::subresource)
            .ok()
            .and_then(|index| self.blocks.get(index))
    }

    pub fn bootstrap_blocks(&self) -> impl Iterator<Item = &RenderArtifactBlockDescriptor> {
        self.blocks
            .iter()
            .filter(|block| block.residency == RenderArtifactResidencyClass::Bootstrap)
    }

    pub fn streamable_blocks(&self) -> impl Iterator<Item = &RenderArtifactBlockDescriptor> {
        self.blocks
            .iter()
            .filter(|block| block.residency == RenderArtifactResidencyClass::Streamable)
    }

    pub fn texture_subresource_layout(
        &self,
        subresource: RenderSubresourceId,
    ) -> Option<RenderArtifactTextureSubresourceLayout> {
        let RenderSubresourceId::TextureMipLayer { mip, layer } = subresource else {
            return None;
        };
        self.layout.texture_layout()?.subresource_layout(mip, layer)
    }

    pub fn mesh_lod_layout(
        &self,
        subresource: RenderSubresourceId,
    ) -> Option<RenderArtifactMeshLodUploadLayout> {
        let RenderSubresourceId::MeshLod { lod } = subresource else {
            return None;
        };
        self.layout.mesh_layout()?.subresource_layout(lod)
    }
}
