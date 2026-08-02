use super::*;

#[test]
fn batch_plan_scale_matrix_prunes_disjoint_rows_and_columns() {
    for item_count in [1_u32, 100, 1_000, 10_000] {
        for columns in [false, true] {
            let long_extent = item_count.saturating_mul(2).max(1);
            let commands = (0..item_count)
                .map(|item| {
                    let offset = item as f32 * 2.0;
                    let frame = if columns {
                        UiSurfaceRect::new(offset, 0.0, 1.0, long_extent as f32)
                    } else {
                        UiSurfaceRect::new(0.0, offset, long_extent as f32, 1.0)
                    };
                    quad(0, frame, [20, 20, 20, 255])
                })
                .collect();
            let surface_size = if columns {
                (long_extent, long_extent.max(1))
            } else {
                (long_extent.max(1), long_extent)
            };
            let draw_list = UiSurfaceDrawList::new(surface_size, None, commands);

            let plan = batch_draw_plan(&draw_list);

            assert_eq!(plan.stats.visible_draw_item_count, u64::from(item_count));
            assert_eq!(plan.stats.batch_dependency_count, 0);
            assert!(
                plan.stats.overlap_candidate_count <= item_count as u64,
                "axis-disjoint scale {item_count} should not perform a full pairwise overlap scan"
            );
            assert_eq!(plan.stats.draw_calls, 1);
            assert_eq!(
                plan.stats.batch_merge_count,
                u64::from(item_count.saturating_sub(1))
            );
            assert_eq!(
                plan.stats.solid_vertex_count,
                u64::from(item_count).saturating_mul(6)
            );
            assert_eq!(plan.stats.solid_instance_count, u64::from(item_count));
            assert!(plan.solid_vertices.is_empty());
            assert_eq!(plan.solid_instances.len(), item_count as usize);
            let [DrawOp::Solid(draw)] = plan.ops.as_slice() else {
                panic!("axis-disjoint guides must share one instanced solid draw");
            };
            assert_eq!(draw.vertex_start, draw.vertex_end);
            assert_eq!(draw.instance_start, 0);
            assert_eq!(draw.instance_end, item_count);
        }
    }
}

#[test]
fn interval_index_uses_linear_pooled_storage_for_sparse_scale() {
    let item_count = 10_000_u32;
    let commands = (0..item_count)
        .map(|item| {
            quad(
                0,
                UiSurfaceRect::new(0.0, item as f32 * 2.0, item_count as f32, 1.0),
                [20, 20, 20, 255],
            )
        })
        .collect();
    let draw_list = UiSurfaceDrawList::new((item_count, item_count * 2), None, commands);
    let items = draw_items(&draw_list);

    let (node_count, crossing_start_count, crossing_end_count) =
        dependency_depths::interval_index_storage_counts(&items);

    assert!(node_count <= item_count as usize);
    assert_eq!(crossing_start_count, item_count as usize);
    assert_eq!(crossing_end_count, item_count as usize);
}

#[test]
fn batch_plan_degenerates_to_one_draw_per_item_when_all_items_overlap() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![
            quad(0, UiSurfaceRect::new(0.0, 0.0, 50.0, 50.0), [10, 0, 0, 255]),
            quad(1, UiSurfaceRect::new(0.0, 0.0, 50.0, 50.0), [20, 0, 0, 255]),
            quad(2, UiSurfaceRect::new(0.0, 0.0, 50.0, 50.0), [30, 0, 0, 255]),
            quad(3, UiSurfaceRect::new(0.0, 0.0, 50.0, 50.0), [40, 0, 0, 255]),
        ],
    );

    let plan = batch_draw_plan(&draw_list);

    assert_eq!(plan.stats.visible_draw_item_count, 4);
    assert_eq!(plan.stats.draw_calls, 4);
    assert_eq!(plan.stats.batch_layer_count, 4);
    assert_eq!(plan.stats.batch_dependency_count, 6);
}

