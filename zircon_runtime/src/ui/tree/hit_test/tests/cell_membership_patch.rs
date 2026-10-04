use super::*;

#[test]
fn dense_batches_match_expected_membership_for_1k_and_10k_entries() {
    for entry_count in [1_000, 10_000] {
        for changed_count in [0, 1, 64] {
            let mut cells = vec![
                UiHitTestCell {
                    entries: (0..entry_count).collect::<Vec<_>>().into(),
                },
                UiHitTestCell::default(),
            ]
            .into();
            let mut patches = UiCellMembershipPatches::default();
            for entry_index in 0..changed_count {
                patches.stage(entry_index, &[0], &[1], false);
            }

            let stats = patches
                .apply(&mut cells, |entry_index| entry_index)
                .unwrap();
            let retained = (changed_count..entry_count).collect::<Vec<_>>();
            let moved = (0..changed_count).collect::<Vec<_>>();

            assert_eq!(cells[0].entries.as_slice(), retained.as_slice());
            assert_eq!(cells[1].entries.as_slice(), moved.as_slice());
            assert_eq!(stats.staged_cell_count, usize::from(changed_count > 0) * 2);
            assert_eq!(
                stats.published_cell_count,
                usize::from(changed_count > 0) * 2
            );
            assert_eq!(
                stats.source_membership_count,
                if changed_count == 0 { 0 } else { entry_count }
            );
            assert_eq!(stats.staged_removal_count, changed_count);
            assert_eq!(stats.staged_addition_count, changed_count);
            assert_eq!(
                stats.materialized_membership_count,
                if changed_count == 0 {
                    0
                } else {
                    2 * entry_count - changed_count
                }
            );
            assert_eq!(
                stats.replacement_buffer_count,
                usize::from(changed_count > 0) * 3
            );
        }
    }
}

#[test]
#[ignore = "managed Windows release paired performance evidence"]
fn dense_batch_release_measurement() {
    const SAMPLE_PAIRS: usize = 17;
    const REPEATS: usize = 32;
    let mut regressions = Vec::new();
    for (entry_count, changed_count, shared_snapshot) in [
        (8, 1, true),
        (8, 1, false),
        (1_000, 1, true),
        (1_000, 64, true),
        (10_000, 1, true),
        (10_000, 64, true),
        (1_000, 64, false),
        (10_000, 64, false),
    ] {
        let template = dense_cells(entry_count);
        let mut expected = dense_cells(entry_count);
        let mut candidate = dense_cells(entry_count);
        for source in [0, 1, 0, 1] {
            legacy_move(&mut expected, changed_count, source);
            batched_move(&mut candidate, changed_count, source);
            assert_eq!(candidate, expected, "paired benchmark output parity");
        }
        let mut batched_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let measure = |batched: bool| {
            // A unique sequence survives every measured move; no retained clone forces COW.
            let mut unique_cells = dense_cells(entry_count);
            let started = std::time::Instant::now();
            for repeat in 0..REPEATS {
                let mut shared_cells;
                let (cells, source) = if shared_snapshot {
                    shared_cells = template.clone();
                    (&mut shared_cells, 0)
                } else {
                    (&mut unique_cells, repeat % 2)
                };
                if batched {
                    batched_move(cells, changed_count, source);
                } else {
                    legacy_move(cells, changed_count, source);
                }
                std::hint::black_box(&*cells);
            }
            started.elapsed().as_nanos()
        };
        std::hint::black_box(measure(false));
        std::hint::black_box(measure(true));
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure(false));
                batched_samples.push(measure(true));
            } else {
                batched_samples.push(measure(true));
                legacy_samples.push(measure(false));
            }
        }
        batched_samples.sort_unstable();
        legacy_samples.sort_unstable();
        let percentile = |samples: &[u128], rank: usize| {
            samples[(SAMPLE_PAIRS * rank / 100).min(SAMPLE_PAIRS - 1)]
        };
        let batched_p95 = percentile(&batched_samples, 95);
        let legacy_p95 = percentile(&legacy_samples, 95);
        println!(
                "ASTRA_HIT_GRID_BATCH_V2 entries={entry_count} changed={changed_count} shared_snapshot={shared_snapshot} batched_p50_ns={} batched_p95_ns={} batched_p99_ns={} legacy_p50_ns={} legacy_p95_ns={} legacy_p99_ns={}",
                percentile(&batched_samples, 50),
                batched_p95,
                percentile(&batched_samples, 99),
                percentile(&legacy_samples, 50),
                legacy_p95,
                percentile(&legacy_samples, 99)
            );
        if (entry_count, changed_count) == (8, 1) {
            if batched_p95.saturating_mul(100) > legacy_p95.saturating_mul(105) {
                regressions.push(format!(
                    "sparse shared={shared_snapshot}: batched={batched_p95} legacy={legacy_p95}"
                ));
            }
        } else if changed_count == 64
            && batched_p95.saturating_mul(100) > legacy_p95.saturating_mul(80)
        {
            regressions.push(format!("dense entries={entry_count} shared={shared_snapshot}: batched={batched_p95} legacy={legacy_p95}"));
        }
    }
    assert!(
        regressions.is_empty(),
        "paired P95 gates failed: {regressions:?}"
    );
}

