use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::core::resource::{ResourceKind, ResourceManager, ResourceRecord};

fn legacy_rows(
    generation: &ResourceReadinessGeneration,
    root_id: AssetId,
    dependency_ids: &[AssetId],
) -> Vec<AssetDependencyReadiness> {
    let initial_capacity = dependency_ids.len();
    let mut rows = Vec::with_capacity(initial_capacity);
    let mut row_by_id = HashMap::with_capacity(initial_capacity);
    let mut expanded = HashSet::with_capacity(initial_capacity.saturating_add(1));
    expanded.insert(root_id);
    let mut queue = VecDeque::with_capacity(initial_capacity);
    for dependency_id in dependency_ids {
        queue.push_back((*dependency_id, 1_u32, true));
    }
    while let Some((dependency_id, depth, direct)) = queue.pop_front() {
        if let Some(index) = row_by_id.get(&dependency_id).copied() {
            let existing: &mut AssetDependencyReadiness = &mut rows[index];
            existing.depth = existing.depth.min(depth);
            existing.direct |= direct;
            continue;
        }
        let readiness_row = generation.row(dependency_id);
        rows.push(dependency_readiness_row(
            dependency_id,
            readiness_row,
            depth,
            direct,
        ));
        row_by_id.insert(dependency_id, rows.len() - 1);
        let Some(readiness_row) = readiness_row else {
            continue;
        };
        if !expanded.insert(dependency_id) {
            continue;
        }
        for nested in &readiness_row.record.dependency_ids {
            queue.push_back((*nested, depth + 1, false));
        }
    }
    rows
}

fn records(count: usize) -> Vec<ResourceRecord> {
    (0..count)
        .map(|index| {
            let uri = AssetUri::parse(&format!("res://astra-readiness/{index}.png")).unwrap();
            ResourceRecord::new(AssetId::from_locator(&uri), ResourceKind::Texture, uri)
        })
        .collect()
}

fn generation(records: Vec<ResourceRecord>) -> Arc<ResourceReadinessGeneration> {
    let manager = ResourceManager::new();
    manager.register_lazy_records(records).unwrap();
    manager.readiness_generation()
}

#[test]
fn astra_m6_breadth_first_rows_preserve_order_depth_direct_and_diagnostics() {
    let mut records = records(5);
    let ids: Vec<_> = records.iter().map(|record| record.id).collect();
    let missing = AssetId::new();
    records[0].dependency_ids = vec![ids[1], ids[2], ids[1], ids[4]];
    records[1].dependency_ids = vec![ids[3], ids[4], missing];
    records[2].dependency_ids = vec![ids[3], missing];
    records[3].dependency_ids = vec![ids[0], ids[1], ids[3]];
    records[4]
        .diagnostics
        .push(ResourceDiagnostic::error("retained diagnostic"));
    let dependencies = records[0].dependency_ids.clone();
    let generation = generation(records);
    let rows = collect_dependency_readiness(&generation, ids[0], &dependencies);
    assert_eq!(rows, legacy_rows(&generation, ids[0], &dependencies));
    assert_eq!(
        rows[..3].iter().map(|row| row.id).collect::<Vec<_>>(),
        [ids[1], ids[2], ids[4]]
    );
    assert_eq!(
        rows[3..5].iter().map(|row| row.id).collect::<HashSet<_>>(),
        HashSet::from([ids[3], missing])
    );
    assert_eq!(rows[5].id, ids[0]);
    assert_eq!(
        rows.iter().map(|row| row.depth).collect::<Vec<_>>(),
        [1, 1, 1, 2, 2, 3]
    );
    assert!(rows[..3].iter().all(|row| row.direct));
    assert!(rows[3..].iter().all(|row| !row.direct));
    assert_eq!(rows[2].diagnostics[0].message, "retained diagnostic");
    let missing_row = rows.iter().find(|row| row.id == missing).unwrap();
    assert_eq!(missing_row.load_state, AssetLoadState::Failed);
    assert!(missing_row.diagnostics[0]
        .message
        .contains("missing asset dependency record"));
}

#[test]
fn astra_m6_traversal_matches_legacy_for_cyclic_shared_and_missing_graphs() {
    for seed in 0..128_u64 {
        let mut records = records(7);
        let ids: Vec<_> = records.iter().map(|record| record.id).collect();
        let mut state = seed + 1;
        for record in &mut records {
            for id in &ids {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                if state >> 62 == 0 {
                    record.dependency_ids.push(*id);
                    if state & 1 == 0 {
                        record.dependency_ids.push(*id);
                    }
                }
            }
        }
        let dependencies = records[0].dependency_ids.clone();
        records.pop();
        let generation = generation(records);
        assert_eq!(
            collect_dependency_readiness(&generation, ids[0], &dependencies),
            legacy_rows(&generation, ids[0], &dependencies),
            "seed {seed}"
        );
    }
    let empty = ResourceReadinessGeneration::default();
    assert!(collect_dependency_readiness(&empty, AssetId::new(), &[]).is_empty());
}

fn fixture(
    width: usize,
    shared: usize,
) -> (Arc<ResourceReadinessGeneration>, AssetId, Vec<AssetId>) {
    let mut records = records(1 + width + shared);
    let ids: Vec<_> = records.iter().map(|record| record.id).collect();
    records[0].dependency_ids = ids[1..1 + width].to_vec();
    for record in &mut records[1..1 + width] {
        record.dependency_ids = ids[1 + width..].to_vec();
    }
    let dependencies = records[0].dependency_ids.clone();
    (generation(records), ids[0], dependencies)
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn astra_m6_readiness_report_release_evidence() {
    for (width, shared) in [(1, 0), (1_000, 0), (10_000, 0), (128, 128)] {
        let (generation, root, dependencies) = fixture(width, shared);
        assert_eq!(
            collect_dependency_readiness(&generation, root, &dependencies),
            legacy_rows(&generation, root, &dependencies)
        );
        let iterations = if width == 1 { 1_000 } else { 1 };
        let measure = |legacy| {
            let start = Instant::now();
            for _ in 0..iterations {
                let rows = if legacy {
                    legacy_rows(black_box(&generation), root, black_box(&dependencies))
                } else {
                    collect_dependency_readiness(
                        black_box(&generation),
                        root,
                        black_box(&dependencies),
                    )
                };
                black_box(rows);
            }
            start.elapsed().as_nanos().max(1)
        };
        for _ in 0..8 {
            measure(true);
            measure(false);
        }
        let mut baseline = Vec::new();
        let mut optimized = Vec::new();
        for sample in 0..101 {
            if sample % 2 == 0 {
                baseline.push(measure(true));
                optimized.push(measure(false));
            } else {
                optimized.push(measure(false));
                baseline.push(measure(true));
            }
        }
        let percentile = |values: &[u128], p: usize| {
            let mut sorted = values.to_vec();
            sorted.sort_unstable();
            sorted[(sorted.len() * p).div_ceil(100) - 1]
        };
        for p in [50, 95, 99] {
            println!("ASTRA_M6_READINESS width={width} shared={shared} iterations={iterations} p{p} baseline_ns={} optimized_ns={}",
                percentile(&baseline, p), percentile(&optimized, p));
        }
        println!("ASTRA_M6_RAW width={width} shared={shared} baseline={baseline:?} optimized={optimized:?}");
        let limit = if shared > 0 { 80 } else { 105 };
        assert!(
            percentile(&optimized, 95) * 100 <= percentile(&baseline, 95) * limit,
            "readiness performance gate failed for width={width} shared={shared}"
        );
    }
}
