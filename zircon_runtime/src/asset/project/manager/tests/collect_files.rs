use std::collections::HashSet;
use std::fs;
use std::hint::black_box;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use crate::asset::AssetImportError;

use super::{
    collect_compound_files_with_limit, collect_files_excluding_subtrees, finish_matching_visit,
    is_auxiliary_source_extension, is_auxiliary_source_file, visit_matching_files,
};

static NEXT_TEST_ROOT: AtomicU64 = AtomicU64::new(1);

#[test]
fn source_collection_ignores_atomic_write_transaction_siblings() {
    let root = std::env::temp_dir().join(format!(
        "zircon_collect_files_atomic_siblings_{}_{}",
        std::process::id(),
        NEXT_TEST_ROOT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&root).unwrap();
    let source = root.join("material.zmaterial");
    let staging = root.join(".material.zmaterial.zr-staging-123-4");
    let backup = root.join(".material.zmaterial.zr-backup-123-5");
    fs::write(&source, "source").unwrap();
    fs::write(&staging, "staging").unwrap();
    fs::write(&backup, "backup").unwrap();

    let mut files = Vec::new();
    collect_files_excluding_subtrees(&root, &mut files, &HashSet::new()).unwrap();

    assert_eq!(files, vec![source]);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn source_collection_ignores_only_canonical_project_transaction_siblings() {
    let root = test_root("project_transaction_siblings");
    fs::create_dir_all(&root).unwrap();
    let basename = "a".repeat(64);
    let transaction_id = format!("{}-42-3", "b".repeat(64));
    let source = root.join("visible.zasset");
    fs::write(&source, b"source").unwrap();

    for role in ["stage", "backup", "rollback-stage"] {
        fs::write(
            root.join(format!(".{basename}.zr-project-{role}-{transaction_id}")),
            b"transaction artifact",
        )
        .unwrap();
    }

    let retained = [
        root.join(format!(
            ".{basename}.zr-project-stage-{transaction_id}-copy"
        )),
        root.join(format!(
            ".{basename}.zr-project-stage-{}-042-3",
            "b".repeat(64)
        )),
        root.join(format!(".{basename}.zr-editor-stage-{transaction_id}")),
        root.join(format!(
            ".{basename}.zr-project-retired-backup-{transaction_id}"
        )),
    ];
    for path in &retained {
        fs::write(path, b"user file").unwrap();
    }

    let mut files = Vec::new();
    collect_files_excluding_subtrees(&root, &mut files, &HashSet::new()).unwrap();
    files.sort();
    let mut expected = vec![source];
    expected.extend(retained);
    expected.sort();
    assert_eq!(files, expected);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn source_collection_ignores_replace_file_zmeta_temps_but_keeps_lookalikes() {
    let root = test_root("replace_file_meta_temps");
    fs::create_dir_all(&root).unwrap();
    let source = root.join("visible.zasset");
    let windows_temp = root.join("pbr_shader.zmeta~RF63fc050.TMP");
    let retained = [
        root.join("user.zmeta~RF63fc05.TMP"),
        root.join("user.TMP"),
        root.join("user.zmeta~RF63fc050.TMP.backup"),
    ];
    fs::write(&source, b"source").unwrap();
    fs::write(&windows_temp, b"replacement temp").unwrap();
    for path in &retained {
        fs::write(path, b"user file").unwrap();
    }

    let mut files = Vec::new();
    collect_files_excluding_subtrees(&root, &mut files, &HashSet::new()).unwrap();
    files.sort();
    let mut expected = vec![source];
    expected.extend(retained);
    expected.sort();
    assert_eq!(files, expected);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn compound_collection_stops_before_growing_past_member_budget() {
    let root = std::env::temp_dir().join(format!(
        "zircon_collect_compound_budget_{}_{}",
        std::process::id(),
        NEXT_TEST_ROOT.fetch_add(1, Ordering::Relaxed)
    ));
    let nested = root.join("nested");
    fs::create_dir_all(&nested).unwrap();
    let first = root.join("first.zcube");
    let second = nested.join("second.zcube");
    fs::write(&first, "first").unwrap();
    fs::write(&second, "second").unwrap();
    fs::write(root.join("buffer.bin"), "auxiliary").unwrap();
    fs::write(root.join("asset.zmeta"), "sidecar").unwrap();

    let mut files = Vec::new();
    collect_compound_files_with_limit(&root, &mut files, 2).unwrap();
    files.sort();
    let mut expected = vec![first, second];
    expected.sort();
    assert_eq!(files, expected);

    fs::write(nested.join("overflow.zcube"), "third").unwrap();
    let mut admitted = Vec::new();
    let error = collect_compound_files_with_limit(&root, &mut admitted, 2)
        .expect_err("third member must fail during collection");
    assert!(error.to_string().contains("2-file limit"));
    assert!(admitted.len() <= 2);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn source_collection_does_not_revisit_admitted_compound_subtrees() {
    let root = std::env::temp_dir().join(format!(
        "zircon_collect_files_compound_subtree_{}_{}",
        std::process::id(),
        NEXT_TEST_ROOT.fetch_add(1, Ordering::Relaxed)
    ));
    let compound = root.join("compound");
    fs::create_dir_all(compound.join("nested")).unwrap();
    let ordinary = root.join("ordinary.zcube");
    fs::write(&ordinary, b"ordinary").unwrap();
    fs::write(compound.join("nested").join("member.zcube"), b"member").unwrap();

    let mut files = Vec::new();
    collect_files_excluding_subtrees(&root, &mut files, &HashSet::from([compound])).unwrap();
    assert_eq!(files, vec![ordinary]);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn matching_visitor_yields_only_nested_regular_matches() {
    let root = test_root("matching_visitor");
    let nested = root.join("nested");
    fs::create_dir_all(&nested).unwrap();
    let first = root.join("first.zmeta");
    let second = nested.join("second.zmeta");
    fs::write(&first, b"first").unwrap();
    fs::write(&second, b"second").unwrap();
    fs::write(nested.join("ignored.zshader"), b"ignored").unwrap();

    let mut visited = Vec::new();
    visit_matching_files(
        &root,
        |path| path.extension().and_then(|extension| extension.to_str()) == Some("zmeta"),
        |path| {
            visited.push(path.to_path_buf());
            Ok(())
        },
    )
    .unwrap();
    visited.sort();
    assert_eq!(visited, vec![first, second]);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn matching_visitor_rejects_a_link_before_visiting_it() {
    let root = test_root("matching_visitor_link");
    let outside = test_root("matching_visitor_link_target");
    fs::create_dir_all(&root).unwrap();
    fs::write(&outside, b"outside").unwrap();
    let linked = root.join("linked.zmeta");
    if !create_file_link(&outside, &linked) {
        fs::remove_dir_all(root).unwrap();
        fs::remove_file(outside).unwrap();
        return;
    }

    let mut visited = false;
    let error = visit_matching_files(
        &root,
        |_| true,
        |_| {
            visited = true;
            Ok(())
        },
    )
    .unwrap_err();
    assert!(matches!(
        error,
        AssetImportError::UnsafeProjectAssetLink { path } if path == linked
    ));
    assert!(!visited);

    fs::remove_file(linked).unwrap();
    fs::remove_dir_all(root).unwrap();
    fs::remove_file(outside).unwrap();
}

#[test]
fn matching_visitor_preserves_walk_error_priority_and_first_processing_error() {
    let first = AssetImportError::Parse("first metadata error".into());
    let walk_error = AssetImportError::UnsafeProjectAssetLink {
        path: Path::new("later-link").to_path_buf(),
    };
    let error = finish_matching_visit(Err(walk_error), Some(first)).unwrap_err();
    assert!(matches!(
        error,
        AssetImportError::UnsafeProjectAssetLink { path }
            if path == Path::new("later-link")
    ));

    let error = finish_matching_visit(
        Ok(()),
        Some(AssetImportError::Parse("first metadata error".into())),
    )
    .unwrap_err();
    assert!(error.to_string().contains("first metadata error"));
}

fn test_root(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "zircon_collect_files_{label}_{}_{}",
        std::process::id(),
        NEXT_TEST_ROOT.fetch_add(1, Ordering::Relaxed)
    ))
}

#[cfg(unix)]
fn create_file_link(target: &Path, link: &Path) -> bool {
    std::os::unix::fs::symlink(target, link).is_ok()
}

#[cfg(windows)]
fn create_file_link(target: &Path, link: &Path) -> bool {
    match std::os::windows::fs::symlink_file(target, link) {
        Ok(()) => true,
        Err(error)
            if error.kind() == std::io::ErrorKind::PermissionDenied
                || error.raw_os_error() == Some(1314) =>
        {
            false
        }
        Err(error) => panic!("create file reparse fixture failed: {error}"),
    }
}

#[test]
fn optimization_batch_gb_runtime484_auxiliary_extension_dispatch_preserves_supported_set() {
    for extension in ["bin", "BIN", "ttf", "OTF", "woff", "WoFf2"] {
        assert!(is_auxiliary_source_extension(extension), "{extension}");
    }
    for extension in ["", "png", "woff22", "font"] {
        assert!(!is_auxiliary_source_extension(extension), "{extension}");
    }
    assert!(is_auxiliary_source_file(Path::new("fonts/interface.WOFF2")));
    assert!(!is_auxiliary_source_file(Path::new(
        "textures/interface.png"
    )));
}

const CHECKS_PER_SAMPLE: usize = 1_048_576;
const SAMPLE_PAIRS: usize = 17;

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_gb_runtime484_auxiliary_extension_dispatch_benchmark() {
    const INPUT: &str = "woff2";
    for _ in 0..4 {
        black_box(measure_checks(INPUT, false));
        black_box(measure_checks(INPUT, true));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_checks(INPUT, false));
            optimized_samples.push(measure_checks(INPUT, true));
        } else {
            optimized_samples.push(measure_checks(INPUT, true));
            legacy_samples.push(measure_checks(INPUT, false));
        }
    }

    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "RUNTIME484_AUXILIARY_EXTENSION_DISPATCH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} value_bytes={} checks_per_sample={CHECKS_PER_SAMPLE} legacy_candidate_comparisons_per_check=5 optimized_candidate_comparisons_per_check=1 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
        INPUT.len(),
        csv(&legacy_samples),
        csv(&optimized_samples),
    );
    assert!(optimized_p95 <= legacy_p95 * 75 / 100);
}

fn measure_checks(input: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        let matched = if optimized {
            is_auxiliary_source_extension(black_box(input))
        } else {
            legacy_is_auxiliary_source_extension(black_box(input))
        };
        black_box(matched);
    }
    started.elapsed().as_nanos().max(1)
}

fn legacy_is_auxiliary_source_extension(extension: &str) -> bool {
    extension.eq_ignore_ascii_case("bin")
        || extension.eq_ignore_ascii_case("ttf")
        || extension.eq_ignore_ascii_case("otf")
        || extension.eq_ignore_ascii_case("woff")
        || extension.eq_ignore_ascii_case("woff2")
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
