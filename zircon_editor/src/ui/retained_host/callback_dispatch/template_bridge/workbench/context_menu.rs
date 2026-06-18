use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{component::UiValue, event_ui::UiNodeId};

use crate::ui::retained_host::WorkbenchContextMenuRequestData;

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

pub(crate) const WORKBENCH_CONTEXT_MENU_CONTROL_ID: &str = "WorkbenchContextMenu";

const OPEN: &str = "open";
const POPUP_OPEN: &str = "popup_open";
const FOCUSED: &str = "focused";
const SELECTED: &str = "selected";
const MENU_ITEMS: &str = "menu_items";
const OPTIONS: &str = "options";
const VALUE: &str = "value";
const VALUE_TEXT: &str = "value_text";
const CONTEXT_TARGET: &str = "context_target";
const CONTEXT_TARGET_PATH: &str = "context_target_path";
const POPUP_ANCHOR_X: &str = "popup_anchor_x";
const POPUP_ANCHOR_Y: &str = "popup_anchor_y";

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn open_context_menu(
        &mut self,
        request: &WorkbenchContextMenuRequestData,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if !self.has_control(WORKBENCH_CONTEXT_MENU_CONTROL_ID) {
            return Ok(false);
        }

        self.set_visible(WORKBENCH_CONTEXT_MENU_CONTROL_ID, true)?;
        self.mutate_control_property(WORKBENCH_CONTEXT_MENU_CONTROL_ID, OPEN, UiValue::Bool(true))?;
        self.mutate_control_property(
            WORKBENCH_CONTEXT_MENU_CONTROL_ID,
            POPUP_OPEN,
            UiValue::Bool(true),
        )?;
        self.mutate_control_property(
            WORKBENCH_CONTEXT_MENU_CONTROL_ID,
            FOCUSED,
            UiValue::Bool(true),
        )?;
        self.mutate_control_property(
            WORKBENCH_CONTEXT_MENU_CONTROL_ID,
            SELECTED,
            UiValue::Bool(true),
        )?;
        self.mutate_control_property(
            WORKBENCH_CONTEXT_MENU_CONTROL_ID,
            MENU_ITEMS,
            UiValue::Array(
                request
                    .menu_items
                    .iter()
                    .map(|item| UiValue::String(item.to_string()))
                    .collect(),
            ),
        )?;
        self.mutate_control_property(
            WORKBENCH_CONTEXT_MENU_CONTROL_ID,
            OPTIONS,
            UiValue::Array(Vec::new()),
        )?;
        self.mutate_control_property(
            WORKBENCH_CONTEXT_MENU_CONTROL_ID,
            VALUE,
            UiValue::String(request.target_value_text.to_string()),
        )?;
        self.mutate_control_property(
            WORKBENCH_CONTEXT_MENU_CONTROL_ID,
            VALUE_TEXT,
            UiValue::String(request.target_value_text.to_string()),
        )?;
        self.mutate_control_property(
            WORKBENCH_CONTEXT_MENU_CONTROL_ID,
            CONTEXT_TARGET,
            UiValue::String(request.target_control_id.to_string()),
        )?;
        self.mutate_control_property(
            WORKBENCH_CONTEXT_MENU_CONTROL_ID,
            CONTEXT_TARGET_PATH,
            UiValue::String(request.target_path.to_string()),
        )?;
        self.mutate_control_property(
            WORKBENCH_CONTEXT_MENU_CONTROL_ID,
            POPUP_ANCHOR_X,
            UiValue::Float(f64::from(request.popup_anchor_x)),
        )?;
        self.mutate_control_property(
            WORKBENCH_CONTEXT_MENU_CONTROL_ID,
            POPUP_ANCHOR_Y,
            UiValue::Float(f64::from(request.popup_anchor_y)),
        )?;
        for node_id in control_node_ids_with_descendants(
            &self.template_surface.surface,
            WORKBENCH_CONTEXT_MENU_CONTROL_ID,
        ) {
            self.mutate_node_bool(node_id, OPEN, true)?;
            self.mutate_node_bool(node_id, POPUP_OPEN, true)?;
            self.mutate_node_bool(node_id, FOCUSED, true)?;
            self.mutate_node_bool(node_id, SELECTED, true)?;
        }
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(true)
    }

    pub(super) fn close_context_menu_if_target(
        &mut self,
        control_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        if control_id != WORKBENCH_CONTEXT_MENU_CONTROL_ID {
            return Ok(());
        }
        self.set_visible(WORKBENCH_CONTEXT_MENU_CONTROL_ID, false)?;
        self.mutate_control_property(
            WORKBENCH_CONTEXT_MENU_CONTROL_ID,
            OPEN,
            UiValue::Bool(false),
        )?;
        self.mutate_control_property(
            WORKBENCH_CONTEXT_MENU_CONTROL_ID,
            MENU_ITEMS,
            UiValue::Array(Vec::new()),
        )?;
        for node_id in control_node_ids_with_descendants(
            &self.template_surface.surface,
            WORKBENCH_CONTEXT_MENU_CONTROL_ID,
        ) {
            self.mutate_node_bool(node_id, OPEN, false)?;
            self.mutate_node_bool(node_id, POPUP_OPEN, false)?;
            self.mutate_node_bool(node_id, FOCUSED, false)?;
            self.mutate_node_bool(node_id, SELECTED, false)?;
        }
        Ok(())
    }
}

fn control_node_id(surface: &UiSurface, control_id: &str) -> Option<UiNodeId> {
    surface.tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
            .filter(|candidate| *candidate == control_id)
            .map(|_| node.node_id)
    })
}

fn control_node_ids_with_descendants(surface: &UiSurface, control_id: &str) -> Vec<UiNodeId> {
    let Some(root_id) = control_node_id(surface, control_id) else {
        return Vec::new();
    };

    let mut node_ids = Vec::new();
    let mut stack = vec![root_id];
    while let Some(node_id) = stack.pop() {
        node_ids.push(node_id);
        if let Some(node) = surface.tree.nodes.get(&node_id) {
            for child_id in node.children.iter().rev() {
                stack.push(*child_id);
            }
        }
    }
    node_ids
}
