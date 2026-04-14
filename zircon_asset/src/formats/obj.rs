//! Wavefront OBJ decoding.

use std::collections::HashMap;
use std::fs;

use zircon_math::{Vec2, Vec3};

use crate::types::{CpuMeshPayload, MeshSource, MeshVertex};

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

pub(crate) fn decode_obj_file(path: &str) -> Result<CpuMeshPayload, String> {
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
