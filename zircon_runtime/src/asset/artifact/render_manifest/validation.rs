use std::cmp::Ordering;
use std::collections::VecDeque;

use thiserror::Error;

use crate::core::resource::{ResourceKind, UntypedResourceHandle};

use super::{
    RENDER_ARTIFACT_MANIFEST_SCHEMA_VERSION, RenderArtifactBlockCodec, RenderArtifactLayout,
    RenderArtifactManifest, RenderArtifactResidencyClass, RenderSubresourceId,
};

mod mesh;
mod texture;

use mesh::{block_at, validate_residency};

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum RenderArtifactManifestError {
    #[error("render artifact manifest schema {actual} does not match supported schema {expected}")]
    SchemaVersionMismatch { expected: u32, actual: u32 },
    #[error("render artifact asset revision must be non-zero")]
    AssetRevisionZero,
    #[error("render artifact target platform must not be empty")]
    EmptyTargetPlatform,
    #[error("render artifact platform format must not be empty")]
    EmptyPlatformFormat,
    #[error(
        "render artifact layout {layout_kind} does not support resource kind {resource_kind:?}"
    )]
    ResourceLayoutKindMismatch {
        resource_kind: ResourceKind,
        layout_kind: &'static str,
    },
    #[error("texture render artifact dimensions must be non-zero")]
    EmptyTextureLayout,
    #[error(
        "texture render artifact block geometry {block_width}x{block_height} at {bytes_per_block} bytes must be non-zero"
    )]
    InvalidTextureBlockGeometry {
        block_width: u32,
        block_height: u32,
        bytes_per_block: u32,
    },
    #[error(
        "texture render artifact mip count {mip_count} exceeds extent {width}x{height} maximum {max_mip_count}"
    )]
    TextureMipCountOutOfRange {
        width: u32,
        height: u32,
        mip_count: u32,
        max_mip_count: u32,
    },
    #[error("texture bootstrap mip {bootstrap_first_mip} is outside mip count {mip_count}")]
    TextureBootstrapMipOutOfRange {
        mip_count: u32,
        bootstrap_first_mip: u32,
    },
    #[error("texture render artifact subresource {subresource:?} upload layout overflows")]
    TextureSubresourceLayoutOverflow { subresource: RenderSubresourceId },
    #[error(
        "texture render artifact subresource {subresource:?} decoded size {actual} does not match tight upload size {expected}"
    )]
    TextureBlockDecodedSizeMismatch {
        subresource: RenderSubresourceId,
        expected: u64,
        actual: u64,
    },
    #[error("mesh render artifact LOD count must be non-zero")]
    EmptyMeshLayout,
    #[error("mesh render artifact LOD count {actual} exceeds the u16 identity space")]
    MeshLodCountOverflow { actual: usize },
    #[error("mesh bootstrap first LOD {bootstrap_first_lod} is outside LOD count {lod_count}")]
    MeshBootstrapLodOutOfRange {
        lod_count: usize,
        bootstrap_first_lod: u16,
    },
    #[error("mesh LOD layout expected canonical LOD {expected}, found {actual}")]
    MeshLodLayoutNotCanonical { expected: u16, actual: u16 },
    #[error("mesh LOD {lod} vertex count must be non-zero")]
    MeshLodVertexCountZero { lod: u16 },
    #[error("mesh LOD {lod} index count must be non-zero")]
    MeshLodIndexCountZero { lod: u16 },
    #[error("mesh LOD {lod} bounds must be finite and canonical")]
    MeshLodBoundsInvalid { lod: u16 },
    #[error("mesh LOD {lod} upload layout overflows")]
    MeshLodLayoutOverflow { lod: u16 },
    #[error(
        "mesh LOD {lod} index offset {actual} must be at least {minimum} and aligned to {alignment}"
    )]
    MeshLodIndexOffsetInvalid {
        lod: u16,
        minimum: u64,
        actual: u64,
        alignment: u64,
    },
    #[error("mesh LOD {lod} decoded size {actual} does not match upload layout size {expected}")]
    MeshLodDecodedSizeMismatch {
        lod: u16,
        expected: u64,
        actual: u64,
    },
    #[error("render artifact block count mismatch: expected {expected}, found {actual}")]
    BlockCountMismatch { expected: usize, actual: usize },
    #[error("render artifact block count does not fit the current platform")]
    BlockCountOverflow,
    #[error("duplicate render artifact subresource {subresource:?}")]
    DuplicateSubresource { subresource: RenderSubresourceId },
    #[error("render artifact blocks are not in canonical subresource order")]
    BlocksNotCanonical,
    #[error("missing render artifact subresource {subresource:?}")]
    MissingSubresource { subresource: RenderSubresourceId },
    #[error("unexpected render artifact subresource {subresource:?}")]
    UnexpectedSubresource { subresource: RenderSubresourceId },
    #[error("render artifact subresource {subresource:?} has an invalid zero content id")]
    ZeroContentId { subresource: RenderSubresourceId },
    #[error("render artifact subresource {subresource:?} has zero encoded or decoded bytes")]
    EmptyBlock {
        subresource: RenderSubresourceId,
        encoded_bytes: u64,
        decoded_bytes: u64,
    },
    #[error(
        "raw render artifact subresource {subresource:?} must have equal encoded and decoded sizes"
    )]
    RawCodecSizeMismatch {
        subresource: RenderSubresourceId,
        encoded_bytes: u64,
        decoded_bytes: u64,
    },
    #[error("render artifact subresource {subresource:?} has invalid alignment {alignment}")]
    InvalidBlockAlignment {
        subresource: RenderSubresourceId,
        alignment: u32,
    },
    #[error(
        "render artifact subresource {subresource:?} platform format does not match its layout"
    )]
    BlockPlatformFormatMismatch { subresource: RenderSubresourceId },
    #[error("render artifact subresource {subresource:?} has the wrong residency class")]
    ResidencyClassMismatch {
        subresource: RenderSubresourceId,
        expected: RenderArtifactResidencyClass,
        actual: RenderArtifactResidencyClass,
    },
    #[error("render artifact asset dependency can not reference the manifest resource")]
    SelfAssetDependency,
    #[error("render artifact asset dependencies are not in canonical order")]
    AssetDependenciesNotCanonical,
    #[error("render artifact block {subresource:?} can not depend on itself")]
    SelfBlockDependency { subresource: RenderSubresourceId },
    #[error("render artifact block {subresource:?} depends on missing block {dependency:?}")]
    MissingBlockDependency {
        subresource: RenderSubresourceId,
        dependency: RenderSubresourceId,
    },
    #[error(
        "bootstrap render artifact block {subresource:?} depends on streamable block {dependency:?}"
    )]
    BootstrapDependsOnStreamable {
        subresource: RenderSubresourceId,
        dependency: RenderSubresourceId,
    },
    #[error("render artifact block dependencies are not in canonical order")]
    BlockDependenciesNotCanonical { subresource: RenderSubresourceId },
    #[error("render artifact block dependency graph contains a cycle")]
    CyclicBlockDependencies,
}

