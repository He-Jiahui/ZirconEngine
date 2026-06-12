use serde::{Deserialize, Serialize};

use super::RenderPhase;

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct RenderPhaseSortKey(i128);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderPhaseSortComponents {
    pub render_queue: i32,
    pub material_queue: i32,
    pub depth: f32,
    pub depth_bias: f32,
    pub order_in_layer: i32,
    pub ui_z_index: i32,
    pub entity_tie_breaker: u64,
}

impl RenderPhaseSortComponents {
    pub const fn new(depth: f32, entity_tie_breaker: u64) -> Self {
        Self {
            render_queue: 0,
            material_queue: 0,
            depth,
            depth_bias: 0.0,
            order_in_layer: 0,
            ui_z_index: 0,
            entity_tie_breaker,
        }
    }

    pub const fn with_render_queue(mut self, render_queue: i32) -> Self {
        self.render_queue = render_queue;
        self
    }

    pub const fn with_material_queue(mut self, material_queue: i32) -> Self {
        self.material_queue = material_queue;
        self
    }

    pub const fn with_depth_bias(mut self, depth_bias: f32) -> Self {
        self.depth_bias = depth_bias;
        self
    }

    pub const fn with_order_in_layer(mut self, order_in_layer: i32) -> Self {
        self.order_in_layer = order_in_layer;
        self
    }

    pub const fn with_ui_z_index(mut self, ui_z_index: i32) -> Self {
        self.ui_z_index = ui_z_index;
        self
    }

    pub fn effective_depth(self) -> f32 {
        self.depth + self.depth_bias
    }

    pub fn depth_key(self) -> i64 {
        depth_sort_key(self.effective_depth())
    }

    pub fn ordered_depth_key(self, phase: RenderPhase) -> i64 {
        let depth_key = self.depth_key();
        if phase.is_transparent() {
            -depth_key
        } else {
            depth_key
        }
    }
}

impl RenderPhaseSortKey {
    pub const fn new(raw: i64) -> Self {
        Self(raw as i128)
    }

    pub const fn raw(self) -> i128 {
        self.0
    }

    pub fn for_components(phase: RenderPhase, components: RenderPhaseSortComponents) -> Self {
        let ordered_depth = components.ordered_depth_key(phase);

        let packed = (render_queue_sort_key(components.render_queue) << 112)
            | (material_queue_sort_key(components.material_queue) << 97)
            | (order_in_layer_sort_key(components.order_in_layer) << 74)
            | (ui_z_index_sort_key(components.ui_z_index) << 51)
            | (ordered_depth_sort_key(ordered_depth) << 16)
            | entity_tie_breaker_sort_key(components.entity_tie_breaker);
        Self(packed)
    }

    pub fn for_mesh(phase: RenderPhase, depth: f32, tie_breaker: u64) -> Self {
        Self::for_components(phase, RenderPhaseSortComponents::new(depth, tie_breaker))
    }

    pub fn for_sprite(phase: RenderPhase, z_order: i32, depth: f32, tie_breaker: u64) -> Self {
        Self::for_components(
            phase,
            RenderPhaseSortComponents::new(depth, tie_breaker).with_order_in_layer(z_order),
        )
    }
}

pub fn packed_sort_key_u64(
    phase: RenderPhase,
    components: RenderPhaseSortComponents,
    pipeline_variant: u32,
    material_discriminant: u16,
) -> u64 {
    let queue_prefix = (signed_15_u64(components.render_queue) << 49)
        | (signed_15_u64(components.material_queue) << 34);
    if phase.is_transparent() {
        return queue_prefix
            | (signed_i64_bits(components.ordered_depth_key(phase), 18) << 16)
            | entity_tie_breaker_u16(components.entity_tie_breaker);
    }

    let state_bucket =
        (((pipeline_variant as u64) & 0x03ff) << 8) | u64::from(material_discriminant & 0x00ff);
    queue_prefix
        | (state_bucket << 16)
        | (coarse_ordered_depth_u8(components.ordered_depth_key(phase)) << 8)
        | entity_tie_breaker_u8(components.entity_tie_breaker)
}

