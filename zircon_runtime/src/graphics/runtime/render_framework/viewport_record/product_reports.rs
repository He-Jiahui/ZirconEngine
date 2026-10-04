//! 产品诊断随视口及帧代际更新，查询端不能拼接不同相机或前后两帧的报告。
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use crate::core::framework::render::RenderVirtualGeometryDebugSnapshot;
use crate::graphics::scene::RenderGraphLightGridReport;

use super::{viewport_record::ViewportRecord, ViewportCameraHistoryKey};

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn record_camera_product_reports(
        &mut self,
        key: &ViewportCameraHistoryKey,
        light_grid_report: Option<RenderGraphLightGridReport>,
        virtual_geometry_debug_snapshot: Option<&Arc<RenderVirtualGeometryDebugSnapshot>>,
    ) {
        if let Some(report) = light_grid_report {
            set_borrowed_hash_value(&mut self.light_grid_reports, key, report);
        } else {
            self.light_grid_reports.remove(key);
        }

        if let Some(snapshot) = virtual_geometry_debug_snapshot {
            set_borrowed_hash_value(
                &mut self.virtual_geometry_debug_snapshots,
                key,
                Arc::clone(snapshot),
            );
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

fn set_borrowed_hash_value<K, V>(values: &mut HashMap<K, V>, key: &K, value: V)
where
    K: Clone + Eq + Hash,
{
    if let Some(current) = values.get_mut(key) {
        *current = value;
    } else {
        values.insert(key.clone(), value);
    }
}

#[cfg(test)]
#[path = "tests/product_reports.rs"]
mod tests;

#[cfg(test)]
#[path = "product_reports/tests/key_reuse_tests.rs"]
mod key_reuse_tests;
