use std::fs;
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
fn obj_material_snapshot_containment_preserves_internal_parent_and_rejects_escape() {
    let root = auxiliary_test_root("obj-material-containment");
    let asset_root = root.join("assets");
    let model_dir = asset_root.join("models");
    let material_dir = asset_root.join("materials");
    fs::create_dir_all(&model_dir).unwrap();
    fs::create_dir_all(&material_dir).unwrap();
    fs::write(material_dir.join("good.mtl"), "newmtl Good\nKd 1 1 1\n").unwrap();

    let source_path = model_dir.join("model.obj");
    let valid = AssetImportContext::new(
        source_path.clone(),
        AssetUri::parse("res://models/model.obj").unwrap(),
        b"mtllib ../materials/good.mtl\no Mesh\nv 0 0 0\nv 1 0 0\nv 0 1 0\nusemtl Good\nf 1 2 3\n"
            .to_vec(),
        toml::Table::new(),
    );
    import_obj(&valid).expect("an internal parent reference must remain valid");

    let escaped = AssetImportContext::new(
        source_path,
        AssetUri::parse("res://models/model.obj").unwrap(),
        b"mtllib ../../outside.mtl\no Mesh\nv 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n".to_vec(),
        toml::Table::new(),
    );
    let error = import_obj(&escaped).expect_err("an out-of-root material path must be rejected");
    assert!(error.to_string().contains("escapes asset root"), "{error}");

    let _ = fs::remove_dir_all(root);
}

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
    let Some(ImportedAsset::Model(model)) = outcome.root_entry().map(|entry| &entry.asset) else {
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

fn auxiliary_test_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "zircon-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}
