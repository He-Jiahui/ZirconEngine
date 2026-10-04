use serde::{Deserialize, Serialize};

use super::packed_sort_key::{
    camera_order_key, domain_key, entity_tie_breaker_key, material_cluster_key, opaque_depth_key,
    order_in_layer_key, pipeline_cluster_key, queue_key, sorting_layer_key, transparent_depth_key,
    ui_z_index_key, y_sort_key,
};
use super::{
    RenderPhase, RenderPhaseSortComponents, RenderPhaseSortDecision, RenderPhaseSortDecisionField,
    RenderPhaseSortKey, RenderQueueValue,
};

/// Diagnostic view of the fields used to build a render phase sort key.
/// 仅供解释；实际次序仍以压缩键及实体次序为准，原始分量可能已截断。/// Diagnostic view of the fields used to build a render phase sort key.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderPhaseSortKeyBreakdown {
    pub phase: RenderPhase,
    pub phase_order: u8,
    pub camera_order: i32,
    pub camera_order_key: u8,
    pub queue: RenderQueueValue,
    pub queue_key: u16,
    pub sorting_layer: i32,
    pub sorting_layer_key: u8,
    pub order_in_layer: i32,
    pub order_in_layer_key: u16,
    pub y_sort: Option<f32>,
    pub y_sort_key: u16,
    pub ui_z_index: i32,
    pub ui_z_index_key: u32,
    pub depth: f32,
    pub depth_bias: f32,
    pub effective_depth: f32,
    pub depth_key: i64,
    pub ordered_depth_key: i64,
    pub opaque_depth_key: u16,
    pub transparent_depth_key: u32,
    pub pipeline_cluster_key: u16,
    pub material_cluster_key: u8,
    pub domain_key: u64,
    pub entity_tie_breaker: u64,
    pub tie_breaker_key: u16,
    pub raw_sort_key: u64,
}

impl RenderPhaseSortKeyBreakdown {
    pub fn from_components(phase: RenderPhase, components: RenderPhaseSortComponents) -> Self {
        let depth_key = components.depth_key();
        let ordered_depth_key = components.ordered_depth_key(phase);
        let raw_sort_key = RenderPhaseSortKey::for_components(phase, components).raw();

        Self {
            phase,
            phase_order: phase.queue_order(),
            camera_order: components.camera_order,
            camera_order_key: camera_order_key(components.camera_order) as u8,
            queue: components.queue,
            queue_key: queue_key(components.queue) as u16,
            sorting_layer: components.sorting_layer,
            sorting_layer_key: sorting_layer_key(components.sorting_layer) as u8,
            order_in_layer: components.order_in_layer,
            order_in_layer_key: order_in_layer_key(components.order_in_layer) as u16,
            y_sort: components.y_sort,
            y_sort_key: y_sort_key(components.y_sort) as u16,
            ui_z_index: components.ui_z_index,
            ui_z_index_key: ui_z_index_key(components.ui_z_index) as u32,
            depth: components.depth,
            depth_bias: components.depth_bias,
            effective_depth: components.effective_depth(),
            depth_key,
            ordered_depth_key,
            opaque_depth_key: opaque_depth_key(components.effective_depth()) as u16,
            transparent_depth_key: transparent_depth_key(components.effective_depth()) as u32,
            pipeline_cluster_key: 0,
            material_cluster_key: 0,
            domain_key: domain_key(phase, components, 0, 0),
            entity_tie_breaker: components.entity_tie_breaker,
            tie_breaker_key: entity_tie_breaker_key(components.entity_tie_breaker) as u16,
            raw_sort_key,
        }
    }

    pub fn from_components_with_clusters(
        phase: RenderPhase,
        components: RenderPhaseSortComponents,
        pipeline_variant: u32,
        material_discriminant: u16,
    ) -> Self {
        let mut breakdown = Self::from_components(phase, components);
        breakdown.pipeline_cluster_key = pipeline_cluster_key(pipeline_variant) as u16;
        breakdown.material_cluster_key = material_cluster_key(material_discriminant) as u8;
        breakdown.domain_key =
            domain_key(phase, components, pipeline_variant, material_discriminant);
        breakdown.raw_sort_key =
            super::packed_sort_key_u64(phase, components, pipeline_variant, material_discriminant);
        breakdown
    }

    /// Returns the first ordering lane that differs using the queue's comparison order.
    pub fn first_difference(self, other: Self) -> Option<RenderPhaseSortDecision> {
        [
            (
                RenderPhaseSortDecisionField::PhaseOrder,
                i64::from(self.phase_order),
                i64::from(other.phase_order),
                u64::from(self.phase_order),
                u64::from(other.phase_order),
            ),
            (
                RenderPhaseSortDecisionField::CameraOrder,
                i64::from(self.camera_order),
                i64::from(other.camera_order),
                u64::from(self.camera_order_key),
                u64::from(other.camera_order_key),
            ),
            (
                RenderPhaseSortDecisionField::Queue,
                i64::from(self.queue.raw()),
                i64::from(other.queue.raw()),
                u64::from(self.queue_key),
                u64::from(other.queue_key),
            ),
            (
                RenderPhaseSortDecisionField::Domain,
                u64_to_i64(self.domain_key),
                u64_to_i64(other.domain_key),
                self.domain_key,
                other.domain_key,
            ),
            (
                RenderPhaseSortDecisionField::TieBreakerKey,
                i64::from(self.tie_breaker_key),
                i64::from(other.tie_breaker_key),
                u64::from(self.tie_breaker_key),
                u64::from(other.tie_breaker_key),
            ),
            (
                RenderPhaseSortDecisionField::EntityTieBreaker,
                u64_to_i64(self.entity_tie_breaker),
                u64_to_i64(other.entity_tie_breaker),
                self.entity_tie_breaker,
                other.entity_tie_breaker,
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

fn u64_to_i64(value: u64) -> i64 {
    value.min(i64::MAX as u64) as i64
}

#[cfg(test)]
#[path = "tests/phase_sort_key_breakdown.rs"]
mod tests;
