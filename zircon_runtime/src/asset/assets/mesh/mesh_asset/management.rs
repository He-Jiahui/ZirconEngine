use serde::{Deserialize, Serialize};

use crate::asset::AssetUri;
use crate::core::framework::render::{RenderMeshBounds, RenderMeshTopology};
use crate::core::resource::ResourceId;

use super::super::{
    MeshAssetUsage, MeshAttributeSummary, MeshIndexFormat, MeshMorphTargetAttributeSummary,
    MeshValidationError,
};

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
