use std::collections::{BTreeMap, HashMap, HashSet};
use std::hint::black_box;
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, Instant};

use serde::Deserialize;

use super::{
    canonical_package_id, canonicalize_package_id_reference_in_place, product_artifact,
    selected_package_index, CargoArtifactPayload, CargoMessageHeader, CargoMetadata,
    CargoMetadataPackage, CargoMetadataTarget,
};
use crate::build::product_build::CargoRuntimeDependencyDeclaration;

#[test]
#[ignore = "explicit release-mode tooling performance evidence"]
fn fused_cargo_artifact_selection_performance_evidence() {
    const PATH_COUNT: usize = 20_000;
    const WARMUP_ROUNDS: usize = 5;
    const SAMPLE_ROUNDS: usize = 51;
    const REQUIRED_PERCENT: u128 = 90;

    let executable = PathBuf::from("target/zircon_runtime.exe");
    let filenames = (0..PATH_COUNT)
        .map(|index| {
            if index % 4 == 0 {
                PathBuf::from(format!("target/symbol_{index:06}.pdb"))
            } else {
                PathBuf::from(format!("target/library_{index:06}.rlib"))
            }
        })
        .chain(std::iter::once(executable.clone()))
        .collect::<Vec<_>>();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) =
            measure_artifact_selection(&filenames, &executable, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }
    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) =
            measure_artifact_selection(&filenames, &executable, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
        baseline_samples.push(baseline.1);
        candidate_samples.push(candidate.1);
    }
    baseline_samples.sort_unstable();
    candidate_samples.sort_unstable();
    let baseline_p50 = percentile_duration(&baseline_samples, 50);
    let baseline_p95 = percentile_duration(&baseline_samples, 95);
    let candidate_p50 = percentile_duration(&candidate_samples, 50);
    let candidate_p95 = percentile_duration(&candidate_samples, 95);

    println!(
        "TOOLING15_FUSED_CARGO_ARTIFACT_SELECTION_BENCH_V1 paths={PATH_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 10%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 10%"
    );
}

fn measure_artifact_selection(
    filenames: &[PathBuf],
    executable: &Path,
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    let measure_baseline = || {
        let payload = CargoArtifactPayload {
            filenames: filenames.to_vec(),
            executable: Some(executable.to_path_buf()),
        };
        let started = Instant::now();
        let executable = payload.executable.unwrap();
        assert!(payload.filenames.iter().any(|path| path == &executable));
        let mut symbols = payload
            .filenames
            .into_iter()
            .filter(|path| {
                path.extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("pdb"))
            })
            .collect::<Vec<_>>();
        symbols.sort();
        symbols.dedup();
        (black_box(symbols.len()), started.elapsed())
    };
    let measure_candidate = || {
        let payload = CargoArtifactPayload {
            filenames: filenames.to_vec(),
            executable: Some(executable.to_path_buf()),
        };
        let started = Instant::now();
        let artifact = product_artifact(payload).unwrap();
        (black_box(artifact.symbol_files.len()), started.elapsed())
    };
    if baseline_first {
        (measure_baseline(), measure_candidate())
    } else {
        let candidate = measure_candidate();
        let baseline = measure_baseline();
        (baseline, candidate)
    }
}

