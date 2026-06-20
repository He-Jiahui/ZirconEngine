use super::{RenderPhase, RenderPhaseSortComponents};

pub const SORT_KEY_CAMERA_ORDER_SHIFT: u32 = 56;
pub const SORT_KEY_QUEUE_SHIFT: u32 = 43;
pub const SORT_KEY_DOMAIN_SHIFT: u32 = 10;

pub(super) const SORT_KEY_CAMERA_ORDER_MASK: u64 = 0xff;
pub(super) const SORT_KEY_QUEUE_MASK: u64 = 0x1fff;
pub(super) const SORT_KEY_DOMAIN_MASK: u64 = 0x1_ffff_ffff;
pub(super) const SORT_KEY_TIE_MASK: u64 = 0x03ff;
pub(super) const Y_SORT_UNITS: f32 = 16.0;

pub fn packed_sort_key_u64(
    phase: RenderPhase,
    components: RenderPhaseSortComponents,
    pipeline_variant: u32,
    material_discriminant: u16,
) -> u64 {
    ((camera_order_key(components.camera_order) & SORT_KEY_CAMERA_ORDER_MASK)
        << SORT_KEY_CAMERA_ORDER_SHIFT)
        | (queue_key(components.queue) << SORT_KEY_QUEUE_SHIFT)
        | ((domain_key(phase, components, pipeline_variant, material_discriminant)
            & SORT_KEY_DOMAIN_MASK)
            << SORT_KEY_DOMAIN_SHIFT)
        | entity_tie_breaker_key(components.entity_tie_breaker)
}

pub(super) fn camera_order_key(value: i32) -> u64 {
    signed_lane(value, -128, 127)
}

pub(super) fn queue_key(value: super::RenderQueueValue) -> u64 {
    u64::from(value.raw()).min(SORT_KEY_QUEUE_MASK)
}

pub(super) fn domain_key(
    phase: RenderPhase,
    components: RenderPhaseSortComponents,
    pipeline_variant: u32,
    material_discriminant: u16,
) -> u64 {
    match phase {
        RenderPhase::Transparent3d => {
            (transparent_depth_key(components.effective_depth()) << 10)
                | pipeline_cluster_key(pipeline_variant)
        }
        RenderPhase::Opaque2d | RenderPhase::AlphaMask2d | RenderPhase::Transparent2d => {
            (sorting_layer_key(components.sorting_layer) << 25)
                | (order_in_layer_key(components.order_in_layer) << 10)
                | y_sort_key(components.y_sort)
        }
        RenderPhase::Ui | RenderPhase::Overlay => ui_z_index_key(components.ui_z_index) << 10,
        _ => {
            (pipeline_cluster_key(pipeline_variant) << 23)
                | (material_cluster_key(material_discriminant) << 15)
                | opaque_depth_key(components.effective_depth())
        }
    }
}

pub(super) fn pipeline_cluster_key(value: u32) -> u64 {
    u64::from(value) & 0x03ff
}

pub(super) fn material_cluster_key(value: u16) -> u64 {
    u64::from(((value >> 8) ^ value) & 0x00ff)
}

pub(super) fn opaque_depth_key(effective_depth: f32) -> u64 {
    quantized_non_negative_depth(effective_depth, 8.0, 0x7fff)
}

pub(super) fn transparent_depth_key(effective_depth: f32) -> u64 {
    0x7f_ffff - quantized_non_negative_depth(effective_depth, 1000.0, 0x7f_ffff)
}

pub(super) fn sorting_layer_key(value: i32) -> u64 {
    signed_lane(value, -128, 127)
}

pub(super) fn order_in_layer_key(value: i32) -> u64 {
    signed_lane(value, -16_384, 16_383)
}

pub(super) fn y_sort_key(value: Option<f32>) -> u64 {
    value
        .filter(|value| value.is_finite())
        .map(|value| {
            let rounded = (value * Y_SORT_UNITS).round() as i32;
            signed_lane(rounded, -512, 511)
        })
        .unwrap_or(512)
}

pub(super) fn ui_z_index_key(value: i32) -> u64 {
    signed_lane(value, -4_194_304, 4_194_303)
}

pub(super) fn entity_tie_breaker_key(value: u64) -> u64 {
    value & SORT_KEY_TIE_MASK
}

