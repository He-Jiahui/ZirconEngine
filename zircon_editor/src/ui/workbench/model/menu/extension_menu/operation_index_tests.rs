use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;
use crate::core::editor_operation::EditorOperationPath;

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826ak_menu_operation_index_covers_nested_items() {
    let existing = operation("editor.view.existing");
    let nested = operation("editor.view.nested");
    let menu_bar = MenuBarModel {
        menus: vec![MenuModel {
            label: "View".to_string(),
            items: vec![
                leaf("Existing", existing.clone()),
                MenuItemModel::branch("Nested", vec![leaf("Nested Item", nested.clone())]),
            ],
        }],
    };

    let mut operation_paths = menu_operation_paths(&menu_bar);

    assert_eq!(operation_paths.len(), 2);
    assert!(!operation_paths.insert(existing));
    assert!(operation_paths.insert(operation("editor.view.new")));
    assert!(operation_paths.contains(nested.as_str()));
}

#[test]
fn optimization_batch_20260826ak_extension_views_use_one_operation_index() {
    let source = include_str!("../extension_menu.rs");

    assert!(source.contains("let mut operation_paths = menu_operation_paths(menu_bar)"));
    assert!(source.contains("operation_paths.insert(operation_path.clone())"));
    assert!(source.contains("collect_menu_operation_paths"));
    assert!(!source.contains("item_contains_operation"));
}

#[test]
#[ignore = "release-only performance contract"]
fn optimization_batch_20260826ak_extension_menu_operation_index_p95() {
    let menu_bar = MenuBarModel {
        menus: vec![MenuModel {
            label: "View".to_string(),
            items: (0..4_096)
                .map(|index| {
                    let path = operation(&format!("editor.view.item_{index:05}"));
                    leaf(&format!("Item {index}"), path)
                })
                .collect(),
        }],
    };
    let probes = (2_048..4_096)
        .rev()
        .map(|index| operation(&format!("editor.view.item_{index:05}")))
        .collect::<Vec<_>>();

    let (baseline_samples, optimized_samples) = paired_samples(
        || {
            black_box(legacy_operation_checksum(
                black_box(&menu_bar),
                black_box(&probes),
            ));
        },
        || {
            black_box(indexed_operation_checksum(
                black_box(&menu_bar),
                black_box(&probes),
            ));
        },
    );
    let baseline_p95 = percentile_95(&baseline_samples);
    let optimized_p95 = percentile_95(&optimized_samples);

    println!(
        "EDITOR01_EXTENSION_MENU_OPERATION_INDEX_BENCH_V1 \
         baseline_p95_ns={} optimized_p95_ns={}",
        baseline_p95.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos().saturating_mul(100) <= baseline_p95.as_nanos().saturating_mul(60),
        "operation-index P95 {optimized_p95:?} exceeded 60% of recursive-scan P95 {baseline_p95:?}",
    );
}

fn operation(value: &str) -> EditorOperationPath {
    EditorOperationPath::parse(value).expect("valid test operation")
}

fn leaf(label: &str, operation_path: EditorOperationPath) -> MenuItemModel {
    MenuItemModel::leaf(label, None, Some(operation_path), None, true)
}

fn legacy_operation_checksum(menu_bar: &MenuBarModel, probes: &[EditorOperationPath]) -> usize {
    probes.iter().fold(0, |checksum, probe| {
        checksum
            + usize::from(menu_bar.menus.iter().any(|menu| {
                menu.items
                    .iter()
                    .any(|item| legacy_item_contains_operation(item, probe))
            }))
    })
}

fn legacy_item_contains_operation(
    item: &MenuItemModel,
    operation_path: &EditorOperationPath,
) -> bool {
    item.operation_path.as_ref() == Some(operation_path)
        || item
            .children
            .iter()
            .any(|child| legacy_item_contains_operation(child, operation_path))
}

fn indexed_operation_checksum(menu_bar: &MenuBarModel, probes: &[EditorOperationPath]) -> usize {
    let operation_paths = menu_operation_paths(menu_bar);
    probes.iter().fold(0, |checksum, probe| {
        checksum + usize::from(operation_paths.contains(probe))
    })
}

fn paired_samples(
    mut baseline: impl FnMut(),
    mut optimized: impl FnMut(),
) -> (Vec<Duration>, Vec<Duration>) {
    let mut baseline_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline_samples.push(measure(&mut baseline));
            optimized_samples.push(measure(&mut optimized));
        } else {
            optimized_samples.push(measure(&mut optimized));
            baseline_samples.push(measure(&mut baseline));
        }
    }
    (baseline_samples, optimized_samples)
}

fn measure(operation: &mut impl FnMut()) -> Duration {
    let started = Instant::now();
    operation();
    started.elapsed()
}

fn percentile_95(samples: &[Duration]) -> Duration {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * 95).div_ceil(100).saturating_sub(1)]
}
