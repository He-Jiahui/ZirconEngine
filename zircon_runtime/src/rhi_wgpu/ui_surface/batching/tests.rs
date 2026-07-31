use crate::rhi::{
    UiSurfaceCommand, UiSurfaceCommandKind, UiSurfaceDrawList, UiSurfaceImagePayload,
    UiSurfaceImageUvRect, UiSurfaceRect, UiSurfaceTextStyle,
};
use zircon_runtime_interface::ui::surface::UiResolvedStyle;

use super::*;

#[test]
fn batch_plan_batches_disjoint_quads_into_one_solid_draw() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![
            quad(
                0,
                UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0),
                [255, 0, 0, 255],
            ),
            quad(
                1,
                UiSurfaceRect::new(20.0, 0.0, 10.0, 10.0),
                [0, 255, 0, 255],
            ),
            quad(
                2,
                UiSurfaceRect::new(40.0, 0.0, 10.0, 10.0),
                [0, 0, 255, 255],
            ),
        ],
    );

    let plan = batch_draw_plan(&draw_list);

    assert_eq!(plan.stats.draw_calls, 1);
    assert_eq!(plan.stats.visible_draw_item_count, 3);
    assert_eq!(plan.stats.batch_merge_count, 2);
    assert_eq!(plan.stats.batch_layer_count, 1);
    assert_eq!(plan.stats.batch_dependency_count, 0);
    assert_eq!(plan.stats.batch_plan_build_count, 0);
    assert_eq!(plan.stats.batch_plan_cache_hit_count, 0);
    assert_eq!(plan.stats.vertex_buffer_create_count, 0);
    assert_eq!(plan.stats.vertex_upload_bytes, 0);
    assert_eq!(plan.stats.solid_vertex_count, 18);
    assert_eq!(plan.stats.image_vertex_count, 0);
    let DrawOp::Solid(draw) = &plan.ops[0] else {
        panic!("expected a solid batch");
    };
    assert_eq!(plan.solid_vertices.len(), 18);
    assert_eq!(draw.vertex_end - draw.vertex_start, 18);
}

#[test]
fn batch_plan_batches_disjoint_rounded_quads_without_fallback() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![
            rounded_quad(
                0,
                UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0),
                [255, 0, 0, 255],
                5.0,
            ),
            rounded_quad(
                1,
                UiSurfaceRect::new(20.0, 0.0, 10.0, 10.0),
                [0, 255, 0, 255],
                5.0,
            ),
        ],
    );

    let plan = batch_draw_plan(&draw_list);

    assert_eq!(plan.stats.draw_calls, 1);
    assert_eq!(plan.stats.visible_draw_item_count, 2);
    let DrawOp::Solid(draw) = &plan.ops[0] else {
        panic!("expected a solid batch");
    };
    assert!(plan.solid_vertices.len() > 12);
    assert!(draw.vertex_end > draw.vertex_start);
}

#[test]
fn batch_plan_splits_overlapping_quads_by_depth() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![
            quad(
                0,
                UiSurfaceRect::new(0.0, 0.0, 20.0, 20.0),
                [255, 0, 0, 255],
            ),
            quad(
                1,
                UiSurfaceRect::new(10.0, 10.0, 20.0, 20.0),
                [0, 255, 0, 255],
            ),
        ],
    );

    let plan = batch_draw_plan(&draw_list);

    assert_eq!(plan.stats.draw_calls, 2);
    assert_eq!(plan.stats.batch_layer_count, 2);
    assert_eq!(plan.stats.batch_dependency_count, 1);
    assert!(matches!(plan.ops[0], DrawOp::Solid(_)));
    assert!(matches!(plan.ops[1], DrawOp::Solid(_)));
}

#[test]
fn batch_plan_keeps_same_z_overlaps_in_original_index_order() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![
            quad(4, UiSurfaceRect::new(0.0, 0.0, 20.0, 20.0), [20, 0, 0, 255]),
            quad(4, UiSurfaceRect::new(5.0, 5.0, 20.0, 20.0), [30, 0, 0, 255]),
        ],
    );

    let plan = batch_draw_plan(&draw_list);

    let DrawOp::Solid(first) = &plan.ops[0] else {
        panic!("expected first solid depth layer");
    };
    let DrawOp::Solid(second) = &plan.ops[1] else {
        panic!("expected second solid depth layer");
    };
    let first_vertex = &plan.solid_vertices[first.vertex_start as usize];
    let second_vertex = &plan.solid_vertices[second.vertex_start as usize];
    assert_eq!(first_vertex.color, [20.0 / 255.0, 0.0, 0.0, 1.0]);
    assert_eq!(second_vertex.color, [30.0 / 255.0, 0.0, 0.0, 1.0]);
}

