use std::collections::BTreeMap;

use zircon_ui::{
    UiFrame, UiInputPolicy, UiNodeId, UiNodePath, UiPoint, UiPointerDispatchEffect,
    UiPointerDispatcher, UiPointerEvent, UiPointerEventKind, UiStateFlags, UiSurface, UiTreeId,
    UiTreeNode,
};

use crate::{
    ActivityDrawerSlot, ShellRegionId, WorkbenchChromeMetrics, WorkbenchShellGeometry,
    WorkbenchViewModel,
};

const ROOT_NODE_ID: UiNodeId = UiNodeId::new(1);
const LEFT_STRIP_NODE_ID: UiNodeId = UiNodeId::new(10);
const RIGHT_STRIP_NODE_ID: UiNodeId = UiNodeId::new(20);
const LEFT_BUTTON_NODE_ID_BASE: u64 = 100;
const RIGHT_BUTTON_NODE_ID_BASE: u64 = 200;

const STRIP_X_INSET: f32 = 3.0;
const STRIP_Y_INSET: f32 = 6.0;
const BUTTON_EXTENT: f32 = 30.0;
const BUTTON_GAP: f32 = 2.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WorkbenchActivityRailPointerSide {
    Left,
    Right,
}

impl WorkbenchActivityRailPointerSide {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            _ => Err(format!("Unknown activity rail side {value}")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorkbenchActivityRailPointerItem {
    pub slot: String,
    pub instance_id: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct WorkbenchActivityRailPointerLayout {
    pub left_strip_frame: UiFrame,
    pub left_tabs: Vec<WorkbenchActivityRailPointerItem>,
    pub right_strip_frame: UiFrame,
    pub right_tabs: Vec<WorkbenchActivityRailPointerItem>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WorkbenchActivityRailPointerRoute {
    Button {
        side: WorkbenchActivityRailPointerSide,
        item_index: usize,
        slot: String,
        instance_id: String,
    },
    Strip(WorkbenchActivityRailPointerSide),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WorkbenchActivityRailPointerDispatch {
    pub route: Option<WorkbenchActivityRailPointerRoute>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum WorkbenchActivityRailPointerTarget {
    Button {
        side: WorkbenchActivityRailPointerSide,
        item_index: usize,
        slot: String,
        instance_id: String,
    },
    Strip(WorkbenchActivityRailPointerSide),
}

#[derive(Default)]
pub(crate) struct WorkbenchActivityRailPointerBridge {
    layout: WorkbenchActivityRailPointerLayout,
    surface: UiSurface,
    dispatcher: UiPointerDispatcher,
    targets: BTreeMap<UiNodeId, WorkbenchActivityRailPointerTarget>,
}

impl WorkbenchActivityRailPointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: WorkbenchActivityRailPointerLayout::default(),
            surface: UiSurface::new(UiTreeId::new("zircon.editor.activity_rail.pointer")),
            dispatcher: UiPointerDispatcher::default(),
            targets: BTreeMap::new(),
        };
        bridge.rebuild_surface();
        bridge
    }

    pub(crate) fn sync(&mut self, layout: WorkbenchActivityRailPointerLayout) {
        self.layout = layout;
        self.rebuild_surface();
    }

    pub(crate) fn handle_click(
        &mut self,
        side: WorkbenchActivityRailPointerSide,
        point: UiPoint,
    ) -> Result<WorkbenchActivityRailPointerDispatch, String> {
        let point = self.global_point_for_side(side, point);
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?;
        Ok(WorkbenchActivityRailPointerDispatch {
            route: route.map(to_public_route),
        })
    }

    fn global_point_for_side(
        &self,
        side: WorkbenchActivityRailPointerSide,
        point: UiPoint,
    ) -> UiPoint {
        let frame = match side {
            WorkbenchActivityRailPointerSide::Left => self.layout.left_strip_frame,
            WorkbenchActivityRailPointerSide::Right => self.layout.right_strip_frame,
        };
        UiPoint::new(frame.x + point.x, frame.y + point.y)
    }

    fn dispatch_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Result<Option<WorkbenchActivityRailPointerTarget>, String> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(&self.dispatcher, event)
            .map_err(|error| error.to_string())?;
        let target_node = dispatch.handled_by.or(dispatch.route.target);
        Ok(target_node.and_then(|node_id| self.targets.get(&node_id).cloned()))
    }

    fn rebuild_surface(&mut self) {
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.activity_rail.pointer"));
        let mut dispatcher = UiPointerDispatcher::default();
        let mut targets = BTreeMap::new();

        surface.tree.insert_root(
            UiTreeNode::new(ROOT_NODE_ID, UiNodePath::new("editor.activity_rail.root"))
                .with_frame(root_frame(&self.layout))
                .with_state_flags(base_state(false)),
        );

        insert_strip(
            &mut surface,
            &mut dispatcher,
            &mut targets,
            ROOT_NODE_ID,
            LEFT_STRIP_NODE_ID,
            "editor.activity_rail.left",
            self.layout.left_strip_frame,
            &self.layout.left_tabs,
            WorkbenchActivityRailPointerSide::Left,
        );
        insert_strip(
            &mut surface,
            &mut dispatcher,
            &mut targets,
            ROOT_NODE_ID,
            RIGHT_STRIP_NODE_ID,
            "editor.activity_rail.right",
            self.layout.right_strip_frame,
            &self.layout.right_tabs,
            WorkbenchActivityRailPointerSide::Right,
        );
        surface.rebuild();

        self.surface = surface;
        self.dispatcher = dispatcher;
        self.targets = targets;
    }
}

pub(crate) fn build_workbench_activity_rail_pointer_layout(
    model: &WorkbenchViewModel,
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
) -> WorkbenchActivityRailPointerLayout {
    let left_tabs = collect_tabs(
        model,
        &[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom],
    );
    let right_tabs = collect_tabs(
        model,
        &[
            ActivityDrawerSlot::RightTop,
            ActivityDrawerSlot::RightBottom,
        ],
    );
    let left_region = geometry.region_frame(ShellRegionId::Left);
    let right_region = geometry.region_frame(ShellRegionId::Right);
    let rail_width = metrics.rail_width.max(0.0);

    let left_strip_frame = if left_region.width > 0.0 && !left_tabs.is_empty() {
        UiFrame::new(
            left_region.x,
            left_region.y,
            rail_width.min(left_region.width.max(0.0)),
            left_region.height.max(0.0),
        )
    } else {
        UiFrame::default()
    };
    let right_strip_frame = if right_region.width > 0.0 && !right_tabs.is_empty() {
        UiFrame::new(
            (right_region.x + right_region.width - rail_width).max(right_region.x),
            right_region.y,
            rail_width.min(right_region.width.max(0.0)),
            right_region.height.max(0.0),
        )
    } else {
        UiFrame::default()
    };

    WorkbenchActivityRailPointerLayout {
        left_strip_frame,
        left_tabs,
        right_strip_frame,
        right_tabs,
    }
}

fn collect_tabs(
    model: &WorkbenchViewModel,
    slots: &[ActivityDrawerSlot],
) -> Vec<WorkbenchActivityRailPointerItem> {
    slots
        .iter()
        .filter_map(|slot| model.tool_windows.get(slot))
        .flat_map(|stack| {
            stack
                .tabs
                .iter()
                .map(move |tab| WorkbenchActivityRailPointerItem {
                    slot: drawer_slot_key(stack.slot).to_string(),
                    instance_id: tab.instance_id.0.clone(),
                })
        })
        .collect()
}

fn insert_strip(
    surface: &mut UiSurface,
    dispatcher: &mut UiPointerDispatcher,
    targets: &mut BTreeMap<UiNodeId, WorkbenchActivityRailPointerTarget>,
    root_node_id: UiNodeId,
    strip_node_id: UiNodeId,
    path: &str,
    frame: UiFrame,
    tabs: &[WorkbenchActivityRailPointerItem],
    side: WorkbenchActivityRailPointerSide,
) {
    if frame.width <= 0.0 || frame.height <= 0.0 {
        return;
    }

    surface
        .tree
        .insert_child(
            root_node_id,
            UiTreeNode::new(strip_node_id, UiNodePath::new(path))
                .with_frame(frame)
                .with_z_index(10)
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(base_state(true)),
        )
        .expect("activity rail root must exist");
    register_handled_pointer_node(dispatcher, strip_node_id);
    targets.insert(
        strip_node_id,
        WorkbenchActivityRailPointerTarget::Strip(side),
    );

    for (item_index, tab) in tabs.iter().enumerate() {
        let node_id = strip_button_node_id(side, item_index);
        surface
            .tree
            .insert_child(
                strip_node_id,
                UiTreeNode::new(
                    node_id,
                    UiNodePath::new(format!("{path}/button_{item_index}")),
                )
                .with_frame(UiFrame::new(
                    frame.x + STRIP_X_INSET,
                    frame.y + STRIP_Y_INSET + item_index as f32 * (BUTTON_EXTENT + BUTTON_GAP),
                    BUTTON_EXTENT,
                    BUTTON_EXTENT,
                ))
                .with_z_index(20 + item_index as i32)
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(base_state(true)),
            )
            .expect("activity rail strip must exist");
        register_handled_pointer_node(dispatcher, node_id);
        targets.insert(
            node_id,
            WorkbenchActivityRailPointerTarget::Button {
                side,
                item_index,
                slot: tab.slot.clone(),
                instance_id: tab.instance_id.clone(),
            },
        );
    }
}

fn strip_button_node_id(side: WorkbenchActivityRailPointerSide, index: usize) -> UiNodeId {
    let base = match side {
        WorkbenchActivityRailPointerSide::Left => LEFT_BUTTON_NODE_ID_BASE,
        WorkbenchActivityRailPointerSide::Right => RIGHT_BUTTON_NODE_ID_BASE,
    };
    UiNodeId::new(base + index as u64)
}

fn root_frame(layout: &WorkbenchActivityRailPointerLayout) -> UiFrame {
    let max_x = [layout.left_strip_frame, layout.right_strip_frame]
        .into_iter()
        .map(|frame| frame.x + frame.width)
        .fold(1.0_f32, f32::max);
    let max_y = [layout.left_strip_frame, layout.right_strip_frame]
        .into_iter()
        .map(|frame| frame.y + frame.height)
        .fold(1.0_f32, f32::max);
    UiFrame::new(0.0, 0.0, max_x.max(1.0), max_y.max(1.0))
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

fn drawer_slot_key(slot: ActivityDrawerSlot) -> &'static str {
    match slot {
        ActivityDrawerSlot::LeftTop => "left_top",
        ActivityDrawerSlot::LeftBottom => "left_bottom",
        ActivityDrawerSlot::RightTop => "right_top",
        ActivityDrawerSlot::RightBottom => "right_bottom",
        ActivityDrawerSlot::BottomLeft => "bottom_left",
        ActivityDrawerSlot::BottomRight => "bottom_right",
    }
}

fn to_public_route(
    target: WorkbenchActivityRailPointerTarget,
) -> WorkbenchActivityRailPointerRoute {
    match target {
        WorkbenchActivityRailPointerTarget::Button {
            side,
            item_index,
            slot,
            instance_id,
        } => WorkbenchActivityRailPointerRoute::Button {
            side,
            item_index,
            slot,
            instance_id,
        },
        WorkbenchActivityRailPointerTarget::Strip(side) => {
            WorkbenchActivityRailPointerRoute::Strip(side)
        }
    }
}