#[test]
#[ignore = "explicit release-mode tooling performance evidence"]
fn package_selection_index_performance_evidence() {
    const PACKAGE_COUNT: usize = 100_000;
    const SELECTED_COUNT: usize = 64;
    const WARMUP_ROUNDS: usize = 5;
    const SAMPLE_ROUNDS: usize = 51;

    let packages = (0..PACKAGE_COUNT)
        .map(|index| CargoMetadataPackage {
            id: format!("registry+fixture#package_{index:06}@1.0.0"),
            name: format!("package_{index:06}"),
            version: "1.0.0".to_string(),
            source: Some("registry+fixture".to_string()),
            checksum: None,
            manifest_path: PathBuf::new(),
            features: BTreeMap::new(),
            targets: vec![CargoMetadataTarget {
                name: format!("target_{index:06}"),
                kind: vec!["lib".to_string()],
                crate_types: vec!["lib".to_string()],
                required_features: Vec::new(),
                edition: "2024".to_string(),
                src_path: PathBuf::new(),
            }],
        })
        .collect();
    let metadata = CargoMetadata {
        packages,
        resolve: None,
        workspace_members: Vec::new(),
        workspace_default_members: Vec::new(),
    };
    let selected_names = metadata.packages[PACKAGE_COUNT - SELECTED_COUNT..]
        .iter()
        .map(|package| package.name.as_str())
        .collect::<HashSet<_>>();

    for _ in 0..WARMUP_ROUNDS {
        black_box(legacy_select_packages(&metadata, &selected_names));
        black_box(selected_package_index(&metadata, &selected_names).unwrap());
    }
    let mut baseline = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        if round % 2 == 0 {
            baseline.push(measure_ns(|| {
                black_box(legacy_select_packages(&metadata, &selected_names));
            }));
            candidate.push(measure_ns(|| {
                black_box(selected_package_index(&metadata, &selected_names).unwrap());
            }));
        } else {
            candidate.push(measure_ns(|| {
                black_box(selected_package_index(&metadata, &selected_names).unwrap());
            }));
            baseline.push(measure_ns(|| {
                black_box(legacy_select_packages(&metadata, &selected_names));
            }));
        }
    }
    baseline.sort_unstable();
    candidate.sort_unstable();
    let baseline_p50 = percentile(&baseline, 50);
    let baseline_p95 = percentile(&baseline, 95);
    let candidate_p50 = percentile(&candidate, 50);
    let candidate_p95 = percentile(&candidate, 95);
    println!(
        "TOOLING15_CARGO_PACKAGE_INDEX_BENCH_V1 packages={PACKAGE_COUNT} selected={SELECTED_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ns={baseline_p50} baseline_p95_ns={baseline_p95} candidate_p50_ns={candidate_p50} candidate_p95_ns={candidate_p95}"
    );
    assert!(candidate_p50.saturating_mul(10) <= baseline_p50.saturating_mul(7));
    assert!(candidate_p95.saturating_mul(10) <= baseline_p95.saturating_mul(7));
}

fn legacy_select_packages<'a>(
    metadata: &'a CargoMetadata,
    selected_names: &HashSet<&str>,
) -> Vec<&'a CargoMetadataPackage> {
    let mut selected = Vec::with_capacity(selected_names.len());
    for name in selected_names {
        let mut matches = metadata
            .packages
            .iter()
            .filter(|package| package.name == *name);
        selected.push(matches.next().unwrap());
        assert!(matches.next().is_none());
    }
    selected
}

