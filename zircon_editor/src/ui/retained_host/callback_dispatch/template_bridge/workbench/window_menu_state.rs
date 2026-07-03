use zircon_runtime::ui::{surface::UiSurface, tree::UiRuntimeTreeLayoutExt};
use zircon_runtime_interface::ui::{
    component::UiValue,
    event_ui::UiNodeId,
    layout::{Anchor, AxisConstraint, Pivot, Position, UiFrame},
};

use crate::ui::retained_host::popup_anchor_metrics::{
    clamp_popup_x_to_bounds, toolbar_popup_render_gap, POPUP_EDGE_MARGIN,
};

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;
use super::module_overflow_menu::{
    WORKBENCH_MODULE_OVERFLOW_MENU_CONTROL_ID, WORKBENCH_MODULE_OVERFLOW_TRIGGER_CONTROL_ID,
};

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn apply_workbench_window_menu_action(
        &mut self,
        source_control_id: &str,
        action_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        let Some(target) = toolbar_menu_for_action(source_control_id, action_id) else {
            return Ok(false);
        };
        let open = !self.control_bool(target.menu_control_id, "popup_open");

        for menu in TOOLBAR_WINDOW_MENUS {
            self.set_toolbar_window_menu_open(menu, open && menu == target)?;
        }
        Ok(true)
    }

    pub(super) fn close_workbench_window_menu_control(
        &mut self,
        menu_control_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        let Some(menu) = TOOLBAR_WINDOW_MENUS
            .iter()
            .find(|menu| menu.menu_control_id == menu_control_id)
        else {
            return Ok(false);
        };
        self.set_toolbar_window_menu_open(menu, false)?;
        Ok(true)
    }

    fn set_toolbar_window_menu_open(
        &mut self,
        menu: &ToolbarWindowMenu,
        open: bool,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        if open && menu.menu_control_id == WORKBENCH_MODULE_OVERFLOW_MENU_CONTROL_ID {
            self.refresh_workbench_module_overflow_menu_items()?;
        }
        if open {
            self.apply_toolbar_window_menu_anchor(menu)?;
        }
        self.set_control_active(menu.trigger_control_id, open)?;
        self.set_visible(menu.menu_control_id, open)?;
        self.set_selected(menu.menu_control_id, open)?;
        self.mutate_control_property(menu.menu_control_id, "popup_open", UiValue::Bool(open))?;
        self.mutate_control_property(menu.menu_control_id, "focused", UiValue::Bool(open))?;
        Ok(())
    }

    fn apply_toolbar_window_menu_anchor(
        &mut self,
        menu: &ToolbarWindowMenu,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let Some(trigger_frame) = self.control_frame(menu.trigger_control_id) else {
            return Ok(());
        };
        let Some(menu_node_id) =
            surface_control_node_id(&self.template_surface.surface, menu.menu_control_id)
        else {
            return Ok(());
        };
        let Some(layout) = toolbar_menu_node_layout(&self.template_surface.surface, menu_node_id)
        else {
            return Ok(());
        };

        let root_frame = self.template_surface.frames.root;
        let toolbar_bottom = self
            .control_frame(WORKBENCH_TOP_TOOLBAR_REGION_CONTROL_ID)
            .map(UiFrame::bottom)
            .unwrap_or_else(|| trigger_frame.bottom());
        let menu_x = toolbar_menu_x(
            root_frame,
            trigger_frame,
            layout.width,
            menu.horizontal_align,
        );
        let menu_frame = UiFrame::new(menu_x, toolbar_bottom, layout.width, layout.height);
        self.apply_toolbar_menu_node_frame(menu_node_id, root_frame, layout, menu_frame)?;
        self.apply_toolbar_menu_popup_metadata(menu.menu_control_id, menu_frame)?;
        Ok(())
    }

    fn apply_toolbar_menu_node_frame(
        &mut self,
        menu_node_id: UiNodeId,
        root_frame: UiFrame,
        layout: ToolbarMenuNodeLayout,
        menu_frame: UiFrame,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let next_position =
            node_position_for_absolute_frame(root_frame, layout.anchor, layout.pivot, menu_frame);
        let changed = {
            let Some(node) = self.template_surface.surface.tree.node_mut(menu_node_id) else {
                return Ok(());
            };
            let changed = !positions_are_near(node.position, next_position);
            if changed {
                node.position = next_position;
            }
            changed
        };
        if changed {
            self.template_surface
                .surface
                .tree
                .mark_layout_dirty(menu_node_id)?;
        }
        Ok(())
    }

    fn apply_toolbar_menu_popup_metadata(
        &mut self,
        menu_control_id: &str,
        menu_frame: UiFrame,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        for (property, value) in [
            ("popup_anchor_x", UiValue::Float(menu_frame.x as f64)),
            ("popup_anchor_y", UiValue::Float(menu_frame.y as f64)),
            ("popup_anchor_width", UiValue::Float(0.0)),
            ("popup_anchor_height", UiValue::Float(0.0)),
            ("popup_offset_x", UiValue::Float(0.0)),
            (
                "popup_offset_y",
                UiValue::Float(-toolbar_popup_render_gap() as f64),
            ),
            ("placement", UiValue::String("bottom-start".to_string())),
            (
                "anchor_origin_horizontal",
                UiValue::String("left".to_string()),
            ),
            (
                "anchor_origin_vertical",
                UiValue::String("bottom".to_string()),
            ),
            (
                "transform_origin_horizontal",
                UiValue::String("left".to_string()),
            ),
            (
                "transform_origin_vertical",
                UiValue::String("top".to_string()),
            ),
        ] {
            self.mutate_control_property(menu_control_id, property, value)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct ToolbarWindowMenu {
    trigger_control_id: &'static str,
    menu_control_id: &'static str,
    action_ids: &'static [&'static str],
    horizontal_align: ToolbarMenuHorizontalAlign,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ToolbarMenuHorizontalAlign {
    Start,
    End,
}

const TOOLBAR_WINDOW_MENUS: &[ToolbarWindowMenu] = &[
    ToolbarWindowMenu {
        trigger_control_id: "WorkbenchToolbarMenu",
        menu_control_id: "WorkbenchToolbarMainMenu",
        action_ids: &["workbench.menu.main.open"],
        horizontal_align: ToolbarMenuHorizontalAlign::Start,
    },
    ToolbarWindowMenu {
        trigger_control_id: "WorkbenchRunMode",
        menu_control_id: "WorkbenchRunModeMenu",
        action_ids: &["workbench.run_mode.menu.open"],
        horizontal_align: ToolbarMenuHorizontalAlign::End,
    },
    ToolbarWindowMenu {
        trigger_control_id: "WorkbenchLayoutGrid",
        menu_control_id: "WorkbenchLayoutMenu",
        action_ids: &["workbench.layout.menu.open"],
        horizontal_align: ToolbarMenuHorizontalAlign::End,
    },
    ToolbarWindowMenu {
        trigger_control_id: WORKBENCH_MODULE_OVERFLOW_TRIGGER_CONTROL_ID,
        menu_control_id: WORKBENCH_MODULE_OVERFLOW_MENU_CONTROL_ID,
        action_ids: &["workbench.module.more.open"],
        horizontal_align: ToolbarMenuHorizontalAlign::Start,
    },
];

fn toolbar_menu_for_action(
    source_control_id: &str,
    action_id: &str,
) -> Option<&'static ToolbarWindowMenu> {
    TOOLBAR_WINDOW_MENUS.iter().find(|menu| {
        menu.trigger_control_id == source_control_id
            || menu
                .action_ids
                .iter()
                .any(|candidate| *candidate == action_id)
    })
}

const WORKBENCH_TOP_TOOLBAR_REGION_CONTROL_ID: &str = "WorkbenchWindowTopToolbarRegion";
const TOOLBAR_MENU_FRAME_EPSILON: f32 = 0.01;

#[derive(Clone, Copy)]
struct ToolbarMenuNodeLayout {
    width: f32,
    height: f32,
    anchor: Anchor,
    pivot: Pivot,
}

fn toolbar_menu_node_layout(
    surface: &UiSurface,
    menu_node_id: UiNodeId,
) -> Option<ToolbarMenuNodeLayout> {
    let node = surface.tree.node(menu_node_id)?;
    Some(ToolbarMenuNodeLayout {
        width: resolved_menu_axis(node.constraints.width, node.layout_cache.frame.width),
        height: resolved_menu_axis(node.constraints.height, node.layout_cache.frame.height),
        anchor: node.anchor,
        pivot: node.pivot,
    })
}

fn toolbar_menu_x(
    root_frame: UiFrame,
    trigger_frame: UiFrame,
    menu_width: f32,
    align: ToolbarMenuHorizontalAlign,
) -> f32 {
    let authored_x = match align {
        ToolbarMenuHorizontalAlign::Start => trigger_frame.x,
        ToolbarMenuHorizontalAlign::End => trigger_frame.right() - menu_width,
    };
    clamp_menu_x_to_root(authored_x, root_frame, menu_width)
}

fn clamp_menu_x_to_root(authored_x: f32, root_frame: UiFrame, menu_width: f32) -> f32 {
    clamp_popup_x_to_bounds(authored_x, root_frame.x, root_frame.width, menu_width)
}

fn node_position_for_absolute_frame(
    root_frame: UiFrame,
    anchor: Anchor,
    pivot: Pivot,
    target_frame: UiFrame,
) -> Position {
    Position::new(
        target_frame.x - root_frame.x - root_frame.width * anchor.x + target_frame.width * pivot.x,
        target_frame.y - root_frame.y - root_frame.height * anchor.y
            + target_frame.height * pivot.y,
    )
}

fn resolved_menu_axis(axis: AxisConstraint, cached_extent: f32) -> f32 {
    [
        axis.preferred,
        axis.min,
        cached_extent,
        axis.max,
        POPUP_EDGE_MARGIN,
    ]
    .into_iter()
    .find(|value| value.is_finite() && *value > 0.0)
    .unwrap_or(POPUP_EDGE_MARGIN)
}

fn positions_are_near(left: Position, right: Position) -> bool {
    (left.x - right.x).abs() <= TOOLBAR_MENU_FRAME_EPSILON
        && (left.y - right.y).abs() <= TOOLBAR_MENU_FRAME_EPSILON
}

fn surface_control_node_id(surface: &UiSurface, control_id: &str) -> Option<UiNodeId> {
    surface.tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
            .filter(|candidate| *candidate == control_id)
            .map(|_| node.node_id)
    })
}
