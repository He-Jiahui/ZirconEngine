use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::asset::AssetUri;
use zircon_runtime_interface::project::MAX_PROJECT_ASSET_ROOTS;

fn manifest(roots: &[&str]) -> ProjectManifest {
    let mut manifest = ProjectManifest::new(
        "Root validation",
        AssetUri::parse("res://main.scene.toml").unwrap(),
        1,
    );
    manifest.asset_roots = roots
        .iter()
        .map(|root| RelPath::parse(root).unwrap())
        .collect();
    manifest
}

fn legacy_asset_validation(roots: &[RelPath]) -> Result<(), ProjectManifestError> {
    if roots.is_empty() {
        return Err(ProjectManifestError::EmptyAssetRoots);
    }
    if roots.len() > MAX_PROJECT_ASSET_ROOTS {
        return Err(ProjectManifestError::Summary(
            ProjectManifestSummaryError::TooManyAssetRoots {
                max: MAX_PROJECT_ASSET_ROOTS,
                found: roots.len(),
            },
        ));
    }
    let mut seen = HashSet::with_capacity(roots.len());
    for root in roots {
        if !seen.insert(root.as_str()) {
            return Err(ProjectManifestError::DuplicateAssetRoot {
                root: root.to_string(),
            });
        }
    }
    for (index, left) in roots.iter().enumerate() {
        for right in &roots[index + 1..] {
            for (ancestor, descendant) in [(left, right), (right, left)] {
                if descendant
                    .as_str()
                    .strip_prefix(ancestor.as_str())
                    .is_some_and(|suffix| suffix.starts_with('/'))
                {
                    return Err(ProjectManifestError::OverlappingAssetRoots {
                        ancestor: ancestor.to_string(),
                        descendant: descendant.to_string(),
                    });
                }
            }
        }
    }
    Ok(())
}

fn legacy_manifest_validation(manifest: &ProjectManifest) -> Result<(), ProjectManifestError> {
    validate_engine_version_req(manifest.engine_version_req.as_deref())?;
    if let Some(receipt) = &manifest.template_receipt {
        if receipt.project_guid() != manifest.project_guid {
            return Err(ProjectManifestError::TemplateReceiptProjectGuidMismatch {
                manifest_guid: manifest.project_guid,
                receipt_guid: receipt.project_guid(),
            });
        }
        if receipt.descriptor().engine_version_req() != manifest.engine_version_req.as_deref() {
            return Err(ProjectManifestError::TemplateReceiptEngineRequirementMismatch);
        }
    }
    legacy_asset_validation(&manifest.asset_roots)?;
    let mut ui_roots = HashSet::with_capacity(manifest.ui_roots.len());
    for root in &manifest.ui_roots {
        if root.scheme() != ResourceScheme::Res {
            return Err(ProjectManifestError::InvalidUiRootScheme {
                root: root.to_string(),
            });
        }
        if root.path().trim().is_empty() {
            return Err(ProjectManifestError::EmptyUiRoot);
        }
        if root.label().is_some() {
            return Err(ProjectManifestError::LabelledUiRoot {
                root: root.to_string(),
            });
        }
        if !ui_roots.insert(root.to_string()) {
            return Err(ProjectManifestError::DuplicateUiRoot {
                root: root.to_string(),
            });
        }
    }
    Ok(())
}

fn assert_same_error(manifest: &ProjectManifest) {
    assert_eq!(
        manifest.validate().map_err(|error| error.to_string()),
        legacy_manifest_validation(manifest).map_err(|error| error.to_string()),
        "roots: {:?}",
        manifest.asset_roots
    );
}

fn check_permutations(roots: &mut [&str], start: usize) {
    if start == roots.len() {
        assert_same_error(&manifest(roots));
        return;
    }
    for index in start..roots.len() {
        roots.swap(start, index);
        check_permutations(roots, start + 1);
        roots.swap(start, index);
    }
}

#[test]
fn astra_m2_root_overlap_preserves_input_error_precedence() {
    check_permutations(&mut ["a", "a/one", "a/one/two", "a-other", "z", "z/one"], 0);
    for roots in [
        vec![],
        vec!["a"],
        vec!["a", "ab", "a-b", "ab/x"],
        vec!["a", "a/x", "z", "z"],
        vec!["a/x", "a/y", "a"],
        vec!["a/x", "a//x/"],
        vec!["Assets", "assets/x"],
        vec!["a/bc", "a/b", "a"],
        vec!["a/x", "z", "z/x", "a"],
    ] {
        assert_same_error(&manifest(&roots));
    }
}

