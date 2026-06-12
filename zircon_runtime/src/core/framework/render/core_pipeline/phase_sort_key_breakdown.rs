use serde::{Deserialize, Serialize};

use super::phase_sort::{
    entity_tie_breaker_sort_key, material_queue_sort_key, order_in_layer_sort_key,
    ordered_depth_sort_key, render_queue_sort_key, ui_z_index_sort_key,
};
use super::{
    RenderPhase, RenderPhaseSortComponents, RenderPhaseSortDecision, RenderPhaseSortDecisionField,
    RenderPhaseSortKey,
};

/// Diagnostic view of the fields used to build a render phase sort key.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderPhaseSortKeyBreakdown {
    pub phase: RenderPhase,
    pub phase_order: u8,
    pub render_queue: i32,
    pub render_queue_sort_key: i128,
    pub material_queue: i32,
    pub material_queue_sort_key: i128,
    pub order_in_layer: i32,
    pub order_in_layer_sort_key: i128,
    pub ui_z_index: i32,
    pub ui_z_index_sort_key: i128,
    pub depth: f32,
    pub depth_bias: f32,
    pub effective_depth: f32,
    pub depth_key: i64,
    pub ordered_depth_key: i64,
    pub ordered_depth_sort_key: i128,
    pub transparent_back_to_front: bool,
    pub entity_tie_breaker: u64,
    pub entity_tie_breaker_key: u16,
    pub entity_tie_breaker_sort_key: i128,
    pub raw_sort_key: i128,
}

impl RenderPhaseSortKeyBreakdown {
    pub fn from_components(phase: RenderPhase, components: RenderPhaseSortComponents) -> Self {
        let ordered_depth_key = components.ordered_depth_key(phase);
        let entity_tie_breaker_key = (components.entity_tie_breaker & 0xffff) as u16;

        Self {
            phase,
            phase_order: phase.queue_order(),
            render_queue: components.render_queue,
            render_queue_sort_key: render_queue_sort_key(components.render_queue),
            material_queue: components.material_queue,
            material_queue_sort_key: material_queue_sort_key(components.material_queue),
            order_in_layer: components.order_in_layer,
            order_in_layer_sort_key: order_in_layer_sort_key(components.order_in_layer),
            ui_z_index: components.ui_z_index,
            ui_z_index_sort_key: ui_z_index_sort_key(components.ui_z_index),
            depth: components.depth,
            depth_bias: components.depth_bias,
            effective_depth: components.effective_depth(),
            depth_key: components.depth_key(),
            ordered_depth_key,
            ordered_depth_sort_key: ordered_depth_sort_key(ordered_depth_key),
            transparent_back_to_front: phase.is_transparent(),
            entity_tie_breaker: components.entity_tie_breaker,
            entity_tie_breaker_key,
            entity_tie_breaker_sort_key: entity_tie_breaker_sort_key(components.entity_tie_breaker),
            raw_sort_key: RenderPhaseSortKey::for_components(phase, components).raw(),
        }
    }

    /// Returns the first ordering lane that differs using the queue's comparison order.
    pub fn first_difference(self, other: Self) -> Option<RenderPhaseSortDecision> {
        [
            (
                RenderPhaseSortDecisionField::PhaseOrder,
                i128::from(self.phase_order),
                i128::from(other.phase_order),
                i128::from(self.phase_order),
                i128::from(other.phase_order),
            ),
            (
                RenderPhaseSortDecisionField::RenderQueue,
                i128::from(self.render_queue),
                i128::from(other.render_queue),
                self.render_queue_sort_key,
                other.render_queue_sort_key,
            ),
            (
                RenderPhaseSortDecisionField::MaterialQueue,
                i128::from(self.material_queue),
                i128::from(other.material_queue),
                self.material_queue_sort_key,
                other.material_queue_sort_key,
            ),
            (
                RenderPhaseSortDecisionField::OrderInLayer,
                i128::from(self.order_in_layer),
                i128::from(other.order_in_layer),
                self.order_in_layer_sort_key,
                other.order_in_layer_sort_key,
            ),
            (
                RenderPhaseSortDecisionField::UiZIndex,
                i128::from(self.ui_z_index),
                i128::from(other.ui_z_index),
                self.ui_z_index_sort_key,
                other.ui_z_index_sort_key,
            ),
            (
                RenderPhaseSortDecisionField::OrderedDepthKey,
                i128::from(self.ordered_depth_key),
                i128::from(other.ordered_depth_key),
                self.ordered_depth_sort_key,
                other.ordered_depth_sort_key,
            ),
            (
                RenderPhaseSortDecisionField::EntityTieBreakerKey,
                i128::from(self.entity_tie_breaker_key),
                i128::from(other.entity_tie_breaker_key),
                self.entity_tie_breaker_sort_key,
                other.entity_tie_breaker_sort_key,
            ),
            (
                RenderPhaseSortDecisionField::EntityTieBreaker,
                i128::from(self.entity_tie_breaker),
                i128::from(other.entity_tie_breaker),
                i128::from(self.entity_tie_breaker),
                i128::from(other.entity_tie_breaker),
            ),
        ]
        .into_iter()
        .find_map(
            |(field, left_value, right_value, left_order_value, right_order_value)| {
                (left_order_value != right_order_value).then(|| {
                    RenderPhaseSortDecision::from_order_values(
                        field,
                        left_value,
                        right_value,
                        left_order_value,
                        right_order_value,
                    )
                })
            },
        )
    }
}

impl RenderPhaseSortKey {
    pub fn breakdown(
        phase: RenderPhase,
        components: RenderPhaseSortComponents,
    ) -> RenderPhaseSortKeyBreakdown {
        RenderPhaseSortKeyBreakdown::from_components(phase, components)
    }
}
