use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::asset::{AssetUri, MeshVertex};
use crate::core::framework::render::{
    RenderMeshBounds, RenderMeshDescriptor, RenderMeshKind, RenderMeshTopology,
};
use crate::core::math::Vec3;
use crate::core::resource::ResourceId;

use super::super::model::{ModelPrimitiveAsset, VirtualGeometryAsset};
use super::{
    MeshAssetUsage, MeshAttributeSummary, MeshAttributeValues, MeshIndexFormat, MeshIndices,
    MeshMorphTargetAsset, MeshMorphTargetAttributeSummary, MeshSkinAsset, MeshValidationError,
    MESH_ATTRIBUTE_COLOR, MESH_ATTRIBUTE_JOINT_INDEX, MESH_ATTRIBUTE_JOINT_WEIGHT,
    MESH_ATTRIBUTE_NORMAL, MESH_ATTRIBUTE_POSITION, MESH_ATTRIBUTE_TANGENT, MESH_ATTRIBUTE_UV0,
};

const DEFAULT_NORMAL: [f32; 3] = [0.0, 0.0, 1.0];
const DEFAULT_UV: [f32; 2] = [0.0, 0.0];
const DEFAULT_JOINT_INDICES: [u16; 4] = [0, 0, 0, 0];
const DEFAULT_JOINT_WEIGHTS: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeshAssetOverview {
    pub uri: AssetUri,
    pub topology: RenderMeshTopology,
    pub bounds: RenderMeshBounds,
    pub vertex_count: usize,
    pub index_count: usize,
    pub index_format: Option<MeshIndexFormat>,
    pub draw_element_count: usize,
    pub render_primitive_count: usize,
    pub attribute_count: usize,
    pub attributes: Vec<MeshAttributeSummary>,
    pub morph_target_count: usize,
    pub morph_target_attribute_count: usize,
    pub morph_target_attributes: Vec<MeshMorphTargetAttributeSummary>,
    pub has_skin: bool,
    pub inverse_bind_matrix_count: usize,
    pub has_virtual_geometry_payload: bool,
    pub asset_usage: MeshAssetUsage,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeshAssetManagementRecord {
    pub mesh_id: ResourceId,
    pub overview: MeshAssetOverview,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeshAssetManagementRecordFailure {
    pub mesh_id: ResourceId,
    pub diagnostic: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeshAssetManagementRecordSetSummary {
    pub mesh_count: usize,
    pub valid_mesh_count: usize,
    pub invalid_mesh_count: usize,
    pub vertex_count: usize,
    pub index_count: usize,
    pub draw_element_count: usize,
    pub render_primitive_count: usize,
    pub attribute_count: usize,
    pub morph_target_count: usize,
    pub morph_target_attribute_count: usize,
    pub skinned_mesh_count: usize,
    pub inverse_bind_matrix_count: usize,
    pub virtual_geometry_mesh_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeshAssetManagementRecordSet {
    pub records: Vec<MeshAssetManagementRecord>,
    pub failures: Vec<MeshAssetManagementRecordFailure>,
    pub summary: MeshAssetManagementRecordSetSummary,
}

impl MeshAssetManagementRecordSetSummary {
    pub fn from_records_and_failures(
        records: &[MeshAssetManagementRecord],
        failures: &[MeshAssetManagementRecordFailure],
    ) -> Self {
        Self {
            mesh_count: records.len() + failures.len(),
            valid_mesh_count: records.len(),
            invalid_mesh_count: failures.len(),
            vertex_count: records
                .iter()
                .map(|record| record.overview.vertex_count)
                .sum(),
            index_count: records
                .iter()
                .map(|record| record.overview.index_count)
                .sum(),
            draw_element_count: records
                .iter()
                .map(|record| record.overview.draw_element_count)
                .sum(),
            render_primitive_count: records
                .iter()
                .map(|record| record.overview.render_primitive_count)
                .sum(),
            attribute_count: records
                .iter()
                .map(|record| record.overview.attribute_count)
                .sum(),
            morph_target_count: records
                .iter()
                .map(|record| record.overview.morph_target_count)
                .sum(),
            morph_target_attribute_count: records
                .iter()
                .map(|record| record.overview.morph_target_attribute_count)
                .sum(),
            skinned_mesh_count: records
                .iter()
                .filter(|record| record.overview.has_skin)
                .count(),
            inverse_bind_matrix_count: records
                .iter()
                .map(|record| record.overview.inverse_bind_matrix_count)
                .sum(),
            virtual_geometry_mesh_count: records
                .iter()
                .filter(|record| record.overview.has_virtual_geometry_payload)
                .count(),
        }
    }
}

impl MeshAssetManagementRecordSet {
    pub fn from_results(
        mut results: Vec<(
            ResourceId,
            Result<MeshAssetManagementRecord, MeshValidationError>,
        )>,
    ) -> Self {
        results.sort_by_key(|(mesh_id, _)| *mesh_id);
        let mut records = Vec::new();
        let mut failures = Vec::new();
        for (mesh_id, result) in results {
            match result {
                Ok(record) => records.push(record),
                Err(error) => failures.push(MeshAssetManagementRecordFailure {
                    mesh_id,
                    diagnostic: error.to_string(),
                }),
            }
        }
        let summary =
            MeshAssetManagementRecordSetSummary::from_records_and_failures(&records, &failures);
        Self {
            records,
            failures,
            summary,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeshAsset {
    pub uri: AssetUri,
    #[serde(default)]
    pub topology: RenderMeshTopology,
    #[serde(default)]
    pub attributes: BTreeMap<String, MeshAttributeValues>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indices: Option<MeshIndices>,
    #[serde(default)]
    pub asset_usage: MeshAssetUsage,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub morph_targets: Vec<MeshMorphTargetAsset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skin: Option<MeshSkinAsset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub virtual_geometry: Option<VirtualGeometryAsset>,
}

impl MeshAsset {
    pub fn new(
        uri: AssetUri,
        topology: RenderMeshTopology,
        attributes: BTreeMap<String, MeshAttributeValues>,
        indices: Option<MeshIndices>,
    ) -> Result<Self, MeshValidationError> {
        let asset = Self {
            uri,
            topology,
            attributes,
            indices,
            asset_usage: MeshAssetUsage::default(),
            morph_targets: Vec::new(),
            skin: None,
            virtual_geometry: None,
        };
        asset.validate()?;
        Ok(asset)
    }

    pub fn from_toml_str(document: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(document)
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn from_model_primitive(uri: AssetUri, primitive: &ModelPrimitiveAsset) -> Self {
        let mut attributes = BTreeMap::new();
        attributes.insert(
            MESH_ATTRIBUTE_POSITION.to_string(),
            MeshAttributeValues::Float32x3(
                primitive
                    .vertices
                    .iter()
                    .map(|vertex| vertex.position)
                    .collect(),
            ),
        );
        attributes.insert(
            MESH_ATTRIBUTE_NORMAL.to_string(),
            MeshAttributeValues::Float32x3(
                primitive
                    .vertices
                    .iter()
                    .map(|vertex| vertex.normal)
                    .collect(),
            ),
        );
        attributes.insert(
            MESH_ATTRIBUTE_UV0.to_string(),
            MeshAttributeValues::Float32x2(
                primitive.vertices.iter().map(|vertex| vertex.uv).collect(),
            ),
        );
        attributes.insert(
            MESH_ATTRIBUTE_JOINT_INDEX.to_string(),
            MeshAttributeValues::Uint16x4(
                primitive
                    .vertices
                    .iter()
                    .map(|vertex| vertex.joint_indices)
                    .collect(),
            ),
        );
        attributes.insert(
            MESH_ATTRIBUTE_JOINT_WEIGHT.to_string(),
            MeshAttributeValues::Float32x4(
                primitive
                    .vertices
                    .iter()
                    .map(|vertex| vertex.joint_weights)
                    .collect(),
            ),
        );

        Self {
            uri,
            topology: RenderMeshTopology::TriangleList,
            attributes,
            indices: Some(MeshIndices::U32(primitive.indices.clone())),
            asset_usage: MeshAssetUsage::default(),
            morph_targets: Vec::new(),
            skin: None,
            virtual_geometry: primitive.virtual_geometry.clone(),
        }
    }

    pub fn to_model_primitive(&self) -> Result<ModelPrimitiveAsset, MeshValidationError> {
        self.validate()?;
        let positions = self.positions()?;
        let normals = optional_float32x3(self, MESH_ATTRIBUTE_NORMAL)?;
        let uvs = optional_float32x2(self, MESH_ATTRIBUTE_UV0)?;
        let joint_indices = optional_uint16x4(self, MESH_ATTRIBUTE_JOINT_INDEX)?;
        let joint_weights = optional_float32x4(self, MESH_ATTRIBUTE_JOINT_WEIGHT)?;

        let vertices = positions
            .iter()
            .enumerate()
            .map(|(index, position)| MeshVertex {
                position: *position,
                normal: normals.map_or(DEFAULT_NORMAL, |values| values[index]),
                uv: uvs.map_or(DEFAULT_UV, |values| values[index]),
                joint_indices: joint_indices.map_or(DEFAULT_JOINT_INDICES, |values| values[index]),
                joint_weights: joint_weights.map_or(DEFAULT_JOINT_WEIGHTS, |values| values[index]),
            })
            .collect();

        Ok(ModelPrimitiveAsset {
            vertices,
            indices: self
                .indices
                .as_ref()
                .map_or_else(Vec::new, MeshIndices::to_u32_vec),
            mesh: None,
            virtual_geometry: self.virtual_geometry.clone(),
        })
    }

    pub fn to_morphed_model_primitive(
        &self,
        morph_weights: &[f32],
    ) -> Result<ModelPrimitiveAsset, MeshValidationError> {
        let mut primitive = self.to_model_primitive()?;
        apply_morph_targets(
            &mut primitive.vertices,
            &self.morph_targets,
            morph_weights,
        )?;
        Ok(primitive)
    }

    pub fn validate(&self) -> Result<(), MeshValidationError> {
        let vertex_count = self.vertex_count()?;
        validate_builtin_attribute_formats(&self.attributes)?;
        for (name, values) in &self.attributes {
            if values.len() != vertex_count {
                return Err(MeshValidationError::AttributeLengthMismatch {
                    attribute: name.clone(),
                    expected: vertex_count,
                    actual: values.len(),
                });
            }
        }
        for (target_index, target) in self.morph_targets.iter().enumerate() {
            for (name, values) in &target.attributes {
                if values.len() != vertex_count {
                    return Err(MeshValidationError::MorphTargetAttributeLengthMismatch {
                        target_index,
                        attribute: name.clone(),
                        expected: vertex_count,
                        actual: values.len(),
                    });
                }
            }
        }
        if let Some(max_index) = self.indices.as_ref().and_then(MeshIndices::max_index) {
            if max_index as usize >= vertex_count {
                return Err(MeshValidationError::IndexOutOfRange {
                    max_index,
                    vertex_count,
                });
            }
        }
        validate_topology_element_count(self.topology, element_count(self, vertex_count))?;
        Ok(())
    }

    pub fn vertex_count(&self) -> Result<usize, MeshValidationError> {
        self.positions().map(|positions| positions.len())
    }

    pub fn index_count(&self) -> usize {
        self.indices.as_ref().map_or(0, MeshIndices::len)
    }

    pub fn index_format(&self) -> Option<MeshIndexFormat> {
        self.indices.as_ref().map(MeshIndices::format)
    }

    pub fn attribute_summaries(&self) -> Vec<MeshAttributeSummary> {
        self.attributes
            .iter()
            .map(|(name, values)| summarize_attribute(name, values))
            .collect()
    }

    pub fn morph_target_attribute_summaries(&self) -> Vec<MeshMorphTargetAttributeSummary> {
        self.morph_targets
            .iter()
            .enumerate()
            .flat_map(|(target_index, target)| {
                target.attributes.iter().map(move |(name, values)| {
                    MeshMorphTargetAttributeSummary {
                        target_index,
                        target_name: target.name.clone(),
                        attribute: summarize_attribute(name, values),
                    }
                })
            })
            .collect()
    }

    pub fn overview(&self) -> Result<MeshAssetOverview, MeshValidationError> {
        let descriptor = self.try_render_mesh_descriptor()?;
        let attributes = self.attribute_summaries();
        let morph_target_attributes = self.morph_target_attribute_summaries();
        Ok(MeshAssetOverview {
            uri: self.uri.clone(),
            topology: self.topology,
            bounds: descriptor.bounds,
            vertex_count: descriptor.vertex_count,
            index_count: descriptor.index_count,
            index_format: self.index_format(),
            draw_element_count: self.draw_element_count()?,
            render_primitive_count: descriptor.primitive_count,
            attribute_count: attributes.len(),
            attributes,
            morph_target_count: self.morph_targets.len(),
            morph_target_attribute_count: morph_target_attributes.len(),
            morph_target_attributes,
            has_skin: self.skin.is_some(),
            inverse_bind_matrix_count: self
                .skin
                .as_ref()
                .map_or(0, |skin| skin.inverse_bind_matrices.len()),
            has_virtual_geometry_payload: descriptor.has_virtual_geometry_payload,
            asset_usage: self.asset_usage,
        })
    }

    pub fn management_record(
        &self,
        mesh_id: ResourceId,
    ) -> Result<MeshAssetManagementRecord, MeshValidationError> {
        Ok(MeshAssetManagementRecord {
            mesh_id,
            overview: self.overview()?,
        })
    }

    pub fn draw_element_count(&self) -> Result<usize, MeshValidationError> {
        self.vertex_count()
            .map(|vertex_count| element_count(self, vertex_count))
    }

    pub fn render_primitive_count(&self) -> Result<usize, MeshValidationError> {
        let element_count = self.draw_element_count()?;
        validate_topology_element_count(self.topology, element_count)?;
        Ok(primitive_count(self.topology, element_count))
    }

    pub fn positions(&self) -> Result<&[[f32; 3]], MeshValidationError> {
        let values = self
            .attributes
            .get(MESH_ATTRIBUTE_POSITION)
            .ok_or(MeshValidationError::MissingPositionAttribute)?;
        values
            .as_float32x3()
            .ok_or(MeshValidationError::InvalidPositionAttributeFormat)
    }

    pub fn bounds(&self) -> Result<RenderMeshBounds, MeshValidationError> {
        self.positions()
            .map(|positions| RenderMeshBounds::from_positions(positions.iter().copied()))
    }

    pub fn render_mesh_descriptor(&self) -> RenderMeshDescriptor {
        let positions = self.positions().unwrap_or(&[]);
        self.render_mesh_descriptor_from_positions(positions)
    }

    pub fn try_render_mesh_descriptor(&self) -> Result<RenderMeshDescriptor, MeshValidationError> {
        self.validate()?;
        let positions = self.positions()?;
        Ok(self.render_mesh_descriptor_from_positions(positions))
    }

    fn render_mesh_descriptor_from_positions(
        &self,
        positions: &[[f32; 3]],
    ) -> RenderMeshDescriptor {
        let vertex_count = positions.len();
        let index_count = self.index_count();
        let element_count = element_count(self, vertex_count);
        let is_planar = positions.iter().all(|position| position[2] == 0.0);
        let bounds = RenderMeshBounds::from_positions(positions.iter().copied());
        RenderMeshDescriptor {
            topology: self.topology,
            bounds,
            primitive_kind: if is_planar {
                RenderMeshKind::Planar2d
            } else {
                RenderMeshKind::Spatial3d
            },
            suitable_for_2d: is_planar,
            suitable_for_3d: true,
            vertex_count,
            index_count,
            primitive_count: primitive_count(self.topology, element_count),
            has_virtual_geometry_payload: self.virtual_geometry.is_some(),
        }
    }
}

fn optional_float32x2<'a>(
    asset: &'a MeshAsset,
    name: &str,
) -> Result<Option<&'a [[f32; 2]]>, MeshValidationError> {
    optional_attribute(asset, name, MeshAttributeValues::as_float32x2, "float32x2")
}

fn optional_float32x3<'a>(
    asset: &'a MeshAsset,
    name: &str,
) -> Result<Option<&'a [[f32; 3]]>, MeshValidationError> {
    optional_attribute(asset, name, MeshAttributeValues::as_float32x3, "float32x3")
}

fn optional_float32x4<'a>(
    asset: &'a MeshAsset,
    name: &str,
) -> Result<Option<&'a [[f32; 4]]>, MeshValidationError> {
    optional_attribute(asset, name, MeshAttributeValues::as_float32x4, "float32x4")
}

fn optional_uint16x4<'a>(
    asset: &'a MeshAsset,
    name: &str,
) -> Result<Option<&'a [[u16; 4]]>, MeshValidationError> {
    optional_attribute(asset, name, MeshAttributeValues::as_uint16x4, "uint16x4")
}

