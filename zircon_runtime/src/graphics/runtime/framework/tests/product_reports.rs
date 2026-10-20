use std::sync::Arc;

use crate::core::framework::render::{
    CameraRenderDescriptor, RenderCameraTarget, RenderViewportDescriptor, RenderViewportRect,
    RenderVirtualGeometryDebugSnapshot, ViewportCameraSnapshot,
};
use crate::core::math::UVec2;
use crate::graphics::scene::RenderGraphLightGridReport;

use super::super::camera_history_key::ViewportCameraHistoryKey;
use super::ViewportRecord;

#[test]
fn viewport_record_keeps_product_reports_per_camera_key() {
    let mut record = ViewportRecord::new(RenderViewportDescriptor::new(UVec2::new(64, 64)));
    let left_key = camera_key(1, UVec2::ZERO);
    let right_key = camera_key(1, UVec2::new(32, 0));
    let left_light = light_report(3);
    let right_light = light_report(9);
    let left_debug = Arc::new(virtual_geometry_debug_snapshot(7));
    let right_debug = Arc::new(virtual_geometry_debug_snapshot(13));

    record.record_camera_product_reports(&left_key, Some(left_light), Some(&left_debug));
    record.record_camera_product_reports(&right_key, Some(right_light), Some(&right_debug));

    assert_eq!(record.camera_light_grid_report(&left_key), Some(left_light));
    assert_eq!(
        record.camera_light_grid_report(&right_key),
        Some(right_light)
    );
    assert_eq!(
        record.camera_virtual_geometry_debug_snapshot(&left_key),
        Some(left_debug.as_ref())
    );
    assert_eq!(
        record.camera_virtual_geometry_debug_snapshot(&right_key),
        Some(right_debug.as_ref())
    );

    record.record_camera_product_reports(&left_key, None, None);

    assert_eq!(record.camera_light_grid_report(&left_key), None);
    assert_eq!(
        record.camera_virtual_geometry_debug_snapshot(&left_key),
        None
    );
    assert_eq!(
        record.camera_light_grid_report(&right_key),
        Some(right_light)
    );
    assert_eq!(
        record.camera_virtual_geometry_debug_snapshot(&right_key),
        Some(right_debug.as_ref())
    );
}

#[test]
fn viewport_record_shares_virtual_geometry_debug_snapshots() {
    let mut record = ViewportRecord::new(RenderViewportDescriptor::new(UVec2::new(64, 64)));
    let key = camera_key(1, UVec2::ZERO);
    let snapshot = Arc::new(virtual_geometry_debug_snapshot(7));

    record.record_camera_product_reports(&key, None, Some(&snapshot));

    assert!(Arc::ptr_eq(
        record
            .virtual_geometry_debug_snapshots
            .get(&key)
            .expect("camera snapshot should be retained"),
        &snapshot,
    ));
}

fn camera_key(entity: u64, position: UVec2) -> ViewportCameraHistoryKey {
    let mut descriptor = CameraRenderDescriptor::from_camera_payload(
        Some(entity),
        ViewportCameraSnapshot::default(),
    );
    descriptor.target = RenderCameraTarget::PrimarySurface;
    descriptor.viewport_rect = Some(RenderViewportRect::new(position, UVec2::new(32, 64)));
    ViewportCameraHistoryKey::from_camera(&descriptor)
}

fn light_report(light_count: usize) -> RenderGraphLightGridReport {
    RenderGraphLightGridReport {
        light_count,
        tile_count: light_count + 1,
        zbin_count: light_count + 2,
        non_empty_tile_count: light_count + 3,
        non_empty_zbin_count: light_count + 4,
        non_empty_cluster_count: light_count + 5,
        peak_lights_per_cluster: light_count + 6,
        average_lights_per_cluster_milli: light_count + 7,
    }
}

fn virtual_geometry_debug_snapshot(
    execution_segment_count: u32,
) -> RenderVirtualGeometryDebugSnapshot {
    RenderVirtualGeometryDebugSnapshot {
        execution_segment_count,
        ..RenderVirtualGeometryDebugSnapshot::default()
    }
}

#[test]
fn physical_submission_binding_requires_original_arc_and_current_frame_epoch() {
    use super::AcceptedPhysicalSubmissions;
    let original = Arc::new(virtual_geometry_debug_snapshot(1));
    let equal_payload = Arc::new(original.as_ref().clone());
    let accepted = AcceptedPhysicalSubmissions {
        scope: (7, 3, 11, 19),
        epoch: (23, 5),
        snapshot: Some(Arc::downgrade(&original)),
        segments: Vec::new(),
    };
    assert!(accepted.matches(&original, &original, 3, (23, 5), Some(&11)));
    assert!(!accepted.matches(&equal_payload, &equal_payload, 3, (23, 5), Some(&11)));
    assert!(!accepted.matches(&original, &equal_payload, 3, (23, 5), Some(&11)));
    assert!(!accepted.matches(&original, &original, 4, (23, 5), Some(&11)));
    assert!(!accepted.matches(&original, &original, 3, (23, 6), Some(&11)));
    assert!(!accepted.matches(&original, &original, 3, (23, 5), Some(&12)));
    assert!(!accepted.matches(&original, &original, 3, (23, 5), None));
}

#[test]
fn physical_submission_binding_matches_scope_and_either_publication_order() {
    use super::AcceptedPhysicalSubmissions;
    let original = Arc::new(virtual_geometry_debug_snapshot(1));
    let identity = (7, 3, 11, 19);
    for report_first in [false, true] {
        let mut accepted = AcceptedPhysicalSubmissions {
            scope: identity,
            epoch: (23, 5),
            snapshot: None,
            segments: Vec::new(),
        };
        if report_first {
            accepted.snapshot = Some(Arc::downgrade(&original));
        } else {
            accepted.bind(identity, Some((23, 5)), Some(&original));
        }
        assert!(accepted.matches(&original, &original, 3, (23, 5), Some(&11)));
        for wrong in [
            (8, 3, 11, 19),
            (7, 4, 11, 19),
            (7, 3, 12, 19),
            (7, 3, 11, 20),
        ] {
            accepted.bind(wrong, Some((23, 5)), None);
            assert!(accepted.matches(&original, &original, 3, (23, 5), Some(&11)));
        }
        accepted.bind(identity, None, None);
        accepted.bind(identity, Some((23, 6)), None);
        assert!(accepted.matches(&original, &original, 3, (23, 5), Some(&11)));
        accepted.bind(identity, Some((23, 5)), None);
        assert!(!accepted.matches(&original, &original, 3, (23, 5), Some(&11)));
    }
}
