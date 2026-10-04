//! 模型汇聚带标签的原语与材质引用；glTF/项目模型导入器创建后，依赖提取器和资源流分别消费引用集合与几何概览。

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::asset::{AssetReference, AssetUri};
use crate::core::framework::render::{
    RenderMeshBounds, RenderMeshDescriptor, RenderMeshKind, RenderMeshTopology,
};
use crate::core::resource::ResourceId;

use super::ModelPrimitiveAsset;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelPrimitiveOverview {
    pub primitive_index: usize,
    pub mesh: Option<AssetReference>,
    pub topology: RenderMeshTopology,
    pub bounds: RenderMeshBounds,
    pub primitive_kind: RenderMeshKind,
    pub suitable_for_2d: bool,
    pub suitable_for_3d: bool,
    pub vertex_count: usize,
    pub index_count: usize,
    pub render_primitive_count: usize,
    pub has_virtual_geometry_payload: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelAssetOverview {
    pub uri: AssetUri,
    pub bounds: RenderMeshBounds,
    pub primitive_count: usize,
    pub vertex_count: usize,
    pub index_count: usize,
    pub mesh_reference_count: usize,
    pub render_primitive_count: usize,
    pub has_virtual_geometry_payload: bool,
    pub primitives: Vec<ModelPrimitiveOverview>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelAssetManagementRecord {
    pub model_id: ResourceId,
    pub overview: ModelAssetOverview,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelAssetManagementRecordSetSummary {
    pub model_count: usize,
    pub primitive_count: usize,
    pub vertex_count: usize,
    pub index_count: usize,
    pub render_primitive_count: usize,
    pub mesh_referenced_model_count: usize,
    pub mesh_reference_count: usize,
    pub virtual_geometry_model_count: usize,
    pub virtual_geometry_primitive_count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelAssetManagementRecordSet {
    pub records: Vec<ModelAssetManagementRecord>,
    pub summary: ModelAssetManagementRecordSetSummary,
}

impl ModelAssetManagementRecordSetSummary {
    pub fn from_records(records: &[ModelAssetManagementRecord]) -> Self {
        Self {
            model_count: records.len(),
            primitive_count: records
                .iter()
                .map(|record| record.overview.primitive_count)
                .sum(),
            vertex_count: records
                .iter()
                .map(|record| record.overview.vertex_count)
                .sum(),
            index_count: records
                .iter()
                .map(|record| record.overview.index_count)
                .sum(),
            render_primitive_count: records
                .iter()
                .map(|record| record.overview.render_primitive_count)
                .sum(),
            mesh_referenced_model_count: records
                .iter()
                .filter(|record| record.overview.mesh_reference_count > 0)
                .count(),
            mesh_reference_count: records
                .iter()
                .map(|record| record.overview.mesh_reference_count)
                .sum(),
            virtual_geometry_model_count: records
                .iter()
                .filter(|record| record.overview.has_virtual_geometry_payload)
                .count(),
            virtual_geometry_primitive_count: records
                .iter()
                .map(|record| {
                    record
                        .overview
                        .primitives
                        .iter()
                        .filter(|primitive| primitive.has_virtual_geometry_payload)
                        .count()
                })
                .sum(),
        }
    }
}

impl ModelAssetManagementRecordSet {
    pub fn from_records(mut records: Vec<ModelAssetManagementRecord>) -> Self {
        records.sort_by_key(|record| record.model_id);
        let summary = ModelAssetManagementRecordSetSummary::from_records(&records);
        Self { records, summary }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelAsset {
    pub uri: AssetUri,
    pub primitives: Vec<ModelPrimitiveAsset>,
}

impl ModelAsset {
    pub fn to_project_toml_string(
        &self,
        resolver: impl FnMut(
            &AssetReference,
        ) -> Result<
            zircon_runtime_interface::project::PersistedAssetReference,
            crate::asset::ReferenceResolutionError,
        >,
    ) -> Result<String, crate::asset::assets::ProjectDocumentError> {
        crate::asset::assets::project_document::serialize_model(self, resolver)
    }

    pub fn from_project_toml_str(
        document: &str,
        resolver: impl FnMut(
            &zircon_runtime_interface::project::PersistedAssetReference,
        ) -> Result<AssetReference, crate::asset::ReferenceResolutionError>,
    ) -> Result<Self, crate::asset::assets::ProjectDocumentError> {
        crate::asset::assets::project_document::deserialize_model(document, resolver)
    }

    #[cfg(test)]
    pub fn from_toml_str(document: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(document)
    }

    #[cfg(test)]
    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn render_mesh_descriptors(&self) -> Vec<RenderMeshDescriptor> {
        self.primitives
            .iter()
            .map(ModelPrimitiveAsset::render_mesh_descriptor)
            .collect()
    }

    pub fn primitive_overviews(&self) -> Vec<ModelPrimitiveOverview> {
        self.primitives
            .iter()
            .enumerate()
            .map(|(primitive_index, primitive)| primitive.overview(primitive_index))
            .collect()
    }

    pub fn overview(&self) -> ModelAssetOverview {
        let primitives = self.primitive_overviews();
        ModelAssetOverview {
            uri: self.uri.clone(),
            bounds: RenderMeshBounds::from_positions(
                self.primitives
                    .iter()
                    .flat_map(|primitive| primitive.vertices.iter().map(|vertex| vertex.position)),
            ),
            primitive_count: primitives.len(),
            vertex_count: primitives
                .iter()
                .map(|primitive| primitive.vertex_count)
                .sum(),
            index_count: primitives
                .iter()
                .map(|primitive| primitive.index_count)
                .sum(),
            mesh_reference_count: primitives
                .iter()
                .filter(|primitive| primitive.mesh.is_some())
                .count(),
            render_primitive_count: primitives
                .iter()
                .map(|primitive| primitive.render_primitive_count)
                .sum(),
            has_virtual_geometry_payload: primitives
                .iter()
                .any(|primitive| primitive.has_virtual_geometry_payload),
            primitives,
        }
    }

    /// 供导入后的依赖提取器读取模型自身的直接资源关系；子网格几何仍由带标签资产单独管理。
    pub fn direct_references(&self) -> Vec<AssetReference> {
        collect_unique_references(
            self.primitives
                .iter()
                .filter_map(|primitive| primitive.mesh.as_ref()),
        )
    }

    pub fn management_record(&self, model_id: ResourceId) -> ModelAssetManagementRecord {
        ModelAssetManagementRecord {
            model_id,
            overview: self.overview(),
        }
    }
}

impl ModelPrimitiveAsset {
    pub fn overview(&self, primitive_index: usize) -> ModelPrimitiveOverview {
        let descriptor = self.render_mesh_descriptor();
        ModelPrimitiveOverview {
            primitive_index,
            mesh: self.mesh.clone(),
            topology: descriptor.topology,
            bounds: descriptor.bounds,
            primitive_kind: descriptor.primitive_kind,
            suitable_for_2d: descriptor.suitable_for_2d,
            suitable_for_3d: descriptor.suitable_for_3d,
            vertex_count: descriptor.vertex_count,
            index_count: descriptor.index_count,
            render_primitive_count: descriptor.primitive_count,
            has_virtual_geometry_payload: descriptor.has_virtual_geometry_payload,
        }
    }
}

fn collect_unique_references<'a>(
    references: impl IntoIterator<Item = &'a AssetReference>,
) -> Vec<AssetReference> {
    let references = references.into_iter();
    let (minimum_references, maximum_references) = references.size_hint();
    let reference_capacity = maximum_references.unwrap_or(minimum_references);
    let mut seen = HashSet::with_capacity(reference_capacity);
    let mut unique = Vec::with_capacity(reference_capacity);
    for reference in references {
        if seen.insert(reference) {
            unique.push(reference.clone());
        }
    }
    unique
}

#[cfg(test)]
#[path = "model_asset/tests/performance_tests.rs"]
mod performance_tests;

#[cfg(test)]
#[path = "tests/model_asset_optimization_batch_20260830cm_runtime_tests.rs"]
mod optimization_batch_20260830cm_runtime_tests;
