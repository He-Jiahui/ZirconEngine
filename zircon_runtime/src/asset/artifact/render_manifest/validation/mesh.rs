use crate::core::resource::ResourceKind;

use super::super::{
    RenderArtifactBlockDescriptor, RenderArtifactManifest, RenderArtifactMeshLayout,
    RenderArtifactResidencyClass, RenderSubresourceId,
};
use super::RenderArtifactManifestError;

pub(super) fn validate_mesh_layout(
    manifest: &RenderArtifactManifest,
    layout: &RenderArtifactMeshLayout,
) -> Result<(), RenderArtifactManifestError> {
    if manifest.resource().kind() != ResourceKind::Mesh {
        return Err(RenderArtifactManifestError::ResourceLayoutKindMismatch {
            resource_kind: manifest.resource().kind(),
            layout_kind: "mesh",
        });
    }
    let lod_count = layout.lod_count();
    if lod_count == 0 {
        return Err(RenderArtifactManifestError::EmptyMeshLayout);
    }
    if lod_count > usize::from(u16::MAX) + 1 {
        return Err(RenderArtifactManifestError::MeshLodCountOverflow { actual: lod_count });
    }
    if usize::from(layout.bootstrap_first_lod()) >= lod_count {
        return Err(RenderArtifactManifestError::MeshBootstrapLodOutOfRange {
            lod_count,
            bootstrap_first_lod: layout.bootstrap_first_lod(),
        });
    }
    if manifest.blocks().len() < lod_count {
        return Err(RenderArtifactManifestError::BlockCountMismatch {
            expected: lod_count,
            actual: manifest.blocks().len(),
        });
    }
    for (index, lod_layout) in layout.lods().iter().enumerate() {
        let expected_lod = u16::try_from(index)
            .map_err(|_| RenderArtifactManifestError::MeshLodCountOverflow { actual: lod_count })?;
        if lod_layout.lod() != expected_lod {
            return Err(RenderArtifactManifestError::MeshLodLayoutNotCanonical {
                expected: expected_lod,
                actual: lod_layout.lod(),
            });
        }
        validate_lod_metadata(layout, expected_lod)?;
        let expected = RenderSubresourceId::MeshLod { lod: expected_lod };
        let block = block_at(manifest, index, expected)?;
        let upload = layout
            .subresource_layout(expected_lod)
            .ok_or(RenderArtifactManifestError::MeshLodLayoutOverflow { lod: expected_lod })?;
        if block.decoded_bytes() != upload.decoded_bytes() {
            return Err(RenderArtifactManifestError::MeshLodDecodedSizeMismatch {
                lod: expected_lod,
                expected: upload.decoded_bytes(),
                actual: block.decoded_bytes(),
            });
        }
        validate_residency(
            block,
            residency_for_lod(expected_lod, layout.bootstrap_first_lod()),
        )?;
    }
    for block in &manifest.blocks()[lod_count..] {
        let RenderSubresourceId::MeshClusterPage { lod, .. } = block.subresource() else {
            return Err(RenderArtifactManifestError::UnexpectedSubresource {
                subresource: block.subresource(),
            });
        };
        if usize::from(lod) >= lod_count {
            return Err(RenderArtifactManifestError::UnexpectedSubresource {
                subresource: block.subresource(),
            });
        }
        validate_residency(block, residency_for_lod(lod, layout.bootstrap_first_lod()))?;
    }
    Ok(())
}

fn validate_lod_metadata(
    layout: &RenderArtifactMeshLayout,
    lod: u16,
) -> Result<(), RenderArtifactManifestError> {
    let stored = layout
        .lod(lod)
        .ok_or(RenderArtifactManifestError::MeshLodLayoutOverflow { lod })?;
    if stored.vertex_count() == 0 {
        return Err(RenderArtifactManifestError::MeshLodVertexCountZero { lod });
    }
    if stored.index_count() == 0 {
        return Err(RenderArtifactManifestError::MeshLodIndexCountZero { lod });
    }
    if !stored.bounds().is_finite_canonical() {
        return Err(RenderArtifactManifestError::MeshLodBoundsInvalid { lod });
    }
    let upload = layout
        .subresource_layout(lod)
        .ok_or(RenderArtifactManifestError::MeshLodLayoutOverflow { lod })?;
    let minimum = upload.vertex_range().end;
    let alignment = u64::from(layout.index_format().byte_width());
    let actual = upload.index_range().start;
    if actual < minimum || actual % alignment != 0 {
        return Err(RenderArtifactManifestError::MeshLodIndexOffsetInvalid {
            lod,
            minimum,
            actual,
            alignment,
        });
    }
    Ok(())
}

pub(super) fn block_at(
    manifest: &RenderArtifactManifest,
    index: usize,
    expected: RenderSubresourceId,
) -> Result<&RenderArtifactBlockDescriptor, RenderArtifactManifestError> {
    let Some(block) = manifest.blocks().get(index) else {
        return Err(RenderArtifactManifestError::MissingSubresource {
            subresource: expected,
        });
    };
    if block.subresource() != expected {
        return Err(RenderArtifactManifestError::MissingSubresource {
            subresource: expected,
        });
    }
    Ok(block)
}

fn residency_for_lod(lod: u16, bootstrap_first_lod: u16) -> RenderArtifactResidencyClass {
    if lod >= bootstrap_first_lod {
        RenderArtifactResidencyClass::Bootstrap
    } else {
        RenderArtifactResidencyClass::Streamable
    }
}

pub(super) fn validate_residency(
    block: &RenderArtifactBlockDescriptor,
    expected: RenderArtifactResidencyClass,
) -> Result<(), RenderArtifactManifestError> {
    if block.residency() != expected {
        return Err(RenderArtifactManifestError::ResidencyClassMismatch {
            subresource: block.subresource(),
            expected,
            actual: block.residency(),
        });
    }
    Ok(())
}