pub(super) fn validate_render_artifact_manifest(
    manifest: &RenderArtifactManifest,
) -> Result<(), RenderArtifactManifestError> {
    if manifest.schema_version() != RENDER_ARTIFACT_MANIFEST_SCHEMA_VERSION {
        return Err(RenderArtifactManifestError::SchemaVersionMismatch {
            expected: RENDER_ARTIFACT_MANIFEST_SCHEMA_VERSION,
            actual: manifest.schema_version(),
        });
    }
    if manifest.asset_revision() == 0 {
        return Err(RenderArtifactManifestError::AssetRevisionZero);
    }
    if manifest.target_platform().trim().is_empty() {
        return Err(RenderArtifactManifestError::EmptyTargetPlatform);
    }
    if manifest.layout().platform_format().trim().is_empty() {
        return Err(RenderArtifactManifestError::EmptyPlatformFormat);
    }
    validate_asset_dependencies(manifest)?;
    validate_blocks(manifest)?;
    match manifest.layout() {
        RenderArtifactLayout::Texture { layout } => {
            texture::validate_texture_layout(manifest, layout)?
        }
        RenderArtifactLayout::Mesh { layout } => mesh::validate_mesh_layout(manifest, layout)?,
    }
    validate_block_dependencies(manifest)
}

fn validate_asset_dependencies(
    manifest: &RenderArtifactManifest,
) -> Result<(), RenderArtifactManifestError> {
    let dependencies = manifest.asset_dependencies();
    for dependency in dependencies {
        if *dependency == manifest.resource() {
            return Err(RenderArtifactManifestError::SelfAssetDependency);
        }
    }
    if dependencies
        .windows(2)
        .any(|pair| compare_resource_handles(&pair[0], &pair[1]) != Ordering::Less)
    {
        return Err(RenderArtifactManifestError::AssetDependenciesNotCanonical);
    }
    Ok(())
}

