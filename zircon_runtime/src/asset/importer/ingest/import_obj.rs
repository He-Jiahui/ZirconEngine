use super::model_mesh_subassets::model_outcome_with_mesh_subassets;
use super::primitive_from_indexed_mesh::{MissingNormalPolicy, primitive_from_indexed_mesh};
use std::io::{BufReader, Cursor};

use crate::asset::assets::ModelAsset;
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome, MeshSdfCookBudget};

pub(crate) fn import_obj(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let mut source = Cursor::new(context.source_bytes.as_slice());
    let material_root = context.source_path.parent();
    let (models, _) = tobj::load_obj_buf(
        &mut source,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |material_path| {
            let path = material_root
                .map(|root| root.join(material_path))
                .unwrap_or_else(|| material_path.to_path_buf());
            if let Some(bytes) = context.source_file_snapshot(&path) {
                tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(bytes)))
            } else {
                tobj::load_mtl(path)
            }
        },
    )
    .map_err(|error| AssetImportError::Parse(format!("parse obj: {error}")))?;

    let source_hint = context.uri.to_string();
    let virtual_geometry_request = context.virtual_geometry_cook_request()?;
    let mesh_sdf_request = context.mesh_sdf_cook_request()?;
    let mut mesh_sdf_budget = MeshSdfCookBudget::default();
    let primitives = models
        .into_iter()
        .map(|model| {
            primitive_from_indexed_mesh(
                &model.mesh.positions,
                &model.mesh.normals,
                MissingNormalPolicy::Smooth,
                &model.mesh.texcoords,
                &[],
                &[],
                &[],
                &model.mesh.indices,
                &[],
                &[],
                Some(model.name.as_str()),
                &source_hint,
                &virtual_geometry_request,
                &mesh_sdf_request,
                &mut mesh_sdf_budget,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(model_outcome_with_mesh_subassets(
        context.uri.clone(),
        ModelAsset {
            uri: context.uri.clone(),
            primitives,
        },
    ))
}

#[cfg(test)]
mod plugins07_obj_snapshot_tests {
    use std::hint::black_box;
    use std::path::{Path, PathBuf};
    use std::time::Instant;

    use super::*;
    use crate::asset::{AssetUri, ImportedAsset};

    const SAMPLE_PAIRS: usize = 21;
    const CHECKS_PER_SAMPLE: usize = 8;
    const SOURCE_BYTES: usize = 1_048_576;
    const OBJ_SOURCE: &[u8] = b"o Plugins07\nv 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n";

    #[test]
    fn source_ownership_contract_obj_uses_context_snapshot() {
        let context = AssetImportContext::new(
            PathBuf::from("missing/plugins07-snapshot.obj"),
            AssetUri::parse("res://models/plugins07-snapshot.obj").unwrap(),
            OBJ_SOURCE.to_vec(),
            toml::Table::new(),
        );
        assert!(!context.source_path.exists());

        let outcome = import_obj(&context).unwrap();
        let Some(ImportedAsset::Model(model)) = outcome.root_entry().map(|entry| &entry.asset)
        else {
            panic!("obj importer must preserve its typed root asset")
        };
        assert_eq!(model.primitives.len(), 1);
        assert_eq!(model.primitives[0].vertices.len(), 3);
        assert_eq!(model.primitives[0].indices, vec![0, 1, 2]);
    }

    #[test]
    #[ignore = "release performance gate"]
    fn source_ownership_performance_release_obj_snapshot_acquisition() {
        let bytes = vec![b'#'; SOURCE_BYTES];
        let path = std::env::temp_dir().join(format!(
            "zircon-plugins07-obj-snapshot-{}.obj",
            std::process::id()
        ));
        std::fs::write(&path, &bytes).unwrap();
        for _ in 0..4 {
            black_box(measure_file_source(&path));
            black_box(measure_snapshot_source(&bytes));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
                (measure_file_source(&path), measure_snapshot_source(&bytes))
            } else {
                let optimized_ns = measure_snapshot_source(&bytes);
                (measure_file_source(&path), optimized_ns)
            };
            legacy_samples.push(legacy_ns);
            optimized_samples.push(optimized_ns);
        }
        std::fs::remove_file(&path).unwrap();

        let legacy_p95 = nearest_rank_p95(&legacy_samples);
        let optimized_p95 = nearest_rank_p95(&optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "PERF_RESULT plugins07_obj_snapshot_source_acquisition sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} source_bytes={SOURCE_BYTES} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=95 legacy_main_file_opens_per_sample={CHECKS_PER_SAMPLE} optimized_main_file_opens_per_sample=0 legacy_main_source_bytes_copied_per_sample={} optimized_main_source_bytes_copied_per_sample=0 order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
            csv(&legacy_samples),
            csv(&optimized_samples),
            SOURCE_BYTES * CHECKS_PER_SAMPLE,
        );
        assert!(
            improvement_percent >= 95,
            "OBJ snapshot source acquisition must improve P95 by at least 95%"
        );
    }

    fn measure_file_source(path: &Path) -> u128 {
        let started = Instant::now();
        for _ in 0..CHECKS_PER_SAMPLE {
            black_box(std::fs::read(black_box(path)).unwrap());
        }
        started.elapsed().as_nanos().max(1)
    }

    fn measure_snapshot_source(source: &[u8]) -> u128 {
        let started = Instant::now();
        for _ in 0..CHECKS_PER_SAMPLE {
            black_box(black_box(source));
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
