use serde::{Deserialize, Serialize};

use crate::ui::event_ui::{UiNodeId, UiTreeId};
use crate::ui::layout::UiFrame;

use super::{
    UiBatchKey, UiBatchPlan, UiBatchPrimitive, UiBatchShader, UiPaintElement, UiPaintPayload,
    UiRenderExtract, UiRenderResourceKey, UiTextRenderMode,
};

/// Canonical renderer-facing rows that runtime and editor can compare without
/// reading backend-private draw commands or painter-specific fallback state.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiRendererParitySnapshot {
    pub tree_id: UiTreeId,
    pub paint_order: Vec<UiRendererParityPaintRow>,
    pub batch_order: Vec<UiRendererParityBatchRow>,
    pub stats: UiRendererParityStats,
}

impl UiRendererParitySnapshot {
    pub fn from_render_extract(extract: &UiRenderExtract) -> Self {
        let elements = extract.list.to_paint_elements();
        let plan = UiBatchPlan::from_paint_elements(&elements);
        Self::from_paint_elements_batches(extract.tree_id.clone(), &elements, &plan)
    }

    pub fn from_paint_elements_batches(
        tree_id: UiTreeId,
        elements: &[UiPaintElement],
        plan: &UiBatchPlan,
    ) -> Self {
        let paint_order = elements
            .iter()
            .enumerate()
            .map(|(paint_index, element)| {
                let batch_index = batch_index_for_paint_index(plan, paint_index);
                let batch_key = UiBatchKey::from_paint_element(element);
                UiRendererParityPaintRow {
                    paint_index,
                    node_id: element.node_id,
                    paint_order: element.paint_order,
                    frame: element.geometry.render_bounds,
                    clip_frame: element.clip.as_ref().map(|clip| clip.frame),
                    clip_key: batch_key.clip.clone(),
                    payload_kind: UiRendererParityPayloadKind::from_payload(&element.payload),
                    batch_key,
                    batch_index,
                    resource: paint_resource_key(element),
                    text_render_mode: paint_text_render_mode(element),
                    opacity: element.effects.opacity,
                    debug_label: element.debug_label.clone(),
                }
            })
            .collect::<Vec<_>>();

        let batch_order = plan
            .batches
            .iter()
            .enumerate()
            .map(|(batch_index, batch)| UiRendererParityBatchRow {
                batch_index,
                first_paint_index: batch.range.first_element,
                paint_count: batch.range.element_count,
                node_ids: batch.node_ids.clone(),
                batch_key: batch.key.clone(),
                primitive: batch.key.primitive,
                shader: batch.key.shader,
                resource: batch.key.resource.clone(),
            })
            .collect::<Vec<_>>();

        let stats = UiRendererParityStats {
            paint_element_count: paint_order.len(),
            batch_count: batch_order.len(),
            clipped_paint_count: paint_order
                .iter()
                .filter(|row| row.clip_frame.is_some())
                .count(),
            resource_bound_paint_count: paint_order
                .iter()
                .filter(|row| row.resource.is_some())
                .count(),
            text_paint_count: paint_order
                .iter()
                .filter(|row| row.payload_kind == UiRendererParityPayloadKind::Text)
                .count(),
        };

        Self {
            tree_id,
            paint_order,
            batch_order,
            stats,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiRendererParityPaintRow {
    pub paint_index: usize,
    pub node_id: UiNodeId,
    pub paint_order: u64,
    pub frame: UiFrame,
    pub clip_frame: Option<UiFrame>,
    pub clip_key: Option<String>,
    pub payload_kind: UiRendererParityPayloadKind,
    pub batch_key: UiBatchKey,
    pub batch_index: Option<usize>,
    pub resource: Option<UiRenderResourceKey>,
    pub text_render_mode: Option<UiTextRenderMode>,
    pub opacity: f32,
    pub debug_label: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiRendererParityBatchRow {
    pub batch_index: usize,
    pub first_paint_index: usize,
    pub paint_count: usize,
    pub node_ids: Vec<UiNodeId>,
    pub batch_key: UiBatchKey,
    pub primitive: UiBatchPrimitive,
    pub shader: UiBatchShader,
    pub resource: Option<UiRenderResourceKey>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiRendererParityPayloadKind {
    #[default]
    Empty,
    Brush,
    Text,
}

impl UiRendererParityPayloadKind {
    fn from_payload(payload: &UiPaintPayload) -> Self {
        match payload {
            UiPaintPayload::Empty => Self::Empty,
            UiPaintPayload::Brush { .. } => Self::Brush,
            UiPaintPayload::Text { .. } => Self::Text,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiRendererParityStats {
    pub paint_element_count: usize,
    pub batch_count: usize,
    pub clipped_paint_count: usize,
    pub resource_bound_paint_count: usize,
    pub text_paint_count: usize,
}

fn batch_index_for_paint_index(plan: &UiBatchPlan, paint_index: usize) -> Option<usize> {
    plan.batches.iter().position(|batch| {
        paint_index >= batch.range.first_element
            && paint_index < batch.range.first_element + batch.range.element_count
    })
}

fn paint_resource_key(element: &UiPaintElement) -> Option<UiRenderResourceKey> {
    UiBatchKey::from_paint_element(element).resource
}

fn paint_text_render_mode(element: &UiPaintElement) -> Option<UiTextRenderMode> {
    match &element.payload {
        UiPaintPayload::Text { text } => Some(text.render_mode),
        UiPaintPayload::Empty | UiPaintPayload::Brush { .. } => None,
    }
}
