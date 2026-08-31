use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent,
    layout::{UiFrame, UiPoint, UiSize},
    surface::UiPointerEventKind,
};

use super::dispatch::AssetReferenceListPointerDispatch;
use super::layout::AssetReferenceListPointerLayout;
use super::metrics::{list_height, row_width, viewport_frame, viewport_y, ROW_GAP, ROW_HEIGHT};
use super::target::{hovered_row_from_target, to_public_route, AssetReferenceListPointerTarget};
use crate::ui::retained_host::asset_pointer::asset_list_pointer_state::AssetListPointerState;
use crate::ui::retained_host::asset_pointer::common::{
    row_index_at_point, AssetPointerSurfaceAuthority,
};

pub(crate) struct AssetReferenceListPointerBridge {
    layout: AssetReferenceListPointerLayout,
    state: AssetListPointerState,
    authority: AssetPointerSurfaceAuthority,
}

impl AssetReferenceListPointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: AssetReferenceListPointerLayout::default(),
            state: AssetListPointerState::default(),
            authority: AssetPointerSurfaceAuthority::new(
                "zircon.editor.asset_reference.pointer",
                "editor.asset_reference.root",
                "editor.asset_reference.viewport",
                UiFrame::default(),
                UiFrame::default(),
            ),
        };
        bridge.patch_surface_geometry();
        bridge
    }

    pub(crate) fn sync(
        &mut self,
        layout: AssetReferenceListPointerLayout,
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
    ) -> Result<AssetReferenceListPointerDispatch, String> {
        self.handle_press(point)
    }

    pub(crate) fn handle_press(
        &mut self,
        point: UiPoint,
    ) -> Result<AssetReferenceListPointerDispatch, String> {
        let route = self.dispatch_target(UiPointerEventKind::Down, point)?;
        self.state.hovered_row_index = hovered_row_from_target(route.as_ref());
        Ok(AssetReferenceListPointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
        })
    }

    pub(crate) fn handle_move(
        &mut self,
        point: UiPoint,
    ) -> Result<AssetReferenceListPointerDispatch, String> {
        let route = self.move_target(point);
        self.state.hovered_row_index = hovered_row_from_target(route.as_ref());
        Ok(AssetReferenceListPointerDispatch {
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

    pub(crate) fn clear_hovered_row(&mut self) -> bool {
        self.state.hovered_row_index.take().is_some()
    }

    pub(crate) fn handle_scroll(
        &mut self,
        point: UiPoint,
        delta: f32,
    ) -> Result<AssetReferenceListPointerDispatch, String> {
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
        Ok(AssetReferenceListPointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
        })
    }

    fn dispatch_target(
        &mut self,
        kind: UiPointerEventKind,
        point: UiPoint,
    ) -> Result<Option<AssetReferenceListPointerTarget>, String> {
        let routed = self
            .authority
            .dispatch_event(UiPointerEvent::new(kind, point))?;
        Ok(routed.then(|| self.move_target(point)).flatten())
    }

    fn move_target(&self, point: UiPoint) -> Option<AssetReferenceListPointerTarget> {
        self.move_row_index(point)
            .map(|row_index| AssetReferenceListPointerTarget::Item {
                row_index,
                asset_uuid: self.layout.entries[row_index].asset_uuid.clone(),
            })
            .or_else(|| {
                viewport_frame(&self.layout)
                    .contains_point(point)
                    .then_some(AssetReferenceListPointerTarget::ListSurface)
            })
    }

    fn move_row_index(&self, point: UiPoint) -> Option<usize> {
        let viewport = viewport_frame(&self.layout);
        if !viewport.contains_point(point) || point.x > row_width(&self.layout) {
            return None;
        }
        let content_y = point.y + self.state.scroll_offset;
        let row_index = row_index_at_point(
            content_y,
            viewport_y(),
            ROW_HEIGHT,
            ROW_GAP,
            self.layout.entries.len(),
        )?;
        self.layout.entries[row_index]
            .known_project_asset
            .then_some(row_index)
    }

    fn clamp_scroll_offset(&mut self) {
        let max_offset =
            (list_height(self.layout.entries.len()) - viewport_frame(&self.layout).height).max(0.0);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::asset_pointer::AssetReferenceListPointerEntry;

    #[test]
    fn pane_size_patch_preserves_reference_projection() {
        let mut bridge = AssetReferenceListPointerBridge::new();
        let entries = (0..1_024)
            .map(|index| AssetReferenceListPointerEntry {
                asset_uuid: format!("asset-{index}"),
                known_project_asset: index % 2 == 0,
            })
            .collect::<Vec<_>>();
        bridge.sync(
            AssetReferenceListPointerLayout {
                pane_size: UiSize::new(240.0, 180.0),
                entries: entries.clone(),
            },
            AssetListPointerState::default(),
        );

        let state_change = bridge.sync_pane_size(UiSize::new(480.0, 360.0));

        assert!(state_change.is_none());
        assert_eq!(bridge.layout.entries, entries);
        assert_eq!(bridge.surface_node_count_for_test(), 2);
    }
}
