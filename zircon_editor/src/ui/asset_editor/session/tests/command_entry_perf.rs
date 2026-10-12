use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::Instant;

use super::*;

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

#[test]
fn optimization_batch_20260826k_editor23_widget_import_hash_index_preserves_replay() {
    let commands = build_widget_import_replay_commands(
        &strings(&["a", "b", "obsolete"]),
        &strings(&["b", "a", "new"]),
    );

    assert!(matches!(
        &commands[0],
        UiAssetEditorDocumentReplayCommand::RemoveWidgetImport { index: 2, reference }
            if reference == "obsolete"
    ));
    assert!(matches!(
        &commands[1],
        UiAssetEditorDocumentReplayCommand::MoveWidgetImport {
            from_index: 1,
            to_index: 0,
            reference,
        } if reference == "b"
    ));
    assert!(matches!(
        &commands[2],
        UiAssetEditorDocumentReplayCommand::InsertWidgetImport { index: 2, reference }
            if reference == "new"
    ));

    let duplicate_fallback =
        build_widget_import_replay_commands(&strings(&["a"]), &strings(&["a", "a"]));
    assert!(matches!(
        duplicate_fallback.as_slice(),
        [UiAssetEditorDocumentReplayCommand::SetWidgetImports { references }]
            if references == &strings(&["a", "a"])
    ));
}

#[test]
fn optimization_batch_20260826k_editor23_widget_import_index_borrows_hash_keys() {
    let source = include_str!("../command_entry.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("asset editor command production source");
    let replay = production
        .split("fn build_widget_import_replay_commands")
        .nth(1)
        .expect("widget import replay")
        .split("fn upsert_node_replay_commands")
        .next()
        .expect("bounded widget import replay");

    assert!(!production.contains("BTreeSet"));
    assert!(replay.contains("widget_import_target_entries(target)"));
    assert!(production.contains("HashSet::with_capacity(target.len())"));
    assert!(production.contains("HashSet::with_capacity(entries.len())"));
    assert!(!replay.contains("target.iter().cloned()"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_20260826k_editor23_widget_import_borrowed_hash_performance_evidence() {
    fn legacy_retained_count(current: &[String], target: &[String]) -> usize {
        let target_entries = target.iter().cloned().collect::<BTreeSet<_>>();
        current
            .iter()
            .filter(|entry| target_entries.contains(*entry))
            .count()
    }

    let target = (0..32_768)
        .map(|index| format!("res://editor/widget/import/{index:05}.zui"))
        .collect::<Vec<_>>();
    let current = (0..32_768)
        .map(|index| {
            if index % 2 == 0 {
                target[index].clone()
            } else {
                format!("res://editor/widget/stale/{index:05}.zui")
            }
        })
        .collect::<Vec<_>>();
    let copied_target_bytes = target.iter().map(String::len).sum::<usize>();
    let mut legacy_samples = Vec::with_capacity(17);
    let mut hash_samples = Vec::with_capacity(17);
    for _ in 0..17 {
        let started = Instant::now();
        black_box(legacy_retained_count(
            black_box(&current),
            black_box(&target),
        ));
        legacy_samples.push(started.elapsed().as_nanos());

        let started = Instant::now();
        let target_entries = widget_import_target_entries(black_box(&target));
        black_box(
            current
                .iter()
                .filter(|entry| target_entries.contains(entry.as_str()))
                .count(),
        );
        hash_samples.push(started.elapsed().as_nanos());
    }

    legacy_samples.sort_unstable();
    hash_samples.sort_unstable();
    let legacy_p95 = legacy_samples[16];
    let hash_p95 = hash_samples[16];
    println!(
        "EDITOR23_WIDGET_IMPORT_BORROWED_HASH_INDEX_BENCH_V1 target_imports={} current_imports={} legacy_p95_ns={} hash_p95_ns={} legacy_string_clones={} hash_string_clones=0 legacy_copied_bytes={} hash_copied_bytes=0 target_ratio_bp=6000",
        target.len(),
        current.len(),
        legacy_p95,
        hash_p95,
        target.len(),
        copied_target_bytes,
    );
    assert!(
        hash_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(6_000),
        "borrowed widget import hash P95 {hash_p95} ns exceeded 60% of legacy {legacy_p95} ns"
    );
}
