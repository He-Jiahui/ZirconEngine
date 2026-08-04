use serde::{Deserialize, Serialize};

use super::{
    RenderPhase, RenderPhaseQueueOrderingKey, RenderPhaseQueueSummary, RENDER_PHASES_BY_QUEUE_ORDER,
};

/// Read-only reporting view that keeps mesh and sprite queue summaries side by side.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderFramePhaseQueueSummary {
    pub geometry: RenderPhaseQueueSummary,
    pub sprites: RenderPhaseQueueSummary,
    pub total_item_count: usize,
    pub geometry_first_ordering_key: Option<RenderPhaseQueueOrderingKey>,
    pub geometry_last_ordering_key: Option<RenderPhaseQueueOrderingKey>,
    pub sprite_first_ordering_key: Option<RenderPhaseQueueOrderingKey>,
    pub sprite_last_ordering_key: Option<RenderPhaseQueueOrderingKey>,
    pub phase_counts: Vec<RenderFramePhaseQueueSummaryPhaseCount>,
    pub phase_order_spans: Vec<RenderFramePhaseQueueSummaryPhaseOrderSpan>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RenderFramePhaseQueueSummaryPhaseCount {
    pub phase: RenderPhase,
    pub diagnostic_name: String,
    pub phase_order: u8,
    pub geometry_item_count: usize,
    pub sprite_item_count: usize,
    pub total_item_count: usize,
}

impl RenderFramePhaseQueueSummaryPhaseCount {
    pub fn diagnostic_name(&self) -> &str {
        &self.diagnostic_name
    }
}

/// Aggregates mesh and sprite counts for phases that share one queue-order bucket.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RenderFramePhaseQueueSummaryPhaseOrderSpan {
    pub phase_order: u8,
    pub diagnostic_name: String,
    pub phases: Vec<RenderPhase>,
    pub geometry_item_count: usize,
    pub sprite_item_count: usize,
    pub total_item_count: usize,
    pub geometry_start_index: Option<usize>,
    pub geometry_end_index_exclusive: Option<usize>,
    pub geometry_first_ordering_key: Option<RenderPhaseQueueOrderingKey>,
    pub geometry_last_ordering_key: Option<RenderPhaseQueueOrderingKey>,
    pub sprite_start_index: Option<usize>,
    pub sprite_end_index_exclusive: Option<usize>,
    pub sprite_first_ordering_key: Option<RenderPhaseQueueOrderingKey>,
    pub sprite_last_ordering_key: Option<RenderPhaseQueueOrderingKey>,
}

impl RenderFramePhaseQueueSummaryPhaseOrderSpan {
    pub fn diagnostic_name(&self) -> &str {
        &self.diagnostic_name
    }
}

impl RenderFramePhaseQueueSummary {
    pub fn new(geometry: RenderPhaseQueueSummary, sprites: RenderPhaseQueueSummary) -> Self {
        let phase_counts = RENDER_PHASES_BY_QUEUE_ORDER
            .iter()
            .map(|phase| {
                let geometry_item_count = geometry.count_for_phase(*phase);
                let sprite_item_count = sprites.count_for_phase(*phase);
                RenderFramePhaseQueueSummaryPhaseCount {
                    phase: *phase,
                    diagnostic_name: phase.diagnostic_name().to_string(),
                    phase_order: phase.queue_order(),
                    geometry_item_count,
                    sprite_item_count,
                    total_item_count: geometry_item_count + sprite_item_count,
                }
            })
            .collect::<Vec<_>>();
        let mut phase_order_spans = Vec::<RenderFramePhaseQueueSummaryPhaseOrderSpan>::new();
        for phase in RENDER_PHASES_BY_QUEUE_ORDER {
            let phase_order = phase.queue_order();
            if let Some(span) = phase_order_spans
                .iter_mut()
                .find(|span| span.phase_order == phase_order)
            {
                span.phases.push(phase);
                span.diagnostic_name = phase_diagnostic_name(&span.phases);
            } else {
                let geometry_span = geometry.span_for_phase_order(phase_order);
                let sprite_span = sprites.span_for_phase_order(phase_order);
                let geometry_item_count = geometry_span
                    .map(|span| span.item_count)
                    .unwrap_or_default();
                let sprite_item_count = sprite_span.map(|span| span.item_count).unwrap_or_default();
                phase_order_spans.push(RenderFramePhaseQueueSummaryPhaseOrderSpan {
                    phase_order,
                    diagnostic_name: phase_diagnostic_name(&[phase]),
                    phases: vec![phase],
                    geometry_item_count,
                    sprite_item_count,
                    total_item_count: geometry_item_count + sprite_item_count,
                    geometry_start_index: geometry_span.and_then(|span| span.start_index),
                    geometry_end_index_exclusive: geometry_span
                        .and_then(|span| span.end_index_exclusive),
                    geometry_first_ordering_key: geometry_span
                        .and_then(|span| span.first_ordering_key),
                    geometry_last_ordering_key: geometry_span
                        .and_then(|span| span.last_ordering_key),
                    sprite_start_index: sprite_span.and_then(|span| span.start_index),
                    sprite_end_index_exclusive: sprite_span
                        .and_then(|span| span.end_index_exclusive),
                    sprite_first_ordering_key: sprite_span.and_then(|span| span.first_ordering_key),
                    sprite_last_ordering_key: sprite_span.and_then(|span| span.last_ordering_key),
                });
            }
        }
        Self {
            total_item_count: geometry.item_count + sprites.item_count,
            geometry_first_ordering_key: geometry.first_ordering_key,
            geometry_last_ordering_key: geometry.last_ordering_key,
            sprite_first_ordering_key: sprites.first_ordering_key,
            sprite_last_ordering_key: sprites.last_ordering_key,
            geometry,
            sprites,
            phase_counts,
            phase_order_spans,
        }
    }