fn depth_sort_key(effective_depth: f32) -> i64 {
    if effective_depth.is_finite() {
        (effective_depth * 1000.0).round() as i64
    } else {
        0
    }
}

pub(super) fn render_queue_sort_key(value: i32) -> i128 {
    signed_15(value)
}

pub(super) fn material_queue_sort_key(value: i32) -> i128 {
    signed_15(value)
}

pub(super) fn order_in_layer_sort_key(value: i32) -> i128 {
    signed_23(value)
}

pub(super) fn ui_z_index_sort_key(value: i32) -> i128 {
    signed_23(value)
}

pub(super) fn ordered_depth_sort_key(value: i64) -> i128 {
    signed_35(value)
}

pub(super) fn entity_tie_breaker_sort_key(value: u64) -> i128 {
    i128::from(value & 0xffff)
}

fn signed_15(value: i32) -> i128 {
    i128::from(value.clamp(-16_384, 16_383) + 16_384)
}

fn signed_15_u64(value: i32) -> u64 {
    u64::from((value.clamp(-16_384, 16_383) + 16_384) as u32)
}

fn signed_23(value: i32) -> i128 {
    i128::from(value.clamp(-(1 << 22), (1 << 22) - 1) + (1 << 22))
}

fn signed_35(value: i64) -> i128 {
    i128::from(value.clamp(-(1_i64 << 34), (1_i64 << 34) - 1) + (1_i64 << 34))
}

fn signed_i64_bits(value: i64, bits: u32) -> u64 {
    debug_assert!(bits > 1 && bits < 63);
    let half_range = 1_i64 << (bits - 1);
    let max = half_range - 1;
    (value.clamp(-half_range, max) + half_range) as u64
}

fn coarse_ordered_depth_u8(value: i64) -> u64 {
    signed_i64_bits(value / 1000, 8)
}

fn entity_tie_breaker_u16(value: u64) -> u64 {
    value & 0xffff
}

fn entity_tie_breaker_u8(value: u64) -> u64 {
    value & 0xff
}

#[cfg(test)]
mod tests {
    use super::{packed_sort_key_u64, RenderPhaseSortComponents};
    use crate::core::framework::render::RenderPhase;

    #[test]
    fn packed_sort_key_clusters_opaque_by_pipeline_before_tie_breaker() {
        let earlier_draw_later_pipeline = packed_sort_key_u64(
            RenderPhase::Opaque3d,
            RenderPhaseSortComponents::new(0.0, 1),
            2,
            0,
        );
        let later_draw_earlier_pipeline = packed_sort_key_u64(
            RenderPhase::Opaque3d,
            RenderPhaseSortComponents::new(0.0, 2),
            1,
            0,
        );

        assert!(later_draw_earlier_pipeline < earlier_draw_later_pipeline);
    }

    #[test]
    fn packed_sort_key_keeps_transparent_depth_before_pipeline() {
        let far_later_pipeline = packed_sort_key_u64(
            RenderPhase::Transparent3d,
            RenderPhaseSortComponents::new(100.0, 1),
            99,
            0,
        );
        let near_earlier_pipeline = packed_sort_key_u64(
            RenderPhase::Transparent3d,
            RenderPhaseSortComponents::new(1.0, 2),
            1,
            0,
        );

        assert!(far_later_pipeline < near_earlier_pipeline);
    }

    #[test]
    fn packed_sort_key_ignores_transparent_pipeline_variant() {
        let first = packed_sort_key_u64(
            RenderPhase::Transparent3d,
            RenderPhaseSortComponents::new(10.0, 5),
            1,
            0,
        );
        let second = packed_sort_key_u64(
            RenderPhase::Transparent3d,
            RenderPhaseSortComponents::new(10.0, 5),
            999,
            128,
        );

        assert_eq!(first, second);
    }
}