pub(super) fn depth_sort_key(effective_depth: f32) -> i64 {
    if effective_depth.is_finite() {
        (effective_depth * 1000.0).round() as i64
    } else {
        0
    }
}

pub(super) fn ordered_depth_key(phase: RenderPhase, depth_key: i64) -> i64 {
    if phase == RenderPhase::Transparent3d {
        -depth_key
    } else {
        depth_key
    }
}

fn signed_lane(value: i32, min: i32, max: i32) -> u64 {
    (value.clamp(min, max) - min) as u64
}

fn quantized_non_negative_depth(effective_depth: f32, units: f32, max: u64) -> u64 {
    if !effective_depth.is_finite() {
        return 0;
    }
    ((effective_depth * units).round() as i64).clamp(0, max as i64) as u64
}

#[cfg(test)]
mod tests {
    use super::{packed_sort_key_u64, SORT_KEY_QUEUE_SHIFT};
    use crate::core::framework::render::{
        RenderPhase, RenderPhaseSortComponents, RenderQueueValue,
    };

    #[test]
    fn render_sort_key_camera_order_dominates_queue() {
        let early_camera_late_queue = packed_sort_key_u64(
            RenderPhase::Transparent3d,
            RenderPhaseSortComponents::new(100.0, 1)
                .with_camera_order(-1)
                .with_queue(RenderQueueValue::TRANSPARENT),
            99,
            0,
        );
        let later_camera_early_queue = packed_sort_key_u64(
            RenderPhase::Opaque3d,
            RenderPhaseSortComponents::new(0.0, 2)
                .with_camera_order(0)
                .with_queue(RenderQueueValue::BACKGROUND),
            0,
            0,
        );

        assert!(early_camera_late_queue < later_camera_early_queue);
    }

    #[test]
    fn render_sort_key_opaque_clusters_pipeline_before_depth() {
        let pipeline_one_far = packed_sort_key_u64(
            RenderPhase::Opaque3d,
            RenderPhaseSortComponents::new(100.0, 1).with_queue(RenderQueueValue::GEOMETRY),
            1,
            0,
        );
        let pipeline_two_near = packed_sort_key_u64(
            RenderPhase::Opaque3d,
            RenderPhaseSortComponents::new(1.0, 2).with_queue(RenderQueueValue::GEOMETRY),
            2,
            0,
        );
        let pipeline_one_near = packed_sort_key_u64(
            RenderPhase::Opaque3d,
            RenderPhaseSortComponents::new(1.0, 3).with_queue(RenderQueueValue::GEOMETRY),
            1,
            0,
        );

        assert!(pipeline_one_far < pipeline_two_near);
        assert!(pipeline_one_near < pipeline_one_far);
    }

    #[test]
    fn render_sort_key_transparent_depth_back_to_front_ignores_cluster() {
        let far_late_pipeline = packed_sort_key_u64(
            RenderPhase::Transparent3d,
            RenderPhaseSortComponents::new(100.0, 1).with_queue(RenderQueueValue::TRANSPARENT),
            99,
            0,
        );
        let near_early_pipeline = packed_sort_key_u64(
            RenderPhase::Transparent3d,
            RenderPhaseSortComponents::new(1.0, 2).with_queue(RenderQueueValue::TRANSPARENT),
            1,
            0,
        );
        let equal_depth_early_pipeline = packed_sort_key_u64(
            RenderPhase::Transparent3d,
            RenderPhaseSortComponents::new(10.0, 3).with_queue(RenderQueueValue::TRANSPARENT),
            1,
            0,
        );
        let equal_depth_late_pipeline = packed_sort_key_u64(
            RenderPhase::Transparent3d,
            RenderPhaseSortComponents::new(10.0, 4).with_queue(RenderQueueValue::TRANSPARENT),
            2,
            0,
        );

        assert!(far_late_pipeline < near_early_pipeline);
        assert!(equal_depth_early_pipeline < equal_depth_late_pipeline);
    }

