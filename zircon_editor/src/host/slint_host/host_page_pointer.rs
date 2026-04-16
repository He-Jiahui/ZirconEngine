use std::collections::BTreeMap;

use zircon_ui::{
    UiFrame, UiInputPolicy, UiNodeId, UiNodePath, UiPoint, UiPointerDispatchEffect,
    UiPointerDispatcher, UiPointerEvent, UiPointerEventKind, UiStateFlags, UiSurface, UiTreeId,
    UiTreeNode,
};

use crate::{WorkbenchChromeMetrics, WorkbenchViewModel};

const ROOT_NODE_ID: UiNodeId = UiNodeId::new(1);
const STRIP_NODE_ID: UiNodeId = UiNodeId::new(2);
const TAB_NODE_ID_BASE: u64 = 100;

const STRIP_X: f32 = 8.0;
const STRIP_Y: f32 = 1.0;
const TAB_MIN_WIDTH: f32 = 68.0;
const TAB_HEIGHT: f32 = 22.0;
const TAB_GAP: f32 = 4.0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorkbenchHostPagePointerItem {
    pub page_id: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct WorkbenchHostPagePointerLayout {
    pub strip_frame: UiFrame,
    pub items: Vec<WorkbenchHostPagePointerItem>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WorkbenchHostPagePointerRoute {
    Tab { item_index: usize, page_id: String },
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WorkbenchHostPagePointerDispatch {
    pub route: Option<WorkbenchHostPagePointerRoute>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum WorkbenchHostPagePointerTarget {
    Tab { item_index: usize, page_id: String },
}

#[derive(Default)]
pub(crate) struct WorkbenchHostPagePointerBridge {
    layout: WorkbenchHostPagePointerLayout,
    measured_frames: Vec<Option<UiFrame>>,
    surface: UiSurface,
    dispatcher: UiPointerDispatcher,
    targets: BTreeMap<UiNodeId, WorkbenchHostPagePointerTarget>,
}

impl WorkbenchHostPagePointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: WorkbenchHostPagePointerLayout {
                strip_frame: UiFrame::default(),
                items: Vec::new(),
            },
            measured_frames: Vec::new(),
            surface: UiSurface::new(UiTreeId::new("zircon.editor.host_page.pointer")),
            dispatcher: UiPointerDispatcher::default(),
            targets: BTreeMap::new(),
        };
        bridge.rebuild_surface();
        bridge
    }

    pub(crate) fn sync(&mut self, layout: WorkbenchHostPagePointerLayout) {
        self.layout = layout;
        self.measured_frames.resize(self.layout.items.len(), None);
        if self.measured_frames.len() > self.layout.items.len() {
            self.measured_frames.truncate(self.layout.items.len());
        }
        self.rebuild_surface();
    }

    pub(crate) fn handle_click(
        &mut self,
        item_index: usize,
        tab_x: f32,
        tab_width: f32,
        point: UiPoint,
    ) -> Result<WorkbenchHostPagePointerDispatch, String> {
        if item_index < self.measured_frames.len() {
            self.measured_frames[item_index] = Some(UiFrame::new(
                self.layout.strip_frame.x + tab_x,
                self.layout.strip_frame.y + STRIP_Y,
                tab_width.max(TAB_MIN_WIDTH),
                TAB_HEIGHT,
            ));
            self.rebuild_surface();
        }
        let point = UiPoint::new(
            self.layout.strip_frame.x + point.x,
            self.layout.strip_frame.y + point.y,
        );
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?;
        Ok(WorkbenchHostPagePointerDispatch {
            route: route.map(to_public_route),
        })
    }

    fn dispatch_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Result<Option<WorkbenchHostPagePointerTarget>, String> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(&self.dispatcher, event)
            .map_err(|error| error.to_string())?;
        let target_node = dispatch.handled_by.or(dispatch.route.target);
        Ok(target_node.and_then(|node_id| self.targets.get(&node_id).cloned()))
    }

    fn rebuild_surface(&mut self) {
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.host_page.pointer"));
        let mut dispatcher = UiPointerDispatcher::default();
        let mut targets = BTreeMap::new();

        surface.tree.insert_root(
            UiTreeNode::new(ROOT_NODE_ID, UiNodePath::new("editor.host_page.root"))
                .with_frame(root_frame(&self.layout))
                .with_state_flags(base_state(false)),
        );
        surface
            .tree
            .insert_child(
                ROOT_NODE_ID,
                UiTreeNode::new(STRIP_NODE_ID, UiNodePath::new("editor.host_page.strip"))
                    .with_frame(self.layout.strip_frame)
                    .with_z_index(10)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_state(true)),
            )
            .expect("host page root must exist");

        let mut next_x = self.layout.strip_frame.x + STRIP_X;
        for (item_index, item) in self.layout.items.iter().enumerate() {
            let node_id = UiNodeId::new(TAB_NODE_ID_BASE + item_index as u64);
            let frame = self
                .measured_frames
                .get(item_index)
                .and_then(|frame| *frame)
                .unwrap_or(UiFrame::new(
                    next_x,
                    self.layout.strip_frame.y + STRIP_Y,
                    TAB_MIN_WIDTH,
                    TAB_HEIGHT,
                ));
            next_x = frame.x + frame.width + TAB_GAP;
            surface
                .tree
                .insert_child(
                    STRIP_NODE_ID,
                    UiTreeNode::new(
                        node_id,
                        UiNodePath::new(format!("editor.host_page/tab_{item_index}")),
                    )
                    .with_frame(frame)
                    .with_z_index(20 + item_index as i32)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_state(true)),
                )
                .expect("host page strip must exist");
            register_handled_pointer_node(&mut dispatcher, node_id);
            targets.insert(
                node_id,
                WorkbenchHostPagePointerTarget::Tab {
                    item_index,
                    page_id: item.page_id.clone(),
                },
            );
        }

        surface.rebuild();
        self.surface = surface;
        self.dispatcher = dispatcher;
        self.targets = targets;
    }
}

pub(crate) fn build_workbench_host_page_pointer_layout(
    model: &WorkbenchViewModel,
    metrics: &WorkbenchChromeMetrics,
) -> WorkbenchHostPagePointerLayout {
    let estimated_width = STRIP_X * 2.0
        + model.host_strip.pages.len() as f32 * TAB_MIN_WIDTH
        + model.host_strip.pages.len().saturating_sub(1) as f32 * TAB_GAP;
    WorkbenchHostPagePointerLayout {
        strip_frame: UiFrame::new(
            0.0,
            metrics.top_bar_height + metrics.separator_thickness,
            estimated_width.max(1.0),
            metrics.host_bar_height.max(TAB_HEIGHT),
        ),
        items: model
            .host_strip
            .pages
            .iter()
            .map(|page| WorkbenchHostPagePointerItem {
                page_id: page.id.0.clone(),
            })
            .collect(),
    }
}

fn root_frame(layout: &WorkbenchHostPagePointerLayout) -> UiFrame {
    UiFrame::new(
        0.0,
        0.0,
        (layout.strip_frame.x + layout.strip_frame.width).max(1.0),
        (layout.strip_frame.y + layout.strip_frame.height).max(1.0),
    )
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

fn to_public_route(target: WorkbenchHostPagePointerTarget) -> WorkbenchHostPagePointerRoute {
    match target {
        WorkbenchHostPagePointerTarget::Tab {
            item_index,
            page_id,
        } => WorkbenchHostPagePointerRoute::Tab {
            item_index,
            page_id,
        },
    }
}
