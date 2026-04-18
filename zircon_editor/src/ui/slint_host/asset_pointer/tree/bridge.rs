use std::collections::BTreeMap;

use zircon_ui::{
    UiAxis, UiContainerKind, UiInputPolicy, UiNodeId, UiNodePath, UiPoint, UiPointerDispatcher,
    UiPointerEvent, UiPointerEventKind, UiScrollState, UiScrollableBoxConfig,
    UiScrollbarVisibility, UiSurface, UiTreeId, UiTreeNode,
};

use super::dispatch::AssetFolderTreePointerDispatch;
use super::layout::AssetFolderTreePointerLayout;
use super::metrics::{
    content_height, row_width, viewport_frame, viewport_y, ROW_GAP, ROW_HEIGHT, ROW_X, ROW_Y,
};
use super::target::{to_public_route, AssetFolderTreePointerTarget};
use crate::ui::slint_host::asset_pointer::asset_list_pointer_state::AssetListPointerState;
use crate::ui::slint_host::asset_pointer::common::{
    base_state, item_node_id, register_handled_pointer_node, ROOT_NODE_ID, VIEWPORT_NODE_ID,
};

#[derive(Default)]
pub(crate) struct AssetFolderTreePointerBridge {
    layout: AssetFolderTreePointerLayout,
    state: AssetListPointerState,
    surface: UiSurface,
    dispatcher: UiPointerDispatcher,
    targets: BTreeMap<UiNodeId, AssetFolderTreePointerTarget>,
}

impl AssetFolderTreePointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: AssetFolderTreePointerLayout::default(),
            state: AssetListPointerState::default(),
            surface: UiSurface::new(UiTreeId::new("zircon.editor.asset_tree.pointer")),
            dispatcher: UiPointerDispatcher::default(),
            targets: BTreeMap::new(),
        };
        bridge.rebuild_surface();
        bridge
    }

    pub(crate) fn sync(
        &mut self,
        layout: AssetFolderTreePointerLayout,
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
    ) -> Result<AssetFolderTreePointerDispatch, String> {
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?;
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
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Move, point))?;
        self.state.hovered_row_index = match route.as_ref() {
            Some(AssetFolderTreePointerTarget::Folder { row_index, .. }) => Some(*row_index),
            Some(AssetFolderTreePointerTarget::TreeSurface) | None => None,
        };
        Ok(AssetFolderTreePointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
        })
    }

    pub(crate) fn handle_scroll(
        &mut self,
        point: UiPoint,
        delta: f32,
    ) -> Result<AssetFolderTreePointerDispatch, String> {
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
        self.state.hovered_row_index = match route.as_ref() {
            Some(AssetFolderTreePointerTarget::Folder { row_index, .. }) => Some(*row_index),
            Some(AssetFolderTreePointerTarget::TreeSurface) | None => self.state.hovered_row_index,
        };
        Ok(AssetFolderTreePointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
        })
    }

    fn dispatch_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Result<Option<AssetFolderTreePointerTarget>, String> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(&self.dispatcher, event)
            .map_err(|error| error.to_string())?;
        let target_node = dispatch.handled_by.or(dispatch.route.target);
        Ok(target_node.and_then(|node_id| self.targets.get(&node_id).cloned()))
    }

    fn clamp_scroll_offset(&mut self) {
        let max_offset = (content_height(self.layout.folder_ids.len())
            - viewport_frame(&self.layout).height)
            .max(0.0);
        self.state.scroll_offset = self.state.scroll_offset.clamp(0.0, max_offset);
    }

    fn rebuild_surface(&mut self) {
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.asset_tree.pointer"));
        let mut dispatcher = UiPointerDispatcher::default();
        let mut targets = BTreeMap::new();

        surface.tree.insert_root(
            UiTreeNode::new(ROOT_NODE_ID, UiNodePath::new("editor.asset_tree.root"))
                .with_frame(zircon_ui::UiFrame::new(
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
                    UiNodePath::new("editor.asset_tree.viewport"),
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
                    content_extent: content_height(self.layout.folder_ids.len()),
                })
                .with_state_flags(base_state(true)),
            )
            .expect("asset tree root must exist");
        register_handled_pointer_node(&mut dispatcher, VIEWPORT_NODE_ID);
        targets.insert(VIEWPORT_NODE_ID, AssetFolderTreePointerTarget::TreeSurface);

        let row_width = row_width(&self.layout);
        for (row_index, folder_id) in self.layout.folder_ids.iter().enumerate() {
            let node_id = item_node_id(row_index);
            surface
                .tree
                .insert_child(
                    VIEWPORT_NODE_ID,
                    UiTreeNode::new(
                        node_id,
                        UiNodePath::new(format!("editor.asset_tree/row_{row_index}")),
                    )
                    .with_frame(zircon_ui::UiFrame::new(
                        ROW_X,
                        viewport_y() + ROW_Y + row_index as f32 * (ROW_HEIGHT + ROW_GAP)
                            - self.state.scroll_offset,
                        row_width,
                        ROW_HEIGHT,
                    ))
                    .with_z_index(20 + row_index as i32)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_state(true)),
                )
                .expect("asset tree viewport must exist");
            register_handled_pointer_node(&mut dispatcher, node_id);
            targets.insert(
                node_id,
                AssetFolderTreePointerTarget::Folder {
                    row_index,
                    folder_id: folder_id.clone(),
                },
            );
        }

        surface.rebuild();
        self.surface = surface;
        self.dispatcher = dispatcher;
        self.targets = targets;
    }
}
