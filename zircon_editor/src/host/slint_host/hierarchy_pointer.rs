use std::collections::BTreeMap;

use zircon_ui::{
    UiAxis, UiContainerKind, UiFrame, UiInputPolicy, UiNodeId, UiNodePath, UiPoint,
    UiPointerDispatchEffect, UiPointerDispatcher, UiPointerEvent, UiPointerEventKind,
    UiScrollState, UiScrollableBoxConfig, UiScrollbarVisibility, UiStateFlags, UiSurface, UiTreeId,
    UiTreeNode,
};

const ROOT_NODE_ID: UiNodeId = UiNodeId::new(1);
const VIEWPORT_NODE_ID: UiNodeId = UiNodeId::new(2);
const ITEM_NODE_ID_BASE: u64 = 100;
const ROW_X: f32 = 8.0;
const ROW_Y: f32 = 8.0;
const ROW_HEIGHT: f32 = 22.0;
const ROW_GAP: f32 = 1.0;
const ROW_WIDTH_INSET: f32 = 16.0;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HierarchyPointerLayout {
    pub pane_width: f32,
    pub pane_height: f32,
    pub node_ids: Vec<String>,
}

impl Default for HierarchyPointerLayout {
    fn default() -> Self {
        Self {
            pane_width: 0.0,
            pane_height: 0.0,
            node_ids: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct HierarchyPointerState {
    pub hovered_item_index: Option<usize>,
    pub scroll_offset: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum HierarchyPointerRoute {
    Node { item_index: usize, node_id: String },
    ListSurface,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HierarchyPointerDispatch {
    pub route: Option<HierarchyPointerRoute>,
    pub state: HierarchyPointerState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum HierarchyPointerTarget {
    Node { item_index: usize, node_id: String },
    ListSurface,
}

#[derive(Default)]
pub(crate) struct HierarchyPointerBridge {
    layout: HierarchyPointerLayout,
    state: HierarchyPointerState,
    surface: UiSurface,
    dispatcher: UiPointerDispatcher,
    targets: BTreeMap<UiNodeId, HierarchyPointerTarget>,
}

impl HierarchyPointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: HierarchyPointerLayout::default(),
            state: HierarchyPointerState::default(),
            surface: UiSurface::new(UiTreeId::new("zircon.editor.hierarchy.pointer")),
            dispatcher: UiPointerDispatcher::default(),
            targets: BTreeMap::new(),
        };
        bridge.rebuild_surface();
        bridge
    }

    pub(crate) fn sync(&mut self, layout: HierarchyPointerLayout, state: HierarchyPointerState) {
        self.layout = layout;
        self.state = state;
        self.clamp_scroll_offset();
        self.rebuild_surface();
    }

    pub(crate) fn handle_click(
        &mut self,
        point: UiPoint,
    ) -> Result<HierarchyPointerDispatch, String> {
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?;
        match route.as_ref() {
            Some(HierarchyPointerTarget::Node { item_index, .. }) => {
                self.state.hovered_item_index = Some(*item_index);
            }
            Some(HierarchyPointerTarget::ListSurface) | None => {
                self.state.hovered_item_index = None;
            }
        }

        Ok(HierarchyPointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
        })
    }

    pub(crate) fn handle_move(
        &mut self,
        point: UiPoint,
    ) -> Result<HierarchyPointerDispatch, String> {
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Move, point))?;
        match route.as_ref() {
            Some(HierarchyPointerTarget::Node { item_index, .. }) => {
                self.state.hovered_item_index = Some(*item_index);
            }
            Some(HierarchyPointerTarget::ListSurface) | None => {
                self.state.hovered_item_index = None;
            }
        }

        Ok(HierarchyPointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
        })
    }

    pub(crate) fn handle_scroll(
        &mut self,
        point: UiPoint,
        delta: f32,
    ) -> Result<HierarchyPointerDispatch, String> {
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

        match route.as_ref() {
            Some(HierarchyPointerTarget::Node { item_index, .. }) => {
                self.state.hovered_item_index = Some(*item_index);
            }
            Some(HierarchyPointerTarget::ListSurface) | None => {}
        }

        Ok(HierarchyPointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
        })
    }

    fn dispatch_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Result<Option<HierarchyPointerTarget>, String> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(&self.dispatcher, event)
            .map_err(|error| error.to_string())?;
        let target_node = dispatch.handled_by.or(dispatch.route.target);
        Ok(target_node.and_then(|node_id| self.targets.get(&node_id).cloned()))
    }

    fn clamp_scroll_offset(&mut self) {
        let max_offset = (content_height(self.layout.node_ids.len())
            - viewport_frame(&self.layout).height)
            .max(0.0);
        self.state.scroll_offset = self.state.scroll_offset.clamp(0.0, max_offset);
    }

    fn rebuild_surface(&mut self) {
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.hierarchy.pointer"));
        let mut dispatcher = UiPointerDispatcher::default();
        let mut targets = BTreeMap::new();

        surface.tree.insert_root(
            UiTreeNode::new(ROOT_NODE_ID, UiNodePath::new("editor.hierarchy.root"))
                .with_frame(UiFrame::new(
                    0.0,
                    0.0,
                    self.layout.pane_width.max(0.0),
                    self.layout.pane_height.max(0.0),
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
                    UiNodePath::new("editor.hierarchy.viewport"),
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
                    content_extent: content_height(self.layout.node_ids.len()),
                })
                .with_state_flags(base_state(true)),
            )
            .expect("hierarchy root must exist");
        register_handled_pointer_node(&mut dispatcher, VIEWPORT_NODE_ID);
        targets.insert(VIEWPORT_NODE_ID, HierarchyPointerTarget::ListSurface);

        let row_width = (self.layout.pane_width - ROW_WIDTH_INSET).max(0.0);
        for (item_index, node_id) in self.layout.node_ids.iter().enumerate() {
            let item_node_id = item_node_id(item_index);
            surface
                .tree
                .insert_child(
                    VIEWPORT_NODE_ID,
                    UiTreeNode::new(
                        item_node_id,
                        UiNodePath::new(format!("editor.hierarchy/item_{item_index}")),
                    )
                    .with_frame(UiFrame::new(
                        ROW_X,
                        ROW_Y + item_index as f32 * (ROW_HEIGHT + ROW_GAP)
                            - self.state.scroll_offset,
                        row_width,
                        ROW_HEIGHT,
                    ))
                    .with_z_index(20 + item_index as i32)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_state(true)),
                )
                .expect("hierarchy viewport must exist");
            register_handled_pointer_node(&mut dispatcher, item_node_id);
            targets.insert(
                item_node_id,
                HierarchyPointerTarget::Node {
                    item_index,
                    node_id: node_id.clone(),
                },
            );
        }

        surface.rebuild();
        self.surface = surface;
        self.dispatcher = dispatcher;
        self.targets = targets;
    }
}

