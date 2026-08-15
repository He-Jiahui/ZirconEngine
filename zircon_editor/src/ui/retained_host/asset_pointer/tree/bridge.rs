use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent,
    layout::{UiFrame, UiPoint},
    surface::UiPointerEventKind,
};

use super::dispatch::AssetFolderTreePointerDispatch;
use super::layout::AssetFolderTreePointerLayout;
use super::metrics::{
    content_height, row_width, viewport_frame, viewport_y, ROW_GAP, ROW_HEIGHT, ROW_X, ROW_Y,
};
use super::target::{to_public_route, AssetFolderTreePointerTarget};
use crate::ui::retained_host::asset_pointer::asset_list_pointer_state::AssetListPointerState;
use crate::ui::retained_host::asset_pointer::common::{
    row_index_at_point, AssetPointerSurfaceAuthority,
};

pub(crate) struct AssetFolderTreePointerBridge {
    layout: AssetFolderTreePointerLayout,
    state: AssetListPointerState,
    authority: AssetPointerSurfaceAuthority,
}

impl AssetFolderTreePointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: AssetFolderTreePointerLayout::default(),
            state: AssetListPointerState::default(),
            authority: AssetPointerSurfaceAuthority::new(
                "zircon.editor.asset_tree.pointer",
                "editor.asset_tree.root",
                "editor.asset_tree.viewport",
                UiFrame::default(),
                UiFrame::default(),
            ),
        };
        bridge.patch_surface_geometry();
        bridge
    }

    pub(crate) fn sync(
        &mut self,
        layout: AssetFolderTreePointerLayout,
        state: AssetListPointerState,
    ) -> bool {
        if self.layout == layout && self.state == state {
            return false;
        }

        self.layout = layout;
        self.state = state;
        self.clamp_scroll_offset();
        self.patch_surface_geometry();
        true
    }

    pub(crate) fn handle_click(
        &mut self,
        point: UiPoint,
    ) -> Result<AssetFolderTreePointerDispatch, String> {
        let route = self.dispatch_target(UiPointerEventKind::Down, point)?;
        self.state.hovered_row_index = match route.as_ref() {
            Some(AssetFolderTreePointerTarget::Folder { row_index, .. }) => Some(*row_index),
            Some(AssetFolderTreePointerTarget::TreeSurface) | None => None,
        };
        Ok(AssetFolderTreePointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
        })
    }

    pub(crate) fn handle_move(
        &mut self,
        point: UiPoint,
    ) -> Result<AssetFolderTreePointerDispatch, String> {
        let route = self.move_target(point);
        self.state.hovered_row_index = match route.as_ref() {
            Some(AssetFolderTreePointerTarget::Folder { row_index, .. }) => Some(*row_index),
            Some(AssetFolderTreePointerTarget::TreeSurface) | None => None,
        };
        Ok(AssetFolderTreePointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
        })
    }

    pub(crate) fn update_hovered_row(&mut self, point: UiPoint) -> Option<AssetListPointerState> {
        let hovered_row_index = self.move_row_index(point);
        if self.state.hovered_row_index == hovered_row_index {
            return None;
        }
        self.state.hovered_row_index = hovered_row_index;
        Some(self.state.clone())
    }

    pub(crate) fn handle_scroll(
        &mut self,
        point: UiPoint,
        delta: f32,
    ) -> Result<AssetFolderTreePointerDispatch, String> {
        let routed = self.authority.dispatch_event(
            UiPointerEvent::new(UiPointerEventKind::Scroll, point).with_scroll_delta(delta),
        )?;
        if routed && delta.is_finite() {
            self.state.scroll_offset += delta;
            self.clamp_scroll_offset();
        }
        let route = routed.then(|| self.move_target(point)).flatten();
        self.state.hovered_row_index = match route.as_ref() {
            Some(AssetFolderTreePointerTarget::Folder { row_index, .. }) => Some(*row_index),
            Some(AssetFolderTreePointerTarget::TreeSurface) | None => self.state.hovered_row_index,
        };
        Ok(AssetFolderTreePointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
        })
    }

    fn dispatch_target(
        &mut self,
        kind: UiPointerEventKind,
        point: UiPoint,
    ) -> Result<Option<AssetFolderTreePointerTarget>, String> {
        let routed = self
            .authority
            .dispatch_event(UiPointerEvent::new(kind, point))?;
        Ok(routed.then(|| self.move_target(point)).flatten())
    }

    fn move_target(&self, point: UiPoint) -> Option<AssetFolderTreePointerTarget> {
        self.move_row_index(point)
            .map(|row_index| AssetFolderTreePointerTarget::Folder {
                row_index,
                folder_id: self.layout.folder_ids[row_index].clone(),
            })
            .or_else(|| {
                viewport_frame(&self.layout)
                    .contains_point(point)
                    .then_some(AssetFolderTreePointerTarget::TreeSurface)
            })
    }

    fn move_row_index(&self, point: UiPoint) -> Option<usize> {
        let viewport = viewport_frame(&self.layout);
        let row_width = row_width(&self.layout);
        if !viewport.contains_point(point) || point.x < ROW_X || point.x > ROW_X + row_width {
            return None;
        }
        let content_y = point.y + self.state.scroll_offset;
        row_index_at_point(
            content_y,
            viewport_y() + ROW_Y,
            ROW_HEIGHT,
            ROW_GAP,
            self.layout.folder_ids.len(),
        )
    }

    fn clamp_scroll_offset(&mut self) {
        let max_offset = (content_height(self.layout.folder_ids.len())
            - viewport_frame(&self.layout).height)
            .max(0.0);
        self.state.scroll_offset = self.state.scroll_offset.clamp(0.0, max_offset);
    }

    fn patch_surface_geometry(&mut self) {
        self.authority.patch_geometry(
            UiFrame::new(
                0.0,
                0.0,
                self.layout.pane_size.width.max(0.0),
                self.layout.pane_size.height.max(0.0),
            ),
            viewport_frame(&self.layout),
        );
    }

    #[cfg(test)]
    pub(crate) fn surface_node_count_for_test(&self) -> usize {
        self.authority.node_count()
    }

    #[cfg(test)]
    pub(crate) const fn surface_authority_generation_for_test(&self) -> u64 {
        self.authority.generation()
    }
}
