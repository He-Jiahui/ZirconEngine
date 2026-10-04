use super::{packed_sort_key_u64, RenderPhaseSortComponents};
use crate::core::framework::render::{RenderPhase, RenderQueueValue};

#[test]
fn packed_sort_key_uses_plan09_camera_queue_domain_and_tie_segments() {
    let components = RenderPhaseSortComponents::new(10.75, 0x1_1234)
        .with_camera_order(12)
        .with_queue(RenderQueueValue::new(2_500));
    let key = packed_sort_key_u64(RenderPhase::Opaque3d, components, 0x1ab, 0x1234);
    let material_cluster = ((0x1234_u16 >> 8) ^ 0x1234_u16) & 0x00ff;
    let expected_domain = ((0x1ab_u64 & 0x03ff) << 23) | (u64::from(material_cluster) << 15) | 86;

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
