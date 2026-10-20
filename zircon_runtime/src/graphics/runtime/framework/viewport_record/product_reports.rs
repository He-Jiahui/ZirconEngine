//! 产品诊断随视口及帧代际更新，查询端不能拼接不同相机或前后两帧的报告。
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use crate::core::framework::render::RenderVirtualGeometryDebugSnapshot;
use crate::graphics::scene::RenderGraphLightGridReport;

use super::{viewport_record::ViewportRecord, ViewportCameraHistoryKey};

pub(super) fn physical_scope_key(
    scope: &crate::graphics::scene::VirtualGeometryReadbackScope,
) -> (u64, u64, u64, u64) {
    (
        scope.viewport.raw(),
        scope.viewport_generation,
        scope.frame_generation,
        scope.scope_id,
    )
}

pub(super) struct AcceptedPhysicalSubmissions {
    pub(super) scope: (u64, u64, u64, u64),
    pub(super) epoch: (u64, u64),
    pub(super) snapshot: Option<std::sync::Weak<RenderVirtualGeometryDebugSnapshot>>,
    pub(super) segments: Vec<crate::core::framework::render::RenderVirtualGeometryExecutionSegment>,
}

impl AcceptedPhysicalSubmissions {
    fn bind(
        &mut self,
        scope: (u64, u64, u64, u64),
        epoch: Option<(u64, u64)>,
        snapshot: Option<&Arc<RenderVirtualGeometryDebugSnapshot>>,
    ) {
        if self.scope == scope && epoch == Some(self.epoch) {
            self.snapshot = snapshot.map(Arc::downgrade);
        }
    }

    fn matches(
        &self,
        original: &Arc<RenderVirtualGeometryDebugSnapshot>,
        current: &Arc<RenderVirtualGeometryDebugSnapshot>,
        generation: u64,
        epoch: (u64, u64),
        frame: Option<&u64>,
    ) -> bool {
        self.scope.1 == generation
            && self.epoch == epoch
            && frame == Some(&self.scope.2)
            && Arc::ptr_eq(original, current)
            && self.snapshot.as_ref().is_some_and(|bound| {
                std::sync::Weak::ptr_eq(bound, &Arc::downgrade(original))
                    && bound.strong_count() != 0
            })
    }
}

impl ViewportRecord {
    // Report publication and history acceptance can occur in either order.
    // The full packet identity binds the original Arc, never a later snapshot.
    pub(in crate::graphics::runtime::render_framework) fn bind_physical_submission_snapshot(
        &mut self,
        key: &ViewportCameraHistoryKey,
        scope: Option<&crate::graphics::scene::VirtualGeometryReadbackScope>,
        snapshot: Option<&Arc<RenderVirtualGeometryDebugSnapshot>>,
    ) {
        let Some(scope) = scope else {
            return;
        };
        if scope.viewport_generation != self.generation {
            return;
        }
        let identity = physical_scope_key(scope);
        let binding = snapshot.map(Arc::downgrade);
        for pending in &mut self.pending_virtual_geometry_readbacks {
            if pending.history == *key
                && physical_scope_key(&pending.packet.scope) == identity
                && self.virtual_geometry_readback_epoch == Some(pending.packet.epoch)
            {
                pending.snapshot = binding.clone();
            }
        }
        if let Some(accepted) = self.accepted_virtual_geometry_submissions.get_mut(key) {
            accepted.bind(
                identity,
                self.virtual_geometry_readback_epoch
                    .map(|epoch| epoch.raw_parts()),
                snapshot,
            );
        }
    }

    pub(in crate::graphics::runtime::render_framework) fn physical_submission_segments(
        &self,
        original: &Arc<RenderVirtualGeometryDebugSnapshot>,
        epoch: crate::graphics::scene::RenderPassDeviceEpoch,
    ) -> Option<&[crate::core::framework::render::RenderVirtualGeometryExecutionSegment]> {
        let mut matches =
            self.accepted_virtual_geometry_submissions
                .iter()
                .filter(|(key, value)| {
                    self.virtual_geometry_debug_snapshots
                        .get(*key)
                        .is_some_and(|current| {
                            value.matches(
                                original,
                                current,
                                self.generation,
                                epoch.raw_parts(),
                                self.last_applied_virtual_geometry_generation.get(*key),
                            )
                        })
                });
        let (_, accepted) = matches.next()?;
        if matches.next().is_some() {
            return None;
        }
        Some(&accepted.segments)
    }

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
#[path = "../tests/product_reports.rs"]
mod tests;

#[cfg(test)]
#[path = "../product_reports/tests/key_reuse.rs"]
mod key_reuse_tests;
