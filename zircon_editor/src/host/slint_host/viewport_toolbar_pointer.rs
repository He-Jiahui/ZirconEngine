use std::collections::{BTreeMap, BTreeSet};

use zircon_ui::{
    UiFrame, UiInputPolicy, UiNodeId, UiNodePath, UiPoint, UiPointerDispatchEffect,
    UiPointerDispatcher, UiPointerEvent, UiPointerEventKind, UiSize, UiStateFlags, UiSurface,
    UiTreeId, UiTreeNode,
};

const ROOT_NODE_ID: UiNodeId = UiNodeId::new(1);
const SURFACE_NODE_ID_BASE: u64 = 10;
const CONTROL_NODE_ID_BASE: u64 = 100;
const SURFACE_VERTICAL_STRIDE: f32 = 64.0;
const SURFACE_WIDTH: f32 = 1024.0;
const SURFACE_HEIGHT: f32 = 32.0;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ViewportToolbarPointerSurface {
    pub key: String,
    pub frame: UiFrame,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ViewportToolbarPointerLayout {
    pub surfaces: Vec<ViewportToolbarPointerSurface>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ViewportToolbarPointerRoute {
    SetTool {
        surface_key: String,
        tool: String,
    },
    SetTransformSpace {
        surface_key: String,
        space: String,
    },
    SetProjectionMode {
        surface_key: String,
        mode: String,
    },
    AlignView {
        surface_key: String,
        orientation: String,
    },
    CycleDisplayMode {
        surface_key: String,
    },
    CycleGridMode {
        surface_key: String,
    },
    CycleTranslateSnap {
        surface_key: String,
    },
    CycleRotateSnapDegrees {
        surface_key: String,
    },
    CycleScaleSnap {
        surface_key: String,
    },
    TogglePreviewLighting {
        surface_key: String,
    },
    TogglePreviewSkybox {
        surface_key: String,
    },
    ToggleGizmosEnabled {
        surface_key: String,
    },
    FrameSelection {
        surface_key: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ViewportToolbarPointerDispatch {
    pub route: Option<ViewportToolbarPointerRoute>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ViewportToolbarPointerTarget {
    route: ViewportToolbarPointerRoute,
}

#[derive(Clone, Debug, PartialEq)]
struct ActiveViewportToolbarControl {
    action_key: String,
    frame: UiFrame,
}

#[derive(Default)]
pub(crate) struct ViewportToolbarPointerBridge {
    layout: ViewportToolbarPointerLayout,
    active_controls: BTreeMap<String, ActiveViewportToolbarControl>,
    surface: UiSurface,
    dispatcher: UiPointerDispatcher,
    targets: BTreeMap<UiNodeId, ViewportToolbarPointerTarget>,
}

impl ViewportToolbarPointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: ViewportToolbarPointerLayout::default(),
            active_controls: BTreeMap::new(),
            surface: UiSurface::new(UiTreeId::new("zircon.editor.viewport_toolbar.pointer")),
            dispatcher: UiPointerDispatcher::default(),
            targets: BTreeMap::new(),
        };
        bridge.rebuild_surface();
        bridge
    }

    pub(crate) fn sync(&mut self, layout: ViewportToolbarPointerLayout) {
        self.layout = layout;
        let valid_surface_keys = self
            .layout
            .surfaces
            .iter()
            .map(|surface| surface.key.clone())
            .collect::<BTreeSet<_>>();
        self.active_controls
            .retain(|surface_key, _| valid_surface_keys.contains(surface_key));
        self.rebuild_surface();
    }

    pub(crate) fn handle_click(
        &mut self,
        surface_key: &str,
        control_id: &str,
        control_x: f32,
        control_y: f32,
        control_width: f32,
        control_height: f32,
        point: UiPoint,
    ) -> Result<ViewportToolbarPointerDispatch, String> {
        let surface_frame = self
            .surface_layout(surface_key)
            .map(|surface| surface.frame)
            .ok_or_else(|| format!("Unknown viewport toolbar surface {surface_key}"))?;
        route_for_control(surface_key, control_id)?;
        self.active_controls.insert(
            surface_key.to_string(),
            ActiveViewportToolbarControl {
                action_key: control_id.to_string(),
                frame: UiFrame::new(
                    surface_frame.x + control_x,
                    surface_frame.y + control_y,
                    control_width.max(1.0),
                    control_height.max(1.0),
                ),
            },
        );
        self.rebuild_surface();

        let point = UiPoint::new(surface_frame.x + point.x, surface_frame.y + point.y);
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?;
        Ok(ViewportToolbarPointerDispatch {
            route: route.map(|target| target.route),
        })
    }

    fn surface_layout(&self, surface_key: &str) -> Option<&ViewportToolbarPointerSurface> {
        self.layout
            .surfaces
            .iter()
            .find(|surface| surface.key == surface_key)
    }

    fn dispatch_event(
        &mut self,
        event: UiPointerEvent,
    ) -> Result<Option<ViewportToolbarPointerTarget>, String> {
        let dispatch = self
            .surface
            .dispatch_pointer_event(&self.dispatcher, event)
            .map_err(|error| error.to_string())?;
        let target_node = dispatch.handled_by.or(dispatch.route.target);
        Ok(target_node.and_then(|node_id| self.targets.get(&node_id).cloned()))
    }

    fn rebuild_surface(&mut self) {
        let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.viewport_toolbar.pointer"));
        let mut dispatcher = UiPointerDispatcher::default();
        let mut targets = BTreeMap::new();

        surface.tree.insert_root(
            UiTreeNode::new(
                ROOT_NODE_ID,
                UiNodePath::new("editor.viewport_toolbar.root"),
            )
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
                        UiNodePath::new(format!("editor.viewport_toolbar/{}", surface_layout.key)),
                    )
                    .with_frame(surface_layout.frame)
                    .with_z_index(10 + surface_index as i32)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_state(true)),
                )
                .expect("viewport toolbar root must exist");

            let Some(active_control) = self.active_controls.get(surface_layout.key.as_str()) else {
                continue;
            };
            let route = route_for_control(surface_layout.key.as_str(), &active_control.action_key)
                .expect("active viewport toolbar action must stay valid");
            let control_node_id = UiNodeId::new(CONTROL_NODE_ID_BASE + surface_index as u64);
            surface
                .tree
                .insert_child(
                    surface_node_id,
                    UiTreeNode::new(
                        control_node_id,
                        UiNodePath::new(format!(
                            "editor.viewport_toolbar/{}/{}",
                            surface_layout.key, active_control.action_key
                        )),
                    )
                    .with_frame(active_control.frame)
                    .with_z_index(100 + surface_index as i32)
                    .with_input_policy(UiInputPolicy::Receive)
                    .with_state_flags(base_state(true)),
                )
                .expect("viewport toolbar surface must exist");
            register_handled_pointer_node(&mut dispatcher, control_node_id);
            targets.insert(control_node_id, ViewportToolbarPointerTarget { route });
        }

        surface.rebuild();
        self.surface = surface;
        self.dispatcher = dispatcher;
        self.targets = targets;
    }
}

