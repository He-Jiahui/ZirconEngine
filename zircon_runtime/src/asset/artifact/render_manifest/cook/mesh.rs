use std::sync::Arc;

use thiserror::Error;

use crate::asset::{
    MESH_ATTRIBUTE_COLOR, MESH_ATTRIBUTE_JOINT_INDEX, MESH_ATTRIBUTE_JOINT_WEIGHT,
    MESH_ATTRIBUTE_NORMAL, MESH_ATTRIBUTE_TANGENT, MESH_ATTRIBUTE_UV0, MESH_ATTRIBUTE_UV1,
    MeshAsset, MeshAttributeValues, MeshIndices, MeshValidationError,
};
use crate::core::framework::render::RenderMeshTopology;
use crate::core::resource::{ResourceKind, UntypedResourceHandle};

use super::super::{
    RenderArtifactBlockCodec, RenderArtifactBlockDescriptor, RenderArtifactContentId,
    RenderArtifactLayout, RenderArtifactManifest, RenderArtifactManifestError,
    RenderArtifactMeshBounds, RenderArtifactMeshIndexFormat, RenderArtifactMeshLayout,
    RenderArtifactMeshLodLayout, RenderArtifactMeshVertexFormat, RenderArtifactResidencyClass,
    RenderSubresourceId,
};
use super::output::{RenderArtifactCookOutput, RenderArtifactCookedBlock};

pub const RENDER_ARTIFACT_STATIC_MESH_FORMAT_V1: &str = "zr-static-mesh-v1";

const DEFAULT_NORMAL: [f32; 3] = [0.0, 0.0, 1.0];
const DEFAULT_UV: [f32; 2] = [0.0, 0.0];
const DEFAULT_TANGENT: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const DEFAULT_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const DEFAULT_JOINT_INDICES: [u16; 4] = [0, 0, 0, 0];
const DEFAULT_JOINT_WEIGHTS: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

#[derive(Clone, Debug)]
pub struct RenderArtifactMeshCookSettings {
    target_platform: Arc<str>,
    block_alignment: u32,
}

impl RenderArtifactMeshCookSettings {
    pub fn new(target_platform: Arc<str>, block_alignment: u32) -> Self {
        Self {
            target_platform,
            block_alignment,
        }
    }

    pub fn target_platform(&self) -> &str {
        self.target_platform.as_ref()
    }

    pub const fn block_alignment(&self) -> u32 {
        self.block_alignment
    }
}

