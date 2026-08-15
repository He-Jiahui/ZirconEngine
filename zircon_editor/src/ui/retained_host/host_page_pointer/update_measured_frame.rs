use zircon_runtime::ui::surface::{UiInvalidationReason, UiSurface};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{AxisConstraint, BoxConstraints, Position, StretchMode, UiFrame, UiSize},
};

use crate::ui::workbench::page_tabs::main_page_tab_close_frame;

use super::host_page_pointer_bridge::HostPagePointerBridge;
use super::root_frame::root_frame;
use super::tab_node_id::{close_node_id, tab_node_id};
use super::HostPagePointerError;

impl HostPagePointerBridge {
    pub(super) fn update_measured_frame(
        &mut self,
        item_index: usize,
        tab_x: f32,
        tab_width: f32,
    ) -> Result<Option<UiFrame>, HostPagePointerError> {
        if !tab_x.is_finite() || !tab_width.is_finite() || tab_width <= 0.0 {
            return Err(HostPagePointerError::InvalidTabFrame {
                item_index,
                x: tab_x,
                width: tab_width,
            });
        }
        let Some(tab_position) = self
            .tab_positions_by_item
            .get(item_index)
            .copied()
            .flatten()
        else {
            return Ok(None);
        };
        let measured_frame = {
            let tab = &self.layout.tabs[tab_position];
            UiFrame::new(
                self.layout.strip_frame.x + tab_x,
                tab.frame.y,
                tab_width,
                tab.frame.height,
            )
        };
        let closeable = self
            .layout
            .items
            .get(item_index)
            .is_some_and(|item| item.close_instance_id.is_some());
        let measured_close_frame = closeable.then(|| main_page_tab_close_frame(measured_frame));
        let current_frames = self
            .measured_frames
            .get(tab_position)
            .copied()
            .flatten()
            .unwrap_or_else(|| {
                let tab = &self.layout.tabs[tab_position];
                (tab.frame, tab.close_frame)
            });
        if current_frames == (measured_frame, measured_close_frame) {
            return Ok(Some(measured_frame));
        }
        self.measured_frames[tab_position] = Some((measured_frame, measured_close_frame));

        let tab_node_id = tab_node_id(item_index);
        let close_node_id = close_node_id(item_index);
        let nodes_exist = self.surface.tree.node(tab_node_id).is_some()
            && (measured_close_frame.is_none() || self.surface.tree.node(close_node_id).is_some());
        if !nodes_exist {
            self.rebuild_surface();
            return Ok(Some(measured_frame));
        }
        patch_fixed_frame(
            &mut self.surface,
            tab_node_id,
            self.layout.strip_frame,
            measured_frame,
        )?;
        if let Some(close_frame) = measured_close_frame {
            patch_fixed_frame(
                &mut self.surface,
                close_node_id,
                self.layout.strip_frame,
                close_frame,
            )?;
        }
        let root = root_frame(&self.layout);
        self.surface
            .rebuild_dirty(UiSize::new(root.width, root.height))?;
        Ok(Some(measured_frame))
    }
}

fn patch_fixed_frame(
    surface: &mut UiSurface,
    node_id: UiNodeId,
    parent_frame: UiFrame,
    frame: UiFrame,
) -> Result<(), HostPagePointerError> {
    let node = surface
        .tree
        .node_mut(node_id)
        .expect("measured host page patch prevalidates every node");
    node.constraints = fixed_constraints(frame);
    node.position = Position::new(frame.x - parent_frame.x, frame.y - parent_frame.y);
    surface.invalidate_node(node_id, UiInvalidationReason::Layout)?;
    Ok(())
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
