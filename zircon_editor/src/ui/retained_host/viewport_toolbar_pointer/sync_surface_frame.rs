use std::sync::Arc;

use zircon_runtime_interface::ui::{layout::UiFrame, surface::UiSurfaceFrame};

use super::rebuild_surface::{
    viewport_toolbar_control_node_id, ViewportToolbarNodeFrameChange, ViewportToolbarSurfaceDelta,
};
use super::route_for_control::control_route_for_id;
use super::viewport_toolbar_pointer_bridge::ViewportToolbarPointerBridge;
use super::viewport_toolbar_pointer_control::ViewportToolbarPointerControl;

impl ViewportToolbarPointerBridge {
    pub(crate) fn sync_surface_frame(
        &mut self,
        surface_key: &str,
        surface_frame: &Arc<UiSurfaceFrame>,
    ) -> Result<bool, String> {
        let (surface_index, surface_origin) = self
            .layout
            .surfaces
            .iter()
            .enumerate()
            .find(|(_, surface)| surface.key == surface_key)
            .map(|(index, surface)| (index, surface.frame))
            .ok_or_else(|| format!("Unknown viewport toolbar surface {surface_key}"))?;
        if self.applied_surface_frames.get(surface_key).is_some_and(
            |(applied_frame, applied_origin)| {
                *applied_origin == surface_origin
                    && std::ptr::eq(applied_frame.as_ptr(), Arc::as_ptr(&surface_frame.hit_grid))
            },
        ) {
            return Ok(false);
        }
        let existing = self
            .controls_by_surface
            .get(surface_key)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let mut controls = Vec::new();
        let mut changes = Vec::new();
        let mut topology_changed = false;
        for entry in surface_frame.hit_grid.entries.iter() {
            let Some(control_id) = entry.control_id.as_deref() else {
                continue;
            };
            if control_route_for_id(control_id).is_none() {
                continue;
            }
            let frame = UiFrame::new(
                surface_origin.x + entry.frame.x,
                surface_origin.y + entry.frame.y,
                entry.frame.width.max(1.0),
                entry.frame.height.max(1.0),
            );
            let control_index = controls.len();
            match existing.get(control_index) {
                Some(current) if current.action_key == control_id => {
                    if current.frame != frame {
                        changes.push(ViewportToolbarNodeFrameChange {
                            node_id: viewport_toolbar_control_node_id(surface_index, control_index),
                            frame,
                        });
                    }
                }
                Some(_) | None => topology_changed = true,
            }
            controls.push(ViewportToolbarPointerControl {
                action_key: control_id.to_string(),
                frame,
            });
        }
        topology_changed |= existing.len() != controls.len();

        self.applied_surface_frames.insert(
            surface_key.to_string(),
            (Arc::downgrade(&surface_frame.hit_grid), surface_origin),
        );

        if !topology_changed && changes.is_empty() {
            self.apply_surface_delta(ViewportToolbarSurfaceDelta::NoChange);
            return Ok(false);
        }

        self.controls_by_surface
            .insert(surface_key.to_string(), controls);
        self.apply_surface_delta(if topology_changed {
            ViewportToolbarSurfaceDelta::Topology
        } else {
            ViewportToolbarSurfaceDelta::Geometry(changes)
        });
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use zircon_runtime_interface::ui::{
        event_ui::{UiNodeId, UiTreeId},
        layout::UiFrame,
        surface::{UiHitRouteNode, UiHitTestEntry, UiHitTestGrid, UiSurfaceFrame},
        tree::UiInputPolicy,
    };

    use super::super::{
        build_viewport_toolbar_pointer_layout_with_size, ViewportToolbarPointerBridge,
    };

    #[test]
    fn stable_frame_identity_skips_reprojecting_toolbar_controls() {
        let mut bridge = ViewportToolbarPointerBridge::new();
        assert!(bridge.sync(build_viewport_toolbar_pointer_layout_with_size(
            ["scene.main"],
            zircon_runtime_interface::ui::layout::UiSize::new(640.0, 28.0),
        )));

        let frame = surface_frame(7, true);
        assert!(bridge.sync_surface_frame("scene.main", &frame).unwrap());
        assert_eq!(bridge.controls_by_surface["scene.main"].len(), 1);
        assert!(!bridge.sync_surface_frame("scene.main", &frame).unwrap());

        let mut focus_only_frame = frame.as_ref().clone();
        focus_only_frame.generation = 8;
        let focus_only_frame = Arc::new(focus_only_frame);
        assert!(!bridge
            .sync_surface_frame("scene.main", &focus_only_frame)
            .unwrap());

        let changed_frame_with_same_local_generation = surface_frame(7, false);
        assert!(bridge
            .sync_surface_frame("scene.main", &changed_frame_with_same_local_generation)
            .unwrap());
        assert!(bridge.controls_by_surface["scene.main"].is_empty());

        let next_generation = surface_frame(8, true);
        assert!(bridge
            .sync_surface_frame("scene.main", &next_generation)
            .unwrap());
        assert_eq!(bridge.controls_by_surface["scene.main"].len(), 1);
    }

    #[test]
    fn legacy_zero_generation_never_reuses_the_applied_cursor() {
        let mut bridge = ViewportToolbarPointerBridge::new();
        bridge.sync(build_viewport_toolbar_pointer_layout_with_size(
            ["scene.main"],
            zircon_runtime_interface::ui::layout::UiSize::new(640.0, 28.0),
        ));

        assert!(bridge
            .sync_surface_frame("scene.main", &surface_frame(0, true))
            .unwrap());
        assert!(bridge
            .sync_surface_frame("scene.main", &surface_frame(0, false))
            .unwrap());
        assert!(bridge.controls_by_surface["scene.main"].is_empty());
    }

    #[test]
    fn stable_control_identity_publishes_only_the_changed_control_geometry() {
        let mut bridge = ViewportToolbarPointerBridge::new();
        bridge.sync(build_viewport_toolbar_pointer_layout_with_size(
            ["scene.main"],
            zircon_runtime_interface::ui::layout::UiSize::new(640.0, 28.0),
        ));
        let initial = surface_frame_with_control_frame(1, UiFrame::new(12.0, 4.0, 24.0, 20.0));
        assert!(bridge.sync_surface_frame("scene.main", &initial).unwrap());

        let moved = surface_frame_with_control_frame(2, UiFrame::new(48.0, 4.0, 24.0, 20.0));
        assert!(bridge.sync_surface_frame("scene.main", &moved).unwrap());

        assert_eq!(bridge.surface.last_rebuild_report.dirty_node_count, 1);
        assert_eq!(
            bridge
                .surface
                .last_rebuild_report
                .arranged_outer_node_visit_count,
            1
        );
        assert_eq!(
            bridge.controls_by_surface["scene.main"][0].action_key,
            "EnterPlayMode"
        );
    }

    #[test]
    fn changed_control_identity_selects_topology_rebuild() {
        let mut bridge = ViewportToolbarPointerBridge::new();
        bridge.sync(build_viewport_toolbar_pointer_layout_with_size(
            ["scene.main"],
            zircon_runtime_interface::ui::layout::UiSize::new(640.0, 28.0),
        ));
        assert!(bridge
            .sync_surface_frame("scene.main", &surface_frame(1, true))
            .unwrap());

        let mut changed = surface_frame(2, true).as_ref().clone();
        changed.hit_grid = Arc::new(UiHitTestGrid {
            entries: vec![UiHitTestEntry {
                control_id: Some("Select".to_string()),
                ..changed.hit_grid.entries[0].clone()
            }]
            .into(),
            ..changed.hit_grid.as_ref().clone()
        });
        let changed = Arc::new(changed);
        let topology_generation = bridge.surface.tree.layout_order_generation();

        assert!(bridge.sync_surface_frame("scene.main", &changed).unwrap());
        assert!(bridge.surface.tree.layout_order_generation() > topology_generation);
        assert_eq!(
            bridge.controls_by_surface["scene.main"][0].action_key,
            "Select"
        );
    }

    fn surface_frame(generation: u64, with_control: bool) -> Arc<UiSurfaceFrame> {
        let entries = with_control
            .then(|| UiHitTestEntry {
                node_id: UiNodeId::new(1),
                frame: UiFrame::new(12.0, 4.0, 24.0, 20.0),
                clip_frame: UiFrame::new(12.0, 4.0, 24.0, 20.0),
                z_index: 0,
                paint_order: 0,
                control_id: Some("EnterPlayMode".to_string()),
                route_node_index: 0,
            })
            .into_iter()
            .collect::<Vec<_>>();
        Arc::new(UiSurfaceFrame {
            generation,
            tree_id: UiTreeId::new("toolbar"),
            hit_grid: Arc::new(UiHitTestGrid {
                route_nodes: Arc::new(
                    with_control
                        .then(|| {
                            vec![UiHitRouteNode {
                                node_id: UiNodeId::new(1),
                                parent_index: UiHitRouteNode::NO_PARENT_INDEX,
                                effective_input_policy: UiInputPolicy::Receive,
                                pointer_path_visible: true,
                                descendant_pointer_path_visible: true,
                                route_valid: true,
                            }]
                        })
                        .unwrap_or_default(),
                ),
                entries: entries.into(),
                ..UiHitTestGrid::default()
            }),
            ..UiSurfaceFrame::default()
        })
    }

    fn surface_frame_with_control_frame(generation: u64, frame: UiFrame) -> Arc<UiSurfaceFrame> {
        let mut surface_frame = surface_frame(generation, true).as_ref().clone();
        surface_frame.hit_grid = Arc::new(UiHitTestGrid {
            entries: vec![UiHitTestEntry {
                frame,
                clip_frame: frame,
                ..surface_frame.hit_grid.entries[0].clone()
            }]
            .into(),
            ..surface_frame.hit_grid.as_ref().clone()
        });
        Arc::new(surface_frame)
    }
}
