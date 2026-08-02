use std::collections::BTreeMap;
use std::sync::Arc;

use super::geometry::{
    DrawItem, ImageVertex, SolidGeometry, SolidInstance, SolidVertex, UI_QUAD_VERTEX_COUNT,
    draw_items, draw_items_with_stats, full_projection_draw_items_with_stats,
};
use crate::rhi::{UiSurfaceDrawList, UiSurfacePresentStats, UiSurfaceRect};

mod dependency_depths;

use dependency_depths::dependency_depths;

#[derive(Clone, Debug)]
pub(super) struct SolidDraw {
    pub(super) layer_index: u32,
    pub(super) item_count: u32,
    pub(super) vertex_start: u32,
    pub(super) vertex_end: u32,
    pub(super) instance_start: u32,
    pub(super) instance_end: u32,
    pub(super) bounds: UiSurfaceRect,
}

#[derive(Clone, Debug)]
pub(super) struct ImageDraw {
    pub(super) layer_index: u32,
    pub(super) item_count: u32,
    pub(super) resource_key: String,
    pub(super) vertex_start: u32,
    pub(super) vertex_end: u32,
    pub(super) bounds: UiSurfaceRect,
}

#[derive(Clone, Debug)]
pub(super) struct TextDraw {
    pub(super) layer_index: u32,
    pub(super) command_indices: Vec<usize>,
    pub(super) batch_index: usize,
    /// Full-projection bounds used to cull whole text batches for a damage scissor.
    pub(super) bounds: UiSurfaceRect,
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
    pub(super) render_pass_count: u64,
    pub(super) visible_draw_item_count: u64,
    pub(super) batch_merge_count: u64,
    pub(super) solid_vertex_count: u64,
    pub(super) solid_instance_count: u64,
    pub(super) image_vertex_count: u64,
    pub(super) batch_layer_count: u64,
    pub(super) batch_dependency_count: u64,
    pub(super) overlap_candidate_count: u64,
    pub(super) batch_plan_build_count: u64,
    pub(super) batch_plan_cache_hit_count: u64,
    pub(super) vertex_buffer_create_count: u64,
    pub(super) vertex_upload_bytes: u64,
    pub(super) retained_cache_copy_bytes: u64,
}

#[derive(Clone, Debug, Default)]
pub(super) struct BatchDrawPlan {
    pub(super) ops: Vec<DrawOp>,
    /// The sole compiled solid vertex payload; draw ops refer to ranges in this buffer.
    pub(super) solid_vertices: Vec<SolidVertex>,
    /// Compact ordinary quads; draw ops refer to instance ranges in this buffer.
    pub(super) solid_instances: Vec<SolidInstance>,
    /// The sole compiled image vertex payload; draw ops refer to ranges in this buffer.
    pub(super) image_vertices: Vec<ImageVertex>,
    /// One ordered source group per image resource for upload preparation.
    pub(super) image_upload_sources: Vec<ImageUploadSource>,
    /// Reused only for an unchanged full projection; damage still derives live visibility stats.
    pub(super) full_draw_list_stats: Option<UiSurfacePresentStats>,
    pub(super) stats: BatchDrawPlanStats,
}

