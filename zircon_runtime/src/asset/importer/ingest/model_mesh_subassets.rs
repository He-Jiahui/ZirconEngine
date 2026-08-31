use crate::asset::assets::{ImportedAsset, MeshAsset, ModelAsset};
use crate::asset::{AssetImportOutcome, AssetReference, AssetUri, ImportedAssetEntry};

pub(super) fn model_outcome_with_mesh_subassets(
    root_uri: AssetUri,
    mut model: ModelAsset,
) -> AssetImportOutcome {
    let primitive_count = model.primitives.len();
    let mut dependencies = Vec::new();
    let mut mesh_entries = Vec::new();
    reserve_model_subasset_outputs(primitive_count, &mut dependencies, &mut mesh_entries);
    for (primitive_index, primitive) in model.primitives.iter_mut().enumerate() {
        if let Some(mesh) = primitive.mesh.as_ref() {
            dependencies.push(mesh.locator.clone());
            continue;
        }
        let mesh_uri = model_primitive_mesh_uri(&root_uri, primitive_index);
        primitive.mesh = Some(AssetReference::from_locator(mesh_uri.clone()));
        let mut mesh = MeshAsset::from_model_primitive(mesh_uri.clone(), primitive);
        mesh.mesh_sdf = primitive.mesh_sdf.take();
        dependencies.push(mesh_uri.clone());
        mesh_entries.push(ImportedAssetEntry::new(mesh_uri, ImportedAsset::Mesh(mesh)));
    }

    let mut outcome = AssetImportOutcome::new(root_uri, ImportedAsset::Model(model));
    outcome.entries.reserve(mesh_entries.len());
    outcome.entries[0].dependencies = dependencies;
    outcome.entries.extend(mesh_entries);
    outcome
}

fn reserve_model_subasset_outputs<Dependency, Entry>(
    primitive_count: usize,
    dependencies: &mut Vec<Dependency>,
    mesh_entries: &mut Vec<Entry>,
) {
    dependencies.reserve(primitive_count);
    mesh_entries.reserve(primitive_count);
}

