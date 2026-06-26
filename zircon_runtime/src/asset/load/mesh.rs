//! Mesh loading and built-in primitives.

use std::path::Path;

use thiserror::Error;

use crate::core::math::{Vec2, Vec3};

use crate::asset::formats::obj;
use crate::asset::types::{CpuMeshPayload, MeshSource, MeshVertex};

pub(crate) type MeshLoadResult<T> = std::result::Result<T, MeshLoadError>;

#[derive(Debug, Error)]
pub(crate) enum MeshLoadError {
    #[error("decode obj mesh {path}: {source}")]
    Obj {
        path: String,
        #[source]
        source: obj::ObjDecodeError,
    },
    #[error(
        "unsupported mesh format `{extension}` for {path}; only .obj is supported in this milestone"
    )]
    UnsupportedFormat { path: String, extension: String },
}

pub(crate) fn load_mesh(source: &MeshSource) -> MeshLoadResult<CpuMeshPayload> {
    match source {
        MeshSource::BuiltinCube => Ok(generate_cube_mesh()),
        MeshSource::Path(path) => decode_mesh_file(path),
    }
}

pub(crate) fn decode_mesh_file(path: &str) -> MeshLoadResult<CpuMeshPayload> {
    let mesh_path = Path::new(path);
    let extension = mesh_path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    match extension.as_str() {
        "obj" => obj::decode_obj_file(path).map_err(|source| MeshLoadError::Obj {
            path: path.to_string(),
            source,
        }),
        _ => Err(MeshLoadError::UnsupportedFormat {
            path: path.to_string(),
            extension,
        }),
    }
}

pub(crate) fn generate_cube_mesh() -> CpuMeshPayload {
    let vertices = vec![
        MeshVertex::new(Vec3::new(-0.5, -0.5, 0.5), Vec3::Z, Vec2::new(0.0, 1.0)),
        MeshVertex::new(Vec3::new(0.5, -0.5, 0.5), Vec3::Z, Vec2::new(1.0, 1.0)),
        MeshVertex::new(Vec3::new(0.5, 0.5, 0.5), Vec3::Z, Vec2::new(1.0, 0.0)),
        MeshVertex::new(Vec3::new(-0.5, 0.5, 0.5), Vec3::Z, Vec2::new(0.0, 0.0)),
        MeshVertex::new(Vec3::new(0.5, -0.5, -0.5), -Vec3::Z, Vec2::new(0.0, 1.0)),
        MeshVertex::new(Vec3::new(-0.5, -0.5, -0.5), -Vec3::Z, Vec2::new(1.0, 1.0)),
        MeshVertex::new(Vec3::new(-0.5, 0.5, -0.5), -Vec3::Z, Vec2::new(1.0, 0.0)),
        MeshVertex::new(Vec3::new(0.5, 0.5, -0.5), -Vec3::Z, Vec2::new(0.0, 0.0)),
        MeshVertex::new(Vec3::new(-0.5, -0.5, -0.5), -Vec3::X, Vec2::new(0.0, 1.0)),
        MeshVertex::new(Vec3::new(-0.5, -0.5, 0.5), -Vec3::X, Vec2::new(1.0, 1.0)),
        MeshVertex::new(Vec3::new(-0.5, 0.5, 0.5), -Vec3::X, Vec2::new(1.0, 0.0)),
        MeshVertex::new(Vec3::new(-0.5, 0.5, -0.5), -Vec3::X, Vec2::new(0.0, 0.0)),
        MeshVertex::new(Vec3::new(0.5, -0.5, 0.5), Vec3::X, Vec2::new(0.0, 1.0)),
        MeshVertex::new(Vec3::new(0.5, -0.5, -0.5), Vec3::X, Vec2::new(1.0, 1.0)),
        MeshVertex::new(Vec3::new(0.5, 0.5, -0.5), Vec3::X, Vec2::new(1.0, 0.0)),
        MeshVertex::new(Vec3::new(0.5, 0.5, 0.5), Vec3::X, Vec2::new(0.0, 0.0)),
        MeshVertex::new(Vec3::new(-0.5, 0.5, 0.5), Vec3::Y, Vec2::new(0.0, 1.0)),
        MeshVertex::new(Vec3::new(0.5, 0.5, 0.5), Vec3::Y, Vec2::new(1.0, 1.0)),
        MeshVertex::new(Vec3::new(0.5, 0.5, -0.5), Vec3::Y, Vec2::new(1.0, 0.0)),
        MeshVertex::new(Vec3::new(-0.5, 0.5, -0.5), Vec3::Y, Vec2::new(0.0, 0.0)),
        MeshVertex::new(Vec3::new(-0.5, -0.5, -0.5), -Vec3::Y, Vec2::new(0.0, 1.0)),
        MeshVertex::new(Vec3::new(0.5, -0.5, -0.5), -Vec3::Y, Vec2::new(1.0, 1.0)),
        MeshVertex::new(Vec3::new(0.5, -0.5, 0.5), -Vec3::Y, Vec2::new(1.0, 0.0)),
        MeshVertex::new(Vec3::new(-0.5, -0.5, 0.5), -Vec3::Y, Vec2::new(0.0, 0.0)),
    ];
    let indices = vec![
        0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 8, 9, 10, 8, 10, 11, 12, 13, 14, 12, 14, 15, 16, 17,
        18, 16, 18, 19, 20, 21, 22, 20, 22, 23,
    ];

    CpuMeshPayload {
        source: MeshSource::BuiltinCube,
        vertices,
        indices,
    }
}
