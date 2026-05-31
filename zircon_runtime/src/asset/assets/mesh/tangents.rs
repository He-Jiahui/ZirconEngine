use crate::core::framework::render::RenderMeshTopology;

use super::attribute::MeshAttributeValues;
use super::constants::{MESH_ATTRIBUTE_NORMAL, MESH_ATTRIBUTE_TANGENT, MESH_ATTRIBUTE_UV0};
use super::indices::MeshIndices;
use super::mesh_asset::MeshAsset;
use super::validation::MeshValidationError;

impl MeshAsset {
    pub fn try_generate_missing_tangents(&mut self) -> Result<bool, MeshValidationError> {
        self.validate()?;

        if self.attributes.contains_key(MESH_ATTRIBUTE_TANGENT) {
            return Ok(false);
        }
        if self.topology != RenderMeshTopology::TriangleList {
            return Err(MeshValidationError::TangentGenerationRequiresTriangleList {
                topology: self.topology,
            });
        }

        let positions = self.positions()?;
        let normals = required_float32x3_attribute(self, MESH_ATTRIBUTE_NORMAL)?;
        let uvs = required_float32x2_attribute(self, MESH_ATTRIBUTE_UV0)?;
        let tangents = tangents_for_mesh(positions, normals, uvs, self.indices.as_ref());
        self.attributes.insert(
            MESH_ATTRIBUTE_TANGENT.to_string(),
            MeshAttributeValues::Float32x4(tangents),
        );
        self.validate()?;
        Ok(true)
    }
}

fn required_float32x3_attribute<'a>(
    asset: &'a MeshAsset,
    name: &'static str,
) -> Result<&'a [[f32; 3]], MeshValidationError> {
    asset
        .attributes
        .get(name)
        .ok_or(MeshValidationError::TangentGenerationMissingAttribute { attribute: name })?
        .as_float32x3()
        .ok_or_else(|| MeshValidationError::InvalidAttributeFormat {
            attribute: name.to_string(),
            expected: "float32x3",
        })
}

fn required_float32x2_attribute<'a>(
    asset: &'a MeshAsset,
    name: &'static str,
) -> Result<&'a [[f32; 2]], MeshValidationError> {
    asset
        .attributes
        .get(name)
        .ok_or(MeshValidationError::TangentGenerationMissingAttribute { attribute: name })?
        .as_float32x2()
        .ok_or_else(|| MeshValidationError::InvalidAttributeFormat {
            attribute: name.to_string(),
            expected: "float32x2",
        })
}

fn tangents_for_mesh(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: Option<&MeshIndices>,
) -> Vec<[f32; 4]> {
    let mut tangent_sums = vec![[0.0, 0.0, 0.0]; positions.len()];
    let mut bitangent_sums = vec![[0.0, 0.0, 0.0]; positions.len()];
    let element_indices = indices.map_or_else(
        || (0..positions.len()).collect::<Vec<_>>(),
        |indices| {
            indices
                .to_u32_vec()
                .into_iter()
                .map(|index| index as usize)
                .collect::<Vec<_>>()
        },
    );

    for triangle in element_indices.chunks_exact(3) {
        accumulate_triangle_tangent(
            [triangle[0], triangle[1], triangle[2]],
            positions,
            uvs,
            &mut tangent_sums,
            &mut bitangent_sums,
        );
    }

    (0..positions.len())
        .map(|index| tangent_for_vertex(normals[index], tangent_sums[index], bitangent_sums[index]))
        .collect()
}

fn accumulate_triangle_tangent(
    [a, b, c]: [usize; 3],
    positions: &[[f32; 3]],
    uvs: &[[f32; 2]],
    tangent_sums: &mut [[f32; 3]],
    bitangent_sums: &mut [[f32; 3]],
) {
    let edge1 = sub3(positions[b], positions[a]);
    let edge2 = sub3(positions[c], positions[a]);
    let uv1 = sub2(uvs[b], uvs[a]);
    let uv2 = sub2(uvs[c], uvs[a]);
    let determinant = uv1[0] * uv2[1] - uv2[0] * uv1[1];
    if determinant.abs() <= f32::EPSILON {
        return;
    }

    let inverse = 1.0 / determinant;
    let tangent = scale3(sub3(scale3(edge1, uv2[1]), scale3(edge2, uv1[1])), inverse);
    let bitangent = scale3(sub3(scale3(edge2, uv1[0]), scale3(edge1, uv2[0])), inverse);
    for index in [a, b, c] {
        add3(&mut tangent_sums[index], tangent);
        add3(&mut bitangent_sums[index], bitangent);
    }
}

fn tangent_for_vertex(
    normal: [f32; 3],
    tangent_sum: [f32; 3],
    bitangent_sum: [f32; 3],
) -> [f32; 4] {
    let normal = normalize3(normal);
    let tangent = orthogonal_tangent(normal, tangent_sum);
    let handedness = if dot3(cross3(normal, tangent), bitangent_sum) < 0.0 {
        -1.0
    } else {
        1.0
    };
    [tangent[0], tangent[1], tangent[2], handedness]
}

fn orthogonal_tangent(normal: [f32; 3], tangent: [f32; 3]) -> [f32; 3] {
    let projected = sub3(tangent, scale3(normal, dot3(normal, tangent)));
    let normalized = normalize3(projected);
    if normalized == [0.0, 0.0, 0.0] {
        fallback_tangent(normal)
    } else {
        normalized
    }
}

fn fallback_tangent(normal: [f32; 3]) -> [f32; 3] {
    if normal == [0.0, 0.0, 0.0] {
        return [1.0, 0.0, 0.0];
    }
    let axis = if normal[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    normalize3(sub3(axis, scale3(normal, dot3(normal, axis))))
}

fn sub2(lhs: [f32; 2], rhs: [f32; 2]) -> [f32; 2] {
    [lhs[0] - rhs[0], lhs[1] - rhs[1]]
}

fn sub3(lhs: [f32; 3], rhs: [f32; 3]) -> [f32; 3] {
    [lhs[0] - rhs[0], lhs[1] - rhs[1], lhs[2] - rhs[2]]
}

fn add3(target: &mut [f32; 3], value: [f32; 3]) {
    target[0] += value[0];
    target[1] += value[1];
    target[2] += value[2];
}

fn scale3(value: [f32; 3], scale: f32) -> [f32; 3] {
    [value[0] * scale, value[1] * scale, value[2] * scale]
}

fn dot3(lhs: [f32; 3], rhs: [f32; 3]) -> f32 {
    lhs[0] * rhs[0] + lhs[1] * rhs[1] + lhs[2] * rhs[2]
}

fn cross3(lhs: [f32; 3], rhs: [f32; 3]) -> [f32; 3] {
    [
        lhs[1] * rhs[2] - lhs[2] * rhs[1],
        lhs[2] * rhs[0] - lhs[0] * rhs[2],
        lhs[0] * rhs[1] - lhs[1] * rhs[0],
    ]
}

fn normalize3(value: [f32; 3]) -> [f32; 3] {
    let length = dot3(value, value).sqrt();
    if length > 0.0 {
        [value[0] / length, value[1] / length, value[2] / length]
    } else {
        [0.0, 0.0, 0.0]
    }
}
