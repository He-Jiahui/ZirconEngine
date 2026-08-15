use crate::ui::binding::EditorUiBinding;
use zircon_runtime::ui::tree::UiRuntimeTreeLayoutExt;
use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue};

use crate::ui::retained_host::menu_popup_contract::menu_popup_content_height;

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

pub(super) const WORKBENCH_MODULE_OVERFLOW_TRIGGER_CONTROL_ID: &str = "WorkbenchModuleMore";
pub(super) const WORKBENCH_MODULE_OVERFLOW_MENU_CONTROL_ID: &str = "WorkbenchModuleOverflowMenu";

const OVERFLOW_COMMANDS: &[OverflowCommand] = &[
    OverflowCommand {
        label: "Perception",
        menu_action_id: "menu.item.perception",
        source_control_id: "WorkbenchModulePerception",
        icon_flag: "icon=grid",
    },
    OverflowCommand {
        label: "Material",
        menu_action_id: "menu.item.material",
        source_control_id: "WorkbenchModuleMaterial",
        icon_flag: "icon=grid",
    },
    OverflowCommand {
        label: "Behavior",
        menu_action_id: "menu.item.behavior",
        source_control_id: "WorkbenchModuleBehavior",
        icon_flag: "icon=grid",
    },
    OverflowCommand {
        label: "Render",
        menu_action_id: "menu.item.render",
        source_control_id: "WorkbenchModuleRender",
        icon_flag: "icon=grid",
    },
    OverflowCommand {
        label: "Assets",
        menu_action_id: "menu.item.assets",
        source_control_id: "WorkbenchModuleAssets",
        icon_flag: "icon=folder",
    },
    OverflowCommand {
        label: "VFX",
        menu_action_id: "menu.item.v_f_x",
        source_control_id: "WorkbenchModuleVfx",
        icon_flag: "icon=grid",
    },
    OverflowCommand {
        label: "HUD",
        menu_action_id: "menu.item.h_u_d",
        source_control_id: "WorkbenchModuleHud",
        icon_flag: "icon=grid",
    },
    OverflowCommand {
        label: "Diff",
        menu_action_id: "menu.item.diff",
        source_control_id: "WorkbenchModuleDiff",
        icon_flag: "icon=grid",
    },
    OverflowCommand {
        label: "Sim",
        menu_action_id: "menu.item.sim",
        source_control_id: "WorkbenchModuleSimulate",
        icon_flag: "icon=play",
    },
];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn refresh_workbench_module_overflow_menu_items(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let commands = OVERFLOW_COMMANDS
            .iter()
            .filter(|command| self.control_frame(command.source_control_id).is_none())
            .collect::<Vec<_>>();
        let items = UiValue::Array(
            commands
                .iter()
                .map(|command| UiValue::String(command.menu_item_value(self)))
                .collect(),
        );
        self.mutate_control_property(
            WORKBENCH_MODULE_OVERFLOW_MENU_CONTROL_ID,
            "menu_items",
            items,
        )?;
        self.apply_workbench_module_overflow_menu_height(commands.len())
    }

    pub(crate) fn dispatch_workbench_module_overflow_menu_item_state(
        &mut self,
        control_id: &str,
        menu_action_id: &str,
    ) -> Result<Option<EditorUiBinding>, BuiltinHostWindowTemplateBridgeError> {
        if control_id != WORKBENCH_MODULE_OVERFLOW_MENU_CONTROL_ID {
            return Ok(None);
        }
        let Some(command) = OVERFLOW_COMMANDS
            .iter()
            .find(|command| command.menu_action_id == menu_action_id)
        else {
            return Ok(None);
        };

        self.dispatch_control_state(command.source_control_id, UiEventKind::Click)
    }

    fn apply_workbench_module_overflow_menu_height(
        &mut self,
        item_count: usize,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let Some(node_id) = self.control_node_id(WORKBENCH_MODULE_OVERFLOW_MENU_CONTROL_ID) else {
            return Ok(());
        };
        let height = menu_popup_content_height(item_count);
        let changed = {
            let Some(node) = self.template_surface.surface.tree.node_mut(node_id) else {
                return Ok(());
            };
            let mut next_height = node.constraints.height;
            next_height.min = height;
            next_height.preferred = height;
            next_height.max = height;
            let changed = node.constraints.height != next_height;
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

struct OverflowCommand {
    label: &'static str,
    menu_action_id: &'static str,
    source_control_id: &'static str,
    icon_flag: &'static str,
}

impl OverflowCommand {
    fn menu_item_value(&self, bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge) -> String {
        let mut flags = vec![self.icon_flag];
        if bridge.control_bool(self.source_control_id, "selected")
            || bridge.control_bool(self.source_control_id, "checked")
        {
            flags.insert(0, "checked");
        }
        format!("{}|{}", self.label, flags.join(","))
    }
}
