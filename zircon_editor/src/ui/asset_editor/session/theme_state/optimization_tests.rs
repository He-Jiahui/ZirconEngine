use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::Instant;

use super::*;

#[test]
fn optimization_batch_20260826l_editor23_theme_hash_indexes_preserve_replay_semantics() {
    let import_commands = build_style_import_replay_commands(
        &strings(&["a", "b", "obsolete"]),
        &strings(&["b", "a", "new"]),
    );
    assert!(matches!(
        &import_commands[0],
        UiAssetEditorDocumentReplayCommand::RemoveStyleImport { index: 2, reference }
            if reference == "obsolete"
    ));
    assert!(matches!(
        &import_commands[1],
        UiAssetEditorDocumentReplayCommand::MoveStyleImport {
            from_index: 1,
            to_index: 0,
            reference,
        } if reference == "b"
    ));
    assert!(matches!(
        &import_commands[2],
        UiAssetEditorDocumentReplayCommand::InsertStyleImport { index: 2, reference }
            if reference == "new"
    ));

    let stylesheet_commands = build_stylesheet_replay_commands(
        &[stylesheet("base", &[".base"]), stylesheet("obsolete", &[])],
        &[stylesheet("base", &[".base"]), stylesheet("new", &[])],
    );
    assert!(matches!(
        &stylesheet_commands[0],
        UiAssetEditorDocumentReplayCommand::RemoveStyleSheet {
            index: 1,
            stylesheet_id,
        } if stylesheet_id == "obsolete"
    ));
    assert!(matches!(
        &stylesheet_commands[1],
        UiAssetEditorDocumentReplayCommand::InsertStyleSheet {
            index: 1,
            stylesheet_id,
            ..
        } if stylesheet_id == "new"
    ));

    let rule_commands = build_style_rule_replay_commands(
        0,
        &[rule(".base"), rule(".obsolete")],
        &[rule(".base"), rule(".new")],
    )
    .expect("unique selectors use incremental replay");
    assert!(matches!(
        &rule_commands[0],
        UiAssetEditorDocumentReplayCommand::RemoveStyleRule {
            stylesheet_index: 0,
            index: 1,
            selector,
        } if selector == ".obsolete"
    ));
    assert!(matches!(
        &rule_commands[1],
        UiAssetEditorDocumentReplayCommand::InsertStyleRule {
            stylesheet_index: 0,
            index: 1,
            selector,
            ..
        } if selector == ".new"
    ));

    let duplicate_fallback =
        build_style_import_replay_commands(&strings(&["a"]), &strings(&["a", "a"]));
    assert!(matches!(
        duplicate_fallback.as_slice(),
        [UiAssetEditorDocumentReplayCommand::SetStyleImports { references }]
            if references == &strings(&["a", "a"])
    ));
}

#[test]
fn optimization_batch_20260826l_editor23_theme_indexes_borrow_all_string_keys() {
    let source = include_str!("../theme_state.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("theme state production source");

    assert!(!production.contains("BTreeSet"));
    assert_eq!(production.matches("borrowed_string_index(").count(), 3);
    assert!(production.contains("fn borrowed_string_index<'a>"));
    assert_eq!(
        production
            .matches("has_duplicate_borrowed_entries(")
            .count(),
        3
    );
    assert!(production.contains("fn has_duplicate_borrowed_entries<'a>"));
    assert!(production.contains("HashSet::with_capacity(entries.len())"));
    assert!(!production.contains("target.iter().cloned()"));
    assert!(!production.contains("stylesheet.id.clone())\n        .collect"));
    assert!(!production.contains("rule.selector.clone())\n        .collect"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_20260826l_editor23_theme_borrowed_hash_performance_evidence() {
    let entries = (0..32_768)
        .map(|index| format!("res://editor/theme/import/{index:05}/long_style_name.zui"))
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
            black_box(borrowed_string_index(
                black_box(&entries).iter().map(String::as_str),
            ));
        }
        hash_samples.push(started.elapsed().as_nanos());
    }

    legacy_samples.sort_unstable();
    hash_samples.sort_unstable();
    let legacy_p95 = legacy_samples[16];
    let hash_p95 = hash_samples[16];
    println!(
        "EDITOR23_THEME_REPLAY_BORROWED_HASH_INDEX_BENCH_V1 entries_per_index={} indexes=3 legacy_p95_ns={} hash_p95_ns={} legacy_string_clones={} hash_string_clones=0 legacy_copied_bytes={} hash_copied_bytes=0 target_ratio_bp=6000",
        entries.len(),
        legacy_p95,
        hash_p95,
        entries.len() * 3,
        copied_bytes * 3,
    );
    assert!(
        hash_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(6_000),
        "borrowed theme replay hash P95 {hash_p95} ns exceeded 60% of legacy {legacy_p95} ns"
    );
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn stylesheet(id: &str, selectors: &[&str]) -> UiStyleSheet {
    UiStyleSheet {
        id: id.to_string(),
        rules: selectors.iter().map(|selector| rule(selector)).collect(),
    }
}

fn rule(selector: &str) -> UiStyleRule {
    UiStyleRule {
        selector: selector.to_string(),
        ..Default::default()
    }
}
