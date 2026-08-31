use std::collections::HashMap;

use crate::core::framework::render::RenderMeshTopology;
use crate::core::math::Vec3;

use super::attribute::MeshAttributeValues;
use super::constants::{
    MESH_ATTRIBUTE_NORMAL, MESH_ATTRIBUTE_POSITION, MESH_ATTRIBUTE_TANGENT, MESH_ATTRIBUTE_UV0,
};
use super::indices::MeshIndices;
use super::mesh_asset::MeshAsset;
use super::validation::MeshValidationError;

impl MeshAsset {
    pub fn try_generate_missing_tangents(&mut self) -> Result<bool, MeshValidationError> {
        self.try_generate_missing_tangents_for_uv(MESH_ATTRIBUTE_UV0)
    }

    pub fn try_generate_missing_tangents_for_uv(
        &mut self,
        uv_attribute: &'static str,
    ) -> Result<bool, MeshValidationError> {
        self.validate()?;

        if self.attributes.contains_key(MESH_ATTRIBUTE_TANGENT) {
            return Ok(false);
        }
        if self.virtual_geometry.is_some() {
            return Err(MeshValidationError::TangentGenerationFailed {
                reason: "tangents must be generated before Virtual Geometry is cooked".to_string(),
            });
        }
        if self.topology != RenderMeshTopology::TriangleList {
            return Err(MeshValidationError::TangentGenerationRequiresTriangleList {
                topology: self.topology,
            });
        }

        let positions = self.positions()?;
        let normals = required_float32x3_attribute(self, MESH_ATTRIBUTE_NORMAL)?;
        let uvs = required_float32x2_attribute(self, uv_attribute)?;
        let corner_tangents =
            mikktspace_corner_tangents_for_mesh(positions, normals, uvs, self.indices.as_ref())?;
        publish_generated_corner_tangents(self, corner_tangents)?;
        Ok(true)
    }