#[derive(Debug, Error)]
pub enum RenderArtifactMeshCookError {
    #[error("mesh render artifact cook requires a Mesh resource, found {actual:?}")]
    ResourceKindMismatch { actual: ResourceKind },
    #[error("mesh render artifact cook does not support {topology:?} topology")]
    UnsupportedTopology { topology: RenderMeshTopology },
    #[error("mesh render artifact cook requires a separate {payload} artifact")]
    UnsupportedPayload { payload: &'static str },
    #[error("mesh render artifact vertex count does not fit the portable u32 layout")]
    VertexCountOverflow,
    #[error("mesh render artifact index count does not fit the portable u32 layout")]
    IndexCountOverflow,
    #[error("mesh render artifact packed byte size does not fit this address space")]
    PackedByteSizeOverflow,
    #[error("mesh render artifact pack produced {actual} bytes but layout requires {expected}")]
    PackedByteCountMismatch { expected: usize, actual: usize },
    #[error("mesh vertex {vertex} attribute {attribute} contains a non-finite value")]
    NonFiniteVertexAttribute {
        vertex: usize,
        attribute: &'static str,
    },
    #[error(transparent)]
    MeshValidation(#[from] MeshValidationError),
    #[error(transparent)]
    Manifest(#[from] RenderArtifactManifestError),
}

pub fn cook_mesh_render_artifact(
    resource: UntypedResourceHandle,
    asset_revision: u64,
    mesh: MeshAsset,
    settings: RenderArtifactMeshCookSettings,
) -> Result<RenderArtifactCookOutput, RenderArtifactMeshCookError> {
    if resource.kind() != ResourceKind::Mesh {
        return Err(RenderArtifactMeshCookError::ResourceKindMismatch {
            actual: resource.kind(),
        });
    }
    mesh.validate()?;
    if mesh.topology != RenderMeshTopology::TriangleList {
        return Err(RenderArtifactMeshCookError::UnsupportedTopology {
            topology: mesh.topology,
        });
    }
    validate_separate_payloads(&mesh)?;

    let vertex_count_usize = mesh.vertex_count()?;
    let vertex_count = u32::try_from(vertex_count_usize)
        .map_err(|_| RenderArtifactMeshCookError::VertexCountOverflow)?;
    let source_index_count = mesh
        .indices
        .as_ref()
        .filter(|indices| !indices.is_empty())
        .map_or(vertex_count_usize, MeshIndices::len);
    let index_count = u32::try_from(source_index_count)
        .map_err(|_| RenderArtifactMeshCookError::IndexCountOverflow)?;
    let vertex_bytes = u64::from(vertex_count)
        .checked_mul(u64::from(
            RenderArtifactMeshVertexFormat::StaticMeshV1.stride(),
        ))
        .ok_or(RenderArtifactMeshCookError::PackedByteSizeOverflow)?;
    let index_bytes = u64::from(index_count)
        .checked_mul(u64::from(
            RenderArtifactMeshIndexFormat::Uint32.byte_width(),
        ))
        .ok_or(RenderArtifactMeshCookError::PackedByteSizeOverflow)?;
    let decoded_bytes = vertex_bytes
        .checked_add(index_bytes)
        .ok_or(RenderArtifactMeshCookError::PackedByteSizeOverflow)?;
    let capacity = usize::try_from(decoded_bytes)
        .map_err(|_| RenderArtifactMeshCookError::PackedByteSizeOverflow)?;

    let mut bytes = Vec::with_capacity(capacity);
    let bounds = pack_vertices(&mesh, &mut bytes)?;
    pack_indices(&mesh, vertex_count, &mut bytes);
    if bytes.len() != capacity {
        return Err(RenderArtifactMeshCookError::PackedByteCountMismatch {
            expected: capacity,
            actual: bytes.len(),
        });
    }
    let layout = RenderArtifactMeshLayout::new(
        Arc::from(RENDER_ARTIFACT_STATIC_MESH_FORMAT_V1),
        RenderArtifactMeshVertexFormat::StaticMeshV1,
        RenderArtifactMeshIndexFormat::Uint32,
        0,
        vec![RenderArtifactMeshLodLayout::new(
            0,
            RenderMeshTopology::TriangleList,
            vertex_count,
            index_count,
            vertex_bytes,
            bounds,
        )],
    );
    let payload = Arc::new(bytes);
    let subresource = RenderSubresourceId::MeshLod { lod: 0 };
    let descriptor = RenderArtifactBlockDescriptor::new(
        subresource,
        RenderArtifactContentId::from_bytes(*blake3::hash(payload.as_slice()).as_bytes()),
        RenderArtifactBlockCodec::Raw,
        decoded_bytes,
        decoded_bytes,
        settings.block_alignment(),
        Arc::from(RENDER_ARTIFACT_STATIC_MESH_FORMAT_V1),
        RenderArtifactResidencyClass::Bootstrap,
        Vec::new(),
    );
    let manifest = RenderArtifactManifest::new(
        resource,
        asset_revision,
        settings.target_platform,
        RenderArtifactLayout::mesh(layout),
        Vec::new(),
        vec![descriptor.clone()],
    )?;
    Ok(RenderArtifactCookOutput::new(
        manifest,
        vec![RenderArtifactCookedBlock::new(
            descriptor,
            payload,
            0..capacity,
        )],
    ))
}

fn validate_separate_payloads(mesh: &MeshAsset) -> Result<(), RenderArtifactMeshCookError> {
    for (present, payload) in [
        (!mesh.morph_targets.is_empty(), "morph-target"),
        (mesh.skin.is_some(), "skin"),
        (mesh.mesh_sdf.is_some(), "mesh-SDF"),
        (mesh.virtual_geometry.is_some(), "virtual-geometry"),
    ] {
        if present {
            return Err(RenderArtifactMeshCookError::UnsupportedPayload { payload });
        }
    }
    Ok(())
}

fn pack_vertices(
    mesh: &MeshAsset,
    output: &mut Vec<u8>,
) -> Result<RenderArtifactMeshBounds, RenderArtifactMeshCookError> {
    let positions = mesh.positions()?;
    let normal = float32x3(mesh, MESH_ATTRIBUTE_NORMAL);
    let uv = float32x2(mesh, MESH_ATTRIBUTE_UV0);
    let joints = uint16x4(mesh, MESH_ATTRIBUTE_JOINT_INDEX);
    let weights = float32x4(mesh, MESH_ATTRIBUTE_JOINT_WEIGHT);
    let tangent = float32x4(mesh, MESH_ATTRIBUTE_TANGENT);
    let color = float32x4(mesh, MESH_ATTRIBUTE_COLOR);
    let uv1 = float32x2(mesh, MESH_ATTRIBUTE_UV1);
    let mut bounds_min = positions.first().copied().unwrap_or([0.0; 3]);
    let mut bounds_max = bounds_min;
    for (index, position) in positions.iter().enumerate() {
        push_f32_checked(output, position, "position", index)?;
        for axis in 0..3 {
            bounds_min[axis] = bounds_min[axis].min(position[axis]);
            bounds_max[axis] = bounds_max[axis].max(position[axis]);
        }
        push_f32_checked(
            output,
            normal.map_or(&DEFAULT_NORMAL, |values| &values[index]),
            "normal",
            index,
        )?;
        push_f32_checked(
            output,
            uv.map_or(&DEFAULT_UV, |values| &values[index]),
            "uv0",
            index,
        )?;
        push_u16(
            output,
            joints.map_or(&DEFAULT_JOINT_INDICES, |values| &values[index]),
        );
        push_f32_checked(
            output,
            weights.map_or(&DEFAULT_JOINT_WEIGHTS, |values| &values[index]),
            "joint_weight",
            index,
        )?;
        push_f32_checked(
            output,
            tangent.map_or(&DEFAULT_TANGENT, |values| &values[index]),
            "tangent",
            index,
        )?;
        push_f32_checked(
            output,
            color.map_or(&DEFAULT_COLOR, |values| &values[index]),
            "color",
            index,
        )?;
        push_f32_checked(
            output,
            uv1.map_or(&DEFAULT_UV, |values| &values[index]),
            "uv1",
            index,
        )?;
    }
    Ok(RenderArtifactMeshBounds::from_min_max(
        bounds_min, bounds_max,
    ))
}

fn pack_indices(mesh: &MeshAsset, vertex_count: u32, output: &mut Vec<u8>) {
    match mesh.indices.as_ref() {
        Some(MeshIndices::U16(indices)) if !indices.is_empty() => {
            for index in indices {
                output.extend_from_slice(&u32::from(*index).to_le_bytes());
            }
        }
        Some(MeshIndices::U32(indices)) if !indices.is_empty() => {
            for index in indices {
                output.extend_from_slice(&index.to_le_bytes());
            }
        }
        Some(MeshIndices::U16(_)) | Some(MeshIndices::U32(_)) | None => {
            for index in 0..vertex_count {
                output.extend_from_slice(&index.to_le_bytes());
            }
        }
    }
}

fn float32x2<'a>(mesh: &'a MeshAsset, name: &str) -> Option<&'a [[f32; 2]]> {
    mesh.attributes
        .get(name)
        .and_then(MeshAttributeValues::as_float32x2)
}

fn float32x3<'a>(mesh: &'a MeshAsset, name: &str) -> Option<&'a [[f32; 3]]> {
    mesh.attributes
        .get(name)
        .and_then(MeshAttributeValues::as_float32x3)
}

fn float32x4<'a>(mesh: &'a MeshAsset, name: &str) -> Option<&'a [[f32; 4]]> {
    mesh.attributes
        .get(name)
        .and_then(MeshAttributeValues::as_float32x4)
}

fn uint16x4<'a>(mesh: &'a MeshAsset, name: &str) -> Option<&'a [[u16; 4]]> {
    mesh.attributes
        .get(name)
        .and_then(MeshAttributeValues::as_uint16x4)
}

fn push_f32_checked<const N: usize>(
    output: &mut Vec<u8>,
    values: &[f32; N],
    attribute: &'static str,
    vertex: usize,
) -> Result<(), RenderArtifactMeshCookError> {
    for value in values {
        if !value.is_finite() {
            return Err(RenderArtifactMeshCookError::NonFiniteVertexAttribute {
                vertex,
                attribute,
            });
        }
        output.extend_from_slice(&value.to_le_bytes());
    }
    Ok(())
}

fn push_u16<const N: usize>(output: &mut Vec<u8>, values: &[u16; N]) {
    for value in values {
        output.extend_from_slice(&value.to_le_bytes());
    }
}

#[cfg(test)]
#[path = "mesh/tests.rs"]
mod tests;
