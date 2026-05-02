use std::collections::BTreeMap;

use zircon_runtime::ui::{
    dispatch::UiPointerDispatcher, surface::UiSurface, tree::UiRuntimeTreeAccessExt,
};
use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent,
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{
        UiAxis, UiContainerKind, UiFrame, UiPoint, UiScrollState, UiScrollableBoxConfig,
        UiScrollbarVisibility,
    },
    surface::UiPointerEventKind,
    tree::{UiInputPolicy, UiTreeNode},
};

use super::dispatch::AssetReferenceListPointerDispatch;
use super::layout::AssetReferenceListPointerLayout;
use super::metrics::{list_height, row_width, viewport_frame, viewport_y, ROW_GAP, ROW_HEIGHT};
use super::target::{hovered_row_from_target, to_public_route, AssetReferenceListPointerTarget};
use crate::ui::slint_host::asset_pointer::asset_list_pointer_state::AssetListPointerState;
use crate::ui::slint_host::asset_pointer::common::{
    base_state, item_node_id, register_handled_pointer_node, ROOT_NODE_ID, VIEWPORT_NODE_ID,
};

#[derive(Default)]
pub(crate) struct AssetReferenceListPointerBridge {
    layout: AssetReferenceListPointerLayout,
    state: AssetListPointerState,
    surface: UiSurface,
    dispatcher: UiPointerDispatcher,
    targets: BTreeMap<UiNodeId, AssetReferenceListPointerTarget>,
}

impl AssetReferenceListPointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: AssetReferenceListPointerLayout::default(),
            state: AssetListPointerState::default(),
            surface: UiSurface::new(UiTreeId::new("zircon.editor.asset_reference.pointer")),
            dispatcher: UiPointerDispatcher::default(),
            targets: BTreeMap::new(),
        };
        bridge.rebuild_surface();
        bridge
    }

    pub(crate) fn sync(
        &mut self,
        layout: AssetReferenceListPointerLayout,
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
    ) -> Result<AssetReferenceListPointerDispatch, String> {
        self.handle_press(point)
    }

    pub(crate) fn handle_press(
        &mut self,
        point: UiPoint,
    ) -> Result<AssetReferenceListPointerDispatch, String> {
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?;
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
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Move, point))?;
        self.state.hovered_row_index = hovered_row_from_target(route.as_ref());
        Ok(AssetReferenceListPointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
        })
    }

    pub(crate) fn handle_scroll(
        &mut self,
        point: UiPoint,
        delta: f32,
    ) -> Result<AssetReferenceListPointerDispatch, String> {
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
        Ok(AssetReferenceListPointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
        })
    }

    fn dispatch_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Result<Option<AssetReferenceListPointerTarget>, String> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(&self.dispatcher, event)
            .map_err(|error| error.to_string())?;
        let target_node = dispatch.handled_by.or(dispatch.route.target);
        Ok(target_node.and_then(|node_id| self.targets.get(&node_id).cloned()))
    }

    fn clamp_scroll_offset(&mut self) {
        let max_offset =
            (list_height(self.layout.entries.len()) - viewport_frame(&self.layout).height).max(0.0);
        self.state.scroll_offset = self.state.scroll_offset.clamp(0.0, max_offset);
    }

    fn rebuild_surface(&mut self) {
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.asset_reference.pointer"));
        let mut dispatcher = UiPointerDispatcher::default();
        let mut targets = BTreeMap::new();

        surface.tree.insert_root(
            UiTreeNode::new(ROOT_NODE_ID, UiNodePath::new("editor.asset_reference.root"))
                .with_frame(UiFrame::new(
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
                    UiNodePath::new("editor.asset_reference.viewport"),
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
                    content_extent: list_height(self.layout.entries.len()),
                })
                .with_state_flags(base_state(true)),
            )
            .expect("asset reference root must exist");
        register_handled_pointer_node(&mut dispatcher, VIEWPORT_NODE_ID);
        targets.insert(
            VIEWPORT_NODE_ID,
            AssetReferenceListPointerTarget::ListSurface,
        );

        let row_width = row_width(&self.layout);
        for (row_index, entry) in self.layout.entries.iter().enumerate() {
            let node_id = item_node_id(row_index);
            let interactive = entry.known_project_asset;
            let input_policy = if interactive {
                UiInputPolicy::Receive
            } else {
                UiInputPolicy::Ignore
            };
            surface
                .tree
                .insert_child(
                    VIEWPORT_NODE_ID,
                    UiTreeNode::new(
                        node_id,
                        UiNodePath::new(format!("editor.asset_reference/item_{row_index}")),
                    )
                    .with_frame(UiFrame::new(
                        0.0,
                        viewport_y() + row_index as f32 * (ROW_HEIGHT + ROW_GAP)
                            - self.state.scroll_offset,
                        row_width,
                        ROW_HEIGHT,
                    ))
                    .with_z_index(20 + row_index as i32)
                    .with_input_policy(input_policy)
                    .with_state_flags(base_state(interactive)),
                )
                .expect("asset reference viewport must exist");
            if interactive {
                register_handled_pointer_node(&mut dispatcher, node_id);
                targets.insert(
                    node_id,
                    AssetReferenceListPointerTarget::Item {
                        row_index,
                        asset_uuid: entry.asset_uuid.clone(),
                    },
                );
            }
        }

        surface.rebuild();
        self.surface = surface;
        self.dispatcher = dispatcher;
        self.targets = targets;
    }
}