    pub fn try_rebuild_morph_tangent_frames_for_uv(
        &mut self,
        rebuild_flat_normals: bool,
        tangent_uv_attribute: Option<&'static str>,
    ) -> Result<bool, MeshValidationError> {
        self.validate()?;
        if self.morph_targets.is_empty()
            || (!rebuild_flat_normals && tangent_uv_attribute.is_none())
        {
            return Ok(false);
        }
        if self.topology != RenderMeshTopology::TriangleList {
            return Err(MeshValidationError::TangentGenerationRequiresTriangleList {
                topology: self.topology,
            });
        }

        let base_positions = self.positions()?.to_vec();
        let base_normals = required_float32x3_attribute(self, MESH_ATTRIBUTE_NORMAL)?.to_vec();
        let indices = self.indices.clone();
        let tangent_inputs = tangent_uv_attribute
            .map(|uv_attribute| {
                Ok::<_, MeshValidationError>((
                    required_float32x2_attribute(self, uv_attribute)?.to_vec(),
                    required_float32x4_attribute(self, MESH_ATTRIBUTE_TANGENT)?.to_vec(),
                ))
            })
            .transpose()?;

        if rebuild_flat_normals {
            validate_face_corner_expanded_mesh(base_positions.len(), indices.as_ref())?;
        }
        let mut target_positions = base_positions.clone();
        let mut target_normals = base_normals.clone();
        let mut rebuilt_frames = Vec::with_capacity(self.morph_targets.len());
        for (target_index, target) in self.morph_targets.iter().enumerate() {
            target_positions.clone_from_slice(&base_positions);
            if let Some(position_deltas) =
                optional_morph_float32x3_attribute(target_index, target, MESH_ATTRIBUTE_POSITION)?
            {
                for (position, delta) in target_positions.iter_mut().zip(position_deltas) {
                    *position = (Vec3::from_array(*position) + Vec3::from_array(*delta)).to_array();
                }
            }

            target_normals.clone_from_slice(&base_normals);
            if rebuild_flat_normals {
                write_flat_corner_normals(&mut target_normals, &target_positions, indices.as_ref());
            } else if let Some(normal_deltas) =
                optional_morph_float32x3_attribute(target_index, target, MESH_ATTRIBUTE_NORMAL)?
            {
                for (normal, delta) in target_normals.iter_mut().zip(normal_deltas) {
                    *normal = (Vec3::from_array(*normal) + Vec3::from_array(*delta))
                        .normalize_or_zero()
                        .to_array();
                }
            }

            let normal_deltas =
                rebuild_flat_normals.then(|| vector_deltas(&target_normals, &base_normals));
            let tangent_deltas = tangent_inputs
                .as_ref()
                .map(|(uvs, base_tangents)| {
                    let target_corner_tangents = mikktspace_corner_tangents_for_mesh(
                        &target_positions,
                        &target_normals,
                        uvs,
                        indices.as_ref(),
                    )?;
                    let target_tangents = morph_vertex_tangents_from_corners(
                        &target_corner_tangents,
                        indices.as_ref(),
                        base_tangents,
                        target_index,
                    )?;
                    for (vertex_index, (base, target)) in
                        base_tangents.iter().zip(&target_tangents).enumerate()
                    {
                        if base[3].signum() != target[3].signum() {
                            return Err(MeshValidationError::MorphTangentHandednessMismatch {
                                target_index,
                                vertex_index,
                            });
                        }
                    }
                    Ok::<_, MeshValidationError>(
                        target_tangents
                            .iter()
                            .zip(base_tangents)
                            .map(|(target, base)| {
                                [
                                    target[0] - base[0],
                                    target[1] - base[1],
                                    target[2] - base[2],
                                ]
                            })
                            .collect(),
                    )
                })
                .transpose()?;
            rebuilt_frames.push((normal_deltas, tangent_deltas));
        }

        for (target, (normal_deltas, tangent_deltas)) in
            self.morph_targets.iter_mut().zip(rebuilt_frames)
        {
            if let Some(normal_deltas) = normal_deltas {
                target.attributes.insert(
                    MESH_ATTRIBUTE_NORMAL.to_string(),
                    MeshAttributeValues::Float32x3(normal_deltas),
                );
            }
            if let Some(tangent_deltas) = tangent_deltas {
                target.attributes.insert(
                    MESH_ATTRIBUTE_TANGENT.to_string(),
                    MeshAttributeValues::Float32x3(tangent_deltas),
                );
            } else if rebuild_flat_normals {
                target.attributes.remove(MESH_ATTRIBUTE_TANGENT);
            }
        }

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

fn required_float32x4_attribute<'a>(
    asset: &'a MeshAsset,
    name: &'static str,
) -> Result<&'a [[f32; 4]], MeshValidationError> {
    asset
        .attributes
        .get(name)
        .ok_or(MeshValidationError::TangentGenerationMissingAttribute { attribute: name })?
        .as_float32x4()
        .ok_or_else(|| MeshValidationError::InvalidAttributeFormat {
            attribute: name.to_string(),
            expected: "float32x4",
        })
}

fn optional_morph_float32x3_attribute<'a>(
    target_index: usize,
    target: &'a super::metadata::MeshMorphTargetAsset,
    name: &str,
) -> Result<Option<&'a [[f32; 3]]>, MeshValidationError> {
    target.attributes.get(name).map_or(Ok(None), |values| {
        values
            .as_float32x3()
            .map(Some)
            .ok_or_else(|| MeshValidationError::InvalidAttributeFormat {
                attribute: format!("morph_targets[{target_index}].{name}"),
                expected: "float32x3",
            })
    })
}