pub(crate) fn build_viewport_toolbar_pointer_layout<I, S>(
    surface_keys: I,
) -> ViewportToolbarPointerLayout
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    build_viewport_toolbar_pointer_layout_with_size(
        surface_keys,
        UiSize::new(SURFACE_WIDTH, SURFACE_HEIGHT),
    )
}

pub(crate) fn build_viewport_toolbar_pointer_layout_with_size<I, S>(
    surface_keys: I,
    surface_size: UiSize,
) -> ViewportToolbarPointerLayout
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    ViewportToolbarPointerLayout {
        surfaces: surface_keys
            .into_iter()
            .enumerate()
            .map(|(index, key)| ViewportToolbarPointerSurface {
                key: key.as_ref().to_string(),
                frame: UiFrame::new(
                    0.0,
                    index as f32 * SURFACE_VERTICAL_STRIDE,
                    surface_size.width.max(1.0),
                    surface_size.height.max(1.0),
                ),
            })
            .collect(),
    }
}

fn root_frame(layout: &ViewportToolbarPointerLayout) -> UiFrame {
    let max_x = layout
        .surfaces
        .iter()
        .map(|surface| surface.frame.x + surface.frame.width)
        .fold(1.0_f32, f32::max);
    let max_y = layout
        .surfaces
        .iter()
        .map(|surface| surface.frame.y + surface.frame.height)
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

fn route_for_control(
    surface_key: &str,
    control_id: &str,
) -> Result<ViewportToolbarPointerRoute, String> {
    let surface_key = surface_key.to_string();
    let route = match control_id {
        "tool.drag" => ViewportToolbarPointerRoute::SetTool {
            surface_key,
            tool: "Drag".to_string(),
        },
        "tool.move" => ViewportToolbarPointerRoute::SetTool {
            surface_key,
            tool: "Move".to_string(),
        },
        "tool.rotate" => ViewportToolbarPointerRoute::SetTool {
            surface_key,
            tool: "Rotate".to_string(),
        },
        "tool.scale" => ViewportToolbarPointerRoute::SetTool {
            surface_key,
            tool: "Scale".to_string(),
        },
        "space.local" | "transform.local" => ViewportToolbarPointerRoute::SetTransformSpace {
            surface_key,
            space: "Local".to_string(),
        },
        "space.global" | "transform.global" => ViewportToolbarPointerRoute::SetTransformSpace {
            surface_key,
            space: "Global".to_string(),
        },
        "projection.perspective" => ViewportToolbarPointerRoute::SetProjectionMode {
            surface_key,
            mode: "Perspective".to_string(),
        },
        "projection.orthographic" => ViewportToolbarPointerRoute::SetProjectionMode {
            surface_key,
            mode: "Orthographic".to_string(),
        },
        "align.pos_x" => ViewportToolbarPointerRoute::AlignView {
            surface_key,
            orientation: "PosX".to_string(),
        },
        "align.neg_x" => ViewportToolbarPointerRoute::AlignView {
            surface_key,
            orientation: "NegX".to_string(),
        },
        "align.pos_y" => ViewportToolbarPointerRoute::AlignView {
            surface_key,
            orientation: "PosY".to_string(),
        },
        "align.neg_y" => ViewportToolbarPointerRoute::AlignView {
            surface_key,
            orientation: "NegY".to_string(),
        },
        "align.pos_z" => ViewportToolbarPointerRoute::AlignView {
            surface_key,
            orientation: "PosZ".to_string(),
        },
        "align.neg_z" => ViewportToolbarPointerRoute::AlignView {
            surface_key,
            orientation: "NegZ".to_string(),
        },
        "display.cycle" => ViewportToolbarPointerRoute::CycleDisplayMode { surface_key },
        "grid.cycle" => ViewportToolbarPointerRoute::CycleGridMode { surface_key },
        "snap.translate" | "translate_snap.cycle" => {
            ViewportToolbarPointerRoute::CycleTranslateSnap { surface_key }
        }
        "snap.rotate" | "rotate_snap.cycle" => {
            ViewportToolbarPointerRoute::CycleRotateSnapDegrees { surface_key }
        }
        "snap.scale" | "scale_snap.cycle" => {
            ViewportToolbarPointerRoute::CycleScaleSnap { surface_key }
        }
        "toggle.lighting" | "preview_lighting.toggle" => {
            ViewportToolbarPointerRoute::TogglePreviewLighting { surface_key }
        }
        "toggle.skybox" | "preview_skybox.toggle" => {
            ViewportToolbarPointerRoute::TogglePreviewSkybox { surface_key }
        }
        "toggle.gizmos" | "gizmos.toggle" => {
            ViewportToolbarPointerRoute::ToggleGizmosEnabled { surface_key }
        }
        "frame.selection" | "frame_selection" => {
            ViewportToolbarPointerRoute::FrameSelection { surface_key }
        }
        _ => {
            return Err(format!(
                "Unknown viewport toolbar control {control_id} on surface {surface_key}"
            ));
        }
    };
    Ok(route)
}