#[test]
fn batch_plan_batches_text_in_same_depth_layer() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![
            text(1, UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0), "A"),
            text(2, UiSurfaceRect::new(20.0, 0.0, 10.0, 10.0), "B"),
        ],
    );

    let plan = batch_draw_plan(&draw_list);

    assert_eq!(plan.stats.draw_calls, 1);
    let DrawOp::Text(draw) = &plan.ops[0] else {
        panic!("expected a text batch");
    };
    assert_eq!(draw.batch_index, 0);
    assert_eq!(draw.command_indices, vec![0, 1]);
}

#[test]
fn batch_plan_splits_text_when_overlapping_geometry_depends() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![
            quad(0, UiSurfaceRect::new(0.0, 0.0, 20.0, 20.0), [0, 0, 0, 255]),
            text(1, UiSurfaceRect::new(10.0, 10.0, 20.0, 20.0), "A"),
        ],
    );

    let plan = batch_draw_plan(&draw_list);

    assert_eq!(plan.stats.batch_layer_count, 2);
    assert!(matches!(plan.ops[0], DrawOp::Solid(_)));
    assert!(matches!(plan.ops[1], DrawOp::Text(_)));
}

#[test]
fn batch_plan_reserves_text_batch_indices_for_empty_then_visible_overlap() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![
            text(0, UiSurfaceRect::new(0.0, 0.0, 20.0, 20.0), "   "),
            text(1, UiSurfaceRect::new(0.0, 0.0, 20.0, 20.0), "Zircon"),
        ],
    );

    let plan = batch_draw_plan(&draw_list);
    let text_draws = plan
        .ops
        .iter()
        .filter_map(|op| match op {
            DrawOp::Text(draw) => Some(draw),
            DrawOp::Solid(_) | DrawOp::Image(_) => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(text_draws.len(), 2);
    assert_eq!(text_draws[0].batch_index, 0);
    assert_eq!(text_draws[0].command_indices, vec![0]);
    assert_eq!(text_draws[1].batch_index, 1);
    assert_eq!(text_draws[1].command_indices, vec![1]);
}

#[test]
fn batch_plan_groups_images_by_resource_key() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![
            image(0, UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0), "atlas-a"),
            image(1, UiSurfaceRect::new(20.0, 0.0, 10.0, 10.0), "atlas-a"),
            image(2, UiSurfaceRect::new(40.0, 0.0, 10.0, 10.0), "atlas-b"),
        ],
    );

    let plan = batch_draw_plan(&draw_list);

    assert_eq!(plan.stats.draw_calls, 2);
    let image_keys = plan
        .ops
        .iter()
        .filter_map(|op| match op {
            DrawOp::Image(draw) => Some(draw.resource_key.as_str()),
            DrawOp::Solid(_) | DrawOp::Text(_) => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(image_keys, vec!["atlas-a", "atlas-b"]);
    let upload_sources = plan
        .image_upload_sources
        .iter()
        .map(|source| (source.resource_key.as_str(), source.command_indices.clone()))
        .collect::<Vec<_>>();
    assert_eq!(
        upload_sources,
        vec![("atlas-a", vec![0, 1]), ("atlas-b", vec![2])]
    );
}

#[test]
fn batch_plan_batches_disjoint_images_with_same_resource_key_into_one_draw() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![
            image(
                0,
                UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0),
                "atlas://editor/icons",
            ),
            image(
                1,
                UiSurfaceRect::new(20.0, 0.0, 10.0, 10.0),
                "atlas://editor/icons",
            ),
            image(
                2,
                UiSurfaceRect::new(40.0, 0.0, 10.0, 10.0),
                "atlas://editor/icons",
            ),
        ],
    );

    let plan = batch_draw_plan(&draw_list);

    assert_eq!(plan.stats.visible_draw_item_count, 3);
    assert_eq!(plan.stats.draw_calls, 1);
    assert_eq!(plan.stats.batch_merge_count, 2);
    assert_eq!(plan.stats.batch_layer_count, 1);
    assert_eq!(plan.stats.batch_dependency_count, 0);
    let [DrawOp::Image(draw)] = plan.ops.as_slice() else {
        panic!("expected one image batch");
    };
    assert_eq!(draw.resource_key, "atlas://editor/icons");
    assert_eq!(plan.image_upload_sources.len(), 1);
    assert_eq!(
        plan.image_upload_sources[0].resource_key,
        "atlas://editor/icons"
    );
    assert_eq!(plan.image_upload_sources[0].command_indices, vec![0, 1, 2]);
    assert_eq!(plan.image_vertices.len(), 18);
    assert_eq!(plan.stats.solid_vertex_count, 0);
    assert_eq!(plan.stats.image_vertex_count, 18);
    assert_eq!(draw.vertex_end - draw.vertex_start, 18);
}