    pub fn count_for_phase(&self, phase: RenderPhase) -> usize {
        self.phase_count_row_for_phase(phase)
            .map(|count| count.total_item_count)
            .unwrap_or(0)
    }

    pub fn count_for_phase_order(&self, phase_order: u8) -> usize {
        self.phase_order_span_for_phase_order(phase_order)
            .map(|span| span.total_item_count)
            .unwrap_or(0)
    }

    pub fn phase_count_row_for_phase(
        &self,
        phase: RenderPhase,
    ) -> Option<&RenderFramePhaseQueueSummaryPhaseCount> {
        self.phase_counts.iter().find(|count| count.phase == phase)
    }

    pub fn active_phase_counts(
        &self,
    ) -> impl Iterator<Item = &RenderFramePhaseQueueSummaryPhaseCount> {
        self.phase_counts
            .iter()
            .filter(|count| count.total_item_count > 0)
    }

    pub fn phase_order_span_for_phase_order(
        &self,
        phase_order: u8,
    ) -> Option<&RenderFramePhaseQueueSummaryPhaseOrderSpan> {
        self.phase_order_spans
            .iter()
            .find(|span| span.phase_order == phase_order)
    }

    pub fn phase_order_span_for_phase(
        &self,
        phase: RenderPhase,
    ) -> Option<&RenderFramePhaseQueueSummaryPhaseOrderSpan> {
        self.phase_order_span_for_phase_order(phase.queue_order())
    }

    pub fn active_phase_order_spans(
        &self,
    ) -> impl Iterator<Item = &RenderFramePhaseQueueSummaryPhaseOrderSpan> {
        self.phase_order_spans
            .iter()
            .filter(|span| span.total_item_count > 0)
    }

    pub fn phase_order_span_for_geometry_queue_index(
        &self,
        queue_index: usize,
    ) -> Option<&RenderFramePhaseQueueSummaryPhaseOrderSpan> {
        self.phase_order_spans.iter().find(|span| {
            span.geometry_start_index
                .zip(span.geometry_end_index_exclusive)
                .map_or(false, |(start, end)| (start..end).contains(&queue_index))
        })
    }

    pub fn phase_order_span_for_sprite_queue_index(
        &self,
        queue_index: usize,
    ) -> Option<&RenderFramePhaseQueueSummaryPhaseOrderSpan> {
        self.phase_order_spans.iter().find(|span| {
            span.sprite_start_index
                .zip(span.sprite_end_index_exclusive)
                .map_or(false, |(start, end)| (start..end).contains(&queue_index))
        })
    }
}

fn phase_diagnostic_name(phases: &[RenderPhase]) -> String {
    let capacity = phases
        .iter()
        .map(|phase| phase.diagnostic_name().len())
        .sum::<usize>()
        .saturating_add(phases.len().saturating_sub(1));
    let mut diagnostic_name = String::with_capacity(capacity);
    for (index, phase) in phases.iter().enumerate() {
        if index > 0 {
            diagnostic_name.push('+');
        }
        diagnostic_name.push_str(phase.diagnostic_name());
    }
    diagnostic_name
}

#[cfg(test)]
mod tests {
    #[test]
    fn frame_summary_builds_diagnostic_names_without_temporary_vec() {
        let source = include_str!("frame_phase_queue_summary.rs");

        assert!(!source.contains(concat!(".collect::<Vec<_>>()", ".join(\"+\")")));
        assert!(source.contains(concat!("String::with_", "capacity(capacity)")));
    }
}
