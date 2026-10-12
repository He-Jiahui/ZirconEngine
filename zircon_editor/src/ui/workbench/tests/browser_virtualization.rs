use super::super::paint_metadata::{AssetContentRect, AssetContentRowGroup};
use super::{
    append_visible_virtual_group_rows, AssetBrowserLogicalPaintGeneration, AssetBrowserPaintItem,
    AssetBrowserThumbnailPaintItem, AssetBrowserVirtualization,
};
use std::hint::black_box;
use std::rc::Rc;
use std::time::Instant;

#[test]
fn empty_logical_paint_generation_has_a_safe_lookup_stride() {
    let generation = AssetBrowserLogicalPaintGeneration::default();

    assert!(generation.is_empty());
    assert_eq!(generation.len(), 0);
    assert!(generation.get(0).is_none());
}

#[test]
fn one_row_scroll_rebinds_only_the_entering_physical_row() {
    let virtualization = virtualization(20, 6, 2);
    let initial = (0..6)
        .map(|slot| virtualization.binding(0.0, slot).unwrap().logical_index)
        .collect::<Vec<_>>();
    let scrolled = (0..6)
        .map(|slot| virtualization.binding(10.0, slot).unwrap().logical_index)
        .collect::<Vec<_>>();

    assert_eq!(initial, vec![0, 1, 2, 3, 4, 5]);
    assert_eq!(scrolled, vec![6, 7, 2, 3, 4, 5]);
    assert_eq!(
        initial
            .iter()
            .zip(&scrolled)
            .filter(|(left, right)| left != right)
            .count(),
        2
    );
    for (slot, logical_index) in scrolled.into_iter().enumerate() {
        assert_eq!(logical_index % 2, slot % 2);
    }
    assert_eq!(virtualization.binding(10.0, 0).unwrap().y_offset, 30.0);
    assert_eq!(virtualization.binding(10.0, 2).unwrap().y_offset, 0.0);
}

#[test]
fn bottom_window_backfills_the_materialized_rows() {
    let virtualization = virtualization(10, 6, 2);
    let mut logical_indices = (0..6)
        .map(|slot| {
            virtualization
                .binding(10_000.0, slot)
                .unwrap()
                .logical_index
        })
        .collect::<Vec<_>>();

    logical_indices.sort_unstable();
    assert_eq!(logical_indices, vec![4, 5, 6, 7, 8, 9]);
}

#[test]
fn partial_row_pool_falls_back_without_omitting_logical_items() {
    let virtualization = virtualization(20, 5, 3);
    let logical_indices = (0..5)
        .map(|slot| virtualization.binding(10.0, slot).unwrap().logical_index)
        .collect::<Vec<_>>();

    assert_eq!(logical_indices, vec![3, 4, 5, 6, 7]);
}

#[test]
fn visible_virtual_groups_match_per_slot_binding_across_scroll_and_pool_boundaries() {
    let viewport = rect(0.0, 0.0, 900.0, 620.0);
    for (logical_count, materialized_count, scroll_px, origin_x, origin_y, damage_clip) in [
        (100, 48, 0.0, 0.0, 0.0, rect(0.0, 0.0, 900.0, 620.0)),
        (
            100_000,
            48,
            158.0 * 2_000.0 + 79.0,
            13.0,
            27.0,
            rect(113.0, 147.0, 500.0, 300.0),
        ),
        (
            100_000,
            48,
            158.0 * 16_666.0,
            0.0,
            0.0,
            rect(0.0, 0.0, 900.0, 620.0),
        ),
        (
            100,
            48,
            158.0 * 15.0,
            0.0,
            0.0,
            rect(0.0, 0.0, 900.0, 620.0),
        ),
        (100, 41, 158.0 * 3.0, 0.0, 0.0, rect(0.0, 0.0, 900.0, 620.0)),
    ] {
        let virtualization = chunked_virtualization(logical_count, materialized_count, 6);
        let groups = virtual_groups(materialized_count, 6);
        assert_virtual_groups_match_retired(
            &groups,
            &virtualization,
            viewport,
            scroll_px,
            origin_x,
            origin_y,
            damage_clip,
        );
        if logical_count == 100_000 && scroll_px > 1_000_000.0 {
            let last_slot = (0..groups.len())
                .find(|&slot| {
                    virtualization
                        .binding(scroll_px, slot)
                        .is_some_and(|binding| binding.logical_index == 99_999)
                })
                .expect("last logical asset must bind to a materialized slot");
            let mut visible_rows = Vec::new();
            append_visible_virtual_group_rows(
                &mut visible_rows,
                &groups,
                &virtualization,
                Some(viewport),
                scroll_px,
                origin_x,
                origin_y,
                damage_clip,
            );
            assert!(
                visible_rows.contains(&groups[last_slot].node_rows[0]),
                "last logical asset must be visible at the 100k-item tail"
            );
        }
    }
    let mut list_virtualization = chunked_virtualization(1_000, 27, 1);
    list_virtualization.row_stride = 28.0;
    let list_groups = (0..27)
        .map(|slot| AssetContentRowGroup {
            top: slot as f32 * 28.0,
            bottom: (slot + 1) as f32 * 28.0,
            node_rows: vec![slot],
        })
        .collect::<Vec<_>>();
    assert_virtual_groups_match_retired(
        &list_groups,
        &list_virtualization,
        viewport,
        28.0 * 300.0,
        0.0,
        0.0,
        rect(0.0, 0.0, 900.0, 620.0),
    );
}