#[test]
#[ignore = "explicit release-mode tooling performance evidence"]
fn cargo_header_only_message_parse_performance_evidence() {
    const MESSAGE_COUNT: usize = 20_000;
    const WARMUP_ROUNDS: usize = 5;
    const SAMPLE_ROUNDS: usize = 51;

    let message = serde_json::to_vec(&serde_json::json!({
        "reason": "compiler-artifact",
        "package_id": "registry+fixture#dependency@1.0.0",
        "target": {"name": "dependency", "kind": ["lib"]},
        "filenames": [
            "C:\\target\\debug\\deps\\dependency.dll",
            "C:\\target\\debug\\deps\\dependency.dll.lib",
            "C:\\target\\debug\\deps\\dependency.pdb",
            "C:\\target\\debug\\deps\\dependency.rlib",
            "C:\\target\\debug\\deps\\dependency.rmeta"
        ],
        "executable": null,
        "fresh": false,
        "profile": {"opt_level": "0", "debuginfo": 2}
    }))
    .unwrap();

    for _ in 0..WARMUP_ROUNDS {
        black_box(parse_legacy_messages(&message, MESSAGE_COUNT));
        black_box(parse_header_messages(&message, MESSAGE_COUNT));
    }
    let mut baseline = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        if round % 2 == 0 {
            baseline.push(measure_ns(|| {
                black_box(parse_legacy_messages(&message, MESSAGE_COUNT));
            }));
            candidate.push(measure_ns(|| {
                black_box(parse_header_messages(&message, MESSAGE_COUNT));
            }));
        } else {
            candidate.push(measure_ns(|| {
                black_box(parse_header_messages(&message, MESSAGE_COUNT));
            }));
            baseline.push(measure_ns(|| {
                black_box(parse_legacy_messages(&message, MESSAGE_COUNT));
            }));
        }
    }
    baseline.sort_unstable();
    candidate.sort_unstable();
    let baseline_p50 = percentile(&baseline, 50);
    let baseline_p95 = percentile(&baseline, 95);
    let candidate_p50 = percentile(&candidate, 50);
    let candidate_p95 = percentile(&candidate, 95);
    println!(
        "TOOLING15_CARGO_HEADER_PARSE_BENCH_V1 messages={MESSAGE_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ns={baseline_p50} baseline_p95_ns={baseline_p95} candidate_p50_ns={candidate_p50} candidate_p95_ns={candidate_p95}"
    );
    assert!(
        candidate_p50.saturating_mul(10) <= baseline_p50.saturating_mul(8),
        "candidate P50 did not improve by at least 20%"
    );
    assert!(
        candidate_p95.saturating_mul(10) <= baseline_p95.saturating_mul(8),
        "candidate P95 did not improve by at least 20%"
    );
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct LegacyCargoMessage {
    reason: String,
    package_id: Option<String>,
    target: Option<LegacyCargoTarget>,
    #[serde(default)]
    filenames: Vec<PathBuf>,
    executable: Option<PathBuf>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct LegacyCargoTarget {
    name: String,
    kind: Vec<String>,
}

fn parse_legacy_messages(message: &[u8], count: usize) -> usize {
    (0..count)
        .map(|_| {
            let parsed: LegacyCargoMessage = serde_json::from_slice(black_box(message)).unwrap();
            parsed.filenames.len()
        })
        .sum()
}

fn parse_header_messages(message: &[u8], count: usize) -> usize {
    (0..count)
        .map(|_| {
            let parsed: CargoMessageHeader<'_> =
                serde_json::from_slice(black_box(message)).unwrap();
            usize::from(parsed.reason == "compiler-artifact")
                + usize::from(parsed.target.is_some_and(|target| target.is_binary))
        })
        .sum()
}

#[test]
#[ignore = "explicit release-mode tooling performance evidence"]
fn borrowed_runtime_declaration_performance_evidence() {
    const DECLARATION_COUNT: usize = 4_096;
    const WARMUP_ROUNDS: usize = 5;
    const SAMPLE_ROUNDS: usize = 51;

    let declarations = (0..DECLARATION_COUNT)
        .map(|index| CargoRuntimeDependencyDeclaration {
            logical_name: format!("runtime-library-{index:04}"),
            relative_path: format!("bin/runtime/library-{index:04}.dll"),
            package: format!("zircon-runtime-package-{index:04}"),
            target: format!("zircon_runtime_target_{index:04}"),
            artifact_file_name: format!("zircon_runtime_artifact_{index:04}.dll"),
        })
        .collect::<Vec<_>>();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) =
            measure_runtime_declaration_capture(&declarations, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }
    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) =
            measure_runtime_declaration_capture(&declarations, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
        baseline_samples.push(baseline.1);
        candidate_samples.push(candidate.1);
    }
    baseline_samples.sort_unstable();
    candidate_samples.sort_unstable();
    let baseline_p50 = percentile(&baseline_samples, 50);
    let baseline_p95 = percentile(&baseline_samples, 95);
    let candidate_p50 = percentile(&candidate_samples, 50);
    let candidate_p95 = percentile(&candidate_samples, 95);
    println!(
        "TOOLING15_BORROWED_RUNTIME_DECLARATION_BENCH_V1 declarations={DECLARATION_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ns={baseline_p50} baseline_p95_ns={baseline_p95} candidate_p50_ns={candidate_p50} candidate_p95_ns={candidate_p95}"
    );
    assert!(
        candidate_p50.saturating_mul(4) <= baseline_p50.saturating_mul(3),
        "candidate P50 did not improve by at least 25%"
    );
    assert!(
        candidate_p95.saturating_mul(4) <= baseline_p95.saturating_mul(3),
        "candidate P95 did not improve by at least 25%"
    );
}

