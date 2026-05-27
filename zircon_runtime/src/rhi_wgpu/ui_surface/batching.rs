use std::collections::BTreeMap;

use super::geometry::{draw_items, DrawItem, ImageVertex, SolidVertex};
use crate::rhi::{UiSurfaceDrawList, UiSurfaceRect};

#[derive(Clone, Debug)]
pub(super) struct SolidDraw {
    pub(super) vertices: Vec<SolidVertex>,
    pub(super) vertex_start: u32,
    pub(super) vertex_end: u32,
}

#[derive(Clone, Debug)]
pub(super) struct ImageDraw {
    pub(super) resource_key: String,
    pub(super) vertices: Vec<ImageVertex>,
    pub(super) vertex_start: u32,
    pub(super) vertex_end: u32,
}

#[derive(Clone, Debug)]
pub(super) struct TextDraw {
    pub(super) command_indices: Vec<usize>,
    pub(super) batch_index: usize,
}

#[derive(Clone, Debug)]
pub(super) enum DrawOp {
    Solid(SolidDraw),
    Image(ImageDraw),
    Text(TextDraw),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct BatchDrawPlanStats {
    pub(super) draw_calls: u64,
    pub(super) visible_draw_item_count: u64,
    pub(super) batch_layer_count: u64,
    pub(super) batch_dependency_count: u64,
}

#[derive(Clone, Debug, Default)]
pub(super) struct BatchDrawPlan {
    pub(super) ops: Vec<DrawOp>,
    pub(super) stats: BatchDrawPlanStats,
}

pub(super) fn batch_draw_plan(draw_list: &UiSurfaceDrawList) -> BatchDrawPlan {
    let items = draw_items(draw_list);
    if items.is_empty() {
        return BatchDrawPlan::default();
    }

    let (depths, layer_count, dependency_count) = dependency_depths(&items);
    let mut layers = vec![Vec::new(); layer_count];
    for (item_index, depth) in depths.into_iter().enumerate() {
        layers[depth].push(item_index);
    }
    for layer in &mut layers {
        layer.sort_by_key(|item_index| items[*item_index].order());
    }

    let mut ops = Vec::new();
    let mut solid_vertex_start = 0;
    let mut image_vertex_start = 0;
    let mut text_batch_index = 0;

    for layer in layers {
        push_layer_solid_draw(&items, &layer, &mut ops, &mut solid_vertex_start);
        push_layer_image_draws(&items, &layer, &mut ops, &mut image_vertex_start);
        push_layer_text_draw(&items, &layer, &mut ops, &mut text_batch_index);
    }

    BatchDrawPlan {
        stats: BatchDrawPlanStats {
            draw_calls: ops.len() as u64,
            visible_draw_item_count: items.len() as u64,
            batch_layer_count: layer_count as u64,
            batch_dependency_count: dependency_count as u64,
        },
        ops,
    }
}

fn dependency_depths(items: &[DrawItem]) -> (Vec<usize>, usize, usize) {
    let mut depths = vec![0; items.len()];
    let mut dependency_count = 0;
    // The old painter order is only mandatory when two visible items overlap.
    // Non-overlapping items stay incomparable and can share a batch layer.
    for later_index in 0..items.len() {
        let later_rect = items[later_index].rect();
        for earlier_index in 0..later_index {
            if !rects_intersect(items[earlier_index].rect(), later_rect) {
                continue;
            }
            dependency_count += 1;
            depths[later_index] = depths[later_index].max(depths[earlier_index] + 1);
        }
    }
    let layer_count = depths
        .iter()
        .copied()
        .max()
        .map(|depth| depth + 1)
        .unwrap_or(0);
    (depths, layer_count, dependency_count)
}

fn push_layer_solid_draw(
    items: &[DrawItem],
    layer: &[usize],
    ops: &mut Vec<DrawOp>,
    solid_vertex_start: &mut u32,
) {
    let vertices = layer
        .iter()
        .filter_map(|item_index| match &items[*item_index] {
            DrawItem::Solid(item) => Some(item.vertices.iter().copied()),
            DrawItem::Image(_) | DrawItem::Text(_) => None,
        })
        .flatten()
        .collect::<Vec<_>>();
    if vertices.is_empty() {
        return;
    }
    let vertex_start = *solid_vertex_start;
    *solid_vertex_start += vertices.len() as u32;
    ops.push(DrawOp::Solid(SolidDraw {
        vertices,
        vertex_start,
        vertex_end: *solid_vertex_start,
    }));
}

fn push_layer_image_draws(
    items: &[DrawItem],
    layer: &[usize],
    ops: &mut Vec<DrawOp>,
    image_vertex_start: &mut u32,
) {
    let mut vertices_by_resource = BTreeMap::<String, Vec<ImageVertex>>::new();
    for item_index in layer {
        let DrawItem::Image(item) = &items[*item_index] else {
            continue;
        };
        vertices_by_resource
            .entry(item.resource_key.clone())
            .or_default()
            .extend(item.vertices);
    }
    for (resource_key, vertices) in vertices_by_resource {
        let vertex_start = *image_vertex_start;
        *image_vertex_start += vertices.len() as u32;
        ops.push(DrawOp::Image(ImageDraw {
            resource_key,
            vertices,
            vertex_start,
            vertex_end: *image_vertex_start,
        }));
    }
}

fn push_layer_text_draw(
    items: &[DrawItem],
    layer: &[usize],
    ops: &mut Vec<DrawOp>,
    text_batch_index: &mut usize,
) {
    let command_indices = layer
        .iter()
        .filter_map(|item_index| match &items[*item_index] {
            DrawItem::Text(item) => Some(item.command_index),
            DrawItem::Solid(_) | DrawItem::Image(_) => None,
        })
        .collect::<Vec<_>>();
    if command_indices.is_empty() {
        return;
    }
    ops.push(DrawOp::Text(TextDraw {
        command_indices,
        batch_index: *text_batch_index,
    }));
    *text_batch_index += 1;
}

fn rects_intersect(left: UiSurfaceRect, right: UiSurfaceRect) -> bool {
    let left_right = left.x + left.width;
    let left_bottom = left.y + left.height;
    let right_right = right.x + right.width;
    let right_bottom = right.y + right.height;
    left.x < right_right && right.x < left_right && left.y < right_bottom && right.y < left_bottom
}

#[cfg(test)]
mod tests {
    use crate::rhi::{
        UiSurfaceCommand, UiSurfaceCommandKind, UiSurfaceDrawList, UiSurfaceImagePayload,
        UiSurfaceImageUvRect, UiSurfaceRect, UiSurfaceTextStyle,
    };

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
        assert_eq!(plan.stats.batch_layer_count, 1);
        assert_eq!(plan.stats.batch_dependency_count, 0);
        let DrawOp::Solid(draw) = &plan.ops[0] else {
            panic!("expected a solid batch");
        };
        assert_eq!(draw.vertices.len(), 18);
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
        assert!(draw.vertices.len() > 12);
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
        assert_eq!(first.vertices[0].color, [20.0 / 255.0, 0.0, 0.0, 1.0]);
        assert_eq!(second.vertices[0].color, [30.0 / 255.0, 0.0, 0.0, 1.0]);
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
        assert_eq!(plan.stats.batch_layer_count, 1);
        assert_eq!(plan.stats.batch_dependency_count, 0);
        let [DrawOp::Image(draw)] = plan.ops.as_slice() else {
            panic!("expected one image batch");
        };
        assert_eq!(draw.resource_key, "atlas://editor/icons");
        assert_eq!(draw.vertices.len(), 18);
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
        assert_eq!(draw.vertices.len(), 12);
        assert_eq!(draw.vertices[0].uv, [0.0, 0.0]);
        assert_eq!(draw.vertices[5].uv, [0.25, 0.5]);
        assert_eq!(draw.vertices[6].uv, [0.25, 0.0]);
        assert_eq!(draw.vertices[11].uv, [0.5, 0.5]);
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
        let [DrawOp::Image(first), DrawOp::Solid(_), DrawOp::Image(third)] = plan.ops.as_slice()
        else {
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
        assert_eq!(plan.stats.batch_layer_count, 2);
        assert_eq!(plan.stats.batch_dependency_count, 1);
        let [DrawOp::Image(draw), DrawOp::Solid(_)] = plan.ops.as_slice() else {
            panic!("expected independent same-resource images to share the first layer");
        };
        assert_eq!(draw.resource_key, "atlas://editor/icons");
        assert_eq!(draw.vertices.len(), 12);
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
}
