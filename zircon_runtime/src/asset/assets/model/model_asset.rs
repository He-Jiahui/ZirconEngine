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
mod performance_tests;

#[cfg(test)]
mod optimization_batch_20260830cm_runtime_tests {
    use std::collections::HashSet;
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const REFERENCES_PER_SAMPLE: usize = 16_384;

    #[test]
    fn optimization_batch_20260830cm_runtime_model_reference_dedup_reserves_iterator_bound() {
        let source = include_str!("model_asset.rs");
        let implementation = source
            .split("mod performance_tests;")
            .next()
            .expect("model asset implementation");

        assert!(implementation.contains("let (minimum_references, maximum_references)"));
        assert!(implementation.contains("maximum_references.unwrap_or(minimum_references)"));
        assert!(implementation.contains("HashSet::with_capacity(reference_capacity)"));
        assert!(implementation.contains("Vec::with_capacity(reference_capacity)"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830cm_runtime_model_reference_dedup_capacity_p95() {
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false));
                optimized.push(measure(true));
            } else {
                optimized.push(measure(true));
                legacy.push(measure(false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME500_MODEL_REFERENCE_DEDUP_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} references_per_sample={REFERENCES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(use_capacity: bool) -> u128 {
        let started = Instant::now();
        let mut seen = if use_capacity {
            HashSet::with_capacity(REFERENCES_PER_SAMPLE)
        } else {
            HashSet::new()
        };
        let mut unique = if use_capacity {
            Vec::with_capacity(REFERENCES_PER_SAMPLE)
        } else {
            Vec::new()
        };
        for reference in 0..REFERENCES_PER_SAMPLE {
            if seen.insert(reference) {
                unique.push(reference);
            }
        }
        std::hint::black_box((seen, unique));
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], p: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
