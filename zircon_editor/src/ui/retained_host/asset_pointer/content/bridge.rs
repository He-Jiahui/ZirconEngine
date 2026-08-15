use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent,
    layout::{UiFrame, UiPoint},
    surface::UiPointerEventKind,
};

use super::dispatch::AssetContentListPointerDispatch;
use super::layout::AssetContentListPointerLayout;
use super::target::{hovered_row_from_target, to_public_route, AssetContentListPointerTarget};
use crate::ui::retained_host::asset_pointer::asset_list_pointer_state::AssetListPointerState;
use crate::ui::retained_host::asset_pointer::common::{
    row_index_at_point, AssetPointerSurfaceAuthority,
};
use crate::ui::workbench::asset_content_layout::{
    AssetContentLayoutMetrics, AssetContentSurfaceProfile, AssetThumbnailGridMetrics,
};
use crate::ui::workbench::snapshot::AssetViewMode;

#[derive(Clone, Copy)]
enum AssetContentMoveHit {
    Folder(usize),
    Item(usize),
    ContentSurface,
}

impl AssetContentMoveHit {
    fn hovered_row_index(self, folder_count: usize) -> Option<usize> {
        match self {
            Self::Folder(folder_index) => Some(folder_index),
            Self::Item(item_index) => Some(folder_count + item_index),
            Self::ContentSurface => None,
        }
    }
}

pub(crate) struct AssetContentListPointerBridge {
    layout: AssetContentListPointerLayout,
    state: AssetListPointerState,
    authority: AssetPointerSurfaceAuthority,
}