fn model_primitive_mesh_uri(root_uri: &AssetUri, primitive_index: usize) -> AssetUri {
    AssetUri::parse(&format!("{root_uri}#Mesh{primitive_index}/Primitive0"))
        .expect("generated model primitive mesh uri must be valid")
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;
    use crate::asset::ModelPrimitiveAsset;

    const SAMPLE_PAIRS: usize = 21;
    const OUTPUTS_PER_SAMPLE: usize = 16_384;

    #[test]
    fn external_mesh_references_are_preserved_as_dependencies() {
        let root_uri = AssetUri::parse("res://models/external.model.toml").unwrap();
        let external_mesh_uri = AssetUri::parse("res://meshes/authoritative.zmesh").unwrap();
        let model = ModelAsset {
            uri: root_uri.clone(),
            primitives: vec![ModelPrimitiveAsset {
                vertices: Vec::new(),
                indices: Vec::new(),
                mesh: Some(AssetReference::from_locator(external_mesh_uri.clone())),
                mesh_sdf: None,
                virtual_geometry: None,
            }],
        };

        let outcome = model_outcome_with_mesh_subassets(root_uri, model);

        assert_eq!(outcome.entries.len(), 1);
        assert_eq!(
            outcome.root_entry().unwrap().dependencies,
            vec![external_mesh_uri.clone()]
        );
        let ImportedAsset::Model(model) = &outcome.root_entry().unwrap().asset else {
            panic!("root import remains a model");
        };
        assert_eq!(
            model.primitives[0].mesh.as_ref().map(|mesh| &mesh.locator),
            Some(&external_mesh_uri)
        );
    }

    #[test]
    fn inline_primitive_is_assetized_without_touching_external_siblings() {
        let root_uri = AssetUri::parse("res://models/mixed.model.toml").unwrap();
        let external_mesh_uri = AssetUri::parse("res://meshes/external.zmesh").unwrap();
        let primitive = |mesh| ModelPrimitiveAsset {
            vertices: Vec::new(),
            indices: Vec::new(),
            mesh,
            mesh_sdf: None,
            virtual_geometry: None,
        };
        let model = ModelAsset {
            uri: root_uri.clone(),
            primitives: vec![
                primitive(Some(AssetReference::from_locator(
                    external_mesh_uri.clone(),
                ))),
                primitive(None),
            ],
        };

        let outcome = model_outcome_with_mesh_subassets(root_uri.clone(), model);
        let generated = model_primitive_mesh_uri(&root_uri, 1);

        assert_eq!(outcome.entries.len(), 2);
        assert_eq!(
            outcome.root_entry().unwrap().dependencies,
            vec![external_mesh_uri, generated.clone()]
        );
        assert!(
            outcome
                .entries
                .iter()
                .any(|entry| entry.locator == generated)
        );
    }

    #[test]
    fn importer_request_publish_contract_model_preallocates_outputs() {
        let mut dependencies = Vec::<usize>::new();
        let mut mesh_entries = Vec::<usize>::new();

        reserve_model_subasset_outputs(128, &mut dependencies, &mut mesh_entries);

        assert!(dependencies.capacity() >= 128);
        assert!(mesh_entries.capacity() >= 128);
        assert!(dependencies.is_empty());
        assert!(mesh_entries.is_empty());
    }

    #[test]
    #[ignore = "release performance gate"]
    fn importer_request_publish_performance_release_model_preallocated_outputs() {
        for _ in 0..4 {
            black_box(measure_output_buffers(false));
            black_box(measure_output_buffers(true));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut legacy_growths = None;
        let mut optimized_growths = None;
        for pair_index in 0..SAMPLE_PAIRS {
            let (legacy, optimized) = if pair_index % 2 == 0 {
                (measure_output_buffers(false), measure_output_buffers(true))
            } else {
                let optimized = measure_output_buffers(true);
                (measure_output_buffers(false), optimized)
            };
            legacy_growths.get_or_insert(legacy.1);
            optimized_growths.get_or_insert(optimized.1);
            assert_eq!(legacy_growths, Some(legacy.1));
            assert_eq!(optimized_growths, Some(optimized.1));
            legacy_samples.push(legacy.0);
            optimized_samples.push(optimized.0);
        }

        let legacy_p95 = nearest_rank_p95(&legacy_samples);
        let optimized_p95 = nearest_rank_p95(&optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "PERF_RESULT plugins07_preallocated_model_subasset_publish sample_pairs={SAMPLE_PAIRS} outputs_per_sample={OUTPUTS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25 legacy_capacity_growths_per_sample={} optimized_capacity_growths_per_sample={} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
            csv(&legacy_samples),
            csv(&optimized_samples),
            legacy_growths.unwrap(),
            optimized_growths.unwrap(),
        );
        assert_eq!(optimized_growths, Some(0));
        assert!(
            improvement_percent >= 25,
            "preallocated model subasset outputs must improve P95 by at least 25%"
        );
    }

    fn measure_output_buffers(preallocated: bool) -> (u128, usize) {
        let started = Instant::now();
        let mut dependencies = Vec::new();
        let mut mesh_entries = Vec::new();
        if preallocated {
            reserve_model_subasset_outputs(
                OUTPUTS_PER_SAMPLE,
                &mut dependencies,
                &mut mesh_entries,
            );
        }
        let mut growths = 0;
        for output in 0..OUTPUTS_PER_SAMPLE {
            let dependency_capacity = dependencies.capacity();
            dependencies.push(black_box(output));
            growths += usize::from(dependencies.capacity() != dependency_capacity);
            let entry_capacity = mesh_entries.capacity();
            mesh_entries.push(black_box(output));
            growths += usize::from(mesh_entries.capacity() != entry_capacity);
        }
        black_box((dependencies, mesh_entries));
        (started.elapsed().as_nanos().max(1), growths)
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
