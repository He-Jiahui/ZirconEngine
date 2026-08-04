use serde::{Deserialize, Serialize};

use super::{
    RenderPhase, RenderPhaseItem, RenderPhaseQueueOrderingKey, RENDER_PHASES_BY_QUEUE_ORDER,
};

/// Diagnostics snapshot for an already-sorted render phase queue.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderPhaseQueueSummary {
    pub item_count: usize,
    pub phase_counts: Vec<RenderPhaseQueueSummaryPhaseCount>,
    pub phase_order_spans: Vec<RenderPhaseQueueSummaryPhaseOrderSpan>,
    pub first_ordering_key: Option<RenderPhaseQueueOrderingKey>,
    pub last_ordering_key: Option<RenderPhaseQueueOrderingKey>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RenderPhaseQueueSummaryPhaseCount {
    pub phase: RenderPhase,
    pub diagnostic_name: String,
    pub phase_order: u8,
    pub item_count: usize,
}

impl RenderPhaseQueueSummaryPhaseCount {
    pub fn new(phase: RenderPhase) -> Self {
        Self {
            phase,
            diagnostic_name: phase.diagnostic_name().to_string(),
            phase_order: phase.queue_order(),
            item_count: 0,
        }
    }

    pub fn diagnostic_name(&self) -> &str {
        &self.diagnostic_name
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RenderPhaseQueueSummaryPhaseOrderSpan {
    pub phase_order: u8,
    pub diagnostic_name: String,
    pub phases: Vec<RenderPhase>,
    pub item_count: usize,
    pub start_index: Option<usize>,
    pub end_index_exclusive: Option<usize>,
    pub first_ordering_key: Option<RenderPhaseQueueOrderingKey>,
    pub last_ordering_key: Option<RenderPhaseQueueOrderingKey>,
}

impl RenderPhaseQueueSummaryPhaseOrderSpan {
    pub fn new(phase_order: u8, phases: Vec<RenderPhase>) -> Self {
        let diagnostic_name = phase_diagnostic_name(&phases);
        Self {
            phase_order,
            diagnostic_name,
            phases,
            item_count: 0,
            start_index: None,
            end_index_exclusive: None,
            first_ordering_key: None,
            last_ordering_key: None,
        }
    }

    fn push_item(&mut self, index: usize, ordering_key: RenderPhaseQueueOrderingKey) {
        self.item_count += 1;
        if self.start_index.is_none() {
            self.start_index = Some(index);
            self.first_ordering_key = Some(ordering_key);
        }
        self.end_index_exclusive = Some(index + 1);
        self.last_ordering_key = Some(ordering_key);
    }

    pub fn diagnostic_name(&self) -> &str {
        &self.diagnostic_name
    }
}

impl RenderPhaseQueueSummary {
    pub(super) fn from_sorted_items(items: &[RenderPhaseItem]) -> Self {
        let mut phase_counts = RENDER_PHASES_BY_QUEUE_ORDER
            .iter()
            .map(|phase| RenderPhaseQueueSummaryPhaseCount::new(*phase))
            .collect::<Vec<_>>();
        let mut phase_order_spans = phase_order_spans();

        for (index, item) in items.iter().enumerate() {
            phase_counts[phase_count_index(item.phase)].item_count += 1;

            if let Some(span) = phase_order_spans.get_mut(usize::from(item.phase.queue_order())) {
                span.push_item(index, item.ordering_key());
            }
        }

        Self {
            item_count: items.len(),
            phase_counts,
            phase_order_spans,
            first_ordering_key: items.first().map(RenderPhaseItem::ordering_key),
            last_ordering_key: items.last().map(RenderPhaseItem::ordering_key),
        }
    }

    pub fn count_for_phase(&self, phase: RenderPhase) -> usize {
        self.phase_count_row_for_phase(phase)
            .map(|count| count.item_count)
            .unwrap_or(0)
    }

    pub fn phase_count_row_for_phase(
        &self,
        phase: RenderPhase,
    ) -> Option<&RenderPhaseQueueSummaryPhaseCount> {
        self.phase_counts.iter().find(|count| count.phase == phase)
    }

    pub fn active_phase_counts(&self) -> impl Iterator<Item = &RenderPhaseQueueSummaryPhaseCount> {
        self.phase_counts
            .iter()
            .filter(|count| count.item_count > 0)
    }

    pub fn count_for_phase_order(&self, phase_order: u8) -> usize {
        self.phase_order_spans
            .iter()
            .find_map(|span| (span.phase_order == phase_order).then_some(span.item_count))
            .unwrap_or(0)
    }

    pub fn span_for_phase_order(
        &self,
        phase_order: u8,
    ) -> Option<&RenderPhaseQueueSummaryPhaseOrderSpan> {
        self.phase_order_spans
            .iter()
            .find(|span| span.phase_order == phase_order)
    }

    pub fn span_for_phase(
        &self,
        phase: RenderPhase,
    ) -> Option<&RenderPhaseQueueSummaryPhaseOrderSpan> {
        self.span_for_phase_order(phase.queue_order())
    }

    pub fn active_phase_order_spans(
        &self,
    ) -> impl Iterator<Item = &RenderPhaseQueueSummaryPhaseOrderSpan> {
        self.phase_order_spans
            .iter()
            .filter(|span| span.item_count > 0)
    }

    pub fn span_for_queue_index(
        &self,
        queue_index: usize,
    ) -> Option<&RenderPhaseQueueSummaryPhaseOrderSpan> {
        self.phase_order_spans.iter().find(|span| {
            span.start_index
                .zip(span.end_index_exclusive)
                .map_or(false, |(start, end)| (start..end).contains(&queue_index))
        })
    }
}

const fn phase_count_index(phase: RenderPhase) -> usize {
    match phase {
        RenderPhase::Prepass => 0,
        RenderPhase::Shadow => 1,
        RenderPhase::Opaque2d => 2,
        RenderPhase::Opaque3d => 3,
        RenderPhase::AlphaMask2d => 4,
        RenderPhase::AlphaMask3d => 5,
        RenderPhase::Deferred => 6,
        RenderPhase::Transparent2d => 7,
        RenderPhase::Transparent3d => 8,
        RenderPhase::PostProcess => 9,
        RenderPhase::Ui => 10,
        RenderPhase::Overlay => 11,
        RenderPhase::Debug => 12,
    }
}

fn phase_order_spans() -> Vec<RenderPhaseQueueSummaryPhaseOrderSpan> {
    let mut spans: Vec<RenderPhaseQueueSummaryPhaseOrderSpan> = Vec::new();
    for phase in RENDER_PHASES_BY_QUEUE_ORDER {
        let phase_order = phase.queue_order();
        if let Some(index) = spans
            .iter()
            .position(|span| span.phase_order == phase_order)
        {
            spans[index].phases.push(phase);
            spans[index].diagnostic_name = phase_diagnostic_name(&spans[index].phases);
        } else {
            spans.push(RenderPhaseQueueSummaryPhaseOrderSpan::new(
                phase_order,
                vec![phase],
            ));
        }
    }
    spans
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
    fn summary_updates_phase_and_order_rows_without_per_item_linear_search() {
        let source = include_str!("phase_queue_summary.rs");

        assert!(source.contains(concat!("phase_count_index", "(item.phase)")));
        assert!(source.contains(concat!("usize::from(item.phase.", "queue_order())")));
        assert!(!source.contains(concat!(".find(|count| count.phase", " == item.phase)")));
        assert!(!source.contains(concat!(
            ".find(|span| span.phase_order",
            " == item.phase.queue_order())"
        )));
        assert!(!source.contains(concat!(".collect::<Vec<_>>()", ".join(\"+\")")));
        assert!(source.contains(concat!("String::with_", "capacity(capacity)")));
    }
}
