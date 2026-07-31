use crate::core::framework::render::RenderVirtualGeometryDebugSnapshot;
use crate::graphics::scene::RenderGraphLightGridReport;
use std::sync::Arc;

use super::{ViewportCameraHistoryKey, viewport_record::ViewportRecord};

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn record_camera_product_reports(
        &mut self,
        key: &ViewportCameraHistoryKey,
        light_grid_report: Option<RenderGraphLightGridReport>,
        virtual_geometry_debug_snapshot: Option<&Arc<RenderVirtualGeometryDebugSnapshot>>,
    ) {
        if let Some(report) = light_grid_report {
            self.light_grid_reports.insert(key.clone(), report);
        } else {
            self.light_grid_reports.remove(key);
        }

        if let Some(snapshot) = virtual_geometry_debug_snapshot {
            self.virtual_geometry_debug_snapshots
                .insert(key.clone(), snapshot.clone());
        } else {
            self.virtual_geometry_debug_snapshots.remove(key);
        }
    }

    #[cfg(test)]
    pub(in crate::graphics::runtime::render_framework) fn camera_light_grid_report(
        &self,
        key: &ViewportCameraHistoryKey,
    ) -> Option<RenderGraphLightGridReport> {
        self.light_grid_reports.get(key).copied()
    }

    #[cfg(test)]
    pub(in crate::graphics::runtime::render_framework) fn camera_virtual_geometry_debug_snapshot(
        &self,
        key: &ViewportCameraHistoryKey,
    ) -> Option<&RenderVirtualGeometryDebugSnapshot> {
        self.virtual_geometry_debug_snapshots
            .get(key)
            .map(Arc::as_ref)
    }
}

#[cfg(test)]
mod tests {
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
}