fn assert_virtual_groups_match_retired(
    groups: &[AssetContentRowGroup],
    virtualization: &AssetBrowserVirtualization,
    viewport: AssetContentRect,
    scroll_px: f32,
    origin_x: f32,
    origin_y: f32,
    damage_clip: AssetContentRect,
) {
    let mut old_rows = vec![usize::MAX];
    let old_count = retired_append_visible_virtual_group_rows(
        &mut old_rows,
        groups,
        virtualization,
        Some(viewport),
        scroll_px,
        origin_x,
        origin_y,
        damage_clip,
    );
    let mut new_rows = vec![usize::MAX];
    let new_count = append_visible_virtual_group_rows(
        &mut new_rows,
        groups,
        virtualization,
        Some(viewport),
        scroll_px,
        origin_x,
        origin_y,
        damage_clip,
    );
    assert_eq!(
        new_count,
        old_count,
        "scroll={scroll_px} pool={}",
        groups.len()
    );
    assert_eq!(
        new_rows,
        old_rows,
        "scroll={scroll_px} pool={}",
        groups.len()
    );
    assert!(new_count > 0, "fixture should select visible rows");
}

#[test]
#[ignore = "run old/new virtual group append comparison in Windows Release"]
fn editor57_virtual_group_single_window_release_benchmark() {
    const MARKER: &str = "EDITOR57_VIRTUAL_GROUP_SINGLE_WINDOW_BENCH_V1";
    const SAMPLES: usize = 31;
    let mut gates = Vec::new();
    for (materialized_count, iterations, max_p95_percent) in [(48, 256, 90), (240, 64, 80)] {
        let virtualization = chunked_virtualization(100_000, materialized_count, 6);
        let groups = virtual_groups(materialized_count, 6);
        let viewport = Some(rect(0.0, 0.0, 900.0, 620.0));
        let clip = rect(13.0, 27.0, 900.0, 620.0);
        for _ in 0..5 {
            black_box(timed_virtual_group_batch(
                &groups,
                &virtualization,
                viewport,
                clip,
                iterations,
                true,
            ));
            black_box(timed_virtual_group_batch(
                &groups,
                &virtualization,
                viewport,
                clip,
                iterations,
                false,
            ));
        }
        let mut old_ns = Vec::with_capacity(SAMPLES);
        let mut new_ns = Vec::with_capacity(SAMPLES);
        for sample in 0..SAMPLES {
            if sample % 2 == 0 {
                old_ns.push(timed_virtual_group_batch(
                    &groups,
                    &virtualization,
                    viewport,
                    clip,
                    iterations,
                    true,
                ));
                new_ns.push(timed_virtual_group_batch(
                    &groups,
                    &virtualization,
                    viewport,
                    clip,
                    iterations,
                    false,
                ));
            } else {
                new_ns.push(timed_virtual_group_batch(
                    &groups,
                    &virtualization,
                    viewport,
                    clip,
                    iterations,
                    false,
                ));
                old_ns.push(timed_virtual_group_batch(
                    &groups,
                    &virtualization,
                    viewport,
                    clip,
                    iterations,
                    true,
                ));
            }
        }
        let mut old_sorted = old_ns.clone();
        let mut new_sorted = new_ns.clone();
        let (old_p50, old_p95, old_p99) = percentiles_ns(&mut old_sorted);
        let (new_p50, new_p95, new_p99) = percentiles_ns(&mut new_sorted);
        println!(
            "PERF_RESULT {MARKER} logical_items=100000 materialized_groups={materialized_count} iterations={iterations} warmups=5 samples={SAMPLES} old_p50_ns={old_p50} old_p95_ns={old_p95} old_p99_ns={old_p99} new_p50_ns={new_p50} new_p95_ns={new_p95} new_p99_ns={new_p99} old_samples_ns={old_ns:?} new_samples_ns={new_ns:?} os={} arch={} package_version={} max_p95_percent={max_p95_percent}",
            std::env::consts::OS,
            std::env::consts::ARCH,
            env!("CARGO_PKG_VERSION"),
        );
        gates.push((materialized_count, old_p95, new_p95, max_p95_percent));
    }
    for (groups, old_p95, new_p95, max_p95_percent) in gates {
        assert!(
            new_p95.saturating_mul(100) <= old_p95.saturating_mul(max_p95_percent),
            "{MARKER} groups={groups}: new p95 {new_p95} ns exceeds {max_p95_percent}% of old p95 {old_p95} ns"
        );
    }
}

