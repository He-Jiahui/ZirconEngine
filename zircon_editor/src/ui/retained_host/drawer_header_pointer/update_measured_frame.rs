use zircon_runtime::ui::surface::{UiInvalidationReason, UiSurface};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{AxisConstraint, BoxConstraints, Position, StretchMode, UiFrame, UiSize},
};

use super::constants::{STRIP_Y, TAB_GAP, TAB_HEIGHT, TAB_MIN_WIDTH, TAB_NODE_ID_BASE};
use super::host_drawer_header_pointer_bridge::HostDrawerHeaderPointerBridge;
use super::root_frame::root_frame;

impl HostDrawerHeaderPointerBridge {
    pub(super) fn update_measured_frame(
        &mut self,
        surface_key: &str,
        item_index: usize,
        tab_x: f32,
        tab_width: f32,
    ) -> Result<(), String> {
        let (surface_index, surface) = self
            .layout
            .surfaces
            .iter()
            .enumerate()
            .find(|(_, surface)| surface.key == surface_key)
            .ok_or_else(|| format!("Unknown drawer header surface {surface_key}"))?;
        let Some(frames) = self.measured_frames.get_mut(surface_key) else {
            return Err(format!("Missing measured frame store for {surface_key}"));
        };
        if item_index >= frames.len() {
            return Err(format!(
                "Drawer header index {item_index} is outside surface {surface_key}"
            ));
        }
        let measured_frame = UiFrame::new(
            surface.strip_frame.x + tab_x,
            surface.strip_frame.y + STRIP_Y,
            tab_width.max(TAB_MIN_WIDTH),
            TAB_HEIGHT,
        );
        if frames[item_index] == Some(measured_frame) {
            return Ok(());
        }
        frames[item_index] = Some(measured_frame);
        let patches = projected_frame_patches(surface, frames, item_index);
        if !patches.iter().all(|patch| {
            self.surface
                .tree
                .node(tab_node_id(surface_index, patch.item_index))
                .is_some()
        }) {
            self.rebuild_surface();
            return Ok(());
        }
        for patch in patches {
            patch_fixed_frame(
                &mut self.surface,
                tab_node_id(surface_index, patch.item_index),
                surface.strip_frame,
                patch.frame,
            )?;
        }
        let root = root_frame(&self.layout);
        self.surface
            .rebuild_dirty(UiSize::new(root.width, root.height))
            .map_err(|error| error.to_string())?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct MeasuredFramePatch {
    item_index: usize,
    frame: UiFrame,
}

fn projected_frame_patches(
    surface: &super::host_drawer_header_pointer_surface::HostDrawerHeaderPointerSurface,
    measured_frames: &[Option<UiFrame>],
    item_index: usize,
) -> Vec<MeasuredFramePatch> {
    let mut patches = Vec::new();
    let mut next_x = measured_frames[item_index]
        .expect("changed measured frame must be present")
        .right()
        + TAB_GAP;
    for index in item_index..surface.items.len() {
        let frame = if index == item_index {
            measured_frames[index].expect("changed measured frame must be present")
        } else if measured_frames[index].is_some() {
            break;
        } else {
            UiFrame::new(
                next_x,
                surface.strip_frame.y + STRIP_Y,
                TAB_MIN_WIDTH,
                TAB_HEIGHT,
            )
        };
        next_x = frame.right() + TAB_GAP;
        patches.push(MeasuredFramePatch {
            item_index: index,
            frame,
        });
    }
    patches
}

fn tab_node_id(surface_index: usize, item_index: usize) -> UiNodeId {
    UiNodeId::new(TAB_NODE_ID_BASE + surface_index as u64 * 100 + item_index as u64)
}

fn patch_fixed_frame(
    surface: &mut UiSurface,
    node_id: UiNodeId,
    parent_frame: UiFrame,
    frame: UiFrame,
) -> Result<(), String> {
    let node = surface
        .tree
        .node_mut(node_id)
        .expect("measured drawer header patch prevalidates every node");
    node.constraints = fixed_constraints(frame);
    node.position = Position::new(frame.x - parent_frame.x, frame.y - parent_frame.y);
    surface
        .invalidate_node(node_id, UiInvalidationReason::Layout)
        .map_err(|error| error.to_string())
}

fn fixed_constraints(frame: UiFrame) -> BoxConstraints {
    BoxConstraints {
        width: fixed_axis(frame.width),
        height: fixed_axis(frame.height),
    }
}

fn fixed_axis(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size,
        max: size,
        preferred: size,
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}
