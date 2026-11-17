//! 切线生成采用 MikkTSpace 兼容约定，供法线贴图与形变上传共用；调用前需要有效位置、UV、法线和三角形拓扑。

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

const MISSING_TANGENT: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

type MorphFrameDeltas = (Option<Vec<[f32; 3]>>, Option<Vec<[f32; 3]>>);

impl MeshAsset {
    pub fn try_generate_missing_tangents(&mut self) -> Result<bool, MeshValidationError> {
        self.try_generate_missing_tangents_for_uv(MESH_ATTRIBUTE_UV0)
    }

    pub fn try_generate_missing_tangents_for_uv(
        &mut self,
        uv_attribute: &'static str,
    ) -> Result<bool, MeshValidationError> {
        Ok(self
            .generate_missing_tangent_basis(uv_attribute, false)?
            .is_some())
    }

    /// Generates a missing base tangent stream and rebuilds its morph frames together.
    /// An undefined generated base sign can take the shared sign of its valid targets.
    /// Existing base tangents are not changed; any error leaves all streams unchanged.
    pub fn try_generate_missing_tangents_and_morph_frames_for_uv(
        &mut self,
        uv_attribute: &'static str,
        rebuild_flat_normals: bool,
    ) -> Result<bool, MeshValidationError> {
        let has_targets = !self.morph_targets.is_empty();
        let Some(mut generated) = self.generate_missing_tangent_basis(uv_attribute, has_targets)?
        else {
            return Ok(false);
        };
        if !has_targets {
            return Ok(true);
        }
        let frames = match self.stage_morph_frames(
            rebuild_flat_normals,
            Some(uv_attribute),
            Some(&mut generated.signs),
        ) {
            Ok(frames) => frames,
            Err(error) => {
                generated.rollback(self);
                return Err(error);
            }
        };
        let Some(MeshAttributeValues::Float32x4(base_tangents)) =
            self.attributes.get_mut(MESH_ATTRIBUTE_TANGENT)
        else {
            unreachable!("base tangents were installed before staging morph frames");
        };
        for (base, sign) in base_tangents.iter_mut().zip(generated.signs) {
            if let Some(sign) = sign {
                base[3] = sign;
            }
        }
        self.publish_morph_frames(rebuild_flat_normals, frames);
        Ok(true)
    }

    fn generate_missing_tangent_basis(
        &mut self,
        uv_attribute: &'static str,
        track_signs: bool,
    ) -> Result<Option<GeneratedTangentBasis>, MeshValidationError> {
        self.validate()?;

        if self.attributes.contains_key(MESH_ATTRIBUTE_TANGENT) {
            return Ok(None);
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
        publish_generated_corner_tangents(self, corner_tangents, track_signs).map(Some)
    }

    /// Rebuilds targets against an existing, fixed base tangent stream. To let
    /// a previously undefined generated base inherit a sign, use the combined
    /// generation method before a base stream has been installed.
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
        let frames = self.stage_morph_frames(rebuild_flat_normals, tangent_uv_attribute, None)?;
        self.publish_morph_frames(rebuild_flat_normals, frames);
        Ok(true)
    }