#[test]
fn batch_plan_uses_clip_reduced_rects_for_dependencies() {
    let mut left = quad(
        0,
        UiSurfaceRect::new(0.0, 0.0, 20.0, 20.0),
        [255, 0, 0, 255],
    );
    left.clip = Some(UiSurfaceRect::new(0.0, 0.0, 10.0, 20.0));
    let mut right = quad(
        1,
        UiSurfaceRect::new(5.0, 0.0, 20.0, 20.0),
        [0, 255, 0, 255],
    );
    right.clip = Some(UiSurfaceRect::new(10.0, 0.0, 20.0, 20.0));
    let draw_list = UiSurfaceDrawList::new((100, 100), None, vec![left, right]);

    let plan = batch_draw_plan(&draw_list);

    assert_eq!(plan.stats.batch_dependency_count, 0);
    assert_eq!(plan.stats.batch_layer_count, 1);
    assert_eq!(plan.stats.draw_calls, 1);
}

#[test]
fn compiled_plan_cache_reuses_an_explicit_unchanged_generation() {
    let draw_list = UiSurfaceDrawList::with_generation(
        (100, 100),
        None,
        vec![quad(
            0,
            UiSurfaceRect::new(0.0, 0.0, 20.0, 20.0),
            [255, 0, 0, 255],
        )],
        41,
    );
    let mut cache = CompiledUiBatchPlanCache::default();

    let first = cache.resolve(&draw_list, false);
    let second = cache.resolve(&draw_list, false);

    assert_eq!(first.batch_plan_build_count, 1);
    assert_eq!(first.batch_plan_cache_hit_count, 0);
    assert_eq!(second.batch_plan_build_count, 0);
    assert_eq!(second.batch_plan_cache_hit_count, 1);
    assert!(Arc::ptr_eq(&first.plan, &second.plan));
}

#[test]
fn compiled_plan_cache_requires_an_explicit_generation() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![quad(
            0,
            UiSurfaceRect::new(0.0, 0.0, 20.0, 20.0),
            [255, 0, 0, 255],
        )],
    );
    let mut cache = CompiledUiBatchPlanCache::default();

    let first = cache.resolve(&draw_list, false);
    let second = cache.resolve(&draw_list, false);

    assert_eq!(first.batch_plan_build_count, 1);
    assert_eq!(second.batch_plan_build_count, 1);
    assert_eq!(second.batch_plan_cache_hit_count, 0);
    assert!(!Arc::ptr_eq(&first.plan, &second.plan));
}

#[test]
fn compiled_plan_cache_reuses_the_full_projection_for_versioned_damage() {
    let draw_list = UiSurfaceDrawList::with_generation(
        (100, 100),
        Some(UiSurfaceRect::new(5.0, 5.0, 10.0, 10.0)),
        vec![
            quad(
                0,
                UiSurfaceRect::new(0.0, 0.0, 20.0, 20.0),
                [255, 0, 0, 255],
            ),
            quad(
                1,
                UiSurfaceRect::new(60.0, 60.0, 20.0, 20.0),
                [0, 255, 0, 255],
            ),
        ],
        41,
    );
    let mut cache = CompiledUiBatchPlanCache::default();

    let first = cache.resolve(&draw_list, false);
    let second = cache.resolve(&draw_list, false);

    assert_eq!(first.batch_plan_build_count, 1);
    assert_eq!(first.plan.stats.visible_draw_item_count, 2);
    assert_eq!(second.batch_plan_build_count, 0);
    assert_eq!(second.batch_plan_cache_hit_count, 1);
    assert!(Arc::ptr_eq(&first.plan, &second.plan));
}

#[test]
fn unversioned_damage_uses_full_projection_when_the_target_requires_a_full_redraw() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        Some(UiSurfaceRect::new(0.0, 0.0, 20.0, 20.0)),
        vec![
            quad(
                0,
                UiSurfaceRect::new(0.0, 0.0, 20.0, 20.0),
                [255, 0, 0, 255],
            ),
            quad(
                1,
                UiSurfaceRect::new(60.0, 60.0, 20.0, 20.0),
                [0, 255, 0, 255],
            ),
        ],
    );
    let mut cache = CompiledUiBatchPlanCache::default();

    let full_redraw = cache.resolve(&draw_list, true);
    let partial_draw = cache.resolve(&draw_list, false);

    assert_eq!(full_redraw.plan.stats.visible_draw_item_count, 2);
    assert_eq!(
        full_redraw
            .draw_list_stats
            .expect("full redraw stats")
            .visible_command_count,
        2
    );
    assert_eq!(partial_draw.plan.stats.visible_draw_item_count, 1);
    assert_eq!(
        partial_draw
            .draw_list_stats
            .expect("partial draw stats")
            .visible_command_count,
        1
    );
}