fn validate_blocks(manifest: &RenderArtifactManifest) -> Result<(), RenderArtifactManifestError> {
    let blocks = manifest.blocks();
    for pair in blocks.windows(2) {
        match pair[0].subresource().cmp(&pair[1].subresource()) {
            Ordering::Equal => {
                return Err(RenderArtifactManifestError::DuplicateSubresource {
                    subresource: pair[0].subresource(),
                });
            }
            Ordering::Greater => return Err(RenderArtifactManifestError::BlocksNotCanonical),
            Ordering::Less => {}
        }
    }
    for block in blocks {
        let subresource = block.subresource();
        if block.content_id().is_zero() {
            return Err(RenderArtifactManifestError::ZeroContentId { subresource });
        }
        if block.encoded_bytes() == 0 || block.decoded_bytes() == 0 {
            return Err(RenderArtifactManifestError::EmptyBlock {
                subresource,
                encoded_bytes: block.encoded_bytes(),
                decoded_bytes: block.decoded_bytes(),
            });
        }
        if block.codec() == RenderArtifactBlockCodec::Raw
            && block.encoded_bytes() != block.decoded_bytes()
        {
            return Err(RenderArtifactManifestError::RawCodecSizeMismatch {
                subresource,
                encoded_bytes: block.encoded_bytes(),
                decoded_bytes: block.decoded_bytes(),
            });
        }
        if block.alignment() == 0 || !block.alignment().is_power_of_two() {
            return Err(RenderArtifactManifestError::InvalidBlockAlignment {
                subresource,
                alignment: block.alignment(),
            });
        }
        if block.platform_format().trim().is_empty()
            || block.platform_format() != manifest.layout().platform_format()
        {
            return Err(RenderArtifactManifestError::BlockPlatformFormatMismatch { subresource });
        }
        if block
            .dependencies()
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
        {
            return Err(RenderArtifactManifestError::BlockDependenciesNotCanonical { subresource });
        }
    }
    Ok(())
}

fn validate_block_dependencies(
    manifest: &RenderArtifactManifest,
) -> Result<(), RenderArtifactManifestError> {
    let blocks = manifest.blocks();
    let mut indegree = vec![0_usize; blocks.len()];
    let mut dependents = vec![Vec::<usize>::new(); blocks.len()];
    for (block_index, block) in blocks.iter().enumerate() {
        for dependency in block.dependencies() {
            if *dependency == block.subresource() {
                return Err(RenderArtifactManifestError::SelfBlockDependency {
                    subresource: block.subresource(),
                });
            }
            let Ok(dependency_index) =
                blocks.binary_search_by_key(dependency, |candidate| candidate.subresource())
            else {
                return Err(RenderArtifactManifestError::MissingBlockDependency {
                    subresource: block.subresource(),
                    dependency: *dependency,
                });
            };
            if block.residency() == RenderArtifactResidencyClass::Bootstrap
                && blocks[dependency_index].residency() == RenderArtifactResidencyClass::Streamable
            {
                return Err(RenderArtifactManifestError::BootstrapDependsOnStreamable {
                    subresource: block.subresource(),
                    dependency: *dependency,
                });
            }
            indegree[block_index] = indegree[block_index].saturating_add(1);
            dependents[dependency_index].push(block_index);
        }
    }

    let mut ready = indegree
        .iter()
        .enumerate()
        .filter_map(|(index, count)| (*count == 0).then_some(index))
        .collect::<VecDeque<_>>();
    let mut visited = 0_usize;
    while let Some(index) = ready.pop_front() {
        visited = visited.saturating_add(1);
        for dependent in &dependents[index] {
            indegree[*dependent] = indegree[*dependent].saturating_sub(1);
            if indegree[*dependent] == 0 {
                ready.push_back(*dependent);
            }
        }
    }
    if visited != blocks.len() {
        return Err(RenderArtifactManifestError::CyclicBlockDependencies);
    }
    Ok(())
}

pub(super) fn compare_resource_handles(
    left: &UntypedResourceHandle,
    right: &UntypedResourceHandle,
) -> Ordering {
    resource_kind_tag(left.kind())
        .cmp(&resource_kind_tag(right.kind()))
        .then_with(|| left.id().cmp(&right.id()))
}

pub(super) const fn resource_kind_tag(kind: ResourceKind) -> u8 {
    match kind {
        ResourceKind::Data => 0,
        ResourceKind::Model => 1,
        ResourceKind::Mesh => 2,
        ResourceKind::Material => 3,
        ResourceKind::MaterialGraph => 4,
        ResourceKind::Texture => 5,
        ResourceKind::Shader => 6,
        ResourceKind::Scene => 7,
        ResourceKind::Sound => 8,
        ResourceKind::Font => 9,
        ResourceKind::PhysicsMaterial => 10,
        ResourceKind::NavMesh => 11,
        ResourceKind::NavigationSettings => 12,
        ResourceKind::Terrain => 13,
        ResourceKind::TerrainLayerStack => 14,
        ResourceKind::TileSet => 15,
        ResourceKind::TileMap => 16,
        ResourceKind::Prefab => 17,
        ResourceKind::AnimationSkeleton => 18,
        ResourceKind::AnimationClip => 19,
        ResourceKind::AnimationSequence => 20,
        ResourceKind::AnimationGraph => 21,
        ResourceKind::AnimationStateMachine => 22,
        ResourceKind::UiLayout => 23,
        ResourceKind::UiWidget => 24,
        ResourceKind::UiStyle => 25,
    }
}