fn batched_move(
    cells: &mut UiPersistentSequence<UiHitTestCell>,
    changed_count: usize,
    source: usize,
) {
    let mut patches = UiCellMembershipPatches::default();
    for entry_index in 0..changed_count {
        patches.stage(entry_index, &[source], &[1 - source], false);
    }
    std::hint::black_box(patches.apply(cells, |entry_index| entry_index).unwrap());
}

fn legacy_move(
    cells: &mut UiPersistentSequence<UiHitTestCell>,
    changed_count: usize,
    source: usize,
) {
    for entry_index in 0..changed_count {
        cells[source]
            .entries
            .retain(|candidate| *candidate != entry_index);
        let insertion = cells[1 - source]
            .entries
            .partition_point(|candidate| *candidate <= entry_index);
        cells[1 - source].entries.insert(insertion, entry_index);
    }
}

fn dense_cells(entry_count: usize) -> UiPersistentSequence<UiHitTestCell> {
    vec![
        UiHitTestCell {
            entries: (0..entry_count).collect::<Vec<_>>().into(),
        },
        UiHitTestCell::default(),
    ]
    .into()
}

#[test]
fn rejects_missing_cell_before_publishing_valid_replacements() {
    let mut cells: UiPersistentSequence<UiHitTestCell> = vec![UiHitTestCell {
        entries: vec![0, 1].into(),
    }]
    .into();
    let before = cells.clone();
    let mut patches = UiCellMembershipPatches::default();
    patches.stage(0, &[0], &[], false);
    patches.stage(1, &[], &[1], false);

    assert_eq!(
        patches.apply(&mut cells, |entry_index| entry_index),
        Err(())
    );
    assert_eq!(cells, before);
}

#[test]
fn unchanged_footprints_stage_no_cell_work() {
    let mut cells = dense_cells(10_000);
    let before = cells.clone();
    let mut patches = UiCellMembershipPatches::default();
    for entry_index in 0..64 {
        patches.stage(entry_index, &[0], &[0], false);
    }

    let stats = patches
        .apply(&mut cells, |entry_index| entry_index)
        .unwrap();

    assert_eq!(stats, UiCellMembershipPatchStats::default());
    assert_eq!(cells, before);
}

#[test]
fn changed_order_reinserts_membership_in_unchanged_cell() {
    let mut cells = vec![
        UiHitTestCell {
            entries: vec![0, 1, 2].into(),
        },
        UiHitTestCell::default(),
    ]
    .into();
    let mut patches = UiCellMembershipPatches::default();
    patches.stage(0, &[0], &[0], true);

    let stats = patches
        .apply(
            &mut cells,
            |entry_index| if entry_index == 0 { 3 } else { entry_index },
        )
        .unwrap();

    assert_eq!(cells[0].entries.as_slice(), &[1, 2, 0]);
    assert_eq!(stats.staged_cell_count, 1);
    assert_eq!(stats.source_membership_count, 3);

    let mut next = UiCellMembershipPatches::default();
    next.stage(0, &[0], &[1], false);
    next.apply(
        &mut cells,
        |entry_index| if entry_index == 0 { 3 } else { entry_index },
    )
    .unwrap();
    assert_eq!(cells[0].entries.as_slice(), &[1, 2]);
    assert_eq!(cells[1].entries.as_slice(), &[0]);
}

#[test]
fn one_cell_batch_deletes_and_inserts_multiple_memberships_once() {
    let mut cells = vec![UiHitTestCell {
        entries: vec![0, 1, 2, 3].into(),
    }]
    .into();
    let mut patches = UiCellMembershipPatches::default();
    patches.stage(1, &[0], &[], false);
    patches.stage(3, &[0], &[], false);
    patches.stage(4, &[], &[0], false);
    patches.stage(5, &[], &[0], false);

    let stats = patches
        .apply(&mut cells, |entry_index| entry_index)
        .unwrap();

    assert_eq!(cells[0].entries.as_slice(), &[0, 2, 4, 5]);
    assert_eq!(stats.staged_cell_count, 1);
    assert_eq!(stats.published_cell_count, 1);
    assert_eq!(stats.source_membership_count, 4);
    assert_eq!(stats.staged_removal_count, 2);
    assert_eq!(stats.staged_addition_count, 2);
}