fn publish_generated_corner_tangents(
    asset: &mut MeshAsset,
    corner_tangents: Vec<[f32; 4]>,
) -> Result<(), MeshValidationError> {
    let vertex_count = asset.vertex_count()?;
    let expected_corner_count = asset
        .indices
        .as_ref()
        .map_or(vertex_count, MeshIndices::len);
    if corner_tangents.len() != expected_corner_count {
        return Err(MeshValidationError::TangentGenerationFailed {
            reason: format!(
                "MikkTSpace produced {} corner tangents for {expected_corner_count} mesh elements",
                corner_tangents.len()
            ),
        });
    }
    if corner_tangents
        .iter()
        .flatten()
        .any(|component| !component.is_finite())
    {
        return Err(MeshValidationError::TangentGenerationFailed {
            reason: "MikkTSpace produced a non-finite corner tangent".to_string(),
        });
    }
    let Some(indices) = asset.indices.as_ref() else {
        return insert_generated_tangents(asset, corner_tangents);
    };

    let source_indices = indices.to_u32_vec();
    let mut split_sources = Vec::new();
    let mut output_tangents = vec![[1.0, 0.0, 0.0, 1.0]; vertex_count];
    let mut primary_group = vec![None; vertex_count];
    let mut extra_groups =
        HashMap::with_capacity(source_indices.len().saturating_sub(vertex_count));
    let mut output_indices = Vec::with_capacity(source_indices.len());

    for (source_index, tangent) in source_indices.iter().copied().zip(corner_tangents) {
        let source_index = source_index as usize;
        let tangent_key = tangent_bits(tangent);
        let output_index = match primary_group[source_index] {
            None => {
                primary_group[source_index] = Some(tangent_key);
                output_tangents[source_index] = tangent;
                source_index
            }
            Some(primary_key) if primary_key == tangent_key => source_index,
            Some(_) => {
                let group_key = (source_index, tangent_key);
                if let Some(output_index) = extra_groups.get(&group_key) {
                    *output_index
                } else {
                    let output_index =
                        vertex_count
                            .checked_add(split_sources.len())
                            .ok_or_else(|| MeshValidationError::TangentGenerationFailed {
                                reason: "MikkTSpace vertex splitting exceeded the usize range"
                                    .to_string(),
                            })?;
                    u32::try_from(output_index).map_err(|_| {
                        MeshValidationError::TangentGenerationFailed {
                            reason: "MikkTSpace vertex splitting exceeded the u32 index range"
                                .to_string(),
                        }
                    })?;
                    split_sources.push(source_index);
                    output_tangents.push(tangent);
                    extra_groups.insert(group_key, output_index);
                    output_index
                }
            }
        };
        output_indices.push(output_index as u32);
    }

    if split_sources.is_empty() {
        return insert_generated_tangents(asset, output_tangents);
    }
    if asset.mesh_sdf.is_some() {
        return Err(MeshValidationError::TangentGenerationFailed {
            reason: "MikkTSpace vertex splitting must complete before Mesh SDF is cooked"
                .to_string(),
        });
    }
    let remapped_indices = remap_index_format(indices, output_indices);

    for values in asset.attributes.values_mut() {
        append_remapped_attribute_values(values, &split_sources);
    }
    for target in &mut asset.morph_targets {
        for values in target.attributes.values_mut() {
            append_remapped_attribute_values(values, &split_sources);
        }
    }
    let previous_tangents = asset.attributes.insert(
        MESH_ATTRIBUTE_TANGENT.to_string(),
        MeshAttributeValues::Float32x4(output_tangents),
    );
    let previous_indices = asset.indices.replace(remapped_indices);
    if let Err(error) = asset.validate() {
        if let Some(previous_tangents) = previous_tangents {
            asset
                .attributes
                .insert(MESH_ATTRIBUTE_TANGENT.to_string(), previous_tangents);
        } else {
            asset.attributes.remove(MESH_ATTRIBUTE_TANGENT);
        }
        for values in asset.attributes.values_mut() {
            truncate_attribute_values(values, vertex_count);
        }
        for target in &mut asset.morph_targets {
            for values in target.attributes.values_mut() {
                truncate_attribute_values(values, vertex_count);
            }
        }
        asset.indices = previous_indices;
        return Err(error);
    }
    Ok(())
}

fn insert_generated_tangents(
    asset: &mut MeshAsset,
    tangents: Vec<[f32; 4]>,
) -> Result<(), MeshValidationError> {
    let previous = asset.attributes.insert(
        MESH_ATTRIBUTE_TANGENT.to_string(),
        MeshAttributeValues::Float32x4(tangents),
    );
    if let Err(error) = asset.validate() {
        if let Some(previous) = previous {
            asset
                .attributes
                .insert(MESH_ATTRIBUTE_TANGENT.to_string(), previous);
        } else {
            asset.attributes.remove(MESH_ATTRIBUTE_TANGENT);
        }
        return Err(error);
    }
    Ok(())
}