fn viewport_frame(layout: &HierarchyPointerLayout) -> UiFrame {
    UiFrame::new(
        0.0,
        0.0,
        layout.pane_width.max(0.0),
        layout.pane_height.max(0.0),
    )
}

fn content_height(item_count: usize) -> f32 {
    if item_count == 0 {
        0.0
    } else {
        ROW_Y + item_count as f32 * ROW_HEIGHT + (item_count as f32 - 1.0) * ROW_GAP + ROW_Y
    }
}

fn item_node_id(index: usize) -> UiNodeId {
    UiNodeId::new(ITEM_NODE_ID_BASE + index as u64)
}

fn register_handled_pointer_node(dispatcher: &mut UiPointerDispatcher, node_id: UiNodeId) {
    dispatcher.register(node_id, UiPointerEventKind::Move, |_context| {
        UiPointerDispatchEffect::handled()
    });
    dispatcher.register(node_id, UiPointerEventKind::Down, |_context| {
        UiPointerDispatchEffect::handled()
    });
}

fn base_state(interactive: bool) -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: interactive,
        clickable: interactive,
        hoverable: interactive,
        focusable: false,
        pressed: false,
        checked: false,
        dirty: false,
    }
}

fn to_public_route(target: HierarchyPointerTarget) -> HierarchyPointerRoute {
    match target {
        HierarchyPointerTarget::Node {
            item_index,
            node_id,
        } => HierarchyPointerRoute::Node {
            item_index,
            node_id,
        },
        HierarchyPointerTarget::ListSurface => HierarchyPointerRoute::ListSurface,
    }
}