fn measure_runtime_declaration_capture(
    declarations: &[CargoRuntimeDependencyDeclaration],
    baseline_first: bool,
) -> ((usize, u128), (usize, u128)) {
    let mut baseline = (0_usize, 0_u128);
    let mut candidate = (0_usize, 0_u128);
    let mut run_baseline = || {
        let started = Instant::now();
        let length = declarations
            .iter()
            .map(|declaration| {
                let declaration = black_box(declaration.clone());
                declaration.logical_name.len() + declaration.relative_path.len()
            })
            .sum();
        baseline = (length, started.elapsed().as_nanos());
    };
    let mut run_candidate = || {
        let started = Instant::now();
        let length = declarations
            .iter()
            .map(|declaration| {
                let names = black_box((
                    declaration.logical_name.clone(),
                    declaration.relative_path.clone(),
                ));
                names.0.len() + names.1.len()
            })
            .sum();
        candidate = (length, started.elapsed().as_nanos());
    };
    if baseline_first {
        run_baseline();
        run_candidate();
    } else {
        run_candidate();
        run_baseline();
    }
    (baseline, candidate)
}

#[test]
#[ignore = "explicit release-mode tooling performance evidence"]
fn cargo_graph_path_buffer_performance_evidence() {
    const PATH_COUNT: usize = 20_000;
    const WARMUP_ROUNDS: usize = 5;
    const SAMPLE_ROUNDS: usize = 51;
    const REQUIRED_PERCENT: u128 = 85;

    let paths = (0..PATH_COUNT)
        .map(|index| {
            PathBuf::from(format!(
                "crates/package_{:04}/src/generated/module_{index:06}.rs",
                index % 1_024
            ))
        })
        .collect::<Vec<_>>();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_graph_paths(&paths, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }
    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_graph_paths(&paths, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
        baseline_samples.push(baseline.1);
        candidate_samples.push(candidate.1);
    }
    baseline_samples.sort_unstable();
    candidate_samples.sort_unstable();
    let baseline_p50 = percentile_duration(&baseline_samples, 50);
    let baseline_p95 = percentile_duration(&baseline_samples, 95);
    let candidate_p50 = percentile_duration(&candidate_samples, 50);
    let candidate_p95 = percentile_duration(&candidate_samples, 95);

    println!(
        "TOOLING15_CARGO_GRAPH_PATH_BUFFER_BENCH_V1 paths={PATH_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 15%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 15%"
    );
}

#[test]
#[ignore = "explicit release-mode tooling performance evidence"]
fn local_manifest_path_reuse_performance_evidence() {
    const PACKAGE_COUNT: usize = 8_192;
    const WARMUP_ROUNDS: usize = 3;
    const SAMPLE_ROUNDS: usize = 51;
    const REQUIRED_PERCENT: u128 = 90;

    let snapshot_root = if cfg!(windows) {
        Path::new("C:/buildsets/source")
    } else {
        Path::new("/buildsets/source")
    };
    let packages = (0..PACKAGE_COUNT)
        .map(|index| CargoMetadataPackage {
            id: format!("path+file:///fixture#package_{index:06}@1.0.0"),
            name: format!("package_{index:06}"),
            version: "1.0.0".to_string(),
            source: None,
            checksum: None,
            manifest_path: snapshot_root.join(format!(
                "crates/family_{:04}/package_{index:06}/Cargo.toml",
                index % 1_024
            )),
            features: BTreeMap::new(),
            targets: Vec::new(),
        })
        .collect::<Vec<_>>();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) =
            measure_local_manifest_paths(&packages, snapshot_root, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }
    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) =
            measure_local_manifest_paths(&packages, snapshot_root, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
        baseline_samples.push(baseline.1);
        candidate_samples.push(candidate.1);
    }
    baseline_samples.sort_unstable();
    candidate_samples.sort_unstable();
    let baseline_p50 = percentile_duration(&baseline_samples, 50);
    let baseline_p95 = percentile_duration(&baseline_samples, 95);
    let candidate_p50 = percentile_duration(&candidate_samples, 50);
    let candidate_p95 = percentile_duration(&candidate_samples, 95);

    println!(
        "TOOLING15_LOCAL_MANIFEST_PATH_REUSE_BENCH_V1 packages={PACKAGE_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 10%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 10%"
    );
}

