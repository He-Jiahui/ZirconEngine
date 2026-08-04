use zircon_runtime_interface::ui::{
    component::UiValue,
    event_ui::UiNodeId,
    layout::UiFrame,
    surface::{UiPointerActivationPhase, UiPointerButton, UiPointerRoute},
    tree::UiTemplateNodeMetadata,
};

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

const WORKBENCH_ICON_TOOLTIP_CONTROL_ID: &str = "WorkbenchIconTooltip";
const ICON_BUTTON_CLASS: &str = "workbench-icon-button";
const DEFAULT_ICON_LABEL: &str = "Tool";

const OPEN: &str = "open";
const POPUP_OPEN: &str = "popup_open";
const TEXT: &str = "text";
const LABEL_TEXT: &str = "label_text";
const PLACEMENT: &str = "placement";
const POPUP_ANCHOR_X: &str = "popup_anchor_x";
const POPUP_ANCHOR_Y: &str = "popup_anchor_y";
const POPUP_ANCHOR_WIDTH: &str = "popup_anchor_width";
const POPUP_ANCHOR_HEIGHT: &str = "popup_anchor_height";
const TOOLTIP_HEIGHT: f32 = 42.0;
const TOOLTIP_GAP: f32 = 8.0;

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn refresh_workbench_icon_tooltip(
        &mut self,
        route: &UiPointerRoute,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if route.activation_phase != UiPointerActivationPhase::Hover {
            return Ok(false);
        }

        let target = route
            .target
            .and_then(|node_id| self.icon_tooltip_target(node_id));
        match target {
            Some(target) => self.show_workbench_icon_tooltip(target),
            None => self.hide_workbench_icon_tooltip(),
        }
    }

    pub(super) fn dismiss_workbench_icon_tooltip_on_primary_press(
        &mut self,
        route: &UiPointerRoute,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if route.activation_phase != UiPointerActivationPhase::PrimaryPress
            || route.button != Some(UiPointerButton::Primary)
        {
            return Ok(false);
        }

        self.hide_workbench_icon_tooltip()
    }

    fn icon_tooltip_target(&self, node_id: UiNodeId) -> Option<IconTooltipTarget> {
        let node = self.template_surface.surface.tree.nodes.get(&node_id)?;
        let metadata = node.template_metadata.as_ref()?;
        let label = icon_tooltip_label(metadata)?;
        let frame = self
            .template_surface
            .surface
            .arranged_tree
            .get(node_id)
            .map(|node| node.frame)?;
        Some(IconTooltipTarget { label, frame })
    }

    fn show_workbench_icon_tooltip(
        &mut self,
        target: IconTooltipTarget,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if !self.has_control(WORKBENCH_ICON_TOOLTIP_CONTROL_ID) {
            return Ok(false);
        }

        let anchor = target.frame;
        let placement = self.tooltip_placement(anchor);
        let anchor_is_current = self
            .control_float(WORKBENCH_ICON_TOOLTIP_CONTROL_ID, POPUP_ANCHOR_X)
            == Some(f64::from(anchor.x))
            && self.control_float(WORKBENCH_ICON_TOOLTIP_CONTROL_ID, POPUP_ANCHOR_Y)
                == Some(f64::from(anchor.y))
            && self.control_float(WORKBENCH_ICON_TOOLTIP_CONTROL_ID, POPUP_ANCHOR_WIDTH)
                == Some(f64::from(anchor.width))
            && self.control_float(WORKBENCH_ICON_TOOLTIP_CONTROL_ID, POPUP_ANCHOR_HEIGHT)
                == Some(f64::from(anchor.height));
        let is_current = self.control_bool(WORKBENCH_ICON_TOOLTIP_CONTROL_ID, POPUP_OPEN)
            && self
                .control_string(WORKBENCH_ICON_TOOLTIP_CONTROL_ID, TEXT)
                .as_deref()
                == Some(target.label.as_str())
            && self
                .control_string(WORKBENCH_ICON_TOOLTIP_CONTROL_ID, LABEL_TEXT)
                .as_deref()
                == Some("")
            && self
                .control_string(WORKBENCH_ICON_TOOLTIP_CONTROL_ID, PLACEMENT)
                .as_deref()
                == Some(placement)
            && anchor_is_current;
        if is_current {
            return Ok(false);
        }

        self.mutate_control_property(
            WORKBENCH_ICON_TOOLTIP_CONTROL_ID,
            TEXT,
            UiValue::String(target.label),
        )?;
        self.mutate_control_property(
            WORKBENCH_ICON_TOOLTIP_CONTROL_ID,
            LABEL_TEXT,
            UiValue::String(String::new()),
        )?;
        self.mutate_control_property(
            WORKBENCH_ICON_TOOLTIP_CONTROL_ID,
            PLACEMENT,
            UiValue::String(placement.to_string()),
        )?;
        self.mutate_control_property(
            WORKBENCH_ICON_TOOLTIP_CONTROL_ID,
            POPUP_ANCHOR_X,
            UiValue::Float(f64::from(anchor.x)),
        )?;
        self.mutate_control_property(
            WORKBENCH_ICON_TOOLTIP_CONTROL_ID,
            POPUP_ANCHOR_Y,
            UiValue::Float(f64::from(anchor.y)),
        )?;
        self.mutate_control_property(
            WORKBENCH_ICON_TOOLTIP_CONTROL_ID,
            POPUP_ANCHOR_WIDTH,
            UiValue::Float(f64::from(anchor.width)),
        )?;
        self.mutate_control_property(
            WORKBENCH_ICON_TOOLTIP_CONTROL_ID,
            POPUP_ANCHOR_HEIGHT,
            UiValue::Float(f64::from(anchor.height)),
        )?;
        self.mutate_control_property(WORKBENCH_ICON_TOOLTIP_CONTROL_ID, OPEN, UiValue::Bool(true))?;
        self.mutate_control_property(
            WORKBENCH_ICON_TOOLTIP_CONTROL_ID,
            POPUP_OPEN,
            UiValue::Bool(true),
        )?;
        self.set_visible(WORKBENCH_ICON_TOOLTIP_CONTROL_ID, true)?;
        Ok(true)
    }

    fn hide_workbench_icon_tooltip(
        &mut self,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if !self.control_bool(WORKBENCH_ICON_TOOLTIP_CONTROL_ID, POPUP_OPEN) {
            return Ok(false);
        }

        self.mutate_control_property(
            WORKBENCH_ICON_TOOLTIP_CONTROL_ID,
            OPEN,
            UiValue::Bool(false),
        )?;
        self.mutate_control_property(
            WORKBENCH_ICON_TOOLTIP_CONTROL_ID,
            POPUP_OPEN,
            UiValue::Bool(false),
        )?;
        self.set_visible(WORKBENCH_ICON_TOOLTIP_CONTROL_ID, false)?;
        Ok(true)
    }

    fn tooltip_placement(&self, anchor: UiFrame) -> &'static str {
        let required_space = TOOLTIP_HEIGHT + TOOLTIP_GAP;
        let space_above = anchor.y.max(0.0);
        let space_below = (self.mount_frame.height - anchor.y - anchor.height).max(0.0);

        // Top placement matches the workbench convention until it would escape the root frame.
        if space_above >= required_space || space_above >= space_below {
            "top"
        } else {
            "bottom"
        }
    }

    fn control_float(&self, control_id: &str, property: &str) -> Option<f64> {
        let node_id = self.control_node_id(control_id)?;
        let value = self
            .template_surface
            .surface
            .tree
            .nodes
            .get(&node_id)
            .and_then(|node| node.template_metadata.as_ref())
            .and_then(|metadata| metadata.attributes.get(property))?;
        match value {
            toml::Value::Float(value) => Some(*value),
            toml::Value::Integer(value) => Some(*value as f64),
            toml::Value::String(value) => value.parse().ok(),
            _ => None,
        }
    }
}

struct IconTooltipTarget {
    label: String,
    frame: UiFrame,
}

fn icon_tooltip_label(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    if !metadata
        .classes
        .iter()
        .any(|class| class.as_str() == ICON_BUTTON_CLASS)
    {
        return None;
    }
    if !metadata
        .attributes
        .get("enabled")
        .and_then(toml::Value::as_bool)
        .unwrap_or(true)
        || metadata
            .attributes
            .get("disabled")
            .and_then(toml::Value::as_bool)
            .unwrap_or(false)
    {
        return None;
    }

    let label = metadata.attributes.get("label")?.as_str()?.trim();
    (!label.is_empty() && label != DEFAULT_ICON_LABEL).then(|| label.to_string())
}