impl AssetContentListPointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: AssetContentListPointerLayout::default(),
            state: AssetListPointerState::default(),
            authority: AssetPointerSurfaceAuthority::new(
                "zircon.editor.asset_content.pointer",
                "editor.asset_content.root",
                "editor.asset_content.viewport",
                UiFrame::default(),
                UiFrame::default(),
            ),
        };
        bridge.patch_surface_geometry();
        bridge
    }

    pub(crate) fn sync(
        &mut self,
        layout: AssetContentListPointerLayout,
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
    ) -> Result<AssetContentListPointerDispatch, String> {
        let route = self.dispatch_target(UiPointerEventKind::Down, point)?;
        self.state.hovered_row_index = hovered_row_from_target(route.as_ref());
        Ok(AssetContentListPointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
        })
    }

    pub(crate) fn handle_press(
        &mut self,
        point: UiPoint,
    ) -> Result<AssetContentListPointerDispatch, String> {
        let route = self.dispatch_target(UiPointerEventKind::Down, point)?;
        self.state.hovered_row_index = hovered_row_from_target(route.as_ref());
        Ok(AssetContentListPointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
        })
    }

    pub(crate) fn handle_move(
        &mut self,
        point: UiPoint,
    ) -> Result<AssetContentListPointerDispatch, String> {
        let route = self.move_target(point);
        self.state.hovered_row_index = hovered_row_from_target(route.as_ref());
        Ok(AssetContentListPointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
        })
    }

    pub(crate) fn update_hovered_row(&mut self, point: UiPoint) -> Option<AssetListPointerState> {
        let hovered_row_index = self
            .move_hit(point)
            .and_then(|hit| hit.hovered_row_index(self.layout.folder_ids.len()));
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
    ) -> Result<AssetContentListPointerDispatch, String> {
        let routed = self.authority.dispatch_event(
            UiPointerEvent::new(UiPointerEventKind::Scroll, point).with_scroll_delta(delta),
        )?;
        if routed && delta.is_finite() {
            self.state.scroll_offset += delta;
            self.clamp_scroll_offset();
        }
        let route = routed.then(|| self.move_target(point)).flatten();
        if let Some(row_index) = hovered_row_from_target(route.as_ref()) {
            self.state.hovered_row_index = Some(row_index);
        }
        Ok(AssetContentListPointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
        })
    }

    fn dispatch_target(
        &mut self,
        kind: UiPointerEventKind,
        point: UiPoint,
    ) -> Result<Option<AssetContentListPointerTarget>, String> {
        let routed = self
            .authority
            .dispatch_event(UiPointerEvent::new(kind, point))?;
        Ok(routed.then(|| self.move_target(point)).flatten())
    }

    fn move_target(&self, point: UiPoint) -> Option<AssetContentListPointerTarget> {
        match self.move_hit(point)? {
            AssetContentMoveHit::Folder(folder_index) => {
                Some(AssetContentListPointerTarget::Folder {
                    row_index: folder_index,
                    folder_index,
                    folder_id: self.layout.folder_ids[folder_index].clone(),
                })
            }
            AssetContentMoveHit::Item(item_index) => Some(AssetContentListPointerTarget::Item {
                row_index: self.layout.folder_ids.len() + item_index,
                item_index,
                asset_uuid: self.layout.item_ids[item_index].clone(),
            }),
            AssetContentMoveHit::ContentSurface => {
                Some(AssetContentListPointerTarget::ContentSurface)
            }
        }
    }

    fn move_hit(&self, point: UiPoint) -> Option<AssetContentMoveHit> {
        let viewport = self.viewport_frame();
        if !viewport.contains_point(point) {
            return None;
        }

        if self.is_browser_thumbnail_grid() {
            let grid = AssetThumbnailGridMetrics::new(
                self.layout.pane_size.width,
                self.layout.item_ids.len(),
            );
            let content_point = UiPoint::new(point.x, point.y + self.state.scroll_offset);
            return grid
                .item_index_at_point(content_point)
                .map(AssetContentMoveHit::Item)
                .or(Some(AssetContentMoveHit::ContentSurface));
        }

        let metrics = self.metrics();
        let row_width = metrics.row_width(self.layout.pane_size.width);
        let content_x = point.x;
        let content_y = point.y + self.state.scroll_offset;
        if content_x < metrics.row_x || content_x > metrics.row_x + row_width {
            return Some(AssetContentMoveHit::ContentSurface);
        }

        if let Some(folder_index) = row_index_at_point(
            content_y,
            metrics.first_row_y(),
            metrics.folder_height,
            metrics.row_gap,
            self.layout.folder_ids.len(),
        ) {
            return Some(AssetContentMoveHit::Folder(folder_index));
        }

        let item_start_y = metrics.first_row_y()
            + self.layout.folder_ids.len() as f32 * (metrics.folder_height + metrics.row_gap);
        if let Some(item_index) = row_index_at_point(
            content_y,
            item_start_y,
            metrics.item_height,
            metrics.row_gap,
            self.layout.item_ids.len(),
        ) {
            return Some(AssetContentMoveHit::Item(item_index));
        }

        Some(AssetContentMoveHit::ContentSurface)
    }

    fn clamp_scroll_offset(&mut self) {
        let viewport = self.viewport_frame();
        let max_offset = (self.content_extent() - viewport.height).max(0.0);
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
            self.viewport_frame(),
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

    fn metrics(&self) -> AssetContentLayoutMetrics {
        AssetContentLayoutMetrics::for_surface(self.layout.surface_profile, self.layout.view_mode)
    }

    fn is_browser_thumbnail_grid(&self) -> bool {
        self.layout.surface_profile == AssetContentSurfaceProfile::Browser
            && self.layout.view_mode == AssetViewMode::Thumbnail
    }

    fn viewport_frame(&self) -> UiFrame {
        if self.is_browser_thumbnail_grid() {
            UiFrame::new(
                0.0,
                0.0,
                self.layout.pane_size.width.max(0.0),
                self.layout.pane_size.height.max(0.0),
            )
        } else {
            self.metrics().viewport_frame(self.layout.pane_size)
        }
    }

    fn content_extent(&self) -> f32 {
        if self.is_browser_thumbnail_grid() {
            AssetThumbnailGridMetrics::new(self.layout.pane_size.width, self.layout.item_ids.len())
                .content_extent()
        } else {
            self.metrics()
                .list_height(self.layout.folder_ids.len(), self.layout.item_ids.len())
        }
    }
}
