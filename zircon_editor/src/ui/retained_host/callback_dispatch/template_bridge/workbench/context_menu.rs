use zircon_runtime::ui::tree::UiRuntimeTreeLayoutExt;
use zircon_runtime_interface::ui::{
    component::UiValue,
    layout::{StretchMode, UiPoint, UiSize},
};

use crate::core::editor_event::MenuAction;
use crate::ui::binding::{
    AssetCommand, EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind,
};
use crate::ui::retained_host::host_contract::{current_host_metrics, menu_popup_text_width};
use crate::ui::retained_host::menu_popup_contract::{
    content_measured_structured_menu_popup_width, menu_popup_content_height,
};
use crate::ui::retained_host::WorkbenchContextMenuRequestData;
use crate::ui::workbench::event::menu_action_binding;

use super::componentized_window::{
    logical_axis_from_physical, BuiltinWorkbenchWindowTemplateSurfaceBridge,
};
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
const LAYOUT_MIN_WIDTH: &str = "layout_min_width";

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn context_menu_item_binding(
        &self,
        menu_control_id: &str,
        action_id: &str,
    ) -> Option<EditorUiBinding> {
        if menu_control_id != WORKBENCH_CONTEXT_MENU_CONTROL_ID {
            return None;
        }
        if action_id == "menu.item.keep_play_changes" {
            return Some(menu_action_binding(&MenuAction::KeepPlayChanges));
        }
        if action_id == "menu.item.asset.delete" {
            let asset_uuid = self
                .control_string(WORKBENCH_CONTEXT_MENU_CONTROL_ID, CONTEXT_TARGET_PATH)?
                .strip_prefix("workbench://asset/")?
                .to_owned();
            return Some(EditorUiBinding::new(
                WORKBENCH_CONTEXT_MENU_CONTROL_ID,
                action_id,
                EditorUiEventKind::Click,
                EditorUiBindingPayload::asset_command(AssetCommand::DeleteAsset { asset_uuid }),
            ));
        }
        None
    }

    pub(crate) fn open_context_menu(
        &mut self,
        request: &WorkbenchContextMenuRequestData,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if !self.has_control(WORKBENCH_CONTEXT_MENU_CONTROL_ID) {
            return Ok(false);
        }

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
        let local_anchor_x = logical_axis_from_physical(
            request.popup_anchor_x,
            self.mount_frame.x,
            self.presentation_scale_factor,
        );
        let local_anchor_y = logical_axis_from_physical(
            request.popup_anchor_y,
            self.mount_frame.y,
            self.presentation_scale_factor,
        );
        let Some(context_menu_node_id) = self.control_node_id(WORKBENCH_CONTEXT_MENU_CONTROL_ID)
        else {
            return Ok(false);
        };
        let _ = self
            .template_surface
            .surface
            .set_popup_pointer_anchor(
                context_menu_node_id,
                UiPoint::new(local_anchor_x, local_anchor_y),
            )
            .map_err(
                |source| BuiltinHostWindowTemplateBridgeError::LayoutMutation {
                    node_id: context_menu_node_id,
                    property: "widget.popup_anchor".to_string(),
                    source,
                },
            )?;
        self.apply_context_menu_extent(request)?;
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
        for node_id in self.control_node_ids_with_descendants(WORKBENCH_CONTEXT_MENU_CONTROL_ID) {
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
        for node_id in self.control_node_ids_with_descendants(WORKBENCH_CONTEXT_MENU_CONTROL_ID) {
            self.mutate_node_bool(node_id, OPEN, false)?;
            self.mutate_node_bool(node_id, POPUP_OPEN, false)?;
            self.mutate_node_bool(node_id, FOCUSED, false)?;
            self.mutate_node_bool(node_id, SELECTED, false)?;
        }
        Ok(())
    }

    fn apply_context_menu_extent(
        &mut self,
        request: &WorkbenchContextMenuRequestData,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let scale_factor = normalized_scale_factor(self.presentation_scale_factor);
        let logical_shell = UiSize::new(
            self.mount_frame.width / scale_factor,
            self.mount_frame.height / scale_factor,
        );
        let metrics = current_host_metrics();
        let trailing_adornment_reserve =
            (metrics.font_large + metrics.gap_m * 2.0 - metrics.input_pad[1]).max(0.0)
                / scale_factor;
        let fallback_width = self
            .control_float(WORKBENCH_CONTEXT_MENU_CONTROL_ID, LAYOUT_MIN_WIDTH)
            .unwrap_or(1.0);
        let width = content_measured_structured_menu_popup_width(
            fallback_width,
            logical_shell.width,
            request.menu_items.iter().map(|item| item.as_str()),
            trailing_adornment_reserve,
            |text| menu_popup_text_width(text) / scale_factor,
        );
        let height = menu_popup_content_height(request.menu_items.len())
            .min(logical_shell.height.max(1.0))
            .max(1.0);
        let Some(node_id) = self.control_node_id(WORKBENCH_CONTEXT_MENU_CONTROL_ID) else {
            return Ok(());
        };
        let changed = {
            let Some(node) = self.template_surface.surface.tree.node_mut(node_id) else {
                return Ok(());
            };
            let mut next_width = node.constraints.width;
            next_width.min = width;
            next_width.preferred = width;
            next_width.max = width;
            next_width.stretch_mode = StretchMode::Fixed;
            let mut next_height = node.constraints.height;
            next_height.min = height;
            next_height.preferred = height;
            next_height.max = height;
            next_height.stretch_mode = StretchMode::Fixed;
            let changed =
                node.constraints.width != next_width || node.constraints.height != next_height;
            node.constraints.width = next_width;
            node.constraints.height = next_height;
            changed
        };
        if changed {
            self.template_surface
                .surface
                .tree
                .mark_layout_dirty(node_id)?;
        }
        Ok(())
    }
}

fn normalized_scale_factor(scale_factor: f32) -> f32 {
    if scale_factor.is_finite() && scale_factor > f32::EPSILON {
        scale_factor
    } else {
        1.0
    }
}