#[derive(Clone, Debug)]
pub(super) struct ImageUploadSource {
    pub(super) resource_key: String,
    pub(super) command_indices: Vec<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BatchPlanCacheKey {
    generation: u64,
    surface_size: (u32, u32),
}

/// Retains immutable geometry and overlap topology for a producer-owned UI generation.
///
/// Damage rendering clips the full projection with a GPU scissor, allowing the same ordered
/// geometry and dependency topology to serve both complete and partial writes.
#[derive(Default)]
pub(super) struct CompiledUiBatchPlanCache {
    key: Option<BatchPlanCacheKey>,
    plan: Option<Arc<BatchDrawPlan>>,
}

pub(super) struct ResolvedBatchDrawPlan {
    pub(super) plan: Arc<BatchDrawPlan>,
    pub(super) batch_plan_build_count: u64,
    pub(super) batch_plan_cache_hit_count: u64,
    pub(super) draw_list_stats: Option<UiSurfacePresentStats>,
}

impl CompiledUiBatchPlanCache {
    pub(super) fn resolve(
        &mut self,
        draw_list: &UiSurfaceDrawList,
        force_full_projection: bool,
    ) -> ResolvedBatchDrawPlan {
        let Some(generation) = draw_list.generation() else {
            let (plan, draw_list_stats) = if force_full_projection {
                let (plan, full_draw_list_stats, _) =
                    full_projection_batch_draw_plan_with_stats(draw_list);
                (plan, full_draw_list_stats)
            } else {
                let (items, draw_list_stats) = draw_items_with_stats(draw_list);
                (batch_draw_plan_from_items(items), draw_list_stats)
            };
            return ResolvedBatchDrawPlan {
                plan: Arc::new(plan),
                batch_plan_build_count: 1,
                batch_plan_cache_hit_count: 0,
                draw_list_stats: Some(draw_list_stats),
            };
        };
        let key = BatchPlanCacheKey {
            generation,
            surface_size: draw_list.surface_size,
        };
        let use_full_draw_list_stats = force_full_projection || draw_list.damage.is_none();
        if self.key == Some(key) {
            if let Some(plan) = &self.plan {
                let draw_list_stats = if use_full_draw_list_stats {
                    plan.full_draw_list_stats.map(|mut stats| {
                        stats.command_visibility_scan_count = 0;
                        stats.command_stats_cache_hit_count = 1;
                        stats
                    })
                } else {
                    Some(draw_list.stats())
                };
                return ResolvedBatchDrawPlan {
                    plan: Arc::clone(plan),
                    batch_plan_build_count: 0,
                    batch_plan_cache_hit_count: 1,
                    draw_list_stats,
                };
            }
        }

        let (plan, full_draw_list_stats, damage_draw_list_stats) =
            full_projection_batch_draw_plan_with_stats(draw_list);
        let plan = Arc::new(plan);
        self.key = Some(key);
        self.plan = Some(Arc::clone(&plan));
        let draw_list_stats = if use_full_draw_list_stats {
            full_draw_list_stats
        } else {
            damage_draw_list_stats.unwrap_or(full_draw_list_stats)
        };
        ResolvedBatchDrawPlan {
            plan,
            batch_plan_build_count: 1,
            batch_plan_cache_hit_count: 0,
            draw_list_stats: Some(draw_list_stats),
        }
    }
}

pub(super) fn batch_draw_plan(draw_list: &UiSurfaceDrawList) -> BatchDrawPlan {
    batch_draw_plan_from_items(draw_items(draw_list))
}

#[cfg(test)]
fn full_projection_batch_draw_plan(draw_list: &UiSurfaceDrawList) -> BatchDrawPlan {
    full_projection_batch_draw_plan_with_stats(draw_list).0
}

fn full_projection_batch_draw_plan_with_stats(
    draw_list: &UiSurfaceDrawList,
) -> (
    BatchDrawPlan,
    UiSurfacePresentStats,
    Option<UiSurfacePresentStats>,
) {
    let (items, full_draw_list_stats, damage_draw_list_stats) =
        full_projection_draw_items_with_stats(draw_list);
    let mut plan = batch_draw_plan_from_items(items);
    plan.full_draw_list_stats = Some(full_draw_list_stats);
    (plan, full_draw_list_stats, damage_draw_list_stats)
}

fn batch_draw_plan_from_items(mut items: Vec<DrawItem>) -> BatchDrawPlan {
    if items.is_empty() {
        return BatchDrawPlan::default();
    }

    let (depths, layer_count, dependency_count, overlap_candidate_count) =
        dependency_depths(&items);
    let mut layered_item_indices = (0..items.len()).collect::<Vec<_>>();
    layered_item_indices
        .sort_unstable_by_key(|item_index| (depths[*item_index], items[*item_index].order()));

    let (solid_vertex_capacity, solid_instance_capacity, image_vertex_capacity) = items
        .iter()
        .fold((0_usize, 0_usize, 0_usize), |capacity, item| match item {
            DrawItem::Solid(item) => match &item.geometry {
                SolidGeometry::Vertices(vertices) => (
                    capacity.0.saturating_add(vertices.len()),
                    capacity.1,
                    capacity.2,
                ),
                SolidGeometry::Instance(_) => {
                    (capacity.0, capacity.1.saturating_add(1), capacity.2)
                }
            },
            DrawItem::Image(_) => (
                capacity.0,
                capacity.1,
                capacity.2.saturating_add(UI_QUAD_VERTEX_COUNT as usize),
            ),
            DrawItem::Text(_) => capacity,
        });
    let mut ops = Vec::with_capacity(items.len());
    let mut solid_vertices = Vec::with_capacity(solid_vertex_capacity);
    let mut solid_instances = Vec::with_capacity(solid_instance_capacity);
    let mut image_vertices = Vec::with_capacity(image_vertex_capacity);
    let mut image_source_indices_by_resource = BTreeMap::<String, Vec<usize>>::new();
    let mut text_batch_index = 0;

    let mut layer_start = 0;
    while layer_start < layered_item_indices.len() {
        let layer_depth = depths[layered_item_indices[layer_start]];
        let layer_end = layered_item_indices[layer_start..]
            .iter()
            .position(|item_index| depths[*item_index] != layer_depth)
            .map_or(layered_item_indices.len(), |offset| layer_start + offset);
        let layer = &layered_item_indices[layer_start..layer_end];
        let layer_index = layer_depth as u32;
        push_layer_solid_draw(
            &mut items,
            layer,
            layer_index,
            &mut ops,
            &mut solid_vertices,
            &mut solid_instances,
        );
        push_layer_image_draws(
            &items,
            layer,
            layer_index,
            &mut ops,
            &mut image_vertices,
            &mut image_source_indices_by_resource,
        );
        push_layer_text_draw(&items, layer, layer_index, &mut ops, &mut text_batch_index);
        layer_start = layer_end;
    }
    let image_upload_sources = image_upload_sources(image_source_indices_by_resource);
    let solid_instance_count = solid_instances.len() as u64;
    let solid_vertex_count = (solid_vertices.len() as u64)
        .saturating_add(solid_instance_count.saturating_mul(u64::from(UI_QUAD_VERTEX_COUNT)));
    let image_vertex_count = image_vertices.len() as u64;

    BatchDrawPlan {
        stats: BatchDrawPlanStats {
            draw_calls: ops.len() as u64,
            render_pass_count: 0,
            visible_draw_item_count: items.len() as u64,
            batch_merge_count: (items.len() as u64).saturating_sub(ops.len() as u64),
            solid_vertex_count,
            solid_instance_count,
            image_vertex_count,
            batch_layer_count: layer_count as u64,
            batch_dependency_count: dependency_count as u64,
            overlap_candidate_count: overlap_candidate_count as u64,
            batch_plan_build_count: 0,
            batch_plan_cache_hit_count: 0,
            vertex_buffer_create_count: 0,
            vertex_upload_bytes: 0,
            retained_cache_copy_bytes: 0,
        },
        ops,
        solid_vertices,
        solid_instances,
        image_vertices,
        image_upload_sources,
        full_draw_list_stats: None,
    }
}

fn image_upload_sources(
    source_indices_by_resource: BTreeMap<String, Vec<usize>>,
) -> Vec<ImageUploadSource> {
    source_indices_by_resource
        .into_iter()
        .map(|(resource_key, mut command_indices)| {
            command_indices.sort_unstable();
            command_indices.dedup();
            ImageUploadSource {
                resource_key,
                command_indices,
            }
        })
        .collect()
}

fn push_layer_solid_draw(
    items: &mut [DrawItem],
    layer: &[usize],
    layer_index: u32,
    ops: &mut Vec<DrawOp>,
    solid_vertices: &mut Vec<SolidVertex>,
    solid_instances: &mut Vec<SolidInstance>,
) {
    let vertex_start = solid_vertices.len() as u32;
    let instance_start = solid_instances.len() as u32;
    let mut vertex_bounds = None;
    let mut instance_bounds = None;
    let mut vertex_item_count = 0_u32;
    let mut instance_item_count = 0_u32;
    for item_index in layer {
        let DrawItem::Solid(item) = &mut items[*item_index] else {
            continue;
        };
        match std::mem::replace(&mut item.geometry, SolidGeometry::Vertices(Vec::new())) {
            SolidGeometry::Instance(instance) => {
                instance_bounds = Some(match instance_bounds {
                    Some(current) => union_rects(current, item.rect),
                    None => item.rect,
                });
                instance_item_count = instance_item_count.saturating_add(1);
                solid_instances.push(instance);
            }
            SolidGeometry::Vertices(mut vertices) => {
                vertex_bounds = Some(match vertex_bounds {
                    Some(current) => union_rects(current, item.rect),
                    None => item.rect,
                });
                vertex_item_count = vertex_item_count.saturating_add(1);
                solid_vertices.append(&mut vertices);
            }
        }
    }
    if let Some(bounds) = instance_bounds {
        ops.push(DrawOp::Solid(SolidDraw {
            layer_index,
            item_count: instance_item_count,
            vertex_start,
            vertex_end: vertex_start,
            instance_start,
            instance_end: solid_instances.len() as u32,
            bounds,
        }));
    }
    if let Some(bounds) = vertex_bounds {
        ops.push(DrawOp::Solid(SolidDraw {
            layer_index,
            item_count: vertex_item_count,
            vertex_start,
            vertex_end: solid_vertices.len() as u32,
            instance_start,
            instance_end: instance_start,
            bounds,
        }));
    }
}

fn push_layer_image_draws(
    items: &[DrawItem],
    layer: &[usize],
    layer_index: u32,
    ops: &mut Vec<DrawOp>,
    image_vertices: &mut Vec<ImageVertex>,
    source_indices_by_resource: &mut BTreeMap<String, Vec<usize>>,
) {
    let mut item_indices_by_resource = BTreeMap::<&str, (Vec<usize>, UiSurfaceRect)>::new();
    for item_index in layer {
        let DrawItem::Image(item) = &items[*item_index] else {
            continue;
        };
        // Keep the per-item key borrowed; the compiled draw owns one key per resource batch.
        let entry = item_indices_by_resource
            .entry(item.resource_key.as_str())
            .or_insert_with(|| (Vec::new(), item.rect));
        entry.0.push(*item_index);
        entry.1 = union_rects(entry.1, item.rect);
    }
    for (resource_key, (item_indices, bounds)) in item_indices_by_resource {
        let vertex_start = image_vertices.len() as u32;
        let mut source_command_indices = Vec::with_capacity(item_indices.len());
        let item_count = item_indices.len() as u32;
        for item_index in item_indices {
            let DrawItem::Image(item) = &items[item_index] else {
                continue;
            };
            image_vertices.extend_from_slice(&item.vertices);
            source_command_indices.push(item.order.command_index);
        }
        source_indices_by_resource
            .entry(resource_key.to_owned())
            .or_default()
            .extend(source_command_indices);
        ops.push(DrawOp::Image(ImageDraw {
            layer_index,
            item_count,
            resource_key: resource_key.to_owned(),
            vertex_start,
            vertex_end: image_vertices.len() as u32,
            bounds,
        }));
    }
}

fn push_layer_text_draw(
    items: &[DrawItem],
    layer: &[usize],
    layer_index: u32,
    ops: &mut Vec<DrawOp>,
    text_batch_index: &mut usize,
) {
    let mut command_indices = Vec::new();
    let mut bounds = None;
    for item_index in layer {
        let DrawItem::Text(item) = &items[*item_index] else {
            continue;
        };
        command_indices.push(item.command_index);
        bounds = Some(match bounds {
            Some(current) => union_rects(current, item.rect),
            None => item.rect,
        });
    }
    if command_indices.is_empty() {
        return;
    }
    let Some(bounds) = bounds else {
        return;
    };
    ops.push(DrawOp::Text(TextDraw {
        layer_index,
        command_indices,
        batch_index: *text_batch_index,
        bounds,
    }));
    *text_batch_index += 1;
}

fn union_rects(left: UiSurfaceRect, right: UiSurfaceRect) -> UiSurfaceRect {
    let min_x = left.x.min(right.x);
    let min_y = left.y.min(right.y);
    let max_x = (left.x + left.width).max(right.x + right.width);
    let max_y = (left.y + left.height).max(right.y + right.height);
    UiSurfaceRect::new(min_x, min_y, max_x - min_x, max_y - min_y)
}

#[cfg(test)]
#[path = "batching/tests.rs"]
mod tests;