#[test]
#[ignore = "explicit release-mode tooling performance evidence"]
fn unchanged_package_id_reuse_performance_evidence() {
    const REFERENCE_COUNT: usize = 8_192;
    const WARMUP_ROUNDS: usize = 5;
    const SAMPLE_ROUNDS: usize = 51;
    const REQUIRED_PERCENT: u128 = 70;

    let references = (0..REFERENCE_COUNT)
        .map(|index| format!("registry+https://example.invalid/index#dependency_{index:06}@1.2.3"))
        .collect::<Vec<_>>();
    let baseline_package_ids = references
        .iter()
        .map(|id| (id.clone(), id.clone()))
        .collect::<HashMap<_, _>>();
    let candidate_package_ids = references
        .iter()
        .map(|id| (id.clone(), None))
        .collect::<HashMap<_, _>>();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_package_id_references(
            &references,
            &baseline_package_ids,
            &candidate_package_ids,
            round % 2 == 0,
        );
        assert_eq!(baseline.0, candidate.0);
    }
    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_package_id_references(
            &references,
            &baseline_package_ids,
            &candidate_package_ids,
            round % 2 == 0,
        );
        assert_eq!(baseline.0, candidate.0);
        baseline_samples.push(baseline.1);
        candidate_samples.push(candidate.1);
    }
    baseline_samples.sort_unstable();
    candidate_samples.sort_unstable();
    let baseline_p50 = percentile_duration(&baseline_samples, 50);
    let baseline_p95 = percentile_duration(&baseline_samples, 95);
    let candidate_p50 = percentile_duration(&candidate_samples, 50);
    let candidate_p95 = percentile_duration(&candidate_samples, 95);

    println!(
        "TOOLING15_UNCHANGED_PACKAGE_ID_REUSE_BENCH_V1 references={REFERENCE_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 30%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 30%"
    );
}

#[test]
#[ignore = "explicit release-mode tooling performance evidence"]
fn external_package_index_reuse_performance_evidence() {
    const PACKAGE_COUNT: usize = 8_192;
    const WARMUP_ROUNDS: usize = 5;
    const SAMPLE_ROUNDS: usize = 51;
    const REQUIRED_PERCENT: u128 = 80;

    let package_ids = (0..PACKAGE_COUNT)
        .map(|index| format!("registry+https://example.invalid/index#dependency_{index:06}@1.2.3"))
        .collect::<Vec<_>>();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_external_package_index(&package_ids, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }
    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_external_package_index(&package_ids, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
        baseline_samples.push(baseline.1);
        candidate_samples.push(candidate.1);
    }
    baseline_samples.sort_unstable();
    candidate_samples.sort_unstable();
    let baseline_p50 = percentile_duration(&baseline_samples, 50);
    let baseline_p95 = percentile_duration(&baseline_samples, 95);
    let candidate_p50 = percentile_duration(&candidate_samples, 50);
    let candidate_p95 = percentile_duration(&candidate_samples, 95);

    println!(
        "TOOLING15_EXTERNAL_PACKAGE_INDEX_REUSE_BENCH_V1 packages={PACKAGE_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 20%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 20%"
    );
}

