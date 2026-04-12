//! Minimal asynchronous asset pipeline for textures and simple meshes.

use crossbeam_channel::unbounded;
use image::{DynamicImage, GenericImageView};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::thread::JoinHandle;
use zircon_core::{spawn_named_thread, ChannelReceiver, ChannelSender, ZirconError};
use zircon_math::{Vec2, Vec3};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TextureSource {
    BuiltinChecker,
    BuiltinGrid,
    Path(String),
}

impl TextureSource {
    pub fn label(&self) -> String {
        match self {
            Self::BuiltinChecker => "builtin://checker".to_string(),
            Self::BuiltinGrid => "builtin://grid".to_string(),
            Self::Path(path) => path.clone(),
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum MeshSource {
    BuiltinCube,
    Path(String),
}

impl MeshSource {
    pub fn label(&self) -> String {
        match self {
            Self::BuiltinCube => "builtin://cube".to_string(),
            Self::Path(path) => path.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

impl MeshVertex {
    pub fn new(position: Vec3, normal: Vec3, uv: Vec2) -> Self {
        Self {
            position: position.to_array(),
            normal: normal.to_array(),
            uv: uv.to_array(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CpuTexturePayload {
    pub source: TextureSource,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct CpuMeshPayload {
    pub source: MeshSource,
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,
}

#[derive(Clone, Debug)]
pub enum CpuAssetPayload {
    Texture(CpuTexturePayload),
    Mesh(CpuMeshPayload),
    Failure {
        request: AssetRequest,
        message: String,
    },
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum AssetRequest {
    Texture(TextureSource),
    Mesh(MeshSource),
}

pub struct AssetWorkerPool {
    request_tx: Option<ChannelSender<AssetRequest>>,
    completion_rx: ChannelReceiver<CpuAssetPayload>,
    joins: Vec<JoinHandle<()>>,
}

impl AssetWorkerPool {
    pub fn new(worker_count: usize) -> Result<Self, ZirconError> {
        let worker_count = worker_count.max(1);
        let (request_tx, request_rx) = unbounded();
        let (completion_tx, completion_rx) = unbounded();
        let mut joins = Vec::with_capacity(worker_count);

        for worker_index in 0..worker_count {
            let request_rx = request_rx.clone();
            let completion_tx = completion_tx.clone();
            joins.push(spawn_named_thread(
                format!("zircon-asset-{worker_index}"),
                move || {
                    while let Ok(request) = request_rx.recv() {
                        let payload = process_request(request);
                        let _ = completion_tx.send(payload);
                    }
                },
            )?);
        }

        Ok(Self {
            request_tx: Some(request_tx),
            completion_rx,
            joins,
        })
    }

    pub fn request(&self, request: AssetRequest) -> Result<(), ZirconError> {
        self.request_tx
            .as_ref()
            .expect("asset worker request sender alive")
            .send(request.clone())
            .map_err(|_| ZirconError::ChannelSend(format!("asset request dropped: {request:?}")))
    }

    pub fn request_sender(&self) -> ChannelSender<AssetRequest> {
        self.request_tx
            .as_ref()
            .expect("asset worker request sender alive")
            .clone()
    }

    pub fn completion_receiver(&self) -> ChannelReceiver<CpuAssetPayload> {
        self.completion_rx.clone()
    }
}

impl Drop for AssetWorkerPool {
    fn drop(&mut self) {
        self.request_tx.take();

        for join in self.joins.drain(..) {
            let _ = join.join();
        }
    }
}

fn process_request(request: AssetRequest) -> CpuAssetPayload {
    match request {
        AssetRequest::Texture(source) => match load_texture(&source) {
            Ok(texture) => CpuAssetPayload::Texture(texture),
            Err(message) => CpuAssetPayload::Failure {
                request: AssetRequest::Texture(source),
                message,
            },
        },
        AssetRequest::Mesh(source) => match load_mesh(&source) {
            Ok(mesh) => CpuAssetPayload::Mesh(mesh),
            Err(message) => CpuAssetPayload::Failure {
                request: AssetRequest::Mesh(source),
                message,
            },
        },
    }
}

fn load_texture(source: &TextureSource) -> Result<CpuTexturePayload, String> {
    match source {
        TextureSource::BuiltinChecker => Ok(generate_checker_texture()),
        TextureSource::BuiltinGrid => Ok(generate_grid_texture()),
        TextureSource::Path(path) => decode_image_file(path),
    }
}

fn load_mesh(source: &MeshSource) -> Result<CpuMeshPayload, String> {
    match source {
        MeshSource::BuiltinCube => Ok(generate_cube_mesh()),
        MeshSource::Path(path) => decode_mesh_file(path),
    }
}

fn decode_image_file(path: &str) -> Result<CpuTexturePayload, String> {
    let image =
        image::open(Path::new(path)).map_err(|error| format!("open image {path}: {error}"))?;
    Ok(image_to_payload(
        TextureSource::Path(path.to_string()),
        image,
    ))
}

fn decode_mesh_file(path: &str) -> Result<CpuMeshPayload, String> {
    let mesh_path = Path::new(path);
    let extension = mesh_path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    match extension.as_str() {
        "obj" => decode_obj_file(path),
        _ => Err(format!(
            "unsupported mesh format for {path}; only .obj is supported in this milestone"
        )),
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct ObjVertexKey {
    position: usize,
    uv: Option<usize>,
    normal: Option<usize>,
}

#[derive(Clone, Copy, Debug)]
struct ParsedObjVertex {
    position: Vec3,
    uv: Vec2,
    normal: Vec3,
    needs_generated_normal: bool,
}

fn decode_obj_file(path: &str) -> Result<CpuMeshPayload, String> {
    let source = fs::read_to_string(path).map_err(|error| format!("read mesh {path}: {error}"))?;
    let mut positions = Vec::<Vec3>::new();
    let mut uvs = Vec::<Vec2>::new();
    let mut normals = Vec::<Vec3>::new();
    let mut vertices = Vec::<ParsedObjVertex>::new();
    let mut indices = Vec::<u32>::new();
    let mut dedup = HashMap::<ObjVertexKey, u32>::new();

    for (line_index, raw_line) in source.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut parts = line.split_whitespace();
        let Some(keyword) = parts.next() else {
            continue;
        };

        match keyword {
            "v" => {
                let x = parse_obj_scalar(parts.next(), path, line_index, "vertex x")?;
                let y = parse_obj_scalar(parts.next(), path, line_index, "vertex y")?;
                let z = parse_obj_scalar(parts.next(), path, line_index, "vertex z")?;
                positions.push(Vec3::new(x, y, z));
            }
            "vt" => {
                let u = parse_obj_scalar(parts.next(), path, line_index, "uv u")?;
                let v = parse_obj_scalar(parts.next(), path, line_index, "uv v")?;
                uvs.push(Vec2::new(u, 1.0 - v));
            }
            "vn" => {
                let x = parse_obj_scalar(parts.next(), path, line_index, "normal x")?;
                let y = parse_obj_scalar(parts.next(), path, line_index, "normal y")?;
                let z = parse_obj_scalar(parts.next(), path, line_index, "normal z")?;
                normals.push(Vec3::new(x, y, z).normalize_or_zero());
            }
            "f" => {
                let tokens: Vec<_> = parts.collect();
                if tokens.len() < 3 {
                    return Err(format!(
                        "face with fewer than 3 vertices at {path}:{}",
                        line_index + 1
                    ));
                }

                let mut face_indices = Vec::with_capacity(tokens.len());
                for token in tokens {
                    let key =
                        parse_obj_face_vertex(token, positions.len(), uvs.len(), normals.len())
                            .map_err(|error| {
                                format!("parse face vertex at {path}:{}: {error}", line_index + 1)
                            })?;
                    let vertex_index = if let Some(index) = dedup.get(&key) {
                        *index
                    } else {
                        let position = positions[key.position];
                        let uv = key.uv.map(|index| uvs[index]).unwrap_or(Vec2::ZERO);
                        let (normal, needs_generated_normal) = key
                            .normal
                            .map(|index| (normals[index], false))
                            .unwrap_or((Vec3::ZERO, true));
                        let index = vertices.len() as u32;
                        vertices.push(ParsedObjVertex {
                            position,
                            uv,
                            normal,
                            needs_generated_normal,
                        });
                        dedup.insert(key, index);
                        index
                    };
                    face_indices.push(vertex_index);
                }

                for triangle in 1..face_indices.len() - 1 {
                    indices.push(face_indices[0]);
                    indices.push(face_indices[triangle]);
                    indices.push(face_indices[triangle + 1]);
                }
            }
            _ => {}
        }
    }

    if positions.is_empty() {
        return Err(format!("mesh {path} did not contain any vertex positions"));
    }
    if indices.is_empty() {
        return Err(format!("mesh {path} did not contain any faces"));
    }

    for triangle in indices.chunks_exact(3) {
        let a = triangle[0] as usize;
        let b = triangle[1] as usize;
        let c = triangle[2] as usize;
        let face_normal = (vertices[b].position - vertices[a].position)
            .cross(vertices[c].position - vertices[a].position)
            .normalize_or_zero();
        if face_normal.length_squared() <= f32::EPSILON {
            continue;
        }

        for &index in triangle {
            let vertex = &mut vertices[index as usize];
            if vertex.needs_generated_normal {
                vertex.normal += face_normal;
            }
        }
    }

    let vertices = vertices
        .into_iter()
        .map(|vertex| {
            MeshVertex::new(
                vertex.position,
                if vertex.needs_generated_normal {
                    let generated = vertex.normal.normalize_or_zero();
                    if generated.length_squared() <= f32::EPSILON {
                        Vec3::Y
                    } else {
                        generated
                    }
                } else {
                    vertex.normal
                },
                vertex.uv,
            )
        })
        .collect();

    Ok(CpuMeshPayload {
        source: MeshSource::Path(path.to_string()),
        vertices,
        indices,
    })
}

fn parse_obj_scalar(
    value: Option<&str>,
    path: &str,
    line_index: usize,
    label: &str,
) -> Result<f32, String> {
    let value = value.ok_or_else(|| format!("missing {label} at {path}:{}", line_index + 1))?;
    value.parse::<f32>().map_err(|error| {
        format!(
            "invalid {label} '{value}' at {path}:{}: {error}",
            line_index + 1
        )
    })
}

fn parse_obj_face_vertex(
    token: &str,
    position_count: usize,
    uv_count: usize,
    normal_count: usize,
) -> Result<ObjVertexKey, String> {
    let mut parts = token.split('/');
    let position = resolve_obj_index(
        parts.next().unwrap_or_default(),
        position_count,
        "position index",
    )?;
    let uv = match parts.next() {
        Some("") | None => None,
        Some(value) => Some(resolve_obj_index(value, uv_count, "uv index")?),
    };
    let normal = match parts.next() {
        Some("") | None => None,
        Some(value) => Some(resolve_obj_index(value, normal_count, "normal index")?),
    };

    Ok(ObjVertexKey {
        position,
        uv,
        normal,
    })
}

fn resolve_obj_index(value: &str, len: usize, label: &str) -> Result<usize, String> {
    if len == 0 {
        return Err(format!("missing source data for {label}"));
    }
    let index = value
        .parse::<isize>()
        .map_err(|error| format!("invalid {label} '{value}': {error}"))?;
    let resolved = if index > 0 {
        index - 1
    } else if index < 0 {
        len as isize + index
    } else {
        return Err(format!("{label} cannot be zero"));
    };
    if !(0..len as isize).contains(&resolved) {
        return Err(format!("{label} {value} is out of bounds"));
    }
    Ok(resolved as usize)
}

fn image_to_payload(source: TextureSource, image: DynamicImage) -> CpuTexturePayload {
    let rgba = image.to_rgba8();
    let (width, height) = image.dimensions();

    CpuTexturePayload {
        source,
        width,
        height,
        rgba: rgba.into_raw(),
    }
}

fn generate_checker_texture() -> CpuTexturePayload {
    let width = 128;
    let height = 128;
    let mut rgba = vec![0_u8; width * height * 4];

    for y in 0..height {
        for x in 0..width {
            let tile = ((x / 16) + (y / 16)) % 2;
            let color = if tile == 0 {
                [220, 220, 220, 255]
            } else {
                [40, 40, 40, 255]
            };
            let offset = (y * width + x) * 4;
            rgba[offset..offset + 4].copy_from_slice(&color);
        }
    }

    CpuTexturePayload {
        source: TextureSource::BuiltinChecker,
        width: width as u32,
        height: height as u32,
        rgba,
    }
}

fn generate_grid_texture() -> CpuTexturePayload {
    let width = 256;
    let height = 256;
    let mut rgba = vec![0_u8; width * height * 4];

    for y in 0..height {
        for x in 0..width {
            let is_major = x % 64 == 0 || y % 64 == 0;
            let is_minor = x % 16 == 0 || y % 16 == 0;
            let color = if is_major {
                [110, 150, 255, 255]
            } else if is_minor {
                [55, 65, 85, 255]
            } else {
                [26, 30, 38, 255]
            };
            let offset = (y * width + x) * 4;
            rgba[offset..offset + 4].copy_from_slice(&color);
        }
    }

    CpuTexturePayload {
        source: TextureSource::BuiltinGrid,
        width: width as u32,
        height: height as u32,
        rgba,
    }
}

fn generate_cube_mesh() -> CpuMeshPayload {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn builtin_checker_texture_has_rgba_payload() {
        let payload = generate_checker_texture();

        assert_eq!(
            payload.rgba.len(),
            payload.width as usize * payload.height as usize * 4
        );
    }

    #[test]
    fn builtin_cube_mesh_has_triangles() {
        let payload = generate_cube_mesh();

        assert_eq!(payload.indices.len() % 3, 0);
        assert!(!payload.vertices.is_empty());
    }

    #[test]
    fn worker_pool_completes_builtin_texture_requests() {
        let pool = AssetWorkerPool::new(1).unwrap();
        let completions = pool.completion_receiver();

        pool.request(AssetRequest::Texture(TextureSource::BuiltinChecker))
            .unwrap();

        let payload = completions.recv().unwrap();
        match payload {
            CpuAssetPayload::Texture(texture) => {
                assert_eq!(texture.source, TextureSource::BuiltinChecker);
                assert_eq!(
                    texture.rgba.len(),
                    texture.width as usize * texture.height as usize * 4
                );
            }
            other => panic!("unexpected payload: {other:?}"),
        }
    }

    #[test]
    fn obj_mesh_file_is_parsed_into_gpu_ready_payload() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("zircon_asset_mesh_{unique}.obj"));
        fs::write(
            &path,
            "\
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 1.0 1.0 0.0
v 0.0 1.0 0.0
vt 0.0 0.0
vt 1.0 0.0
vt 1.0 1.0
vt 0.0 1.0
f 1/1 2/2 3/3 4/4
",
        )
        .unwrap();

        let payload = decode_obj_file(path.to_str().unwrap()).unwrap();
        let _ = fs::remove_file(&path);

        assert_eq!(payload.indices.len(), 6);
        assert_eq!(payload.vertices.len(), 4);
        assert!(payload
            .vertices
            .iter()
            .all(|vertex| Vec3::from_array(vertex.normal).length() > 0.0));
    }
}
