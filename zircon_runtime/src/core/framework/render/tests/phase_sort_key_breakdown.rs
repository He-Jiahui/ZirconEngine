use super::RenderPhaseSortKeyBreakdown;
use crate::core::framework::render::{
    packed_sort_key_u64, RenderPhase, RenderPhaseSortComponents, RenderQueueValue,
};

#[test]
fn render_sort_key_breakdown_roundtrip() {
    let components = RenderPhaseSortComponents::new(10.25, 0x1_1234)
        .with_camera_order(7)
        .with_queue(RenderQueueValue::ALPHA_TEST.with_material_offset_i32(25))
        .with_sorting_layer(3)
        .with_order_in_layer(9)
        .with_y_sort(Some(2.0))
        .with_depth_bias(0.5)
        .with_ui_z_index(11);

    let breakdown = RenderPhaseSortKeyBreakdown::from_components_with_clusters(
        RenderPhase::Opaque3d,
        components,
        0x1ab,
        0x1234,
    );

    assert_eq!(breakdown.phase, RenderPhase::Opaque3d);
    assert_eq!(breakdown.camera_order, 7);
    assert_eq!(breakdown.camera_order_key, 135);
    assert_eq!(breakdown.queue, RenderQueueValue::new(2_475));
    assert_eq!(breakdown.queue_key, 2_475);
    assert_eq!(breakdown.sorting_layer, 3);
    assert_eq!(breakdown.order_in_layer, 9);
    assert_eq!(breakdown.y_sort, Some(2.0));
    assert_eq!(breakdown.ui_z_index, 11);
    assert_eq!(breakdown.effective_depth, 10.75);
    assert_eq!(breakdown.opaque_depth_key, 86);
    assert_eq!(breakdown.pipeline_cluster_key, 0x1ab);
    assert_eq!(breakdown.material_cluster_key, 0x26);
    assert_eq!(breakdown.tie_breaker_key, 0x234);
    assert_eq!(
        breakdown.raw_sort_key,
        packed_sort_key_u64(RenderPhase::Opaque3d, components, 0x1ab, 0x1234)
    );
}