fn optional_attribute<'a, TValue>(
    asset: &'a MeshAsset,
    name: &str,
    accessor: impl FnOnce(&'a MeshAttributeValues) -> Option<TValue>,
    expected: &'static str,
) -> Result<Option<TValue>, MeshValidationError> {
    asset.attributes.get(name).map_or(Ok(None), |values| {
        accessor(values)
            .map(Some)
            .ok_or_else(|| MeshValidationError::InvalidAttributeFormat {
                attribute: name.to_string(),
                expected,
            })
    })
}

fn apply_morph_targets(
    vertices: &mut [MeshVertex],
    morph_targets: &[MeshMorphTargetAsset],
    morph_weights: &[f32],
) -> Result<(), MeshValidationError> {
    let mut morphed_normals = vec![false; vertices.len()];

    for (target_index, target) in morph_targets.iter().enumerate() {
        let weight = morph_weights.get(target_index).copied().unwrap_or_default();
        if weight.abs() <= f32::EPSILON {
            continue;
        }

        if let Some(position_deltas) =
            morph_target_float32x3_attribute(target_index, target, MESH_ATTRIBUTE_POSITION)?
        {
            for (vertex, delta) in vertices.iter_mut().zip(position_deltas.iter()) {
                vertex.position =
                    (Vec3::from_array(vertex.position) + Vec3::from_array(*delta) * weight)
                        .to_array();
            }
        }

        if let Some(normal_deltas) =
            morph_target_float32x3_attribute(target_index, target, MESH_ATTRIBUTE_NORMAL)?
        {
            for (vertex_index, (vertex, delta)) in
                vertices.iter_mut().zip(normal_deltas.iter()).enumerate()
            {
                vertex.normal =
                    (Vec3::from_array(vertex.normal) + Vec3::from_array(*delta) * weight)
                        .to_array();
                morphed_normals[vertex_index] = true;
            }
        }
    }

    for (vertex, morphed) in vertices.iter_mut().zip(morphed_normals.iter()) {
        if *morphed {
            vertex.normal = Vec3::from_array(vertex.normal)
                .normalize_or_zero()
                .to_array();
        }
    }

    Ok(())
}

