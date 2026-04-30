use std::collections::BTreeMap;

use zircon_runtime::ui::{
    dispatch::{UiPointerDispatcher, UiPointerEvent},
    event_ui::UiNodeId,
    event_ui::UiNodePath,
    event_ui::UiTreeId,
    layout::UiAxis,
    layout::UiContainerKind,
    layout::UiPoint,
    layout::UiScrollState,
    layout::UiScrollableBoxConfig,
    layout::UiScrollbarVisibility,
    surface::UiPointerEventKind,
    surface::UiSurface,
    tree::UiInputPolicy,
    tree::UiTreeNode,
};

use super::dispatch::AssetContentListPointerDispatch;
use super::layout::AssetContentListPointerLayout;
use super::metrics::{
    folder_height, item_height, list_height, row_width, viewport_frame, viewport_y, ROW_GAP, ROW_X,
    ROW_Y,
};
use super::target::{hovered_row_from_target, to_public_route, AssetContentListPointerTarget};
use crate::ui::slint_host::asset_pointer::asset_list_pointer_state::AssetListPointerState;
use crate::ui::slint_host::asset_pointer::common::{
    base_state, item_node_id, register_handled_pointer_node, ROOT_NODE_ID, VIEWPORT_NODE_ID,
};

#[derive(Default)]
pub(crate) struct AssetContentListPointerBridge {
    layout: AssetContentListPointerLayout,
    state: AssetListPointerState,
    surface: UiSurface,
    dispatcher: UiPointerDispatcher,
    targets: BTreeMap<UiNodeId, AssetContentListPointerTarget>,
}

