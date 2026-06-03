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
}

impl RenderPhaseSortKey {
    pub const fn new(raw: i64) -> Self {
        Self(raw as i128)
    }

    pub const fn raw(self) -> i128 {
        self.0
    }

    pub fn for_components(phase: RenderPhase, components: RenderPhaseSortComponents) -> Self {
        let effective_depth = components.depth + components.depth_bias;
        let depth_key = if effective_depth.is_finite() {
            (effective_depth * 1000.0).round() as i64
        } else {
            0
        };
        let ordered_depth = if phase.is_transparent() {
            -depth_key
        } else {
            depth_key
        };

        let packed = (signed_15(components.render_queue) << 112)
            | (signed_15(components.material_queue) << 97)
            | (signed_23(components.order_in_layer) << 74)
            | (signed_23(components.ui_z_index) << 51)
            | (signed_35(ordered_depth) << 16)
            | i128::from(components.entity_tie_breaker & 0xffff);
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

fn signed_15(value: i32) -> i128 {
    i128::from(value.clamp(-16_384, 16_383) + 16_384)
}

fn signed_23(value: i32) -> i128 {
    i128::from(value.clamp(-(1 << 22), (1 << 22) - 1) + (1 << 22))
}

fn signed_35(value: i64) -> i128 {
    i128::from(value.clamp(-(1_i64 << 34), (1_i64 << 34) - 1) + (1_i64 << 34))
}