fn morph_target_float32x3_attribute<'a>(
    target_index: usize,
    target: &'a MeshMorphTargetAsset,
    name: &str,
) -> Result<Option<&'a [[f32; 3]]>, MeshValidationError> {
    target.attributes.get(name).map_or(Ok(None), |values| {
        values.as_float32x3().map(Some).ok_or_else(|| {
            MeshValidationError::InvalidAttributeFormat {
                attribute: format!("morph_targets[{target_index}].{name}"),
                expected: "float32x3",
            }
        })
    })
}

fn validate_builtin_attribute_formats(
    attributes: &BTreeMap<String, MeshAttributeValues>,
) -> Result<(), MeshValidationError> {
    for (name, values) in attributes {
        let expected = match name.as_str() {
            MESH_ATTRIBUTE_NORMAL if !matches!(values, MeshAttributeValues::Float32x3(_)) => {
                Some("float32x3")
            }
            MESH_ATTRIBUTE_TANGENT if !matches!(values, MeshAttributeValues::Float32x4(_)) => {
                Some("float32x4")
            }
            MESH_ATTRIBUTE_UV0 if !matches!(values, MeshAttributeValues::Float32x2(_)) => {
                Some("float32x2")
            }
            MESH_ATTRIBUTE_COLOR if !matches!(values, MeshAttributeValues::Float32x4(_)) => {
                Some("float32x4")
            }
            MESH_ATTRIBUTE_JOINT_INDEX if !matches!(values, MeshAttributeValues::Uint16x4(_)) => {
                Some("uint16x4")
            }
            MESH_ATTRIBUTE_JOINT_WEIGHT if !matches!(values, MeshAttributeValues::Float32x4(_)) => {
                Some("float32x4")
            }
            _ => None,
        };

        if let Some(expected) = expected {
            return Err(MeshValidationError::InvalidAttributeFormat {
                attribute: name.clone(),
                expected,
            });
        }
    }

    Ok(())
}

