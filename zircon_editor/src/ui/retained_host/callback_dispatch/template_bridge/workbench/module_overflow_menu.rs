use crate::ui::binding::EditorUiBinding;
use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue};

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

pub(super) const WORKBENCH_MODULE_OVERFLOW_TRIGGER_CONTROL_ID: &str = "WorkbenchModuleMore";
pub(super) const WORKBENCH_MODULE_OVERFLOW_MENU_CONTROL_ID: &str = "WorkbenchModuleOverflowMenu";

const OVERFLOW_MODULES: &[OverflowModule] = &[
    OverflowModule {
        label: "Behavior",
        menu_action_id: "menu.item.behavior",
        tab_control_id: "WorkbenchModuleBehavior",
        icon_flag: "icon=grid",
    },
    OverflowModule {
        label: "Render",
        menu_action_id: "menu.item.render",
        tab_control_id: "WorkbenchModuleRender",
        icon_flag: "icon=grid",
    },
    OverflowModule {
        label: "Assets",
        menu_action_id: "menu.item.assets",
        tab_control_id: "WorkbenchModuleAssets",
        icon_flag: "icon=folder",
    },
    OverflowModule {
        label: "VFX",
        menu_action_id: "menu.item.v_f_x",
        tab_control_id: "WorkbenchModuleVfx",
        icon_flag: "icon=grid",
    },
    OverflowModule {
        label: "HUD",
        menu_action_id: "menu.item.h_u_d",
        tab_control_id: "WorkbenchModuleHud",
        icon_flag: "icon=grid",
    },
];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn refresh_workbench_module_overflow_menu_items(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let items = UiValue::Array(
            OVERFLOW_MODULES
                .iter()
                .map(|module| UiValue::String(module.menu_item_value(self)))
                .collect(),
        );
        self.mutate_control_property(
            WORKBENCH_MODULE_OVERFLOW_MENU_CONTROL_ID,
            "menu_items",
            items,
        )
    }

    pub(crate) fn dispatch_workbench_module_overflow_menu_item_state(
        &mut self,
        control_id: &str,
        menu_action_id: &str,
    ) -> Result<Option<EditorUiBinding>, BuiltinHostWindowTemplateBridgeError> {
        if control_id != WORKBENCH_MODULE_OVERFLOW_MENU_CONTROL_ID {
            return Ok(None);
        }
        let Some(module) = OVERFLOW_MODULES
            .iter()
            .find(|module| module.menu_action_id == menu_action_id)
        else {
            return Ok(None);
        };

        self.dispatch_control_state(module.tab_control_id, UiEventKind::Click)
    }
}

struct OverflowModule {
    label: &'static str,
    menu_action_id: &'static str,
    tab_control_id: &'static str,
    icon_flag: &'static str,
}

impl OverflowModule {
    fn menu_item_value(&self, bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge) -> String {
        let mut flags = vec![self.icon_flag];
        if bridge.control_bool(self.tab_control_id, "selected")
            || bridge.control_bool(self.tab_control_id, "checked")
        {
            flags.insert(0, "checked");
        }
        format!("{}|{}", self.label, flags.join(","))
    }
}