#[test]
fn versioned_damage_cache_retains_undamaged_stats_for_a_later_full_redraw() {
    let commands = vec![
        quad(
            0,
            UiSurfaceRect::new(0.0, 0.0, 20.0, 20.0),
            [255, 0, 0, 255],
        ),
        quad(
            1,
            UiSurfaceRect::new(60.0, 60.0, 20.0, 20.0),
            [0, 255, 0, 255],
        ),
    ];
    let damaged = UiSurfaceDrawList::with_generation(
        (100, 100),
        Some(UiSurfaceRect::new(0.0, 0.0, 20.0, 20.0)),
        commands.clone(),
        99,
    );
    let undamaged = UiSurfaceDrawList::with_generation((100, 100), None, commands, 99);
    let mut cache = CompiledUiBatchPlanCache::default();

    let damaged_result = cache.resolve(&damaged, false);
    let full_redraw = cache.resolve(&undamaged, true);
    let stats = full_redraw.draw_list_stats.expect("cached full stats");

    assert_eq!(damaged_result.plan.stats.visible_draw_item_count, 2);
    assert_eq!(full_redraw.batch_plan_cache_hit_count, 1);
    assert_eq!(stats.visible_command_count, 2);
    assert_eq!(stats.visible_draw_item_count, 2);
    assert_eq!(stats.command_visibility_scan_count, 0);
    assert_eq!(stats.command_stats_cache_hit_count, 1);
}

#[test]
fn text_batch_keeps_union_bounds_for_damage_culling() {
    let draw_list = UiSurfaceDrawList::with_generation(
        (100, 100),
        None,
        vec![
            text(0, UiSurfaceRect::new(8.0, 12.0, 10.0, 6.0), "first"),
            text(1, UiSurfaceRect::new(32.0, 20.0, 12.0, 8.0), "second"),
        ],
        42,
    );

    let plan = full_projection_batch_draw_plan(&draw_list);
    let text_draw = plan
        .ops
        .iter()
        .find_map(|op| match op {
            DrawOp::Text(draw) => Some(draw),
            DrawOp::Solid(_) | DrawOp::Image(_) => None,
        })
        .expect("text draw");

    assert_eq!(text_draw.bounds, UiSurfaceRect::new(8.0, 12.0, 36.0, 16.0));
}

#[test]
fn solid_and_image_batches_keep_union_bounds_for_damage_culling() {
    let draw_list = UiSurfaceDrawList::with_generation(
        (100, 100),
        None,
        vec![
            quad(0, UiSurfaceRect::new(4.0, 5.0, 6.0, 5.0), [255, 0, 0, 255]),
            quad(1, UiSurfaceRect::new(20.0, 8.0, 4.0, 4.0), [0, 255, 0, 255]),
            image(2, UiSurfaceRect::new(50.0, 7.0, 5.0, 5.0), "atlas"),
            image(3, UiSurfaceRect::new(70.0, 12.0, 4.0, 8.0), "atlas"),
        ],
        43,
    );

    let plan = full_projection_batch_draw_plan(&draw_list);
    let solid_draw = plan
        .ops
        .iter()
        .find_map(|op| match op {
            DrawOp::Solid(draw) => Some(draw),
            DrawOp::Image(_) | DrawOp::Text(_) => None,
        })
        .expect("solid draw");
    let image_draw = plan
        .ops
        .iter()
        .find_map(|op| match op {
            DrawOp::Image(draw) => Some(draw),
            DrawOp::Solid(_) | DrawOp::Text(_) => None,
        })
        .expect("image draw");

    assert_eq!(solid_draw.bounds, UiSurfaceRect::new(4.0, 5.0, 20.0, 7.0));
    assert_eq!(image_draw.bounds, UiSurfaceRect::new(50.0, 7.0, 24.0, 13.0));
}
