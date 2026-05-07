use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;
use crate::ui::layout::UiFrame;

use super::{
    UiBatchKey, UiBatchPlan, UiBatchPrimitive, UiBatchShader, UiBatchSplitReason, UiBrushPayload,
    UiPaintElement, UiPaintPayload, UiRenderCacheInvalidationReason, UiRenderCachePlan,
    UiRenderCacheStatus, UiRenderResourceKey, UiTextRenderMode,
};

/// Replay-friendly render inspection payload for Widget Reflector style panels.
/// It is derived from paint elements and batches, not from renderer-private state.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiRenderVisualizerSnapshot {
    pub paint_elements: Vec<UiRenderVisualizerPaintElement>,
    pub batch_groups: Vec<UiRenderVisualizerBatchGroup>,
    pub overlays: Vec<UiRenderVisualizerOverlay>,
    pub overdraw_regions: Vec<UiRenderVisualizerOverdrawRegion>,
    pub resource_bindings: Vec<UiRenderVisualizerResourceBinding>,
    pub text: UiRenderVisualizerTextStats,
    pub stats: UiRenderVisualizerStats,
}

impl UiRenderVisualizerSnapshot {
    pub fn from_paint_elements_batches_cache(
        elements: &[UiPaintElement],
        plan: &UiBatchPlan,
        cache: &UiRenderCachePlan,
    ) -> Self {
        let mut resource_bindings = Vec::new();
        let paint_elements = elements
            .iter()
            .enumerate()
            .map(|(paint_index, element)| {
                let batch_index = batch_index_for_paint_index(plan, paint_index);
                for resource in paint_resource_keys(element) {
                    add_resource_binding(
                        &mut resource_bindings,
                        resource,
                        Some(paint_index),
                        batch_index,
                    );
                }

                let key = UiBatchKey::from_paint_element(element);
                UiRenderVisualizerPaintElement {
                    paint_index,
                    node_id: element.node_id,
                    frame: element.geometry.render_bounds,
                    clip_frame: element.clip.as_ref().map(|clip| clip.frame),
                    z_index: element.z_index,
                    paint_order: element.paint_order,
                    payload_kind: UiRenderVisualizerPaintPayloadKind::from_payload(
                        &element.payload,
                    ),
                    primitive: key.primitive,
                    shader: key.shader,
                    resource: key.resource,
                    text_backend: key.text_backend,
                    opacity: element.effects.opacity,
                    batch_index,
                    cache_status: cache
                        .paint_entries
                        .iter()
                        .find(|entry| entry.paint_index == paint_index)
                        .map(|entry| entry.status),
                    debug_label: element.debug_label.clone(),
                }
            })
            .collect::<Vec<_>>();

        let batch_groups = plan
            .batches
            .iter()
            .enumerate()
            .map(|(batch_index, batch)| {
                if let Some(resource) = batch.key.resource.clone() {
                    add_resource_binding(&mut resource_bindings, resource, None, Some(batch_index));
                }
                let cache_entry = cache
                    .batch_entries
                    .iter()
                    .find(|entry| entry.batch_index == batch_index);
                UiRenderVisualizerBatchGroup {
                    batch_index,
                    key: batch.key.clone(),
                    first_element: batch.range.first_element,
                    element_count: batch.range.element_count,
                    node_ids: batch.node_ids.clone(),
                    split_reason: batch.split_reason,
                    cache_status: cache_entry.map(|entry| entry.status),
                    cache_reason: cache_entry.map(|entry| entry.reason),
                }
            })
            .collect::<Vec<_>>();

        let overdraw_regions = overdraw_regions(elements);
        let text = UiRenderVisualizerTextStats::from_paint_elements(elements);
        let overlays = visualizer_overlays(elements, plan, &overdraw_regions);
        let stats = UiRenderVisualizerStats {
            paint_element_count: paint_elements.len(),
            batch_group_count: batch_groups.len(),
            draw_call_count: plan.stats.draw_call_count,
            overlay_count: overlays.len(),
            overdraw_region_count: overdraw_regions.len(),
            clipped_element_count: paint_elements
                .iter()
                .filter(|element| element.clip_frame.is_some())
                .count(),
            material_batch_count: batch_groups
                .iter()
                .filter(|batch| batch.key.shader == UiBatchShader::Material)
                .count(),
            resource_binding_count: resource_bindings.len(),
            text_element_count: text.text_element_count,
            sdf_text_element_count: text.sdf_text_count,
            cached_paint_count: cache.stats.reused_paint_count,
            rebuilt_paint_count: cache.stats.rebuilt_paint_count,
        };

        Self {
            paint_elements,
            batch_groups,
            overlays,
            overdraw_regions,
            resource_bindings,
            text,
            stats,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiRenderVisualizerPaintElement {
    pub paint_index: usize,
    pub node_id: UiNodeId,
    pub frame: UiFrame,
    pub clip_frame: Option<UiFrame>,
    pub z_index: i32,
    pub paint_order: u64,
    pub payload_kind: UiRenderVisualizerPaintPayloadKind,
    pub primitive: UiBatchPrimitive,
    pub shader: UiBatchShader,
    pub resource: Option<UiRenderResourceKey>,
    pub text_backend: Option<UiTextRenderMode>,
    pub opacity: f32,
    pub batch_index: Option<usize>,
    pub cache_status: Option<UiRenderCacheStatus>,
    pub debug_label: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiRenderVisualizerBatchGroup {
    pub batch_index: usize,
    pub key: UiBatchKey,
    pub first_element: usize,
    pub element_count: usize,
    pub node_ids: Vec<UiNodeId>,
    pub split_reason: UiBatchSplitReason,
    pub cache_status: Option<UiRenderCacheStatus>,
    pub cache_reason: Option<UiRenderCacheInvalidationReason>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiRenderVisualizerOverlay {
    pub kind: UiRenderVisualizerOverlayKind,
    pub frame: UiFrame,
    pub node_id: Option<UiNodeId>,
    pub paint_index: Option<usize>,
    pub batch_index: Option<usize>,
    pub label: Option<String>,
    pub color: Option<String>,
    pub intensity: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiRenderVisualizerOverlayKind {
    #[default]
    Wireframe,
    ClipScissor,
    BatchBounds,
    OverdrawHeat,
    TextGlyphBounds,
    TextBaseline,
    ResourceAtlas,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiRenderVisualizerOverdrawRegion {
    pub frame: UiFrame,
    pub paint_count: usize,
    pub node_ids: Vec<UiNodeId>,
    pub heat: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiRenderVisualizerResourceBinding {
    pub resource: UiRenderResourceKey,
    pub paint_indices: Vec<usize>,
    pub batch_indices: Vec<usize>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiRenderVisualizerPaintPayloadKind {
    #[default]
    Empty,
    Solid,
    Rounded,
    Border,
    Image,
    Box,
    Vector,
    Gradient,
    Material,
    Text,
}

impl UiRenderVisualizerPaintPayloadKind {
    fn from_payload(payload: &UiPaintPayload) -> Self {
        match payload {
            UiPaintPayload::Empty => Self::Empty,
            UiPaintPayload::Text { .. } => Self::Text,
            UiPaintPayload::Brush { brushes } => brushes
                .fill
                .as_ref()
                .or(brushes.border.as_ref())
                .map(Self::from_brush)
                .unwrap_or(Self::Empty),
        }
    }

    fn from_brush(brush: &UiBrushPayload) -> Self {
        match brush {
            UiBrushPayload::Solid(_) => Self::Solid,
            UiBrushPayload::Rounded(_) => Self::Rounded,
            UiBrushPayload::Border(_) => Self::Border,
            UiBrushPayload::Image(_) => Self::Image,
            UiBrushPayload::Box(_) => Self::Box,
            UiBrushPayload::Vector(_) => Self::Vector,
            UiBrushPayload::Gradient(_) => Self::Gradient,
            UiBrushPayload::Material(_) => Self::Material,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiRenderVisualizerStats {
    pub paint_element_count: usize,
    pub batch_group_count: usize,
    pub draw_call_count: usize,
    pub overlay_count: usize,
    pub overdraw_region_count: usize,
    pub clipped_element_count: usize,
    pub material_batch_count: usize,
    pub resource_binding_count: usize,
    pub text_element_count: usize,
    pub sdf_text_element_count: usize,
    pub cached_paint_count: usize,
    pub rebuilt_paint_count: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiRenderVisualizerTextStats {
    pub text_element_count: usize,
    pub auto_text_count: usize,
    pub native_text_count: usize,
    pub sdf_text_count: usize,
    pub shaped_line_count: usize,
    pub glyph_count: usize,
    pub decoration_count: usize,
    pub selection_count: usize,
    pub caret_count: usize,
    pub composition_count: usize,
}

impl UiRenderVisualizerTextStats {
    fn from_paint_elements(elements: &[UiPaintElement]) -> Self {
        let mut stats = Self::default();
        for element in elements {
            let UiPaintPayload::Text { text } = &element.payload else {
                continue;
            };
            stats.text_element_count += 1;
            match text.render_mode {
                UiTextRenderMode::Auto => stats.auto_text_count += 1,
                UiTextRenderMode::Native => stats.native_text_count += 1,
                UiTextRenderMode::Sdf => stats.sdf_text_count += 1,
            }
            stats.decoration_count += text.decorations.len();
            stats.selection_count += if text.selection.is_some() { 1 } else { 0 };
            stats.caret_count += if text.caret.is_some() { 1 } else { 0 };
            stats.composition_count += if text.composition.is_some() { 1 } else { 0 };
            if let Some(shaped) = &text.shaped {
                stats.shaped_line_count += shaped.lines.len();
                stats.glyph_count += shaped
                    .lines
                    .iter()
                    .map(|line| line.glyphs.len())
                    .sum::<usize>();
            }
        }
        stats
    }
}

fn visualizer_overlays(
    elements: &[UiPaintElement],
    plan: &UiBatchPlan,
    overdraw_regions: &[UiRenderVisualizerOverdrawRegion],
) -> Vec<UiRenderVisualizerOverlay> {
    let mut overlays = Vec::new();
    for (paint_index, element) in elements.iter().enumerate() {
        overlays.push(UiRenderVisualizerOverlay {
            kind: UiRenderVisualizerOverlayKind::Wireframe,
            frame: element.geometry.render_bounds,
            node_id: Some(element.node_id),
            paint_index: Some(paint_index),
            batch_index: batch_index_for_paint_index(plan, paint_index),
            label: element.debug_label.clone(),
            color: Some("#40c4ff".to_string()),
            intensity: 1.0,
        });

        if let Some(clip) = &element.clip {
            overlays.push(UiRenderVisualizerOverlay {
                kind: UiRenderVisualizerOverlayKind::ClipScissor,
                frame: clip.frame,
                node_id: Some(element.node_id),
                paint_index: Some(paint_index),
                batch_index: batch_index_for_paint_index(plan, paint_index),
                label: Some(format!("{:?}", clip.mode)),
                color: Some("#ffca28".to_string()),
                intensity: 1.0,
            });
        }

        if let UiPaintPayload::Text { text } = &element.payload {
            if let Some(shaped) = &text.shaped {
                for line in &shaped.lines {
                    overlays.push(UiRenderVisualizerOverlay {
                        kind: UiRenderVisualizerOverlayKind::TextBaseline,
                        frame: UiFrame::new(
                            line.frame.x,
                            line.frame.y + line.baseline,
                            line.measured_width,
                            1.0,
                        ),
                        node_id: Some(element.node_id),
                        paint_index: Some(paint_index),
                        batch_index: batch_index_for_paint_index(plan, paint_index),
                        label: Some(format!("{:?}", shaped.render_mode)),
                        color: Some("#ab47bc".to_string()),
                        intensity: 1.0,
                    });
                    for glyph in &line.glyphs {
                        overlays.push(UiRenderVisualizerOverlay {
                            kind: UiRenderVisualizerOverlayKind::TextGlyphBounds,
                            frame: glyph.visual_frame,
                            node_id: Some(element.node_id),
                            paint_index: Some(paint_index),
                            batch_index: batch_index_for_paint_index(plan, paint_index),
                            label: Some(glyph.glyph_id.to_string()),
                            color: Some("#7e57c2".to_string()),
                            intensity: 1.0,
                        });
                    }
                }
            }
        }
    }

    for (batch_index, batch) in plan.batches.iter().enumerate() {
        if let Some(frame) = batch_bounds(
            elements,
            batch.range.first_element,
            batch.range.element_count,
        ) {
            overlays.push(UiRenderVisualizerOverlay {
                kind: UiRenderVisualizerOverlayKind::BatchBounds,
                frame,
                node_id: None,
                paint_index: None,
                batch_index: Some(batch_index),
                label: Some(format!("{:?}", batch.split_reason)),
                color: Some("#66bb6a".to_string()),
                intensity: 1.0,
            });
        }
        if batch.key.resource.is_some() {
            if let Some(frame) = batch_bounds(
                elements,
                batch.range.first_element,
                batch.range.element_count,
            ) {
                overlays.push(UiRenderVisualizerOverlay {
                    kind: UiRenderVisualizerOverlayKind::ResourceAtlas,
                    frame,
                    node_id: None,
                    paint_index: None,
                    batch_index: Some(batch_index),
                    label: batch
                        .key
                        .resource
                        .as_ref()
                        .map(|resource| resource.id.clone()),
                    color: Some("#26a69a".to_string()),
                    intensity: 1.0,
                });
            }
        }
    }

    for region in overdraw_regions {
        overlays.push(UiRenderVisualizerOverlay {
            kind: UiRenderVisualizerOverlayKind::OverdrawHeat,
            frame: region.frame,
            node_id: None,
            paint_index: None,
            batch_index: None,
            label: Some(region.paint_count.to_string()),
            color: Some("#ef5350".to_string()),
            intensity: region.heat,
        });
    }

    overlays
}

fn overdraw_regions(elements: &[UiPaintElement]) -> Vec<UiRenderVisualizerOverdrawRegion> {
    let visible_elements = elements
        .iter()
        .enumerate()
        .filter_map(|(_, element)| {
            if element.effects.opacity <= 0.0 || element.payload == UiPaintPayload::Empty {
                return None;
            }
            let frame = visible_paint_frame(element)?;
            Some((element.node_id, frame))
        })
        .collect::<Vec<_>>();
    let mut regions: Vec<UiRenderVisualizerOverdrawRegion> = Vec::new();
    for left_index in 0..visible_elements.len() {
        for right_index in left_index + 1..visible_elements.len() {
            let (left_node_id, left_frame) = visible_elements[left_index];
            let (_, right_frame) = visible_elements[right_index];
            if let Some(frame) = left_frame.intersection(right_frame) {
                let mut node_ids = vec![left_node_id];
                for (node_id, candidate_frame) in &visible_elements {
                    if candidate_frame.intersection(frame).is_some() {
                        node_ids.push(*node_id);
                    }
                }
                node_ids.sort();
                node_ids.dedup();
                let paint_count = node_ids.len();
                if regions.iter().any(|region| region.frame == frame) {
                    continue;
                }
                regions.push(UiRenderVisualizerOverdrawRegion {
                    frame,
                    paint_count,
                    node_ids,
                    heat: paint_count as f32,
                });
            }
        }
    }
    regions
}

fn visible_paint_frame(element: &UiPaintElement) -> Option<UiFrame> {
    let frame = element.geometry.render_bounds;
    if frame.width <= 0.0 || frame.height <= 0.0 {
        return None;
    }
    if let Some(clip) = element.clip.as_ref() {
        return frame.intersection(clip.frame);
    }
    if let Some(clip) = element.geometry.clip_frame {
        return frame.intersection(clip);
    }
    Some(frame)
}

fn batch_bounds(
    elements: &[UiPaintElement],
    first_element: usize,
    element_count: usize,
) -> Option<UiFrame> {
    elements
        .get(first_element..first_element.saturating_add(element_count))?
        .iter()
        .map(|element| element.geometry.render_bounds)
        .reduce(union_frame)
}

fn union_frame(left: UiFrame, right: UiFrame) -> UiFrame {
    let x = left.x.min(right.x);
    let y = left.y.min(right.y);
    let right_edge = left.right().max(right.right());
    let bottom_edge = left.bottom().max(right.bottom());
    UiFrame::new(x, y, right_edge - x, bottom_edge - y)
}

fn batch_index_for_paint_index(plan: &UiBatchPlan, paint_index: usize) -> Option<usize> {
    plan.batches.iter().position(|batch| {
        paint_index >= batch.range.first_element
            && paint_index < batch.range.first_element + batch.range.element_count
    })
}

fn paint_resource_keys(element: &UiPaintElement) -> Vec<UiRenderResourceKey> {
    let mut resources = Vec::new();
    match &element.payload {
        UiPaintPayload::Brush { brushes } => {
            for brush in brushes.fill.iter().chain(brushes.border.iter()) {
                brush_resource_keys(brush, &mut resources);
            }
        }
        UiPaintPayload::Text { text } => {
            if let Some(shaped) = &text.shaped {
                push_unique_resource(&mut resources, shaped.font_key.clone());
                push_unique_resource(&mut resources, shaped.atlas_resource.clone());
                for line in &shaped.lines {
                    for glyph in &line.glyphs {
                        push_unique_resource(&mut resources, glyph.atlas_resource.clone());
                    }
                }
            }
        }
        UiPaintPayload::Empty => {}
    }
    resources
}

fn brush_resource_keys(brush: &UiBrushPayload, resources: &mut Vec<UiRenderResourceKey>) {
    match brush {
        UiBrushPayload::Image(payload) | UiBrushPayload::Box(payload) => {
            push_unique_resource(resources, Some(payload.resource.clone()));
        }
        UiBrushPayload::Vector(payload) => {
            push_unique_resource(resources, Some(payload.resource.clone()));
        }
        UiBrushPayload::Material(payload) => {
            push_unique_resource(resources, Some(payload.resource_key()));
        }
        UiBrushPayload::Solid(_)
        | UiBrushPayload::Rounded(_)
        | UiBrushPayload::Border(_)
        | UiBrushPayload::Gradient(_) => {}
    }
}

fn push_unique_resource(
    resources: &mut Vec<UiRenderResourceKey>,
    resource: Option<UiRenderResourceKey>,
) {
    if let Some(resource) = resource {
        if !resources.contains(&resource) {
            resources.push(resource);
        }
    }
}

fn add_resource_binding(
    bindings: &mut Vec<UiRenderVisualizerResourceBinding>,
    resource: UiRenderResourceKey,
    paint_index: Option<usize>,
    batch_index: Option<usize>,
) {
    if let Some(binding) = bindings
        .iter_mut()
        .find(|binding| binding.resource == resource)
    {
        if let Some(paint_index) = paint_index {
            push_unique_usize(&mut binding.paint_indices, paint_index);
        }
        if let Some(batch_index) = batch_index {
            push_unique_usize(&mut binding.batch_indices, batch_index);
        }
        return;
    }

    bindings.push(UiRenderVisualizerResourceBinding {
        resource,
        paint_indices: paint_index.into_iter().collect(),
        batch_indices: batch_index.into_iter().collect(),
    });
}

fn push_unique_usize(values: &mut Vec<usize>, value: usize) {
    if !values.contains(&value) {
        values.push(value);
    }
}