impl AssetContentListPointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: AssetContentListPointerLayout::default(),
            state: AssetListPointerState::default(),
            surface: UiSurface::new(UiTreeId::new("zircon.editor.asset_content.pointer")),
            dispatcher: UiPointerDispatcher::default(),
            targets: BTreeMap::new(),
        };
        bridge.rebuild_surface();
        bridge
    }

    pub(crate) fn sync(
        &mut self,
        layout: AssetContentListPointerLayout,
        state: AssetListPointerState,
    ) {
        self.layout = layout;
        self.state = state;
        self.clamp_scroll_offset();
        self.rebuild_surface();
    }

    pub(crate) fn handle_click(
        &mut self,
        point: UiPoint,
    ) -> Result<AssetContentListPointerDispatch, String> {
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?;
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
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?;
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
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Move, point))?;
        self.state.hovered_row_index = hovered_row_from_target(route.as_ref());
        Ok(AssetContentListPointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
        })
    }

    pub(crate) fn handle_scroll(
        &mut self,
        point: UiPoint,
        delta: f32,
    ) -> Result<AssetContentListPointerDispatch, String> {
        let route = self.dispatch_event(
            UiPointerEvent::new(UiPointerEventKind::Scroll, point).with_scroll_delta(delta),
        )?;
        if let Some(viewport) = self.surface.tree.node(VIEWPORT_NODE_ID) {
            let offset = viewport.scroll_state.unwrap_or_default().offset;
            if (self.state.scroll_offset - offset).abs() > f32::EPSILON {
                self.state.scroll_offset = offset;
                self.rebuild_surface();
            }
        }
        if let Some(row_index) = hovered_row_from_target(route.as_ref()) {
            self.state.hovered_row_index = Some(row_index);
        }
        Ok(AssetContentListPointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
        })
    }

    fn dispatch_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Result<Option<AssetContentListPointerTarget>, String> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(&self.dispatcher, event)
            .map_err(|error| error.to_string())?;
        let target_node = dispatch.handled_by.or(dispatch.route.target);
        Ok(target_node.and_then(|node_id| self.targets.get(&node_id).cloned()))
    }

    fn clamp_scroll_offset(&mut self) {
        let max_offset = (list_height(&self.layout) - viewport_frame(&self.layout).height).max(0.0);
        self.state.scroll_offset = self.state.scroll_offset.clamp(0.0, max_offset);
    }

    fn rebuild_surface(&mut self) {
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.asset_content.pointer"));
        let mut dispatcher = UiPointerDispatcher::default();
        let mut targets = BTreeMap::new();

        surface.tree.insert_root(
            UiTreeNode::new(ROOT_NODE_ID, UiNodePath::new("editor.asset_content.root"))
                .with_frame(zircon_runtime::ui::layout::UiFrame::new(
                    0.0,
                    0.0,
                    self.layout.pane_size.width.max(0.0),
                    self.layout.pane_size.height.max(0.0),
                ))
                .with_state_flags(base_state(false)),
        );

        let viewport = viewport_frame(&self.layout);
        surface
            .tree
            .insert_child(
                ROOT_NODE_ID,
                UiTreeNode::new(
                    VIEWPORT_NODE_ID,
                    UiNodePath::new("editor.asset_content.viewport"),
                )
                .with_frame(viewport)
                .with_z_index(10)
                .with_input_policy(UiInputPolicy::Receive)
                .with_clip_to_bounds(true)
                .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                    axis: UiAxis::Vertical,
                    gap: 0.0,
                    scrollbar_visibility: UiScrollbarVisibility::Auto,
                    virtualization: None,
                }))
                .with_scroll_state(UiScrollState {
                    offset: self.state.scroll_offset,
                    viewport_extent: viewport.height.max(0.0),
                    content_extent: list_height(&self.layout),
                })
                .with_state_flags(base_state(true)),
            )
            .expect("asset content root must exist");
        register_handled_pointer_node(&mut dispatcher, VIEWPORT_NODE_ID);
        targets.insert(
            VIEWPORT_NODE_ID,
            AssetContentListPointerTarget::ContentSurface,
        );

        let row_width = row_width(&self.layout);
        let mut row_y = viewport_y() + ROW_Y - self.state.scroll_offset;
        let mut row_index = 0usize;

        for (folder_index, folder_id) in self.layout.folder_ids.iter().enumerate() {
            let node_id = item_node_id(row_index);
            let row_height = folder_height(self.layout.view_mode);
            surface
                .tree
                .insert_child(
                    VIEWPORT_NODE_ID,
                    UiTreeNode::new(
                        node_id,
                        UiNodePath::new(format!("editor.asset_content/folder_{folder_index}")),
                    )
                    .with_frame(zircon_runtime::ui::layout::UiFrame::new(
                        ROW_X, row_y, row_width, row_height,
                    ))
                    .with_z_index(20 + row_index as i32)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_state(true)),
                )
                .expect("asset content viewport must exist");
            register_handled_pointer_node(&mut dispatcher, node_id);
            targets.insert(
                node_id,
                AssetContentListPointerTarget::Folder {
                    row_index,
                    folder_index,
                    folder_id: folder_id.clone(),
                },
            );
            row_index += 1;
            row_y += row_height + ROW_GAP;
        }

        for (item_index, asset_uuid) in self.layout.item_ids.iter().enumerate() {
            let node_id = item_node_id(row_index);
            let row_height = item_height(self.layout.view_mode);
            surface
                .tree
                .insert_child(
                    VIEWPORT_NODE_ID,
                    UiTreeNode::new(
                        node_id,
                        UiNodePath::new(format!("editor.asset_content/item_{item_index}")),
                    )
                    .with_frame(zircon_runtime::ui::layout::UiFrame::new(
                        ROW_X, row_y, row_width, row_height,
                    ))
                    .with_z_index(20 + row_index as i32)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_state(true)),
                )
                .expect("asset content viewport must exist");
            register_handled_pointer_node(&mut dispatcher, node_id);
            targets.insert(
                node_id,
                AssetContentListPointerTarget::Item {
                    row_index,
                    item_index,
                    asset_uuid: asset_uuid.clone(),
                },
            );
            row_index += 1;
            row_y += row_height + ROW_GAP;
        }

        surface.rebuild();
        self.surface = surface;
        self.dispatcher = dispatcher;
        self.targets = targets;
    }
}
