use std::collections::{btree_map::Entry, BTreeMap};
use std::hint::black_box;
use std::time::Instant;

use super::EditorLocalizationBundle;
use crate::core::i18n::EditorLocale;

#[test]
fn bundle_normalizes_locales_and_rejects_invalid_resources() {
    let bundle = EditorLocalizationBundle::from_locale_maps(
        "fixture.editor",
        BTreeMap::from([(
            "zh-cn".to_string(),
            BTreeMap::from([("settings.fixture.label".to_string(), "示例".to_string())]),
        )]),
    )
    .expect("valid plugin bundle should be accepted");

    assert_eq!(
        bundle
            .translation(
                &EditorLocale::parse("zh-CN").unwrap(),
                "settings.fixture.label"
            )
            .as_deref(),
        Some("示例")
    );
    assert!(EditorLocalizationBundle::from_locale_maps(
        "fixture.editor",
        BTreeMap::from([("en".to_string(), BTreeMap::new())]),
    )
    .is_err());
    assert!(
        EditorLocalizationBundle::from_locale_maps(
            "fixture.editor",
            BTreeMap::from([
                (
                    "zh-CN".to_string(),
                    BTreeMap::from([("plugin.fixture.label".to_string(), "甲".to_string())]),
                ),
                (
                    "zh-cn".to_string(),
                    BTreeMap::from([("plugin.fixture.label".to_string(), "乙".to_string())]),
                ),
            ]),
        )
        .is_err(),
        "locale aliases that normalize to one identity must not overwrite each other"
    );
}

#[test]
fn optimization_batch_ie_editor615_duplicate_locale_precedes_empty_translation_error() {
    let error = EditorLocalizationBundle::from_locale_maps(
        "fixture.editor",
        BTreeMap::from([
            (
                "zh-CN".to_string(),
                BTreeMap::from([("plugin.fixture.label".to_string(), "first".to_string())]),
            ),
            ("zh-cn".to_string(), BTreeMap::new()),
        ]),
    )
    .expect_err("normalized locale alias should remain a duplicate");

    assert_eq!(error, "editor translation bundle repeats locale `zh-CN`");
}

#[test]
fn optimization_batch_ie_editor615_locale_admission_uses_single_entry_probe() {
    let source = include_str!("../bundle.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("use std::collections::{btree_map::Entry, BTreeMap};"));
    assert!(production.contains("Entry::Occupied"));
    assert!(production.contains("Entry::Vacant"));
    assert!(!production.contains("locales.contains_key(&locale)"));
}

fn double_probe_admission(keys: &[String]) -> usize {
    let mut admitted = BTreeMap::new();
    for key in keys {
        if admitted.contains_key(key) {
            continue;
        }
        admitted.insert(key.clone(), ());
    }
    admitted.len()
}

fn entry_admission(keys: &[String]) -> usize {
    let mut admitted = BTreeMap::new();
    for key in keys {
        match admitted.entry(key.clone()) {
            Entry::Occupied(_) => {}
            Entry::Vacant(entry) => {
                entry.insert(());
            }
        }
    }
    admitted.len()
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_ie_editor615_single_probe_locale_admission_performance_evidence() {
    const LOCALES: usize = 32_768;
    const SAMPLE_PAIRS: usize = 17;
    let suffix = "x".repeat(128);
    let keys = (0..LOCALES)
        .map(|index| format!("locale.identity.{suffix}.{index:05}"))
        .collect::<Vec<_>>();
    assert_eq!(double_probe_admission(&keys), LOCALES);
    assert_eq!(entry_admission(&keys), LOCALES);
    let mut double_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut entry_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            let started = Instant::now();
            black_box(double_probe_admission(black_box(&keys)));
            double_samples.push(started.elapsed());
            let started = Instant::now();
            black_box(entry_admission(black_box(&keys)));
            entry_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(entry_admission(black_box(&keys)));
            entry_samples.push(started.elapsed());
            let started = Instant::now();
            black_box(double_probe_admission(black_box(&keys)));
            double_samples.push(started.elapsed());
        }
    }
    double_samples.sort_unstable();
    entry_samples.sort_unstable();
    let double_p95 = double_samples[(SAMPLE_PAIRS - 1) * 95 / 100];
    let entry_p95 = entry_samples[(SAMPLE_PAIRS - 1) * 95 / 100];
    println!(
        "EDITOR615_SINGLE_PROBE_LOCALE_ADMISSION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} locales={LOCALES} locale_key_bytes={} double_p95_ns={} entry_p95_ns={} target_ratio_bp=7000",
        keys[0].len(),
        double_p95.as_nanos(),
        entry_p95.as_nanos(),
    );
    assert!(
        entry_p95.as_nanos() * 10_000 <= double_p95.as_nanos() * 7_000,
        "entry admission P95 {:?} exceeded 70% of double-probe P95 {:?}",
        entry_p95,
        double_p95,
    );
}
