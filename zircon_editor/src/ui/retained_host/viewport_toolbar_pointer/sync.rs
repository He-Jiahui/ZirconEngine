use std::collections::BTreeSet;

use zircon_runtime_interface::ui::layout::{UiFrame, UiSize};

use super::constants::ROOT_NODE_ID;
use super::viewport_toolbar_pointer_bridge::ViewportToolbarPointerBridge;
use super::viewport_toolbar_pointer_layout::ViewportToolbarPointerLayout;
use super::viewport_toolbar_pointer_surface::ViewportToolbarPointerSurface;
use super::{
    rebuild_surface::{
        viewport_toolbar_control_node_id, viewport_toolbar_surface_node_id,
        ViewportToolbarNodeFrameChange, ViewportToolbarSurfaceDelta,
    },
    root_frame::root_frame,
};

impl ViewportToolbarPointerBridge {
    pub(crate) fn sync_single_surface(&mut self, surface_key: &str, surface_size: UiSize) -> bool {
        let surface_frame = UiFrame::new(
            0.0,
            0.0,
            surface_size.width.max(1.0),
            surface_size.height.max(1.0),
        );
        if self
            .layout
            .surfaces
            .as_slice()
            .first()
            .is_some_and(|existing| {
                self.layout.surfaces.len() == 1
                    && existing.key == surface_key
                    && existing.frame == surface_frame
            })
        {
            return false;
        }

        self.sync(ViewportToolbarPointerLayout {
            surfaces: vec![ViewportToolbarPointerSurface {
                key: surface_key.to_string(),
                frame: surface_frame,
            }],
        })
    }

    pub(crate) fn sync(&mut self, layout: ViewportToolbarPointerLayout) -> bool {
        if self.layout == layout {
            return false;
        }

        let topology_changed = self.layout.surfaces.len() != layout.surfaces.len()
            || self
                .layout
                .surfaces
                .iter()
                .zip(&layout.surfaces)
                .any(|(current, next)| current.key != next.key);
        let previous_layout = std::mem::replace(&mut self.layout, layout);
        let valid_surface_keys = self
            .layout
            .surfaces
            .iter()
            .map(|surface| surface.key.as_str())
            .collect::<BTreeSet<_>>();
        self.controls_by_surface
            .retain(|surface_key, _| valid_surface_keys.contains(surface_key.as_str()));
        self.applied_surface_frames
            .retain(|surface_key, _| valid_surface_keys.contains(surface_key.as_str()));
        if topology_changed {
            self.apply_surface_delta(ViewportToolbarSurfaceDelta::Topology);
            return true;
        }

        let mut changes = Vec::new();
        let previous_root_frame = root_frame(&previous_layout);
        let next_root_frame = root_frame(&self.layout);
        push_frame_change(
            &mut changes,
            ROOT_NODE_ID,
            previous_root_frame,
            next_root_frame,
        );
        for (surface_index, (previous, next)) in previous_layout
            .surfaces
            .iter()
            .zip(&self.layout.surfaces)
            .enumerate()
        {
            push_frame_change(
                &mut changes,
                viewport_toolbar_surface_node_id(surface_index),
                previous.frame,
                next.frame,
            );
            let offset_x = next.frame.x - previous.frame.x;
            let offset_y = next.frame.y - previous.frame.y;
            if offset_x != 0.0 || offset_y != 0.0 {
                if let Some(controls) = self.controls_by_surface.get_mut(next.key.as_str()) {
                    for (control_index, control) in controls.iter_mut().enumerate() {
                        control.frame.x += offset_x;
                        control.frame.y += offset_y;
                        changes.push(ViewportToolbarNodeFrameChange {
                            node_id: viewport_toolbar_control_node_id(surface_index, control_index),
                            frame: control.frame,
                        });
                    }
                }
            }
            if let Some((_, applied_origin)) =
                self.applied_surface_frames.get_mut(next.key.as_str())
            {
                *applied_origin = next.frame;
            }
        }
        self.apply_surface_delta(if changes.is_empty() {
            ViewportToolbarSurfaceDelta::NoChange
        } else {
            ViewportToolbarSurfaceDelta::Geometry(changes)
        });
        true
    }
}

fn push_frame_change(
    changes: &mut Vec<ViewportToolbarNodeFrameChange>,
    node_id: zircon_runtime_interface::ui::event_ui::UiNodeId,
    previous: UiFrame,
    next: UiFrame,
) {
    if previous != next {
        changes.push(ViewportToolbarNodeFrameChange {
            node_id,
            frame: next,
        });
    }
}
