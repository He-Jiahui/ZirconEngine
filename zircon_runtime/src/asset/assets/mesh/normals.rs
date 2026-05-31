use crate::core::framework::render::RenderMeshTopology;

use super::attribute::MeshAttributeValues;
use super::constants::MESH_ATTRIBUTE_NORMAL;
use super::indices::MeshIndices;
use super::mesh_asset::MeshAsset;
use super::validation::MeshValidationError;

impl MeshAsset {
    pub fn try_generate_missing_normals(&mut self) -> Result<bool, MeshValidationError> {
        self.validate()?;

        if self.attributes.contains_key(MESH_ATTRIBUTE_NORMAL) {
            return Ok(false);
        }
        ensure_triangle_list_topology(self.topology)?;

        let normals = match &self.indices {
            Some(indices) => smooth_normals_for_positions_and_indices(self.positions()?, indices),
            None => flat_normals_for_positions(self.positions()?),
        };
        self.insert_generated_normals(normals)
    }

    pub fn try_generate_missing_flat_normals(&mut self) -> Result<bool, MeshValidationError> {
        self.validate()?;

        if self.attributes.contains_key(MESH_ATTRIBUTE_NORMAL) {
            return Ok(false);
        }
        ensure_triangle_list_topology(self.topology)?;
        if self.indices.is_some() {
            return Err(MeshValidationError::FlatNormalGenerationRequiresUnindexedMesh);
        }

        let normals = flat_normals_for_positions(self.positions()?);
        self.insert_generated_normals(normals)
    }

    pub fn try_generate_missing_smooth_normals(&mut self) -> Result<bool, MeshValidationError> {
        self.validate()?;

        if self.attributes.contains_key(MESH_ATTRIBUTE_NORMAL) {
            return Ok(false);
        }
        ensure_triangle_list_topology(self.topology)?;
        let indices = self
            .indices
            .as_ref()
            .ok_or(MeshValidationError::SmoothNormalGenerationRequiresIndexedMesh)?;

        let normals = smooth_normals_for_positions_and_indices(self.positions()?, indices);
        self.insert_generated_normals(normals)
    }

    fn insert_generated_normals(
        &mut self,
        normals: Vec<[f32; 3]>,
    ) -> Result<bool, MeshValidationError> {
        self.attributes.insert(
            MESH_ATTRIBUTE_NORMAL.to_string(),
            MeshAttributeValues::Float32x3(normals),
        );
        self.validate()?;
        Ok(true)
    }
}

fn ensure_triangle_list_topology(topology: RenderMeshTopology) -> Result<(), MeshValidationError> {
    if topology == RenderMeshTopology::TriangleList {
        Ok(())
    } else {
        Err(MeshValidationError::NormalGenerationRequiresTriangleList { topology })
    }
}

fn flat_normals_for_positions(positions: &[[f32; 3]]) -> Vec<[f32; 3]> {
    let mut normals = Vec::with_capacity(positions.len());
    for triangle in positions.chunks_exact(3) {
        let normal = triangle_normal(triangle[0], triangle[1], triangle[2]);
        normals.extend([normal, normal, normal]);
    }
    normals
}

fn smooth_normals_for_positions_and_indices(
    positions: &[[f32; 3]],
    indices: &MeshIndices,
) -> Vec<[f32; 3]> {
    let mut normal_sums = vec![[0.0, 0.0, 0.0]; positions.len()];
    for triangle in indices.to_u32_vec().chunks_exact(3) {
        accumulate_angle_weighted_triangle_normal(
            [
                triangle[0] as usize,
                triangle[1] as usize,
                triangle[2] as usize,
            ],
            positions,
            &mut normal_sums,
        );
    }
    normal_sums.into_iter().map(normalize).collect()
}

fn accumulate_angle_weighted_triangle_normal(
    [a, b, c]: [usize; 3],
    positions: &[[f32; 3]],
    normal_sums: &mut [[f32; 3]],
) {
    let pa = positions[a];
    let pb = positions[b];
    let pc = positions[c];

    let ab = sub(pb, pa);
    let ba = sub(pa, pb);
    let bc = sub(pc, pb);
    let cb = sub(pb, pc);
    let ca = sub(pa, pc);
    let ac = sub(pc, pa);
    let normal = triangle_normal(pa, pb, pc);

    add_scaled(&mut normal_sums[a], normal, corner_angle(ab, ac));
    add_scaled(&mut normal_sums[b], normal, corner_angle(ba, bc));
    add_scaled(&mut normal_sums[c], normal, corner_angle(ca, cb));
}

fn triangle_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    normalize(cross(sub(b, a), sub(c, a)))
}

fn sub(lhs: [f32; 3], rhs: [f32; 3]) -> [f32; 3] {
    [lhs[0] - rhs[0], lhs[1] - rhs[1], lhs[2] - rhs[2]]
}

fn cross(lhs: [f32; 3], rhs: [f32; 3]) -> [f32; 3] {
    [
        lhs[1] * rhs[2] - lhs[2] * rhs[1],
        lhs[2] * rhs[0] - lhs[0] * rhs[2],
        lhs[0] * rhs[1] - lhs[1] * rhs[0],
    ]
}

fn dot(lhs: [f32; 3], rhs: [f32; 3]) -> f32 {
    lhs[0] * rhs[0] + lhs[1] * rhs[1] + lhs[2] * rhs[2]
}

fn length_squared(vector: [f32; 3]) -> f32 {
    dot(vector, vector)
}

fn corner_angle(lhs: [f32; 3], rhs: [f32; 3]) -> f32 {
    let length_product = length_squared(lhs) * length_squared(rhs);
    if length_product <= f32::EPSILON {
        return 0.0;
    }

    (dot(lhs, rhs) / length_product.sqrt())
        .clamp(-1.0, 1.0)
        .acos()
}

fn add_scaled(target: &mut [f32; 3], vector: [f32; 3], scale: f32) {
    target[0] += vector[0] * scale;
    target[1] += vector[1] * scale;
    target[2] += vector[2] * scale;
}

fn normalize(vector: [f32; 3]) -> [f32; 3] {
    let length = (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt();
    if length > 0.0 {
        [vector[0] / length, vector[1] / length, vector[2] / length]
    } else {
        [0.0, 0.0, 0.0]
    }
}