fn chunked_virtualization(
    item_count: usize,
    materialized_count: usize,
    columns: usize,
) -> AssetBrowserVirtualization {
    let chunks = (0..item_count)
        .step_by(256)
        .map(|start| {
            (start..(start + 256).min(item_count))
                .map(|index| {
                    AssetBrowserPaintItem::Thumbnail(AssetBrowserThumbnailPaintItem {
                        name: format!("Asset {index}"),
                        source_file_name: String::new(),
                        file_extension: String::new(),
                        name_continuation: String::new(),
                        type_label: String::new(),
                        type_label_width: 0.0,
                        state_label: String::new(),
                        visual_variant: String::new(),
                        preview_artifact_path: String::new(),
                    })
                })
                .collect::<Vec<_>>()
                .into()
        })
        .collect::<Vec<Rc<[AssetBrowserPaintItem]>>>();
    AssetBrowserVirtualization::new(
        AssetBrowserLogicalPaintGeneration::from_chunks(chunks),
        Vec::new(),
        materialized_count,
        columns,
        0.0,
        158.0,
        0.0,
        2,
    )
}

fn virtual_groups(materialized_count: usize, columns: usize) -> Vec<AssetContentRowGroup> {
    (0..materialized_count)
        .map(|slot| {
            let top = (slot / columns) as f32 * 158.0;
            AssetContentRowGroup {
                top,
                bottom: top + 150.0,
                node_rows: (slot * 9..slot * 9 + 9).collect(),
            }
        })
        .collect()
}

fn rect(x: f32, y: f32, width: f32, height: f32) -> AssetContentRect {
    AssetContentRect {
        x,
        y,
        width,
        height,
    }
}

fn timed_virtual_group_batch(
    groups: &[AssetContentRowGroup],
    virtualization: &AssetBrowserVirtualization,
    viewport: Option<AssetContentRect>,
    clip: AssetContentRect,
    iterations: usize,
    retired: bool,
) -> u128 {
    let mut rows = Vec::with_capacity(groups.len() * 9);
    let started = Instant::now();
    for _ in 0..iterations {
        rows.clear();
        let visible = if retired {
            retired_append_visible_virtual_group_rows(
                &mut rows,
                groups,
                virtualization,
                viewport,
                158.0 * 2_000.0,
                13.0,
                27.0,
                clip,
            )
        } else {
            append_visible_virtual_group_rows(
                &mut rows,
                groups,
                virtualization,
                viewport,
                158.0 * 2_000.0,
                13.0,
                27.0,
                clip,
            )
        };
        black_box((visible, &rows));
    }
    started.elapsed().as_nanos()
}

fn percentiles_ns(samples: &mut [u128]) -> (u128, u128, u128) {
    samples.sort_unstable();
    let percentile = |percent: usize| samples[(samples.len() * percent).div_ceil(100) - 1];
    (percentile(50), percentile(95), percentile(99))
}

// The retired loop recalculates the same window start for every physical slot.
fn retired_append_visible_virtual_group_rows(
    rows: &mut Vec<usize>,
    groups: &[AssetContentRowGroup],
    virtualization: &AssetBrowserVirtualization,
    viewport: Option<AssetContentRect>,
    scroll_px: f32,
    origin_x: f32,
    origin_y: f32,
    damage_clip: AssetContentRect,
) -> usize {
    let Some(viewport) = viewport.map(|viewport| viewport.translated(origin_x, origin_y)) else {
        return 0;
    };
    let Some(visible) = viewport.intersect(damage_clip) else {
        return 0;
    };
    let visible_top = visible.y - origin_y + scroll_px.max(0.0);
    let visible_bottom = visible.bottom() - origin_y + scroll_px.max(0.0);
    let mut visible_item_count = 0;
    for (slot_index, group) in groups.iter().enumerate() {
        let Some(binding) = virtualization.binding(scroll_px, slot_index) else {
            continue;
        };
        let top = group.top + binding.y_offset;
        let bottom = group.bottom + binding.y_offset;
        if bottom <= visible_top || top >= visible_bottom {
            continue;
        }
        rows.extend_from_slice(&group.node_rows);
        visible_item_count += 1;
    }
    visible_item_count
}

fn virtualization(
    item_count: usize,
    materialized_item_count: usize,
    columns: usize,
) -> AssetBrowserVirtualization {
    let items = (0..item_count)
        .map(|index| {
            AssetBrowserPaintItem::Thumbnail(AssetBrowserThumbnailPaintItem {
                name: format!("Asset {index}"),
                source_file_name: String::new(),
                file_extension: String::new(),
                name_continuation: String::new(),
                type_label: String::new(),
                type_label_width: 0.0,
                state_label: String::new(),
                visual_variant: String::new(),
                preview_artifact_path: String::new(),
            })
        })
        .collect::<Vec<_>>();
    AssetBrowserVirtualization::new(
        AssetBrowserLogicalPaintGeneration::from_chunks(vec![items.into()]),
        Vec::new(),
        materialized_item_count,
        columns,
        0.0,
        10.0,
        0.0,
        0,
    )
}
