use super::{
    has_paintable_thumbnail_extent, thumbnail_surface_rect, FrameRect, TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

#[test]
fn thumbnail_geometry_rejects_collapsed_non_finite_and_overflowed_frames() {
    let node = TemplatePaneNodeData::default();
    let metrics = super::super::asset_visual_metrics_from_host(METRICS);
    let valid = FrameRect {
        x: 12.0,
        y: 8.0,
        width: 74.0,
        height: 42.0,
    };

    assert!(has_paintable_thumbnail_extent(&valid));
    assert!(thumbnail_surface_rect(&node, &valid, metrics).is_some());
    assert!(!has_paintable_thumbnail_extent(&FrameRect {
        width: 0.0,
        ..valid.clone()
    }));
    assert!(thumbnail_surface_rect(
        &node,
        &FrameRect {
            x: f32::NAN,
            ..valid.clone()
        },
        metrics,
    )
    .is_none());
    assert!(!has_paintable_thumbnail_extent(&FrameRect {
        // BUG: [CR-EDITOR-PAINT-ROWS-0004] f32::MAX + 74.0 仍舍入为有限的 f32::MAX；
        // 此用例断言非有限边界却给了不会溢出的宽度，当前测试会在此处失败。
        // 改为同量级宽度，才能验证 has_paintable_thumbnail_extent 的溢出门槛。
        x: f32::MAX,
        ..valid
    }));
}
