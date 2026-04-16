use std::collections::BTreeMap;

use zircon_ui::{
    UiFrame, UiInputPolicy, UiNodeId, UiNodePath, UiPoint, UiPointerDispatchEffect,
    UiPointerDispatcher, UiPointerEvent, UiPointerEventKind, UiStateFlags, UiSurface, UiTreeId,
    UiTreeNode,
};

use crate::{WorkbenchChromeMetrics, WorkbenchShellGeometry, WorkbenchViewModel};

const ROOT_NODE_ID: UiNodeId = UiNodeId::new(1);
const SURFACE_NODE_ID_BASE: u64 = 10;
const TAB_NODE_ID_BASE: u64 = 100;

const STRIP_X: f32 = 6.0;
const STRIP_Y: f32 = 2.0;
const TAB_GAP: f32 = 4.0;
const TAB_HEIGHT: f32 = 22.0;
const TAB_MIN_WIDTH: f32 = 68.0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorkbenchDrawerHeaderPointerItem {
    pub slot: String,
    pub instance_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WorkbenchDrawerHeaderPointerSurface {
    pub key: String,
    pub strip_frame: UiFrame,
    pub items: Vec<WorkbenchDrawerHeaderPointerItem>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct WorkbenchDrawerHeaderPointerLayout {
    pub surfaces: Vec<WorkbenchDrawerHeaderPointerSurface>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WorkbenchDrawerHeaderPointerRoute {
    Tab {
        surface_key: String,
        item_index: usize,
        slot: String,
        instance_id: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WorkbenchDrawerHeaderPointerDispatch {
    pub route: Option<WorkbenchDrawerHeaderPointerRoute>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum WorkbenchDrawerHeaderPointerTarget {
    Tab {
        surface_key: String,
        item_index: usize,
        slot: String,
        instance_id: String,
    },
}

#[derive(Default)]
pub(crate) struct WorkbenchDrawerHeaderPointerBridge {
    layout: WorkbenchDrawerHeaderPointerLayout,
    measured_frames: BTreeMap<String, Vec<Option<UiFrame>>>,
    surface: UiSurface,
    dispatcher: UiPointerDispatcher,
    targets: BTreeMap<UiNodeId, WorkbenchDrawerHeaderPointerTarget>,
}

impl WorkbenchDrawerHeaderPointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: WorkbenchDrawerHeaderPointerLayout::default(),
            measured_frames: BTreeMap::new(),
            surface: UiSurface::new(UiTreeId::new("zircon.editor.drawer_header.pointer")),
            dispatcher: UiPointerDispatcher::default(),
            targets: BTreeMap::new(),
        };
        bridge.rebuild_surface();
        bridge
    }

    pub(crate) fn sync(&mut self, layout: WorkbenchDrawerHeaderPointerLayout) {
        self.layout = layout;
        self.measured_frames = self
            .layout
            .surfaces
            .iter()
            .map(|surface| (surface.key.clone(), vec![None; surface.items.len()]))
            .collect();
        self.rebuild_surface();
    }

    pub(crate) fn handle_click(
        &mut self,
        surface_key: &str,
        item_index: usize,
        tab_x: f32,
        tab_width: f32,
        point: UiPoint,
    ) -> Result<WorkbenchDrawerHeaderPointerDispatch, String> {
        self.update_measured_frame(surface_key, item_index, tab_x, tab_width)?;
        let point = self.global_point(surface_key, point)?;
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?;
        Ok(WorkbenchDrawerHeaderPointerDispatch {
            route: route.map(to_public_route),
        })
    }

    fn update_measured_frame(
        &mut self,
        surface_key: &str,
        item_index: usize,
        tab_x: f32,
        tab_width: f32,
    ) -> Result<(), String> {
        let surface = self
            .layout
            .surfaces
            .iter()
            .find(|surface| surface.key == surface_key)
            .ok_or_else(|| format!("Unknown drawer header surface {surface_key}"))?;
        let Some(frames) = self.measured_frames.get_mut(surface_key) else {
            return Err(format!("Missing measured frame store for {surface_key}"));
        };
        if item_index >= frames.len() {
            return Err(format!(
                "Drawer header index {item_index} is outside surface {surface_key}"
            ));
        }
        frames[item_index] = Some(UiFrame::new(
            surface.strip_frame.x + tab_x,
            surface.strip_frame.y + STRIP_Y,
            tab_width.max(TAB_MIN_WIDTH),
            TAB_HEIGHT,
        ));
        self.rebuild_surface();
        Ok(())
    }

    fn global_point(&self, surface_key: &str, point: UiPoint) -> Result<UiPoint, String> {
        let strip_frame = self
            .layout
            .surfaces
            .iter()
            .find(|surface| surface.key == surface_key)
            .map(|surface| surface.strip_frame)
            .ok_or_else(|| format!("Unknown drawer header surface {surface_key}"))?;
        Ok(UiPoint::new(
            strip_frame.x + point.x,
            strip_frame.y + point.y,
        ))
    }

    fn dispatch_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Result<Option<WorkbenchDrawerHeaderPointerTarget>, String> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(&self.dispatcher, event)
            .map_err(|error| error.to_string())?;
        let target_node = dispatch.handled_by.or(dispatch.route.target);
        Ok(target_node.and_then(|node_id| self.targets.get(&node_id).cloned()))
    }

    fn rebuild_surface(&mut self) {
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.drawer_header.pointer"));
        let mut dispatcher = UiPointerDispatcher::default();
        let mut targets = BTreeMap::new();

        surface.tree.insert_root(
            UiTreeNode::new(ROOT_NODE_ID, UiNodePath::new("editor.drawer_header.root"))
                .with_frame(root_frame(&self.layout))
                .with_state_flags(base_state(false)),
        );

        for (surface_index, surface_layout) in self.layout.surfaces.iter().enumerate() {
            let surface_node_id = UiNodeId::new(SURFACE_NODE_ID_BASE + surface_index as u64);
            surface
                .tree
                .insert_child(
                    ROOT_NODE_ID,
                    UiTreeNode::new(
                        surface_node_id,
                        UiNodePath::new(format!("editor.drawer_header/{}", surface_layout.key)),
                    )
                    .with_frame(surface_layout.strip_frame)
                    .with_z_index(10 + surface_index as i32)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_state(true)),
                )
                .expect("drawer header root must exist");

            let measured = self
                .measured_frames
                .get(surface_layout.key.as_str())
                .cloned()
                .unwrap_or_else(|| vec![None; surface_layout.items.len()]);
            let mut next_x = surface_layout.strip_frame.x + STRIP_X;
            for (item_index, item) in surface_layout.items.iter().enumerate() {
                let frame = measured
                    .get(item_index)
                    .and_then(|frame| *frame)
                    .unwrap_or_else(|| {
                        UiFrame::new(
                            next_x,
                            surface_layout.strip_frame.y + STRIP_Y,
                            TAB_MIN_WIDTH,
                            TAB_HEIGHT,
                        )
                    });
                next_x = frame.x + frame.width + TAB_GAP;
                let node_id = UiNodeId::new(
                    TAB_NODE_ID_BASE + surface_index as u64 * 100 + item_index as u64,
                );
                surface
                    .tree
                    .insert_child(
                        surface_node_id,
                        UiTreeNode::new(
                            node_id,
                            UiNodePath::new(format!(
                                "editor.drawer_header/{}/tab_{item_index}",
                                surface_layout.key
                            )),
                        )
                        .with_frame(frame)
                        .with_z_index(20 + item_index as i32)
                        .with_input_policy(UiInputPolicy::Receive)
                        .with_state_flags(base_state(true)),
                    )
                    .expect("drawer header surface must exist");
                register_handled_pointer_node(&mut dispatcher, node_id);
                targets.insert(
                    node_id,
                    WorkbenchDrawerHeaderPointerTarget::Tab {
                        surface_key: surface_layout.key.clone(),
                        item_index,
                        slot: item.slot.clone(),
                        instance_id: item.instance_id.clone(),
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

pub(crate) fn build_workbench_drawer_header_pointer_layout(
    model: &WorkbenchViewModel,
    geometry: &WorkbenchShellGeometry,
    metrics: &WorkbenchChromeMetrics,
) -> WorkbenchDrawerHeaderPointerLayout {
    let mut surfaces = Vec::new();
    if let Some(surface) = build_surface(
        "left",
        geometry.region_frame(crate::ShellRegionId::Left),
        model,
        &[
            crate::ActivityDrawerSlot::LeftTop,
            crate::ActivityDrawerSlot::LeftBottom,
        ],
        metrics,
        true,
    ) {
        surfaces.push(surface);
    }
    if let Some(surface) = build_surface(
        "right",
        geometry.region_frame(crate::ShellRegionId::Right),
        model,
        &[
            crate::ActivityDrawerSlot::RightTop,
            crate::ActivityDrawerSlot::RightBottom,
        ],
        metrics,
        false,
    ) {
        surfaces.push(surface);
    }
    if let Some(surface) = build_surface(
        "bottom",
        geometry.region_frame(crate::ShellRegionId::Bottom),
        model,
        &[
            crate::ActivityDrawerSlot::BottomLeft,
            crate::ActivityDrawerSlot::BottomRight,
        ],
        metrics,
        false,
    ) {
        surfaces.push(surface);
    }

    WorkbenchDrawerHeaderPointerLayout { surfaces }
}

fn build_surface(
    key: &str,
    region_frame: UiFrame,
    model: &WorkbenchViewModel,
    slots: &[crate::ActivityDrawerSlot],
    metrics: &WorkbenchChromeMetrics,
    side_with_rail: bool,
) -> Option<WorkbenchDrawerHeaderPointerSurface> {
    let items = slots
        .iter()
        .filter_map(|slot| model.tool_windows.get(slot))
        .flat_map(|stack| {
            stack
                .tabs
                .iter()
                .map(move |tab| WorkbenchDrawerHeaderPointerItem {
                    slot: drawer_slot_key(stack.slot).to_string(),
                    instance_id: tab.instance_id.0.clone(),
                })
        })
        .collect::<Vec<_>>();
    if items.is_empty() {
        return None;
    }

    let strip_frame = if side_with_rail {
        if region_frame.width <= metrics.rail_width + metrics.separator_thickness {
            return None;
        }
        UiFrame::new(
            region_frame.x + metrics.rail_width + metrics.separator_thickness,
            region_frame.y,
            (region_frame.width - metrics.rail_width - metrics.separator_thickness).max(0.0),
            metrics.panel_header_height,
        )
    } else if key == "right" {
        if region_frame.width <= metrics.rail_width + metrics.separator_thickness {
            return None;
        }
        UiFrame::new(
            region_frame.x,
            region_frame.y,
            (region_frame.width - metrics.rail_width - metrics.separator_thickness).max(0.0),
            metrics.panel_header_height,
        )
    } else {
        UiFrame::new(
            region_frame.x,
            region_frame.y,
            region_frame.width.max(0.0),
            metrics.panel_header_height,
        )
    };

    Some(WorkbenchDrawerHeaderPointerSurface {
        key: key.to_string(),
        strip_frame,
        items,
    })
}

fn root_frame(layout: &WorkbenchDrawerHeaderPointerLayout) -> UiFrame {
    let max_x = layout
        .surfaces
        .iter()
        .map(|surface| surface.strip_frame.x + surface.strip_frame.width)
        .fold(1.0_f32, f32::max);
    let max_y = layout
        .surfaces
        .iter()
        .map(|surface| surface.strip_frame.y + surface.strip_frame.height)
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

fn drawer_slot_key(slot: crate::ActivityDrawerSlot) -> &'static str {
    match slot {
        crate::ActivityDrawerSlot::LeftTop => "left_top",
        crate::ActivityDrawerSlot::LeftBottom => "left_bottom",
        crate::ActivityDrawerSlot::RightTop => "right_top",
        crate::ActivityDrawerSlot::RightBottom => "right_bottom",
        crate::ActivityDrawerSlot::BottomLeft => "bottom_left",
        crate::ActivityDrawerSlot::BottomRight => "bottom_right",
    }
}

fn to_public_route(
    target: WorkbenchDrawerHeaderPointerTarget,
) -> WorkbenchDrawerHeaderPointerRoute {
    match target {
        WorkbenchDrawerHeaderPointerTarget::Tab {
            surface_key,
            item_index,
            slot,
            instance_id,
        } => WorkbenchDrawerHeaderPointerRoute::Tab {
            surface_key,
            item_index,
            slot,
            instance_id,
        },
    }
}