    fn stage_morph_frames(
        &self,
        rebuild_flat_normals: bool,
        tangent_uv_attribute: Option<&'static str>,
        mut generated_signs: Option<&mut [Option<f32>]>,
    ) -> Result<Vec<MorphFrameDeltas>, MeshValidationError> {
        if self.topology != RenderMeshTopology::TriangleList {
            return Err(MeshValidationError::TangentGenerationRequiresTriangleList {
                topology: self.topology,
            });
        }

        // Publish only after every target succeeds, so the base streams stay borrowed.
        let base_positions = self.positions()?;
        let base_normals = required_float32x3_attribute(self, MESH_ATTRIBUTE_NORMAL)?;
        let indices = self.indices.as_ref();
        let tangent_inputs = tangent_uv_attribute
            .map(|uv_attribute| {
                Ok::<_, MeshValidationError>((
                    required_float32x2_attribute(self, uv_attribute)?,
                    required_float32x4_attribute(self, MESH_ATTRIBUTE_TANGENT)?,
                ))
            })
            .transpose()?;

        if rebuild_flat_normals {
            validate_face_corner_expanded_mesh(base_positions.len(), indices)?;
        }
        let mut target_positions = base_positions.to_vec();
        let mut target_normals = base_normals.to_vec();
        let mut rebuilt_frames = Vec::with_capacity(self.morph_targets.len());
        for (target_index, target) in self.morph_targets.iter().enumerate() {
            target_positions.clone_from_slice(base_positions);
            if let Some(position_deltas) =
                optional_morph_float32x3_attribute(target_index, target, MESH_ATTRIBUTE_POSITION)?
            {
                for (position, delta) in target_positions.iter_mut().zip(position_deltas) {
                    *position = (Vec3::from_array(*position) + Vec3::from_array(*delta)).to_array();
                }
            }

            target_normals.clone_from_slice(base_normals);
            if rebuild_flat_normals {
                write_flat_corner_normals(&mut target_normals, &target_positions, indices);
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
                rebuild_flat_normals.then(|| vector_deltas(&target_normals, base_normals));
            let tangent_deltas = tangent_inputs
                .map(|(uvs, base_tangents)| {
                    let target_corner_tangents = mikktspace_corner_tangents_for_mesh(
                        &target_positions,
                        &target_normals,
                        uvs,
                        indices,
                    )?;
                    morph_vertex_tangent_deltas_from_corners(
                        &target_corner_tangents,
                        indices,
                        base_tangents,
                        generated_signs.as_deref_mut(),
                        target_index,
                    )
                })
                .transpose()?;
            rebuilt_frames.push((normal_deltas, tangent_deltas));
        }
        Ok(rebuilt_frames)
    }

    fn publish_morph_frames(&mut self, rebuild_flat_normals: bool, frames: Vec<MorphFrameDeltas>) {
        // Staging preserves vertex counts and constructs the exact stream formats.
        for (target, (normal_deltas, tangent_deltas)) in self.morph_targets.iter_mut().zip(frames) {
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

struct GeneratedTangentBasis {
    original_vertex_count: usize,
    previous_indices: Option<MeshIndices>,
    signs: Vec<Option<f32>>,
}

impl GeneratedTangentBasis {
    fn rollback(self, asset: &mut MeshAsset) {
        asset.attributes.remove(MESH_ATTRIBUTE_TANGENT);
        if let Some(indices) = self.previous_indices {
            for values in asset.attributes.values_mut() {
                truncate_attribute_values(values, self.original_vertex_count);
            }
            for target in &mut asset.morph_targets {
                for values in target.attributes.values_mut() {
                    truncate_attribute_values(values, self.original_vertex_count);
                }
            }
            asset.indices = Some(indices);
        }
    }
}

fn publish_generated_corner_tangents(
    asset: &mut MeshAsset,
    corner_tangents: Vec<Option<[f32; 4]>>,
    track_signs: bool,
) -> Result<GeneratedTangentBasis, MeshValidationError> {
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
    let mut basis = GeneratedTangentBasis {
        original_vertex_count: vertex_count,
        previous_indices: None,
        signs: if track_signs {
            vec![None; vertex_count]
        } else {
            Vec::new()
        },
    };
    let Some(indices) = asset.indices.as_ref() else {
        let tangents = corner_tangents
            .into_iter()
            .enumerate()
            .map(|(vertex, computed)| {
                if track_signs {
                    basis.signs[vertex] = computed.map(|tangent| tangent[3]);
                }
                computed.unwrap_or(MISSING_TANGENT)
            })
            .collect();
        return install_generated_tangents(asset, tangents, basis);
    };

    let mut split_sources = Vec::new();
    let mut output_tangents = vec![MISSING_TANGENT; vertex_count];
    let mut primary_group = vec![None; vertex_count];
    let mut extra_groups = HashMap::new();
    let mut output_indices: Option<Vec<u32>> = None;

    for (corner_index, computed) in corner_tangents.into_iter().enumerate() {
        let tangent = computed.unwrap_or(MISSING_TANGENT);
        let source_index = mesh_vertex_index(Some(indices), corner_index);
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
                    if track_signs {
                        basis.signs.push(None);
                    }
                    extra_groups.insert(group_key, output_index);
                    output_index
                }
            }
        };
        // Keep the existing split key. If both a fallback and a computed
        // corner land in the same group, only the latter constrains the sign.
        if track_signs {
            if let Some(computed) = computed {
                basis.signs[output_index] = Some(computed[3]);
            }
        }
        if output_index != source_index || output_indices.is_some() {
            // Before the first split every output index equals its source index.
            let output_indices = output_indices.get_or_insert_with(|| {
                let mut output_indices = Vec::with_capacity(indices.len());
                output_indices.extend(
                    (0..corner_index).map(|corner| mesh_vertex_index(Some(indices), corner) as u32),
                );
                output_indices
            });
            output_indices.push(output_index as u32);
        }
    }

    let Some(output_indices) = output_indices else {
        return install_generated_tangents(asset, output_tangents, basis);
    };
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
    basis.previous_indices = asset.indices.replace(remapped_indices);
    install_generated_tangents(asset, output_tangents, basis)
}

fn install_generated_tangents(
    asset: &mut MeshAsset,
    tangents: Vec<[f32; 4]>,
    basis: GeneratedTangentBasis,
) -> Result<GeneratedTangentBasis, MeshValidationError> {
    debug_assert!(!asset.attributes.contains_key(MESH_ATTRIBUTE_TANGENT));
    asset.attributes.insert(
        MESH_ATTRIBUTE_TANGENT.to_string(),
        MeshAttributeValues::Float32x4(tangents),
    );
    if let Err(error) = asset.validate() {
        basis.rollback(asset);
        return Err(error);
    }
    Ok(basis)
}

fn append_remapped_attribute_values(values: &mut MeshAttributeValues, split_sources: &[usize]) {
    macro_rules! append {
        ($values:expr) => {{
            $values.reserve(split_sources.len());
            for source_index in split_sources {
                let value = $values[*source_index];
                $values.push(value);
            }
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

#[derive(Clone, Copy)]
enum MorphVertexBasis {
    Unreferenced,
    Missing,
    Computed([f32; 4]),
}

fn morph_vertex_tangent_deltas_from_corners(
    corner_tangents: &[Option<[f32; 4]>],
    indices: Option<&MeshIndices>,
    base_tangents: &[[f32; 4]],
    mut generated_signs: Option<&mut [Option<f32>]>,
    target_index: usize,
) -> Result<Vec<[f32; 3]>, MeshValidationError> {
    let mut targets = vec![MorphVertexBasis::Unreferenced; base_tangents.len()];
    for (corner_index, tangent) in corner_tangents.iter().copied().enumerate() {
        let vertex_index = mesh_vertex_index(indices, corner_index);
        let target = &mut targets[vertex_index];
        match (*target, tangent) {
            (MorphVertexBasis::Computed(previous), Some(next))
                if tangent_bits(previous) != tangent_bits(next) =>
            {
                return Err(MeshValidationError::MorphTangentCornerMismatch {
                    target_index,
                    vertex_index,
                });
            }
            (_, Some(computed)) => *target = MorphVertexBasis::Computed(computed),
            (MorphVertexBasis::Unreferenced, None) => *target = MorphVertexBasis::Missing,
            (_, None) => {}
        }
    }
    targets
        .into_iter()
        .zip(base_tangents)
        .enumerate()
        .map(|(vertex_index, (target, base))| {
            let target = match target {
                MorphVertexBasis::Unreferenced => return Ok([0.0; 3]),
                MorphVertexBasis::Missing => MISSING_TANGENT,
                MorphVertexBasis::Computed(target) => {
                    let sign = target[3];
                    let previous = match generated_signs.as_deref_mut() {
                        Some(signs) => *signs[vertex_index].get_or_insert(sign),
                        None => base[3].signum(),
                    };
                    if previous != sign {
                        return Err(MeshValidationError::MorphTangentHandednessMismatch {
                            target_index,
                            vertex_index,
                        });
                    }
                    target
                }
            };
            Ok([
                target[0] - base[0],
                target[1] - base[1],
                target[2] - base[2],
            ])
        })
        .collect()
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
    corner_tangents: Vec<Option<[f32; 4]>>,
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
        // The encoded sign already matches ZE's cross(N,T)*w convention for
        // the positions, normals and UVs we pass unmodified to MikkTSpace.
        self.corner_tangents[face * 3 + vertex] =
            tangent_space.map(|basis| basis.tangent_encoded());
    }
}

fn mikktspace_corner_tangents_for_mesh(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: Option<&MeshIndices>,
) -> Result<Vec<Option<[f32; 4]>>, MeshValidationError> {
    let corner_count = indices.map_or(positions.len(), MeshIndices::len);
    let mut geometry = MikktspaceMesh {
        indices,
        positions,
        normals,
        uvs,
        corner_tangents: vec![None; corner_count],
    };
    bevy_mikktspace::generate_tangents(&mut geometry).map_err(|error| {
        MeshValidationError::TangentGenerationFailed {
            reason: error.to_string(),
        }
    })?;

    if geometry
        .corner_tangents
        .iter()
        .flatten()
        .flatten()
        .any(|value| !value.is_finite())
    {
        return Err(MeshValidationError::TangentGenerationFailed {
            reason: "MikkTSpace produced a non-finite corner tangent".to_string(),
        });
    }
    Ok(geometry.corner_tangents)
}