fn is_builtin_attribute_name(name: &str) -> bool {
    matches!(
        name,
        MESH_ATTRIBUTE_POSITION
            | MESH_ATTRIBUTE_NORMAL
            | MESH_ATTRIBUTE_TANGENT
            | MESH_ATTRIBUTE_UV0
            | MESH_ATTRIBUTE_COLOR
            | MESH_ATTRIBUTE_JOINT_INDEX
            | MESH_ATTRIBUTE_JOINT_WEIGHT
    )
}

fn summarize_attribute(name: &str, values: &MeshAttributeValues) -> MeshAttributeSummary {
    MeshAttributeSummary {
        name: name.to_string(),
        format: values.format(),
        len: values.len(),
        is_builtin: is_builtin_attribute_name(name),
    }
}

fn validate_topology_element_count(
    topology: RenderMeshTopology,
    element_count: usize,
) -> Result<(), MeshValidationError> {
    let required_multiple = match topology {
        RenderMeshTopology::TriangleList => 3,
        RenderMeshTopology::LineList => 2,
        RenderMeshTopology::TriangleStrip
        | RenderMeshTopology::LineStrip
        | RenderMeshTopology::PointList => return Ok(()),
    };

    if element_count % required_multiple != 0 {
        return Err(MeshValidationError::IncompleteTopologyElement {
            topology,
            required_multiple,
            actual_elements: element_count,
        });
    }

    Ok(())
}

fn element_count(asset: &MeshAsset, vertex_count: usize) -> usize {
    let index_count = asset.index_count();
    if index_count == 0 {
        vertex_count
    } else {
        index_count
    }
}

fn primitive_count(topology: RenderMeshTopology, element_count: usize) -> usize {
    match topology {
        RenderMeshTopology::TriangleList => element_count / 3,
        RenderMeshTopology::TriangleStrip => element_count.saturating_sub(2),
        RenderMeshTopology::LineList => element_count / 2,
        RenderMeshTopology::LineStrip => element_count.saturating_sub(1),
        RenderMeshTopology::PointList => element_count,
    }
}
