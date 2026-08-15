use zircon_runtime::ui::surface::{UiInvalidationReason, UiSurface};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{AxisConstraint, BoxConstraints, Position, StretchMode, UiFrame, UiSize},
};

use super::constants::{
    CLOSE_EXTENT, CLOSE_X_OFFSET, CLOSE_Y_OFFSET, STRIP_Y, TAB_GAP, TAB_HEIGHT,
};
use super::helper::{close_node_id, root_frame, tab_min_width, tab_node_id};
use super::host_document_tab_pointer_bridge::HostDocumentTabPointerBridge;

impl HostDocumentTabPointerBridge {
    pub(in crate::ui::retained_host::document_tab_pointer) fn update_measured_frame(
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
            .ok_or_else(|| format!("Unknown document tab surface {surface_key}"))?;
        let Some(frames) = self.measured_frames.get_mut(surface_key) else {
            return Err(format!("Missing measured frame store for {surface_key}"));
        };
        if item_index >= frames.len() {
            return Err(format!(
                "Document tab index {item_index} is outside surface {surface_key}"
            ));
        }
        let measured_frame = UiFrame::new(
            surface.strip_frame.x + tab_x,
            surface.strip_frame.y + STRIP_Y,
            tab_width.max(tab_min_width(surface, item_index)),
            TAB_HEIGHT,
        );
        if frames[item_index] == Some(measured_frame) {
            return Ok(());
        }
        frames[item_index] = Some(measured_frame);
        let patches = projected_frame_patches(surface, frames, item_index);
        if !patch_nodes_exist(&self.surface, surface_index, &patches) {
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
            if patch.closeable {
                patch_fixed_frame(
                    &mut self.surface,
                    close_node_id(surface_index, patch.item_index),
                    surface.strip_frame,
                    UiFrame::new(
                        patch.frame.x + patch.frame.width - CLOSE_X_OFFSET,
                        patch.frame.y + CLOSE_Y_OFFSET,
                        CLOSE_EXTENT,
                        CLOSE_EXTENT,
                    ),
                )?;
            }
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
    closeable: bool,
}

fn projected_frame_patches(
    surface: &super::host_document_tab_pointer_surface::HostDocumentTabPointerSurface,
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
                tab_min_width(surface, index),
                TAB_HEIGHT,
            )
        };
        next_x = frame.right() + TAB_GAP;
        patches.push(MeasuredFramePatch {
            item_index: index,
            frame,
            closeable: surface.items[index].closeable,
        });
    }
    patches
}

fn patch_nodes_exist(
    surface: &UiSurface,
    surface_index: usize,
    patches: &[MeasuredFramePatch],
) -> bool {
    patches.iter().all(|patch| {
        surface
            .tree
            .node(tab_node_id(surface_index, patch.item_index))
            .is_some()
            && (!patch.closeable
                || surface
                    .tree
                    .node(close_node_id(surface_index, patch.item_index))
                    .is_some())
    })
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
        .expect("measured document tab patch prevalidates every node");
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
