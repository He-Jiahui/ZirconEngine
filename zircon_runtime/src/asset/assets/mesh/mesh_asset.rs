use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::asset::{AssetUri, MeshVertex};
use crate::core::framework::render::{
    RenderMeshBounds, RenderMeshDescriptor, RenderMeshKind, RenderMeshTopology,
};

use super::super::model::{ModelPrimitiveAsset, VirtualGeometryAsset};
use super::{
    MeshAssetUsage, MeshAttributeValues, MeshIndices, MeshValidationError,
    MESH_ATTRIBUTE_JOINT_INDEX, MESH_ATTRIBUTE_JOINT_WEIGHT, MESH_ATTRIBUTE_NORMAL,
    MESH_ATTRIBUTE_POSITION, MESH_ATTRIBUTE_UV0,
};

const DEFAULT_NORMAL: [f32; 3] = [0.0, 0.0, 1.0];
const DEFAULT_UV: [f32; 2] = [0.0, 0.0];
const DEFAULT_JOINT_INDICES: [u16; 4] = [0, 0, 0, 0];
const DEFAULT_JOINT_WEIGHTS: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

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
            virtual_geometry: self.virtual_geometry.clone(),
        })
    }

    pub fn validate(&self) -> Result<(), MeshValidationError> {
        let vertex_count = self.vertex_count()?;
        for (name, values) in &self.attributes {
            if values.len() != vertex_count {
                return Err(MeshValidationError::AttributeLengthMismatch {
                    attribute: name.clone(),
                    expected: vertex_count,
                    actual: values.len(),
                });
            }
        }
        Ok(())
    }

    pub fn vertex_count(&self) -> Result<usize, MeshValidationError> {
        self.positions().map(|positions| positions.len())
    }

    pub fn index_count(&self) -> usize {
        self.indices.as_ref().map_or(0, MeshIndices::len)
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

    pub fn render_mesh_descriptor(&self) -> RenderMeshDescriptor {
        let positions = self.positions().unwrap_or(&[]);
        let vertex_count = positions.len();
        let index_count = self.index_count();
        let element_count = if index_count == 0 {
            vertex_count
        } else {
            index_count
        };
        let is_planar = positions.iter().all(|position| position[2] == 0.0);
        RenderMeshDescriptor {
            topology: self.topology,
            bounds: RenderMeshBounds::from_positions(positions.iter().copied()),
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

fn primitive_count(topology: RenderMeshTopology, element_count: usize) -> usize {
    match topology {
        RenderMeshTopology::TriangleList => element_count / 3,
        RenderMeshTopology::TriangleStrip => element_count.saturating_sub(2),
        RenderMeshTopology::LineList => element_count / 2,
        RenderMeshTopology::LineStrip => element_count.saturating_sub(1),
        RenderMeshTopology::PointList => element_count,
    }
}