#[test]
#[ignore = "explicit release-mode tooling performance evidence"]
fn unstable_cargo_graph_sort_performance_evidence() {
    const VALUE_COUNT: usize = 20_000;
    const WARMUP_ROUNDS: usize = 5;
    const SAMPLE_ROUNDS: usize = 51;
    const REQUIRED_PERCENT: u128 = 90;

    let values = (0..VALUE_COUNT)
        .rev()
        .map(|index| format!("registry+fixture#dependency_{index:06}@1.2.3"))
        .collect::<Vec<_>>();

    for round in 0..WARMUP_ROUNDS {
        let (baseline, candidate) = measure_graph_value_sort(&values, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
    }
    let mut baseline_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_ROUNDS);
    for round in 0..SAMPLE_ROUNDS {
        let (baseline, candidate) = measure_graph_value_sort(&values, round % 2 == 0);
        assert_eq!(baseline.0, candidate.0);
        baseline_samples.push(baseline.1);
        candidate_samples.push(candidate.1);
    }
    baseline_samples.sort_unstable();
    candidate_samples.sort_unstable();
    let baseline_p50 = percentile_duration(&baseline_samples, 50);
    let baseline_p95 = percentile_duration(&baseline_samples, 95);
    let candidate_p50 = percentile_duration(&candidate_samples, 50);
    let candidate_p95 = percentile_duration(&candidate_samples, 95);

    println!(
        "TOOLING15_UNSTABLE_CARGO_GRAPH_SORT_BENCH_V1 values={VALUE_COUNT} rounds={SAMPLE_ROUNDS} baseline_p50_ms={:.4} baseline_p95_ms={:.4} candidate_p50_ms={:.4} candidate_p95_ms={:.4}",
        baseline_p50.as_secs_f64() * 1_000.0,
        baseline_p95.as_secs_f64() * 1_000.0,
        candidate_p50.as_secs_f64() * 1_000.0,
        candidate_p95.as_secs_f64() * 1_000.0,
    );
    assert!(
        candidate_p50.as_nanos() * 100 <= baseline_p50.as_nanos() * REQUIRED_PERCENT,
        "candidate P50 did not improve by at least 10%"
    );
    assert!(
        candidate_p95.as_nanos() * 100 <= baseline_p95.as_nanos() * REQUIRED_PERCENT,
        "candidate P95 did not improve by at least 10%"
    );
}

fn measure_graph_value_sort(
    values: &[String],
    baseline_first: bool,
) -> ((Vec<String>, Duration), (Vec<String>, Duration)) {
    let mut baseline_values = values.to_vec();
    let mut candidate_values = values.to_vec();
    let measure_baseline = |values: &mut Vec<String>| {
        let started = Instant::now();
        values.sort();
        (black_box(std::mem::take(values)), started.elapsed())
    };
    let measure_candidate = |values: &mut Vec<String>| {
        let started = Instant::now();
        values.sort_unstable();
        (black_box(std::mem::take(values)), started.elapsed())
    };
    if baseline_first {
        (
            measure_baseline(&mut baseline_values),
            measure_candidate(&mut candidate_values),
        )
    } else {
        let candidate = measure_candidate(&mut candidate_values);
        let baseline = measure_baseline(&mut baseline_values);
        (baseline, candidate)
    }
}

fn measure_local_manifest_paths(
    packages: &[CargoMetadataPackage],
    snapshot_root: &Path,
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    let measure_baseline = || {
        let started = Instant::now();
        let length = packages
            .iter()
            .map(|package| {
                let manifest =
                    super::canonical_snapshot_path(snapshot_root, &package.manifest_path).unwrap();
                let canonical_id = format!(
                    "path+build-set:///{manifest}#{}@{}",
                    package.name, package.version
                );
                let canonical_manifest_path = PathBuf::from(
                    super::canonical_snapshot_path(snapshot_root, &package.manifest_path).unwrap(),
                );
                canonical_id.len() + canonical_manifest_path.as_os_str().len()
            })
            .sum();
        (black_box(length), started.elapsed())
    };
    let measure_candidate = || {
        let started = Instant::now();
        let length = packages
            .iter()
            .map(|package| {
                let (canonical_id, canonical_manifest_path) =
                    canonical_package_id(package, snapshot_root).unwrap();
                canonical_id.unwrap().len() + canonical_manifest_path.as_os_str().len()
            })
            .sum();
        (black_box(length), started.elapsed())
    };
    if baseline_first {
        (measure_baseline(), measure_candidate())
    } else {
        let candidate = measure_candidate();
        let baseline = measure_baseline();
        (baseline, candidate)
    }
}