#[test]
fn batch_plan_batches_disjoint_atlas_images_with_same_key_and_distinct_uvs() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![
            atlas_image(
                0,
                UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0),
                "atlas://editor/icons",
                UiSurfaceImageUvRect {
                    min: [0.0, 0.0],
                    max: [0.25, 0.5],
                },
            ),
            atlas_image(
                1,
                UiSurfaceRect::new(20.0, 0.0, 10.0, 10.0),
                "atlas://editor/icons",
                UiSurfaceImageUvRect {
                    min: [0.25, 0.0],
                    max: [0.5, 0.5],
                },
            ),
        ],
    );

    let plan = batch_draw_plan(&draw_list);

    assert_eq!(plan.stats.visible_draw_item_count, 2);
    assert_eq!(plan.stats.draw_calls, 1);
    let [DrawOp::Image(draw)] = plan.ops.as_slice() else {
        panic!("expected one atlas image batch");
    };
    assert_eq!(draw.resource_key, "atlas://editor/icons");
    let vertices = &plan.image_vertices[draw.vertex_start as usize..draw.vertex_end as usize];
    assert_eq!(vertices.len(), 12);
    assert_eq!(vertices[0].uv, [0.0, 0.0]);
    assert_eq!(vertices[5].uv, [0.25, 0.5]);
    assert_eq!(vertices[6].uv, [0.25, 0.0]);
    assert_eq!(vertices[11].uv, [0.5, 0.5]);
}

#[test]
fn batch_plan_splits_overlapping_images_even_with_same_resource_key() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![
            image(
                0,
                UiSurfaceRect::new(0.0, 0.0, 20.0, 20.0),
                "atlas://editor/icons",
            ),
            image(
                1,
                UiSurfaceRect::new(10.0, 0.0, 20.0, 20.0),
                "atlas://editor/icons",
            ),
        ],
    );

    let plan = batch_draw_plan(&draw_list);

    assert_eq!(plan.stats.visible_draw_item_count, 2);
    assert_eq!(plan.stats.draw_calls, 2);
    assert_eq!(plan.stats.batch_layer_count, 2);
    assert_eq!(plan.stats.batch_dependency_count, 1);
    let [DrawOp::Image(first), DrawOp::Image(second)] = plan.ops.as_slice() else {
        panic!("expected overlapping images to split into image batches");
    };
    assert_eq!(first.resource_key, "atlas://editor/icons");
    assert_eq!(second.resource_key, "atlas://editor/icons");
}

#[test]
fn batch_plan_keeps_same_resource_upload_candidates_in_command_order() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![
            image(
                1,
                UiSurfaceRect::new(0.0, 0.0, 20.0, 20.0),
                "atlas://editor/icons",
            ),
            image(
                0,
                UiSurfaceRect::new(10.0, 0.0, 20.0, 20.0),
                "atlas://editor/icons",
            ),
        ],
    );

    let plan = batch_draw_plan(&draw_list);

    assert!(matches!(
        plan.ops.as_slice(),
        [DrawOp::Image(_), DrawOp::Image(_)]
    ));
    assert_eq!(plan.image_upload_sources.len(), 1);
    assert_eq!(plan.image_upload_sources[0].command_indices, vec![0, 1]);
}

#[test]
fn batch_plan_preserves_overlap_chain_between_same_resource_images() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![
            image(
                0,
                UiSurfaceRect::new(0.0, 0.0, 20.0, 20.0),
                "atlas://editor/icons",
            ),
            quad(
                1,
                UiSurfaceRect::new(10.0, 0.0, 20.0, 20.0),
                [255, 0, 0, 255],
            ),
            image(
                2,
                UiSurfaceRect::new(24.0, 0.0, 20.0, 20.0),
                "atlas://editor/icons",
            ),
        ],
    );

    let plan = batch_draw_plan(&draw_list);

    assert_eq!(plan.stats.visible_draw_item_count, 3);
    assert_eq!(plan.stats.draw_calls, 3);
    assert_eq!(plan.stats.batch_layer_count, 3);
    assert_eq!(plan.stats.batch_dependency_count, 2);
    let [DrawOp::Image(first), DrawOp::Solid(_), DrawOp::Image(third)] = plan.ops.as_slice() else {
        panic!("expected image-solid-image painter order across dependency layers");
    };
    assert_eq!(first.resource_key, "atlas://editor/icons");
    assert_eq!(third.resource_key, "atlas://editor/icons");
}

