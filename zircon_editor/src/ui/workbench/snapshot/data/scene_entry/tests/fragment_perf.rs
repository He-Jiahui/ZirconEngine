use std::collections::{HashMap, HashSet};
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::scene::WorldInspectionHierarchyRow;

use crate::core::editor_message::{
    SceneInspectionFieldsDelta, SceneInspectionHierarchyAnchor, SceneInspectionMessage,
    SceneInspectionSelectionDelta,
};

use super::{SceneInspectionHierarchyFragment, SceneInspectionHierarchyFragmentError};

#[test]
fn optimization_batch_dj_patch_row_consumption_preserves_validation() {
    let rows = vec![hierarchy_row(1, 11), hierarchy_row(2, 22)];
    let valid = SceneInspectionHierarchyFragment::patch(
        patch_message(vec![anchor(2, 22), anchor(1, 11)]),
        rows.clone(),
    )
    .expect("reordered one-to-one rows remain valid");
    assert_eq!(valid.changed_rows().map(<[_]>::len), Some(2));

    let duplicate = SceneInspectionHierarchyFragment::patch(
        patch_message(vec![anchor(1, 11), anchor(1, 11)]),
        rows,
    )
    .expect_err("duplicate anchors must remain invalid");
    assert_eq!(
        duplicate,
        SceneInspectionHierarchyFragmentError::PatchRowMismatch { entity: 1 }
    );
}

#[test]
fn optimization_batch_dj_patch_validation_uses_one_entity_index_source() {
    let source = include_str!("../fragment.rs");
    let function = source
        .split("pub(crate) fn patch")
        .nth(1)
        .expect("patch constructor")
        .split("pub(crate) fn reflow")
        .next()
        .expect("patch body");

    assert!(function.contains("HashMap<_, _>"));
    assert!(function.contains("rows_by_entity.get_mut(&anchor.entity())"));
    assert!(!function.contains("HashSet"));
    assert!(!function.contains("anchored_entities"));
}

#[test]
#[ignore = "release-only alternating p95 performance gate"]
fn optimization_batch_dj_single_patch_entity_index_p95() {
    const SAMPLE_PAIRS: usize = 17;
    const ROW_COUNT: usize = 16_384;

    let rows = (0..ROW_COUNT as u64)
        .map(|entity| (entity, entity.wrapping_mul(17)))
        .collect::<Vec<_>>();
    let anchors = rows.clone();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_patch_validation(&rows, &anchors, true));
            optimized_samples.push(measure_patch_validation(&rows, &anchors, false));
        } else {
            optimized_samples.push(measure_patch_validation(&rows, &anchors, false));
            legacy_samples.push(measure_patch_validation(&rows, &anchors, true));
        }
    }

    let legacy_p95 = p95(&mut legacy_samples);
    let optimized_p95 = p95(&mut optimized_samples);
    println!(
        "EDITOR346_SINGLE_PATCH_ENTITY_INDEX_BENCH_V1 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
        optimized_p95 as f64 / legacy_p95.max(1) as f64
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
        "single patch entity index p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
    );
}

fn patch_message(anchors: Vec<SceneInspectionHierarchyAnchor>) -> SceneInspectionMessage {
    SceneInspectionMessage::delta(
        1,
        2,
        None,
        Vec::new(),
        anchors,
        Vec::new(),
        false,
        SceneInspectionFieldsDelta::unchanged(None),
        SceneInspectionSelectionDelta::unchanged(),
    )
}

fn anchor(entity: u64, subtree_hash: u64) -> SceneInspectionHierarchyAnchor {
    SceneInspectionHierarchyAnchor::new(entity, None, 0, subtree_hash)
}

fn hierarchy_row(entity: u64, subtree_hash: u64) -> WorldInspectionHierarchyRow {
    WorldInspectionHierarchyRow {
        entity,
        parent: None,
        depth: 0,
        display_name: format!("Entity {entity}"),
        kind: "Entity".to_string(),
        subtree_hash,
        active_in_hierarchy: true,
        has_children: false,
    }
}

fn measure_patch_validation(rows: &[(u64, u64)], anchors: &[(u64, u64)], legacy: bool) -> u128 {
    let started_at = Instant::now();
    let valid = if legacy {
        legacy_patch_validation(black_box(rows), black_box(anchors))
    } else {
        optimized_patch_validation(black_box(rows), black_box(anchors))
    };
    black_box(valid);
    started_at.elapsed().as_nanos()
}

fn legacy_patch_validation(rows: &[(u64, u64)], anchors: &[(u64, u64)]) -> bool {
    let rows_by_entity = rows
        .iter()
        .map(|row| (row.0, row))
        .collect::<HashMap<_, _>>();
    let mut anchored_entities = HashSet::with_capacity(anchors.len());
    let invalid = anchors.iter().any(|anchor| {
        !anchored_entities.insert(anchor.0)
            || rows_by_entity
                .get(&anchor.0)
                .is_none_or(|row| row.1 != anchor.1)
    });
    anchors.len() == rows.len()
        && rows_by_entity.len() == rows.len()
        && anchored_entities.len() == anchors.len()
        && !invalid
}

fn optimized_patch_validation(rows: &[(u64, u64)], anchors: &[(u64, u64)]) -> bool {
    let mut rows_by_entity = rows
        .iter()
        .map(|row| (row.0, (row, false)))
        .collect::<HashMap<_, _>>();
    let invalid = anchors.iter().any(|anchor| {
        let Some((row, seen)) = rows_by_entity.get_mut(&anchor.0) else {
            return true;
        };
        if *seen || row.1 != anchor.1 {
            return true;
        }
        *seen = true;
        false
    });
    anchors.len() == rows.len() && rows_by_entity.len() == rows.len() && !invalid
}

fn p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    let index = samples
        .len()
        .saturating_mul(95)
        .div_ceil(100)
        .saturating_sub(1);
    samples[index]
}
