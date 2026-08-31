use crate::asset::assets::{ImportedAsset, ZMeshDocument};
use crate::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, cook_mesh_sdf_or_fallback_single,
};

pub(crate) fn import_zmesh(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_str()?;
    let zmesh = ZMeshDocument::from_toml_str(document)
        .map_err(|error| AssetImportError::Parse(format!("parse zmesh toml: {error}")))?;
    let mut mesh = zmesh
        .into_mesh_asset(context.uri.clone())
        .map_err(|error| AssetImportError::Parse(format!("validate zmesh: {error}")))?;
    if mesh.mesh_sdf.is_none() {
        if let Some(settings) = context.mesh_sdf_cook_request()?.settings() {
            let primitive = mesh
                .to_model_primitive()
                .map_err(|error| AssetImportError::Parse(format!("validate zmesh: {error}")))?;
            mesh.mesh_sdf =
                cook_mesh_sdf_or_fallback_single(&primitive.vertices, &primitive.indices, settings)
                    .map_err(|error| AssetImportError::Parse(format!("cook mesh SDF: {error}")))?;
        }
    }
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Mesh(mesh),
    ))
}

#[cfg(test)]
mod plugins07_zmesh_source_tests {
    use std::collections::BTreeMap;
    use std::hint::black_box;
    use std::path::PathBuf;
    use std::time::Instant;

    use super::*;
    use crate::asset::{
        AssetUri, MESH_ATTRIBUTE_POSITION, MeshAttributeValues, MeshIndices, ZMESH_DOCUMENT_VERSION,
    };
    use crate::core::framework::render::RenderMeshTopology;

    const SAMPLE_PAIRS: usize = 21;
    const CHECKS_PER_SAMPLE: usize = 16;
    const SOURCE_BYTES: usize = 1_048_576;

    #[test]
    fn borrowed_toml_source_contract_zmesh_import() {
        let uri = AssetUri::parse("res://meshes/plugins07.zmesh").unwrap();
        let document = ZMeshDocument {
            version: ZMESH_DOCUMENT_VERSION,
            name: Some("Plugins07".to_string()),
            topology: RenderMeshTopology::TriangleList,
            attributes: BTreeMap::from([(
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                ]),
            )]),
            indices: Some(MeshIndices::U16(vec![0, 1, 2])),
            asset_usage: Default::default(),
            morph_targets: Vec::new(),
            skin: None,
            mesh_sdf: None,
            virtual_geometry: None,
        };
        let context = AssetImportContext::new(
            PathBuf::from("meshes/plugins07.zmesh"),
            uri,
            document.to_toml_string().unwrap().into_bytes(),
            toml::Table::new(),
        );

        let outcome = import_zmesh(&context).unwrap();
        let Some(ImportedAsset::Mesh(mesh)) = outcome.root_entry().map(|entry| &entry.asset) else {
            panic!("zmesh importer must preserve its typed root asset")
        };
        assert_eq!(mesh.vertex_count().unwrap(), 3);
        assert_eq!(mesh.index_count(), 3);
    }

    #[test]
    #[ignore = "release performance gate"]
    fn borrowed_toml_source_performance_release_zmesh() {
        run_release_gate("plugins07_borrowed_zmesh_toml_source");
    }

    fn run_release_gate(marker: &str) {
        let context = benchmark_context();
        for _ in 0..4 {
            black_box(measure_owned(&context));
            black_box(measure_borrowed(&context));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
                (measure_owned(&context), measure_borrowed(&context))
            } else {
                let optimized_ns = measure_borrowed(&context);
                (measure_owned(&context), optimized_ns)
            };
            legacy_samples.push(legacy_ns);
            optimized_samples.push(optimized_ns);
        }

        let legacy_p95 = nearest_rank_p95(&legacy_samples);
        let optimized_p95 = nearest_rank_p95(&optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "PERF_RESULT {marker} sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} source_bytes={SOURCE_BYTES} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=40 legacy_source_string_allocations_per_sample={CHECKS_PER_SAMPLE} optimized_source_string_allocations_per_sample=0 order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(
            improvement_percent >= 40,
            "borrowed zmesh source preparation must improve P95 by at least 40%"
        );
    }

    fn benchmark_context() -> AssetImportContext {
        AssetImportContext::new(
            PathBuf::from("meshes/plugins07-large.zmesh"),
            AssetUri::parse("res://meshes/plugins07-large.zmesh").unwrap(),
            vec![b'a'; SOURCE_BYTES],
            toml::Table::new(),
        )
    }

    fn measure_owned(context: &AssetImportContext) -> u128 {
        let started = Instant::now();
        for _ in 0..CHECKS_PER_SAMPLE {
            black_box(black_box(context).source_text().unwrap());
        }
        started.elapsed().as_nanos().max(1)
    }

    fn measure_borrowed(context: &AssetImportContext) -> u128 {
        let started = Instant::now();
        for _ in 0..CHECKS_PER_SAMPLE {
            black_box(black_box(context).source_str().unwrap());
        }
        started.elapsed().as_nanos().max(1)
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * 95).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
