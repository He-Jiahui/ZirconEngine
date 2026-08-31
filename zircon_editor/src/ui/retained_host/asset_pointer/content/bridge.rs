use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent,
    layout::{UiFrame, UiPoint, UiSize},
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
            self.layout = layout;
            return false;
        }

        self.layout = layout;
        self.state = state;
        self.clamp_scroll_offset();
        self.patch_surface_geometry();
        true
    }

    pub(crate) fn sync_pane_size(&mut self, pane_size: UiSize) -> Option<AssetListPointerState> {
        if self.layout.pane_size == pane_size {
            return None;
        }

        let previous_state = self.state.clone();
        self.layout.pane_size = pane_size;
        self.clamp_scroll_offset();
        self.patch_surface_geometry();
        (self.state != previous_state).then(|| self.state.clone())
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

    pub(crate) fn route_at(&self, point: UiPoint) -> Option<super::AssetPointerContentRoute> {
        self.move_target(point).map(to_public_route)
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
                asset_uuid: self.layout.item_uuid(item_index)?.to_owned(),
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
                self.layout.item_count(),
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
            self.layout.item_count(),
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
            AssetThumbnailGridMetrics::new(self.layout.pane_size.width, self.layout.item_count())
                .content_extent()
        } else {
            self.metrics()
                .list_height(self.layout.folder_ids.len(), self.layout.item_count())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pane_size_patch_preserves_content_projection() {
        let mut bridge = AssetContentListPointerBridge::new();
        let item_ids = (0..1_024)
            .map(|index| format!("asset-{index}"))
            .collect::<Vec<_>>();
        bridge.sync(
            AssetContentListPointerLayout::for_test(
                UiSize::new(320.0, 180.0),
                AssetContentSurfaceProfile::Browser,
                AssetViewMode::List,
                vec![String::from("res://materials")],
                item_ids.clone(),
            ),
            AssetListPointerState::default(),
        );

        let state_change = bridge.sync_pane_size(UiSize::new(640.0, 360.0));

        assert!(state_change.is_none());
        assert_eq!(
            bridge
                .layout
                .items
                .iter()
                .map(|item| &item.uuid)
                .collect::<Vec<_>>(),
            item_ids.iter().collect::<Vec<_>>()
        );
        assert_eq!(
            bridge.layout.folder_ids,
            vec![String::from("res://materials")]
        );
        assert_eq!(bridge.surface_node_count_for_test(), 2);
    }

    #[test]
    fn payload_only_generation_refresh_preserves_pointer_geometry() {
        let mut bridge = AssetContentListPointerBridge::new();
        let layout = AssetContentListPointerLayout::for_test(
            UiSize::new(320.0, 180.0),
            AssetContentSurfaceProfile::Browser,
            AssetViewMode::List,
            Vec::new(),
            vec!["asset-runtime-material".to_string()],
        );
        let source_items = layout.items.clone();
        assert!(bridge.sync(layout, AssetListPointerState::default()));

        let mut changed = source_items[0].clone();
        changed.display_name.push_str(" updated");
        let next_items = source_items
            .replace_existing_items([changed])
            .expect("payload-only replacement must preserve item identity");
        assert!(source_items.shares_item_identity_with(&next_items));
        assert!(!source_items.shares_items_with(&next_items));

        let authority_generation = bridge.surface_authority_generation_for_test();
        let next_layout = AssetContentListPointerLayout {
            items: next_items.clone(),
            ..bridge.layout.clone()
        };
        assert!(!bridge.sync(next_layout, AssetListPointerState::default()));
        assert_eq!(
            bridge.surface_authority_generation_for_test(),
            authority_generation
        );
        assert!(bridge.layout.items.shares_items_with(&next_items));
    }

    #[test]
    fn route_at_resolves_the_stable_asset_uuid_without_dispatching_input() {
        let mut bridge = AssetContentListPointerBridge::new();
        bridge.sync(
            AssetContentListPointerLayout::for_test(
                UiSize::new(320.0, 180.0),
                AssetContentSurfaceProfile::Browser,
                AssetViewMode::Thumbnail,
                Vec::new(),
                vec!["asset-runtime-material".to_string()],
            ),
            AssetListPointerState::default(),
        );

        assert!(matches!(
            bridge.route_at(UiPoint::new(24.0, 24.0)),
            Some(AssetPointerContentRoute::Item { asset_uuid, .. })
                if asset_uuid == "asset-runtime-material"
        ));
    }
}