#[test]
fn astra_m2_ui_roots_preserve_normalization_and_error_priority() {
    let mut manifest = manifest(&["assets"]);
    manifest.ui_roots = ["res://ui/base", "res://ui/base-extra"]
        .map(|value| AssetUri::parse(value).unwrap())
        .into();
    manifest.validate().unwrap();
    manifest
        .ui_roots
        .push(AssetUri::parse("res://ui//base").unwrap());
    assert!(
        matches!(manifest.validate(), Err(ProjectManifestError::DuplicateUiRoot { root }) if root == "res://ui/base")
    );
    manifest.ui_roots = ["res://ui/base", "lib://ui/base"]
        .map(|value| AssetUri::parse(value).unwrap())
        .into();
    assert!(matches!(
        manifest.validate(),
        Err(ProjectManifestError::InvalidUiRootScheme { .. })
    ));
    manifest.ui_roots = ["res://ui/base", "res://ui/base#label"]
        .map(|value| AssetUri::parse(value).unwrap())
        .into();
    assert!(matches!(
        manifest.validate(),
        Err(ProjectManifestError::LabelledUiRoot { .. })
    ));
    assert_same_error(&manifest);
    assert!(AssetUri::parse("res://").is_err());
    manifest.ui_roots = ["res://ui/base"]
        .map(|value| AssetUri::parse(value).unwrap())
        .into();
    manifest.engine_version_req = Some("not-a-version".to_owned());
    assert_same_error(&manifest);
    assert!(manifest.validate().is_err());
}

#[test]
fn astra_m2_maximum_sibling_roots_are_valid_and_overlap_is_detected_at_budget() {
    let mut manifest = manifest(&["assets"]);
    manifest.asset_roots = (0..MAX_PROJECT_ASSET_ROOTS - 1)
        .map(|id| RelPath::parse(format!("assets/root_{id:05}")).unwrap())
        .collect();
    manifest.validate().unwrap();
    manifest.asset_roots.push(RelPath::parse("assets").unwrap());
    assert!(
        matches!(manifest.validate(), Err(ProjectManifestError::OverlappingAssetRoots { ancestor, descendant }) if ancestor == "assets" && descendant == "assets/root_00000")
    );
}

#[test]
fn astra_m2_asset_root_count_matches_interface_admission_limit() {
    let mut manifest = manifest(&["assets"]);
    manifest.asset_roots = (0..MAX_PROJECT_ASSET_ROOTS)
        .map(|id| RelPath::parse(format!("root-{id:04}")).unwrap())
        .collect();
    manifest.validate().unwrap();

    manifest
        .asset_roots
        .push(RelPath::parse("root-over-limit").unwrap());
    assert!(matches!(
        manifest.validate(),
        Err(ProjectManifestError::Summary(
            ProjectManifestSummaryError::TooManyAssetRoots { max, found }
        )) if max == MAX_PROJECT_ASSET_ROOTS && found == MAX_PROJECT_ASSET_ROOTS + 1
    ));
}

fn percentiles(samples: &mut [u128]) -> [u128; 3] {
    samples.sort_unstable();
    [50, 95, 99].map(|p| samples[(samples.len() * p).div_ceil(100) - 1])
}

#[test]
#[ignore = "Windows release evidence through the coordinator"]
fn astra_m2_asset_root_validation_release_evidence() {
    assert!(
        !cfg!(debug_assertions),
        "performance evidence requires release"
    );
    const SAMPLES: usize = 101;
    for (kind, count) in [
        ("asset", 1),
        ("asset", 1_000),
        ("asset", MAX_PROJECT_ASSET_ROOTS),
        ("ui", 1),
        ("ui", 1_000),
        ("ui", 10_000),
    ] {
        let mut manifest = manifest(&["assets"]);
        if kind == "asset" {
            manifest.asset_roots = (0..count)
                .map(|id| RelPath::parse(format!("assets/root_{id:05}")).unwrap())
                .collect();
        } else {
            manifest.ui_roots = (0..count)
                .map(|id| AssetUri::parse(&format!("res://ui/root_{id:05}")).unwrap())
                .collect();
        }
        assert_same_error(&manifest);
        let validate = |optimized| {
            if optimized {
                black_box(&manifest).validate()
            } else {
                legacy_manifest_validation(black_box(&manifest))
            }
        };
        for _ in 0..8 {
            validate(false).unwrap();
            validate(true).unwrap();
        }
        let iterations = if count == 1 { 1_000 } else { 1 };
        let mut before = Vec::with_capacity(SAMPLES);
        let mut after = Vec::with_capacity(SAMPLES);
        for sample in 0..SAMPLES {
            for optimized in if sample % 2 == 0 {
                [false, true]
            } else {
                [true, false]
            } {
                let started = Instant::now();
                for _ in 0..iterations {
                    validate(optimized).unwrap();
                }
                let elapsed = started.elapsed().as_nanos();
                if optimized {
                    after.push(elapsed);
                } else {
                    before.push(elapsed);
                }
            }
        }
        let before = percentiles(&mut before);
        let after = percentiles(&mut after);
        let limit = if count == 1 {
            105
        } else if kind == "asset" {
            20
        } else {
            80
        };
        println!("ASTRA_M2_ROOTS profile=release kind={kind} roots={count} samples={SAMPLES} warmup=8 iterations={iterations} before_p50_p95_p99_ns={before:?} after_p50_p95_p99_ns={after:?} p95_limit_percent={limit}");
        assert!(
            after[1] * 100 <= before[1] * limit,
            "root validation p95 target missed: {before:?} -> {after:?}"
        );
    }
}
