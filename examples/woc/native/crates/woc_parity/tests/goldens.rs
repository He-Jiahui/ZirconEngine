use std::path::{Path, PathBuf};

use woc_parity::{GoldenError, GoldenSuite, GoldenUpdateGuard};

fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

#[test]
fn all_54_current_head_goldens_are_present_and_hash_locked() {
    let suite = GoldenSuite::load(project_root().join("reference/current-head/parity"))
        .expect("committed parity suite must load");
    assert_eq!(suite.scenarios().len(), 54);
    for scenario in suite.scenarios() {
        suite
            .read_expected(&scenario.name)
            .expect("every manifest row must have a hash-matched golden");
    }
}

#[test]
fn comparison_reports_the_first_structural_difference() {
    let suite = GoldenSuite::load(project_root().join("reference/current-head/parity"))
        .expect("committed parity suite must load");
    let scenario = &suite.scenarios()[0];
    let mut actual = suite
        .read_expected(&scenario.name)
        .expect("fixture golden must load");
    actual["scenario"] = serde_json::Value::String("changed".to_string());
    let error = suite
        .compare(&scenario.name, &actual)
        .expect_err("changed trace must fail");
    assert!(matches!(error, GoldenError::Difference { ref path, .. } if path == "$.scenario"));
}

#[test]
fn double_run_compares_duplicates_before_the_golden() {
    let suite = GoldenSuite::load(project_root().join("reference/current-head/parity"))
        .expect("committed parity suite must load");
    let scenario = &suite.scenarios()[0];
    let expected = suite
        .read_expected(&scenario.name)
        .expect("fixture golden must load");
    suite
        .compare_double_run(&scenario.name, || expected.clone())
        .expect("identical duplicate runs must match the golden");
}

#[test]
fn golden_updates_are_disabled_without_two_factor_confirmation() {
    assert!(!GoldenUpdateGuard::disabled().is_enabled());
    assert!(!GoldenUpdateGuard::from_values(Some("1"), None).is_enabled());
    assert!(!GoldenUpdateGuard::from_values(Some("0"), Some("confirm")).is_enabled());
}