#[test]
fn batch_plan_batches_independent_same_resource_images_around_overlap() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![
            image(
                0,
                UiSurfaceRect::new(0.0, 0.0, 20.0, 20.0),
                "atlas://editor/icons",
            ),
            quad(
                1,
                UiSurfaceRect::new(10.0, 0.0, 20.0, 20.0),
                [255, 0, 0, 255],
            ),
            image(
                2,
                UiSurfaceRect::new(40.0, 0.0, 20.0, 20.0),
                "atlas://editor/icons",
            ),
        ],
    );

    let plan = batch_draw_plan(&draw_list);

    assert_eq!(plan.stats.visible_draw_item_count, 3);
    assert_eq!(plan.stats.draw_calls, 2);
    assert_eq!(plan.stats.batch_merge_count, 1);
    assert_eq!(plan.stats.batch_layer_count, 2);
    assert_eq!(plan.stats.batch_dependency_count, 1);
    let [DrawOp::Image(draw), DrawOp::Solid(_)] = plan.ops.as_slice() else {
        panic!("expected independent same-resource images to share the first layer");
    };
    assert_eq!(draw.resource_key, "atlas://editor/icons");
    assert_eq!(draw.vertex_end - draw.vertex_start, 12);
    assert_eq!(plan.image_vertices.len(), 12);
}

#[test]
fn batch_plan_batches_disjoint_list_rows_by_depth_and_material() {
    let draw_list = UiSurfaceDrawList::new(
        (200, 120),
        None,
        vec![
            quad(
                0,
                UiSurfaceRect::new(0.0, 0.0, 200.0, 20.0),
                [20, 20, 20, 255],
            ),
            text(1, UiSurfaceRect::new(8.0, 2.0, 80.0, 16.0), "Row 1"),
            quad(
                0,
                UiSurfaceRect::new(0.0, 24.0, 200.0, 20.0),
                [24, 24, 24, 255],
            ),
            text(1, UiSurfaceRect::new(8.0, 26.0, 80.0, 16.0), "Row 2"),
            quad(
                0,
                UiSurfaceRect::new(0.0, 48.0, 200.0, 20.0),
                [28, 28, 28, 255],
            ),
            text(1, UiSurfaceRect::new(8.0, 50.0, 80.0, 16.0), "Row 3"),
        ],
    );

    let plan = batch_draw_plan(&draw_list);

    assert_eq!(plan.stats.visible_draw_item_count, 6);
    assert_eq!(plan.stats.draw_calls, 2);
    assert_eq!(plan.stats.batch_layer_count, 2);
    assert_eq!(plan.stats.batch_dependency_count, 3);
    assert!(matches!(plan.ops[0], DrawOp::Solid(_)));
    let DrawOp::Text(text_draw) = &plan.ops[1] else {
        panic!("expected row labels to share one text batch");
    };
    assert_eq!(text_draw.command_indices, vec![1, 3, 5]);
}

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
        }
    }
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

fn quad(z_index: i32, frame: UiSurfaceRect, color: [u8; 4]) -> UiSurfaceCommand {
    UiSurfaceCommand {
        z_index,
        frame,
        clip: None,
        kind: UiSurfaceCommandKind::Quad {
            color,
            corner_radius: 0.0,
        },
    }
}

fn rounded_quad(
    z_index: i32,
    frame: UiSurfaceRect,
    color: [u8; 4],
    corner_radius: f32,
) -> UiSurfaceCommand {
    UiSurfaceCommand {
        z_index,
        frame,
        clip: None,
        kind: UiSurfaceCommandKind::Quad {
            color,
            corner_radius,
        },
    }
}

fn text(z_index: i32, frame: UiSurfaceRect, value: &str) -> UiSurfaceCommand {
    UiSurfaceCommand {
        z_index,
        frame,
        clip: None,
        kind: UiSurfaceCommandKind::Text {
            text: value.to_string(),
            color: [255, 255, 255, 255],
            font_family: None,
            font_weight: UiResolvedStyle::DEFAULT_FONT_WEIGHT,
            font_size: 12.0,
            line_height: 14.0,
            style: UiSurfaceTextStyle::Regular,
        },
    }
}

fn image(z_index: i32, frame: UiSurfaceRect, resource_key: &str) -> UiSurfaceCommand {
    UiSurfaceCommand {
        z_index,
        frame,
        clip: None,
        kind: UiSurfaceCommandKind::Image {
            payload: UiSurfaceImagePayload {
                resource_key: resource_key.to_string(),
                width: 2,
                height: 2,
                upload_bytes: 16,
                rgba: Some(vec![255; 16]),
                atlas_uv: None,
            },
        },
    }
}

fn atlas_image(
    z_index: i32,
    frame: UiSurfaceRect,
    resource_key: &str,
    atlas_uv: UiSurfaceImageUvRect,
) -> UiSurfaceCommand {
    UiSurfaceCommand {
        z_index,
        frame,
        clip: None,
        kind: UiSurfaceCommandKind::Image {
            payload: UiSurfaceImagePayload {
                resource_key: resource_key.to_string(),
                width: 64,
                height: 64,
                upload_bytes: 64 * 64 * 4,
                rgba: Some(vec![255; 64 * 64 * 4]),
                atlas_uv: Some(atlas_uv),
            },
        },
    }
}