fn append_remapped_attribute_values(values: &mut MeshAttributeValues, split_sources: &[usize]) {
    macro_rules! append {
        ($values:expr) => {{
            let appended = split_sources
                .iter()
                .map(|source_index| $values[*source_index])
                .collect::<Vec<_>>();
            $values.extend(appended);
        }};
    }
    match values {
        MeshAttributeValues::Float32x2(values) => append!(values),
        MeshAttributeValues::Float32x3(values) => append!(values),
        MeshAttributeValues::Float32x4(values) => append!(values),
        MeshAttributeValues::Uint16x4(values) => append!(values),
        MeshAttributeValues::Uint32x4(values) => append!(values),
    }
}

fn truncate_attribute_values(values: &mut MeshAttributeValues, vertex_count: usize) {
    match values {
        MeshAttributeValues::Float32x2(values) => values.truncate(vertex_count),
        MeshAttributeValues::Float32x3(values) => values.truncate(vertex_count),
        MeshAttributeValues::Float32x4(values) => values.truncate(vertex_count),
        MeshAttributeValues::Uint16x4(values) => values.truncate(vertex_count),
        MeshAttributeValues::Uint32x4(values) => values.truncate(vertex_count),
    }
}

fn remap_index_format(source: &MeshIndices, indices: Vec<u32>) -> MeshIndices {
    match source {
        MeshIndices::U16(_) if indices.iter().all(|index| u16::try_from(*index).is_ok()) => {
            MeshIndices::U16(indices.into_iter().map(|index| index as u16).collect())
        }
        MeshIndices::U16(_) | MeshIndices::U32(_) => MeshIndices::U32(indices),
    }
}

fn morph_vertex_tangents_from_corners(
    corner_tangents: &[[f32; 4]],
    indices: Option<&MeshIndices>,
    base_tangents: &[[f32; 4]],
    target_index: usize,
) -> Result<Vec<[f32; 4]>, MeshValidationError> {
    let mut target_tangents = base_tangents.to_vec();
    let mut assigned = vec![false; base_tangents.len()];
    for (corner_index, tangent) in corner_tangents.iter().copied().enumerate() {
        let vertex_index = mesh_vertex_index(indices, corner_index);
        if assigned[vertex_index]
            && tangent_bits(target_tangents[vertex_index]) != tangent_bits(tangent)
        {
            return Err(MeshValidationError::MorphTangentCornerMismatch {
                target_index,
                vertex_index,
            });
        }
        target_tangents[vertex_index] = tangent;
        assigned[vertex_index] = true;
    }
    Ok(target_tangents)
}

fn tangent_bits(tangent: [f32; 4]) -> [u32; 4] {
    tangent.map(|component| {
        if component == 0.0 {
            0.0_f32.to_bits()
        } else {
            component.to_bits()
        }
    })
}

fn validate_face_corner_expanded_mesh(
    vertex_count: usize,
    indices: Option<&MeshIndices>,
) -> Result<(), MeshValidationError> {
    let element_count = indices.map_or(vertex_count, MeshIndices::len);
    let mut visited = vec![false; vertex_count];
    for element in 0..element_count {
        let vertex_index = mesh_vertex_index(indices, element);
        if visited[vertex_index] {
            return Err(MeshValidationError::TangentGenerationFailed {
                reason: "flat morph normal rebuild requires face-corner-expanded vertices"
                    .to_string(),
            });
        }
        visited[vertex_index] = true;
    }
    if visited.iter().any(|visited| !visited) {
        return Err(MeshValidationError::TangentGenerationFailed {
            reason: "flat morph normal rebuild found an unreferenced vertex".to_string(),
        });
    }
    Ok(())
}

