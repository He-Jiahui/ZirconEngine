//! CPU asset decoding and retained-payload byte accounting.

use crate::asset::load::{mesh, texture};
use crate::asset::types::{AssetRequest, CpuAssetPayload, MeshSource, MeshVertex, TextureSource};

pub(super) fn process_request(request: AssetRequest) -> CpuAssetPayload {
    match request {
        AssetRequest::Texture(source) => match texture::load_texture(&source) {
            Ok(texture) => CpuAssetPayload::Texture(texture),
            Err(error) => CpuAssetPayload::Failure {
                request: AssetRequest::Texture(source),
                message: error.to_string(),
            },
        },
        AssetRequest::Mesh(source) => match mesh::load_mesh(&source) {
            Ok(mesh) => CpuAssetPayload::Mesh(mesh),
            Err(error) => CpuAssetPayload::Failure {
                request: AssetRequest::Mesh(source),
                message: error.to_string(),
            },
        },
    }
}

pub(super) fn payload_bytes(payload: &CpuAssetPayload) -> usize {
    let inline_bytes = std::mem::size_of_val(payload);
    inline_bytes.saturating_add(match payload {
        CpuAssetPayload::Texture(texture) => texture
            .rgba
            .capacity()
            .saturating_add(texture_source_bytes(&texture.source)),
        CpuAssetPayload::Mesh(mesh) => std::mem::size_of::<MeshVertex>()
            .saturating_mul(mesh.vertices.capacity())
            .saturating_add(std::mem::size_of::<u32>().saturating_mul(mesh.indices.capacity()))
            .saturating_add(mesh_source_bytes(&mesh.source)),
        CpuAssetPayload::Failure { request, message } => {
            request_bytes(request).saturating_add(message.capacity())
        }
    })
}

fn request_bytes(request: &AssetRequest) -> usize {
    std::mem::size_of_val(request).saturating_add(match request {
        AssetRequest::Texture(source) => texture_source_bytes(source),
        AssetRequest::Mesh(source) => mesh_source_bytes(source),
    })
}

fn texture_source_bytes(source: &TextureSource) -> usize {
    match source {
        TextureSource::Path(path) => path.capacity(),
        TextureSource::BuiltinChecker | TextureSource::BuiltinGrid => 0,
    }
}

fn mesh_source_bytes(source: &MeshSource) -> usize {
    match source {
        MeshSource::Path(path) => path.capacity(),
        MeshSource::BuiltinCube => 0,
    }
}
