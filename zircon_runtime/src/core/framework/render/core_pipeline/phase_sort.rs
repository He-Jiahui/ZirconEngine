use serde::{Deserialize, Serialize};

use super::packed_sort_key::{depth_sort_key, ordered_depth_key, packed_sort_key_u64};
use super::{RenderPhase, RenderQueueValue};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct RenderPhaseSortKey(u64);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderPhaseSortComponents {
    pub camera_order: i32,
    pub queue: RenderQueueValue,
    pub sorting_layer: i32,
    pub order_in_layer: i32,
    pub y_sort: Option<f32>,
    pub depth: f32,
    pub depth_bias: f32,
    pub ui_z_index: i32,
    pub entity_tie_breaker: u64,
}

impl RenderPhaseSortComponents {
    pub const fn new(depth: f32, entity_tie_breaker: u64) -> Self {
        Self {
            camera_order: 0,
            queue: RenderQueueValue::GEOMETRY,
            sorting_layer: 0,
            order_in_layer: 0,
            y_sort: None,
            depth,
            depth_bias: 0.0,
            ui_z_index: 0,
            entity_tie_breaker,
        }
    }

    pub const fn with_camera_order(mut self, camera_order: i32) -> Self {
        self.camera_order = camera_order;
        self
    }

    pub const fn with_queue(mut self, queue: RenderQueueValue) -> Self {
        self.queue = queue;
        self
    }

    pub fn with_queue_offset(mut self, offset: i32) -> Self {
        self.queue = self.queue.with_material_offset_i32(offset);
        self
    }

    pub const fn with_sorting_layer(mut self, sorting_layer: i32) -> Self {
        self.sorting_layer = sorting_layer;
        self
    }

    pub const fn with_order_in_layer(mut self, order_in_layer: i32) -> Self {
        self.order_in_layer = order_in_layer;
        self
    }

    pub const fn with_y_sort(mut self, y_sort: Option<f32>) -> Self {
        self.y_sort = y_sort;
        self
    }

    pub const fn with_depth_bias(mut self, depth_bias: f32) -> Self {
        self.depth_bias = depth_bias;
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
        ordered_depth_key(phase, self.depth_key())
    }
}

impl RenderPhaseSortKey {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    pub fn for_components(phase: RenderPhase, components: RenderPhaseSortComponents) -> Self {
        Self(packed_sort_key_u64(phase, components, 0, 0))
    }

    pub fn for_mesh(phase: RenderPhase, depth: f32, tie_breaker: u64) -> Self {
        Self::for_components(
            phase,
            RenderPhaseSortComponents::new(depth, tie_breaker)
                .with_queue(default_queue_for_phase(phase)),
        )
    }

    pub fn for_sprite(phase: RenderPhase, z_order: i32, depth: f32, tie_breaker: u64) -> Self {
        Self::for_components(
            phase,
            RenderPhaseSortComponents::new(depth, tie_breaker)
                .with_queue(default_queue_for_phase(phase))
                .with_order_in_layer(z_order),
        )
    }
}

fn default_queue_for_phase(phase: RenderPhase) -> RenderQueueValue {
    match phase {
        RenderPhase::AlphaMask2d | RenderPhase::AlphaMask3d => RenderQueueValue::ALPHA_TEST,
        RenderPhase::Transparent2d | RenderPhase::Transparent3d => RenderQueueValue::TRANSPARENT,
        RenderPhase::Ui | RenderPhase::Overlay | RenderPhase::Debug => RenderQueueValue::OVERLAY,
        _ => RenderQueueValue::GEOMETRY,
    }
}

#[cfg(test)]
mod tests {
    use super::{packed_sort_key_u64, RenderPhaseSortComponents};
    use crate::core::framework::render::{RenderPhase, RenderQueueValue};

    #[test]
    fn packed_sort_key_uses_plan09_camera_queue_domain_and_tie_segments() {
        let components = RenderPhaseSortComponents::new(10.75, 0x1_1234)
            .with_camera_order(12)
            .with_queue(RenderQueueValue::new(2_500));
        let key = packed_sort_key_u64(RenderPhase::Opaque3d, components, 0x1ab, 0x1234);
        let material_cluster = ((0x1234_u16 >> 8) ^ 0x1234_u16) & 0x00ff;
        let expected_domain =
            ((0x1ab_u64 & 0x03ff) << 23) | (u64::from(material_cluster) << 15) | 86;

        assert_eq!(key >> 56, 140);
        assert_eq!((key >> 43) & 0x1fff, 2_500);
        assert_eq!((key >> 10) & 0x1_ffff_ffff, expected_domain);
        assert_eq!(key & 0x03ff, 0x234);
    }

    #[test]
    fn packed_sort_key_clusters_opaque_by_pipeline_before_tie_breaker() {
        let earlier_draw_later_pipeline = packed_sort_key_u64(
            RenderPhase::Opaque3d,
            RenderPhaseSortComponents::new(0.0, 1).with_queue(RenderQueueValue::GEOMETRY),
            2,
            0,
        );
        let later_draw_earlier_pipeline = packed_sort_key_u64(
            RenderPhase::Opaque3d,
            RenderPhaseSortComponents::new(0.0, 2).with_queue(RenderQueueValue::GEOMETRY),
            1,
            0,
        );

        assert!(later_draw_earlier_pipeline < earlier_draw_later_pipeline);
    }

    #[test]
    fn packed_sort_key_keeps_transparent_depth_before_pipeline() {
        let far_later_pipeline = packed_sort_key_u64(
            RenderPhase::Transparent3d,
            RenderPhaseSortComponents::new(100.0, 1).with_queue(RenderQueueValue::TRANSPARENT),
            99,
            0,
        );
        let near_earlier_pipeline = packed_sort_key_u64(
            RenderPhase::Transparent3d,
            RenderPhaseSortComponents::new(1.0, 2).with_queue(RenderQueueValue::TRANSPARENT),
            1,
            0,
        );

        assert!(far_later_pipeline < near_earlier_pipeline);
    }

    #[test]
    fn packed_sort_key_uses_transparent_pipeline_only_inside_equal_depth_bucket() {
        let later_pipeline = packed_sort_key_u64(
            RenderPhase::Transparent3d,
            RenderPhaseSortComponents::new(10.0, 5).with_queue(RenderQueueValue::TRANSPARENT),
            2,
            0,
        );
        let earlier_pipeline = packed_sort_key_u64(
            RenderPhase::Transparent3d,
            RenderPhaseSortComponents::new(10.0, 5).with_queue(RenderQueueValue::TRANSPARENT),
            1,
            128,
        );

        assert!(earlier_pipeline < later_pipeline);
    }
}