fn write_flat_corner_normals(
    normals: &mut [[f32; 3]],
    positions: &[[f32; 3]],
    indices: Option<&MeshIndices>,
) {
    let element_count = indices.map_or(positions.len(), MeshIndices::len);
    for triangle in 0..element_count / 3 {
        let vertex_indices = [
            mesh_vertex_index(indices, triangle * 3),
            mesh_vertex_index(indices, triangle * 3 + 1),
            mesh_vertex_index(indices, triangle * 3 + 2),
        ];
        let [a, b, c] = vertex_indices.map(|index| Vec3::from_array(positions[index]));
        let mut face_normal = (b - a).cross(c - a).normalize_or_zero();
        if face_normal.length_squared() <= f32::EPSILON {
            face_normal = Vec3::Y;
        }
        for vertex_index in vertex_indices {
            normals[vertex_index] = face_normal.to_array();
        }
    }
}

fn mesh_vertex_index(indices: Option<&MeshIndices>, element: usize) -> usize {
    match indices {
        Some(MeshIndices::U16(indices)) => indices[element] as usize,
        Some(MeshIndices::U32(indices)) => indices[element] as usize,
        None => element,
    }
}

fn vector_deltas(target: &[[f32; 3]], base: &[[f32; 3]]) -> Vec<[f32; 3]> {
    target
        .iter()
        .zip(base)
        .map(|(target, base)| {
            [
                target[0] - base[0],
                target[1] - base[1],
                target[2] - base[2],
            ]
        })
        .collect()
}

struct MikktspaceMesh<'a> {
    indices: Option<&'a MeshIndices>,
    positions: &'a [[f32; 3]],
    normals: &'a [[f32; 3]],
    uvs: &'a [[f32; 2]],
    corner_tangents: Vec<[f32; 4]>,
}

impl MikktspaceMesh<'_> {
    fn index(&self, face: usize, vertex: usize) -> usize {
        let index_index = face * 3 + vertex;
        match self.indices {
            Some(MeshIndices::U16(indices)) => indices[index_index] as usize,
            Some(MeshIndices::U32(indices)) => indices[index_index] as usize,
            None => index_index,
        }
    }
}

impl bevy_mikktspace::Geometry for MikktspaceMesh<'_> {
    fn num_faces(&self) -> usize {
        self.indices
            .map(|indices| match indices {
                MeshIndices::U16(indices) => indices.len(),
                MeshIndices::U32(indices) => indices.len(),
            })
            .unwrap_or(self.positions.len())
            / 3
    }

    fn num_vertices_of_face(&self, _face: usize) -> usize {
        3
    }

    fn position(&self, face: usize, vertex: usize) -> [f32; 3] {
        self.positions[self.index(face, vertex)]
    }

    fn normal(&self, face: usize, vertex: usize) -> [f32; 3] {
        self.normals[self.index(face, vertex)]
    }

    fn tex_coord(&self, face: usize, vertex: usize) -> [f32; 2] {
        self.uvs[self.index(face, vertex)]
    }

    fn set_tangent(
        &mut self,
        tangent_space: Option<bevy_mikktspace::TangentSpace>,
        face: usize,
        vertex: usize,
    ) {
        self.corner_tangents[face * 3 + vertex] =
            tangent_space.unwrap_or_default().tangent_encoded();
    }
}

fn mikktspace_corner_tangents_for_mesh(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: Option<&MeshIndices>,
) -> Result<Vec<[f32; 4]>, MeshValidationError> {
    let corner_count = indices.map_or(positions.len(), MeshIndices::len);
    let mut geometry = MikktspaceMesh {
        indices,
        positions,
        normals,
        uvs,
        corner_tangents: vec![[0.0; 4]; corner_count],
    };
    bevy_mikktspace::generate_tangents(&mut geometry).map_err(|error| {
        MeshValidationError::TangentGenerationFailed {
            reason: error.to_string(),
        }
    })?;

    // MikkTSpace encodes a left-handed basis; Zircon reconstructs a right-handed basis.
    for tangent in &mut geometry.corner_tangents {
        tangent[3] = -tangent[3];
    }
    Ok(geometry.corner_tangents)
}
