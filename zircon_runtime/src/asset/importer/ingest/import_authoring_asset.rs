use crate::asset::{
    AssetAuthoringError, AssetImportContext, AssetImportError, AssetImportOutcome, ImportedAsset,
    MaterialGraphAsset, TerrainAsset, TileMapAsset,
};

pub(super) fn import_prefab(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    parse_typed_toml(context, "prefab toml")
        .map(|asset| AssetImportOutcome::new(context.uri.clone(), ImportedAsset::Prefab(asset)))
}

pub(super) fn import_material_graph(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let graph: MaterialGraphAsset = parse_typed_toml(context, "material graph toml")?;
    graph
        .validate_output_node()
        .map_err(asset_authoring_parse_error)?;
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::MaterialGraph(graph),
    ))
}

pub(super) fn import_terrain(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let terrain: TerrainAsset = parse_typed_toml(context, "terrain toml")?;
    terrain
        .validate_dimensions()
        .map_err(asset_authoring_parse_error)?;
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Terrain(terrain),
    ))
}

pub(super) fn import_terrain_layer_stack(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    parse_typed_toml(context, "terrain layer stack toml").map(|asset| {
        AssetImportOutcome::new(context.uri.clone(), ImportedAsset::TerrainLayerStack(asset))
    })
}

pub(super) fn import_tileset(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    parse_typed_toml(context, "tileset toml")
        .map(|asset| AssetImportOutcome::new(context.uri.clone(), ImportedAsset::TileSet(asset)))
}

pub(super) fn import_tilemap(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let tilemap: TileMapAsset = parse_typed_toml(context, "tilemap toml")?;
    tilemap
        .validate_layers()
        .map_err(asset_authoring_parse_error)?;
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::TileMap(tilemap),
    ))
}

pub(super) fn import_navmesh(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    parse_typed_toml(context, "navmesh toml")
        .map(|asset| AssetImportOutcome::new(context.uri.clone(), ImportedAsset::NavMesh(asset)))
}

pub(super) fn import_navigation_settings(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    parse_typed_toml(context, "navigation settings toml").map(|asset| {
        AssetImportOutcome::new(
            context.uri.clone(),
            ImportedAsset::NavigationSettings(asset),
        )
    })
}

fn parse_typed_toml<T: serde::de::DeserializeOwned>(
    context: &AssetImportContext,
    label: &str,
) -> Result<T, AssetImportError> {
    let document = context.source_str()?;
    toml::from_str::<T>(document)
        .map_err(|error| AssetImportError::Parse(format!("parse {label}: {error}")))
}

fn asset_authoring_parse_error(error: AssetAuthoringError) -> AssetImportError {
    AssetImportError::Parse(error.to_string())
}

#[cfg(test)]
mod plugins07_authoring_source_tests {
    use std::hint::black_box;
    use std::path::PathBuf;
    use std::time::Instant;

    use super::*;
    use crate::asset::{AssetUri, TerrainLayerStackAsset};

    const SAMPLE_PAIRS: usize = 21;
    const CHECKS_PER_SAMPLE: usize = 16;
    const SOURCE_BYTES: usize = 1_048_576;

    #[test]
    fn borrowed_toml_source_contract_authoring_import() {
        let uri = AssetUri::parse("res://terrain/plugins07.layers.toml").unwrap();
        let document = TerrainLayerStackAsset {
            uri: uri.clone(),
            layers: Vec::new(),
        };
        let context = AssetImportContext::new(
            PathBuf::from("terrain/plugins07.layers.toml"),
            uri.clone(),
            toml::to_string(&document).unwrap().into_bytes(),
            toml::Table::new(),
        );

        let outcome = import_terrain_layer_stack(&context).unwrap();
        let Some(ImportedAsset::TerrainLayerStack(stack)) =
            outcome.root_entry().map(|entry| &entry.asset)
        else {
            panic!("authoring importer must preserve its typed root asset")
        };
        assert_eq!(stack.uri, uri);
        assert!(stack.layers.is_empty());
    }

    #[test]
    #[ignore = "release performance gate"]
    fn borrowed_toml_source_performance_release_authoring() {
        run_release_gate("plugins07_borrowed_authoring_toml_source");
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
            "borrowed authoring source preparation must improve P95 by at least 40%"
        );
    }

    fn benchmark_context() -> AssetImportContext {
        AssetImportContext::new(
            PathBuf::from("terrain/plugins07-large.layers.toml"),
            AssetUri::parse("res://terrain/plugins07-large.layers.toml").unwrap(),
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
