mod parse_sfnt;

use std::path::{Path, PathBuf};

use crate::asset::assets::{
    FontAsset, FontBlobArtifact, ImportedAsset, decode_font_source, validate_font_source_file_len,
};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome, AssetUri};

pub(crate) fn import_font_asset(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_str()?;
    let mut asset = FontAsset::from_toml_str(document).map_err(AssetImportError::FontDocument)?;
    let source_path = resolve_manifest_source_path(&context.source_path, &asset.source)?;
    let source_size = std::fs::metadata(&source_path)
        .map_err(|source| AssetImportError::FontSourceIo {
            path: source_path.clone(),
            source,
        })?
        .len();
    validate_font_source_file_len(source_size).map_err(|source| {
        AssetImportError::FontSourceBudget {
            path: source_path.clone(),
            source,
        }
    })?;
    let source_bytes =
        std::fs::read(&source_path).map_err(|source| AssetImportError::FontSourceIo {
            path: source_path.clone(),
            source,
        })?;
    let source = decode_font_source(source_bytes).map_err(|source| match source {
        crate::asset::assets::FontSourceDecodeError::Budget(source) => {
            AssetImportError::FontSourceBudget {
                path: source_path.clone(),
                source,
            }
        }
        source => AssetImportError::FontSourceDecode {
            path: source_path.clone(),
            source,
        },
    })?;
    let metadata = parse_sfnt::parse_font_metadata(&source).map_err(|source| {
        AssetImportError::FontMetadata {
            path: source_path.clone(),
            source,
        }
    })?;
    let cooked_blob =
        FontBlobArtifact::from_decoded_bytes(source.source_format(), source.into_bytes());

    apply_parsed_defaults(&mut asset, metadata, cooked_blob);

    let dependency = dependency_uri_for_source(&context.uri, &asset.source);
    let mut outcome = AssetImportOutcome::new(context.uri.clone(), ImportedAsset::Font(asset));
    if let Some(dependency) = dependency {
        outcome = outcome.with_dependency(dependency);
    }
    Ok(outcome)
}

fn apply_parsed_defaults(
    asset: &mut FontAsset,
    mut metadata: crate::asset::assets::FontAssetMetadata,
    cooked_blob: FontBlobArtifact,
) {
    if asset.family_members.is_empty() {
        asset.family_members = metadata
            .faces
            .iter()
            .filter_map(|face| face.family_member())
            .collect();
    }
    if asset.variable_instances.is_empty() {
        asset.variable_instances = metadata
            .faces
            .iter()
            .find(|face| face.face_index == asset.face_index)
            .map(|face| face.named_instances.clone())
            .unwrap_or_default();
    }
    if asset.family.is_none() {
        asset.family = metadata
            .faces
            .iter()
            .find(|face| face.face_index == asset.face_index)
            .and_then(|face| face.family.clone())
            .or_else(|| metadata.faces.first().and_then(|face| face.family.clone()));
    }
    metadata.cooked_blob = Some(cooked_blob);
    asset.metadata = Some(metadata);
}

fn resolve_manifest_source_path(
    manifest_path: &Path,
    source: &str,
) -> Result<PathBuf, AssetImportError> {
    let source = source.trim();
    if source.is_empty() {
        return Err(AssetImportError::FontSourcePath {
            manifest_path: manifest_path.to_path_buf(),
            reason: "source is empty",
        });
    }

    let source_path = PathBuf::from(source);
    if source_path.is_absolute() {
        return Err(AssetImportError::FontSourcePath {
            manifest_path: manifest_path.to_path_buf(),
            reason: "source must be relative to the manifest",
        });
    }

    Ok(manifest_path
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(source_path))
}

fn dependency_uri_for_source(manifest_uri: &AssetUri, source: &str) -> Option<AssetUri> {
    if !manifest_uri.to_string().starts_with("res://") {
        return None;
    }
    let parent = Path::new(manifest_uri.path()).parent()?;
    let dependency_path = parent
        .join(source.trim())
        .to_string_lossy()
        .replace('\\', "/");
    AssetUri::parse(&format!("res://{dependency_path}")).ok()
}

#[cfg(test)]
mod plugins07_font_manifest_source_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 21;
    const CHECKS_PER_SAMPLE: usize = 16;
    const SOURCE_BYTES: usize = 1_048_576;

    #[test]
    fn source_ownership_contract_font_manifest_import() {
        let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("fonts")
            .join("plugins07.font.toml");
        let context = AssetImportContext::new(
            source_path,
            AssetUri::parse("res://fonts/plugins07.font.toml").unwrap(),
            br#"
source = "FiraMono-subset.ttf"
family = "Fira Mono"
render_mode = "sdf"
"#
            .to_vec(),
            toml::Table::new(),
        );

        let outcome = import_font_asset(&context).unwrap();
        let Some(ImportedAsset::Font(font)) = outcome.root_entry().map(|entry| &entry.asset) else {
            panic!("font importer must preserve its typed root asset")
        };
        assert_eq!(font.source, "FiraMono-subset.ttf");
        assert!(
            font.metadata
                .as_ref()
                .is_some_and(|metadata| !metadata.faces.is_empty())
        );
    }

    #[test]
    #[ignore = "release performance gate"]
    fn source_ownership_performance_release_font_manifest() {
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
            "PERF_RESULT plugins07_borrowed_font_manifest_source sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} source_bytes={SOURCE_BYTES} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=40 legacy_source_string_allocations_per_sample={CHECKS_PER_SAMPLE} optimized_source_string_allocations_per_sample=0 order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(
            improvement_percent >= 40,
            "borrowed font manifest source preparation must improve P95 by at least 40%"
        );
    }

    fn benchmark_context() -> AssetImportContext {
        AssetImportContext::new(
            PathBuf::from("fonts/plugins07-large.font.toml"),
            AssetUri::parse("res://fonts/plugins07-large.font.toml").unwrap(),
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
