use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ui::template::{
    UiAssetHeader, UiAssetKind, UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
};

use super::*;

#[test]
fn optimization_batch_20260826m_editor23_borrowed_merge_indexes_preserve_collisions() {
    let existing_tokens = BTreeMap::from([
        ("accent".to_string(), Value::String("#fff".to_string())),
        (
            "theme_accent".to_string(),
            Value::String("#eee".to_string()),
        ),
    ]);
    let imported_tokens = BTreeMap::from([
        ("accent".to_string(), Value::String("#000".to_string())),
        ("panel".to_string(), Value::String("#111".to_string())),
    ]);
    assert_eq!(
        build_imported_token_rename_map(&existing_tokens, &imported_tokens, "theme"),
        BTreeMap::from([
            ("accent".to_string(), "theme_accent_2".to_string()),
            ("panel".to_string(), "panel".to_string()),
        ])
    );

    let mut stylesheets = vec![stylesheet("base"), stylesheet("theme_base")];
    merge_detached_stylesheets(
        &mut stylesheets,
        &[stylesheet("base"), stylesheet("fresh")],
        &BTreeMap::new(),
        "theme",
    );
    assert_eq!(
        stylesheets
            .iter()
            .map(|stylesheet| stylesheet.id.as_str())
            .collect::<Vec<_>>(),
        vec!["theme_base_2", "fresh", "base", "theme_base"]
    );

    let mut document = document("local-theme");
    document.imports.styles = vec!["base.zui".to_string(), "shared.zui".to_string()];
    let mut imported = self::document("imported-theme");
    imported.imports.styles = vec![
        "shared.zui".to_string(),
        "nested.zui".to_string(),
        "nested.zui".to_string(),
    ];
    merge_detached_style_imports(&mut document, "base.zui", &imported, 0, false);
    assert_eq!(
        document.imports.styles,
        vec!["nested.zui".to_string(), "shared.zui".to_string()]
    );
}

#[test]
fn optimization_batch_20260826m_editor23_theme_merge_indexes_borrow_existing_ids() {
    let source = include_str!("../merge.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("theme merge production source");

    assert!(production.contains("struct UsedIdentifierSet"));
    assert_eq!(
        production
            .matches("UsedIdentifierSet::from_existing")
            .count(),
        3
    );
    assert!(production.contains("HashSet::with_capacity(entries.len())"));
    assert!(production.contains("HashSet::with_capacity(admitted_capacity)"));
    assert!(production.contains("fn unique_identifier(used_names: &UsedIdentifierSet<'_>"));
    assert!(!production.contains("existing_tokens.keys().cloned()"));
    assert!(!production.contains("stylesheet.id.clone()))\n        .collect::<BTreeSet"));
    assert!(!production.contains(".imports\n        .styles\n        .iter()\n        .cloned()"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_20260826m_editor23_theme_merge_borrowed_hash_performance_evidence() {
    let entries = (0..32_768)
        .map(|index| format!("editor_theme_existing_identifier_{index:05}_long_name"))
        .collect::<Vec<_>>();
    let copied_bytes = entries.iter().map(String::len).sum::<usize>();
    let mut legacy_samples = Vec::with_capacity(17);
    let mut hash_samples = Vec::with_capacity(17);
    for _ in 0..17 {
        let started = Instant::now();
        for _ in 0..3 {
            black_box(black_box(&entries).iter().cloned().collect::<BTreeSet<_>>());
        }
        legacy_samples.push(started.elapsed().as_nanos());

        let started = Instant::now();
        for _ in 0..3 {
            black_box(UsedIdentifierSet::from_existing(
                black_box(&entries).iter().map(String::as_str),
                0,
            ));
        }
        hash_samples.push(started.elapsed().as_nanos());
    }

    legacy_samples.sort_unstable();
    hash_samples.sort_unstable();
    let legacy_p95 = legacy_samples[16];
    let hash_p95 = hash_samples[16];
    println!(
        "EDITOR23_THEME_MERGE_BORROWED_ID_INDEX_BENCH_V1 entries_per_index={} indexes=3 legacy_p95_ns={} hash_p95_ns={} legacy_string_clones={} hash_string_clones=0 legacy_copied_bytes={} hash_copied_bytes=0 target_ratio_bp=6000",
        entries.len(),
        legacy_p95,
        hash_p95,
        entries.len() * 3,
        copied_bytes * 3,
    );
    assert!(
        hash_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(6_000),
        "borrowed theme merge hash P95 {hash_p95} ns exceeded 60% of legacy {legacy_p95} ns"
    );
}

fn stylesheet(id: &str) -> UiStyleSheet {
    UiStyleSheet {
        id: id.to_string(),
        rules: Vec::new(),
    }
}

fn document(id: &str) -> UiAssetDocument {
    UiAssetDocument {
        asset: UiAssetHeader {
            kind: UiAssetKind::Style,
            id: id.to_string(),
            version: UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
            display_name: id.to_string(),
        },
        imports: Default::default(),
        tokens: BTreeMap::new(),
        root: None,
        components: BTreeMap::new(),
        stylesheets: Vec::new(),
    }
}
