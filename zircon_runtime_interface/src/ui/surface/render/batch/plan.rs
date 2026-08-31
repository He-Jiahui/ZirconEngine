use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;

use super::{clip::UiBatchClipStates, UiBatchKey, UiBatchRange, UiBatchSplitReason, UiBatchStats};
use crate::ui::surface::UiPaintElement;

/// Ordered draw-call plan derived from paint elements.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiBatchPlan {
    pub batches: Vec<UiBatch>,
    /// Source indices in compositing order. They preserve the link from an
    /// ordered batch back to the extraction input used by caches and tooling.
    pub ordered_element_indices: Vec<usize>,
    pub stats: UiBatchStats,
    clip_states: UiBatchClipStates,
}

impl UiBatchPlan {
    pub fn from_paint_elements(elements: &[UiPaintElement]) -> Self {
        let ordered_element_indices = ordered_element_indices(elements);

        let mut clip_states = UiBatchClipStates::default();
        let mut batches = Vec::new();
        let mut active_key: Option<UiBatchKey> = None;
        let mut active_layer = 0_i32;
        let mut active_start = 0_usize;
        let mut active_source_indices = Vec::new();
        let mut active_node_ids = Vec::new();
        let mut active_split_reason = UiBatchSplitReason::FirstBatch;

        for (ordered_index, &source_index) in ordered_element_indices.iter().enumerate() {
            let element = &elements[source_index];
            let key = UiBatchKey::from_paint_element_with_clip_states(element, &mut clip_states);
            if let Some(current_key) = active_key.as_ref() {
                if active_layer == element.z_index && current_key == &key {
                    active_source_indices.push(source_index);
                    active_node_ids.push(element.node_id);
                    continue;
                }

                batches.push(UiBatch {
                    layer: active_layer,
                    key: current_key.clone(),
                    range: UiBatchRange {
                        first_element: active_start,
                        element_count: ordered_index - active_start,
                    },
                    source_indices: std::mem::take(&mut active_source_indices),
                    node_ids: std::mem::take(&mut active_node_ids),
                    split_reason: active_split_reason,
                });
                active_split_reason = if active_layer != element.z_index {
                    UiBatchSplitReason::LayerChanged
                } else {
                    UiBatchSplitReason::between(current_key, &key)
                };
                active_layer = element.z_index;
                active_start = ordered_index;
                active_key = Some(key);
                active_source_indices.push(source_index);
                active_node_ids.push(element.node_id);
            } else {
                active_layer = element.z_index;
                active_key = Some(key);
                active_source_indices.push(source_index);
                active_node_ids.push(element.node_id);
            }
        }

        if let Some(key) = active_key {
            batches.push(UiBatch {
                layer: active_layer,
                key,
                range: UiBatchRange {
                    first_element: active_start,
                    element_count: ordered_element_indices.len() - active_start,
                },
                source_indices: active_source_indices,
                node_ids: active_node_ids,
                split_reason: active_split_reason,
            });
        }

        let stats = UiBatchStats {
            element_count: elements.len(),
            batch_count: batches.len(),
            draw_call_count: batches.len(),
        };
        Self {
            batches,
            ordered_element_indices,
            stats,
            clip_states,
        }
    }

    #[cfg(test)]
    pub(super) fn clip_states(&self) -> &UiBatchClipStates {
        &self.clip_states
    }
}

fn ordered_element_indices(elements: &[UiPaintElement]) -> Vec<usize> {
    let mut indices = (0..elements.len()).collect::<Vec<_>>();
    if elements.windows(2).all(|pair| {
        (pair[0].z_index, pair[0].paint_order) <= (pair[1].z_index, pair[1].paint_order)
    }) {
        return indices;
    }

    indices.sort_by_key(|&index| {
        let element = &elements[index];
        (element.z_index, element.paint_order, index)
    });
    indices
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiBatch {
    /// Layer controls ordering only; it is deliberately excluded from `key`.
    pub layer: i32,
    pub key: UiBatchKey,
    /// Range into `UiBatchPlan::ordered_element_indices`.
    pub range: UiBatchRange,
    /// Exact extraction indices for cache and debug consumers.
    pub source_indices: Vec<usize>,
    pub node_ids: Vec<UiNodeId>,
    pub split_reason: UiBatchSplitReason,
}
