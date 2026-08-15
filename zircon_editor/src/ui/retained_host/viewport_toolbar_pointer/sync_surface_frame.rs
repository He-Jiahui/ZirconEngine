use zircon_runtime_interface::ui::{layout::UiFrame, surface::UiSurfaceFrame};

use super::route_for_control::route_for_control;
use super::viewport_toolbar_pointer_bridge::ViewportToolbarPointerBridge;
use super::viewport_toolbar_pointer_control::ViewportToolbarPointerControl;

impl ViewportToolbarPointerBridge {
    pub(crate) fn sync_surface_frame(
        &mut self,
        surface_key: &str,
        surface_frame: &UiSurfaceFrame,
    ) -> Result<bool, String> {
        let surface_origin = self
            .surface_layout(surface_key)
            .map(|surface| surface.frame)
            .ok_or_else(|| format!("Unknown viewport toolbar surface {surface_key}"))?;
        if surface_frame.generation != 0
            && self
                .applied_surface_frames
                .get(surface_key)
                .is_some_and(|applied| *applied == (surface_frame.generation, surface_origin))
        {
            return Ok(false);
        }
        let controls = surface_frame
            .arranged_tree
            .nodes
            .iter()
            .filter_map(|node| {
                let control_id = node.control_id.as_deref()?;
                route_for_control(surface_key, control_id).ok()?;
                Some(ViewportToolbarPointerControl {
                    action_key: control_id.to_string(),
                    frame: UiFrame::new(
                        surface_origin.x + node.frame.x,
                        surface_origin.y + node.frame.y,
                        node.frame.width.max(1.0),
                        node.frame.height.max(1.0),
                    ),
                })
            })
            .collect::<Vec<_>>();

        if surface_frame.generation == 0 {
            self.applied_surface_frames.remove(surface_key);
        } else {
            self.applied_surface_frames.insert(
                surface_key.to_string(),
                (surface_frame.generation, surface_origin),
            );
        }

        if self
            .controls_by_surface
            .get(surface_key)
            .is_some_and(|existing| *existing == controls)
        {
            return Ok(false);
        }

        self.controls_by_surface
            .insert(surface_key.to_string(), controls);
        self.rebuild_surface();
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ui::{
        event_ui::{UiNodeId, UiNodePath, UiTreeId},
        layout::UiFrame,
        surface::{UiArrangedNode, UiArrangedTree, UiSurfaceFrame},
        tree::{UiInputPolicy, UiVisibility},
    };

    use super::super::{
        build_viewport_toolbar_pointer_layout_with_size, ViewportToolbarPointerBridge,
    };

    #[test]
    fn stable_surface_generation_skips_reprojecting_toolbar_controls() {
        let mut bridge = ViewportToolbarPointerBridge::new();
        assert!(bridge.sync(build_viewport_toolbar_pointer_layout_with_size(
            ["scene.main"],
            zircon_runtime_interface::ui::layout::UiSize::new(640.0, 28.0),
        )));

        let frame = surface_frame(7, true);
        assert!(bridge.sync_surface_frame("scene.main", &frame).unwrap());
        assert_eq!(bridge.controls_by_surface["scene.main"].len(), 1);

        let inconsistent_same_generation = surface_frame(7, false);
        assert!(!bridge
            .sync_surface_frame("scene.main", &inconsistent_same_generation)
            .unwrap());
        assert_eq!(bridge.controls_by_surface["scene.main"].len(), 1);

        let next_generation = surface_frame(8, false);
        assert!(bridge
            .sync_surface_frame("scene.main", &next_generation)
            .unwrap());
        assert!(bridge.controls_by_surface["scene.main"].is_empty());
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

    fn surface_frame(generation: u64, with_control: bool) -> UiSurfaceFrame {
        let nodes = with_control
            .then(|| UiArrangedNode {
                node_id: UiNodeId::new(1),
                node_path: UiNodePath::new("toolbar.play"),
                parent: None,
                children: Vec::new(),
                frame: UiFrame::new(12.0, 4.0, 24.0, 20.0),
                clip_frame: UiFrame::new(12.0, 4.0, 24.0, 20.0),
                z_index: 0,
                paint_order: 0,
                visibility: UiVisibility::Visible,
                input_policy: UiInputPolicy::Receive,
                pointer_events: Default::default(),
                enabled: true,
                clickable: true,
                hoverable: true,
                focusable: false,
                clip_to_bounds: false,
                control_id: Some("EnterPlayMode".to_string()),
                slot: None,
            })
            .into_iter()
            .collect::<Vec<_>>();
        UiSurfaceFrame {
            generation,
            tree_id: UiTreeId::new("toolbar"),
            arranged_tree: UiArrangedTree {
                tree_id: UiTreeId::new("toolbar"),
                roots: nodes.iter().map(|node| node.node_id).collect(),
                draw_order: nodes.iter().map(|node| node.node_id).collect(),
                nodes,
                ..UiArrangedTree::default()
            },
            ..UiSurfaceFrame::default()
        }
    }
}