    #[test]
    fn render_sort_key_2d_sorting_layer_then_order_then_y() {
        let base = RenderPhaseSortComponents::new(0.0, 1).with_queue(RenderQueueValue::GEOMETRY);
        let layer_one = base.with_sorting_layer(1).with_order_in_layer(-10);
        let order_one = base.with_order_in_layer(1).with_y_sort(Some(-10.0));
        let y_one = base.with_y_sort(Some(1.0));

        assert!(
            packed_sort_key_u64(RenderPhase::Opaque2d, base, 99, 99)
                < packed_sort_key_u64(RenderPhase::Opaque2d, layer_one, 0, 0)
        );
        assert!(
            packed_sort_key_u64(RenderPhase::Opaque2d, base, 99, 99)
                < packed_sort_key_u64(RenderPhase::Opaque2d, order_one, 0, 0)
        );
        assert!(
            packed_sort_key_u64(RenderPhase::Opaque2d, base, 99, 99)
                < packed_sort_key_u64(RenderPhase::Opaque2d, y_one, 0, 0)
        );
    }

    #[test]
    fn render_sort_key_ui_z_index_maps_into_overlay_segment() {
        let low_z = packed_sort_key_u64(
            RenderPhase::Overlay,
            RenderPhaseSortComponents::new(0.0, 1)
                .with_queue(RenderQueueValue::OVERLAY)
                .with_ui_z_index(-10),
            99,
            99,
        );
        let high_z = packed_sort_key_u64(
            RenderPhase::Overlay,
            RenderPhaseSortComponents::new(0.0, 2)
                .with_queue(RenderQueueValue::OVERLAY)
                .with_ui_z_index(10),
            0,
            0,
        );

        assert_eq!(
            (low_z >> SORT_KEY_QUEUE_SHIFT) & 0x1fff,
            u64::from(RenderQueueValue::OVERLAY.raw())
        );
        assert!(low_z < high_z);
    }

    #[test]
    fn render_sort_key_fixed_representative_order_snapshot() {
        #[derive(Clone, Copy)]
        struct Sample {
            name: &'static str,
            phase: RenderPhase,
            components: RenderPhaseSortComponents,
            pipeline_variant: u32,
            material_discriminant: u16,
        }

        let mut samples = [
            Sample {
                name: "transparent-near",
                phase: RenderPhase::Transparent3d,
                components: RenderPhaseSortComponents::new(1.0, 4)
                    .with_queue(RenderQueueValue::TRANSPARENT),
                pipeline_variant: 1,
                material_discriminant: 0,
            },
            Sample {
                name: "opaque-pipeline-two",
                phase: RenderPhase::Opaque3d,
                components: RenderPhaseSortComponents::new(0.0, 2)
                    .with_queue(RenderQueueValue::GEOMETRY),
                pipeline_variant: 2,
                material_discriminant: 0,
            },
            Sample {
                name: "overlay",
                phase: RenderPhase::Overlay,
                components: RenderPhaseSortComponents::new(0.0, 5)
                    .with_queue(RenderQueueValue::OVERLAY)
                    .with_ui_z_index(2),
                pipeline_variant: 0,
                material_discriminant: 0,
            },
            Sample {
                name: "transparent-far",
                phase: RenderPhase::Transparent3d,
                components: RenderPhaseSortComponents::new(100.0, 3)
                    .with_queue(RenderQueueValue::TRANSPARENT),
                pipeline_variant: 9,
                material_discriminant: 0,
            },
            Sample {
                name: "opaque-pipeline-one",
                phase: RenderPhase::Opaque3d,
                components: RenderPhaseSortComponents::new(100.0, 1)
                    .with_queue(RenderQueueValue::GEOMETRY),
                pipeline_variant: 1,
                material_discriminant: 0,
            },
            Sample {
                name: "alpha-mask",
                phase: RenderPhase::AlphaMask3d,
                components: RenderPhaseSortComponents::new(2.0, 6)
                    .with_queue(RenderQueueValue::ALPHA_TEST),
                pipeline_variant: 0,
                material_discriminant: 0,
            },
        ];

        samples.sort_by_key(|sample| {
            (
                sample.phase.queue_order(),
                packed_sort_key_u64(
                    sample.phase,
                    sample.components,
                    sample.pipeline_variant,
                    sample.material_discriminant,
                ),
                sample.components.entity_tie_breaker,
            )
        });

        assert_eq!(
            samples.map(|sample| sample.name),
            [
                "opaque-pipeline-one",
                "opaque-pipeline-two",
                "alpha-mask",
                "transparent-far",
                "transparent-near",
                "overlay",
            ]
        );
    }
}