fn measure_external_package_index(
    package_ids: &[String],
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    let measure_baseline = || {
        let started = Instant::now();
        let index = package_ids
            .iter()
            .map(|id| (id.clone(), id.clone()))
            .collect::<HashMap<_, _>>();
        let count = black_box(index.len());
        black_box(index);
        (count, started.elapsed())
    };
    let measure_candidate = || {
        let started = Instant::now();
        let index = package_ids
            .iter()
            .map(|id| (id.clone(), None))
            .collect::<HashMap<String, Option<String>>>();
        let count = black_box(index.len());
        black_box(index);
        (count, started.elapsed())
    };
    if baseline_first {
        (measure_baseline(), measure_candidate())
    } else {
        let candidate = measure_candidate();
        let baseline = measure_baseline();
        (baseline, candidate)
    }
}

fn measure_package_id_references(
    references: &[String],
    baseline_package_ids: &HashMap<String, String>,
    candidate_package_ids: &HashMap<String, Option<String>>,
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    let mut baseline_references = references.to_vec();
    let mut candidate_references = references.to_vec();
    let measure_baseline = |references: &mut [String]| {
        let started = Instant::now();
        for id in references.iter_mut() {
            *id = baseline_package_ids.get(id.as_str()).unwrap().clone();
        }
        let length = black_box(references.iter().map(String::len).sum::<usize>());
        (length, started.elapsed())
    };
    let measure_candidate = |references: &mut [String]| {
        let started = Instant::now();
        for id in references.iter_mut() {
            canonicalize_package_id_reference_in_place(id, candidate_package_ids).unwrap();
        }
        let length = black_box(references.iter().map(String::len).sum::<usize>());
        (length, started.elapsed())
    };
    if baseline_first {
        (
            measure_baseline(&mut baseline_references),
            measure_candidate(&mut candidate_references),
        )
    } else {
        let candidate = measure_candidate(&mut candidate_references);
        let baseline = measure_baseline(&mut baseline_references);
        (baseline, candidate)
    }
}

fn measure_graph_paths(
    paths: &[PathBuf],
    baseline_first: bool,
) -> ((usize, Duration), (usize, Duration)) {
    let mut baseline = (0_usize, Duration::ZERO);
    let mut candidate = (0_usize, Duration::ZERO);
    let mut run_baseline = || {
        let started = Instant::now();
        let length = paths
            .iter()
            .map(|path| legacy_canonical_relative_path(black_box(path)).len())
            .sum();
        baseline = (length, started.elapsed());
    };
    let mut run_candidate = || {
        let started = Instant::now();
        let length = paths
            .iter()
            .map(|path| {
                super::canonical_relative_path(black_box(path), "benchmark path")
                    .unwrap()
                    .len()
            })
            .sum();
        candidate = (length, started.elapsed());
    };
    if baseline_first {
        run_baseline();
        run_candidate();
    } else {
        run_candidate();
        run_baseline();
    }
    (baseline, candidate)
}

fn legacy_canonical_relative_path(path: &Path) -> String {
    let mut canonical = String::new();
    for component in path.components() {
        let Component::Normal(component) = component else {
            panic!("benchmark fixture path must be normalized");
        };
        if !canonical.is_empty() {
            canonical.push('/');
        }
        canonical.push_str(component.to_str().unwrap());
    }
    canonical
}

fn measure_ns(run: impl FnOnce()) -> u128 {
    let started = Instant::now();
    run();
    started.elapsed().as_nanos()
}

fn percentile_duration(samples: &[Duration], percentile: usize) -> Duration {
    samples[(samples.len() - 1) * percentile / 100]
}

fn percentile(sorted: &[u128], percentile: usize) -> u128 {
    let index = (sorted.len() - 1) * percentile / 100;
    sorted[index]
}
