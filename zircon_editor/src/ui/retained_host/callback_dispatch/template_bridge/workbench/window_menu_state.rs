use zircon_runtime_interface::ui::component::UiValue;

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

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
        self.set_control_active(menu.trigger_control_id, open)?;
        self.set_visible(menu.menu_control_id, open)?;
        self.set_selected(menu.menu_control_id, open)?;
        self.mutate_control_property(menu.menu_control_id, "popup_open", UiValue::Bool(open))?;
        self.mutate_control_property(menu.menu_control_id, "focused", UiValue::Bool(open))?;
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct ToolbarWindowMenu {
    trigger_control_id: &'static str,
    menu_control_id: &'static str,
    action_ids: &'static [&'static str],
}

const TOOLBAR_WINDOW_MENUS: &[ToolbarWindowMenu] = &[
    ToolbarWindowMenu {
        trigger_control_id: "WorkbenchToolbarMenu",
        menu_control_id: "WorkbenchToolbarMainMenu",
        action_ids: &["workbench.menu.main.open"],
    },
    ToolbarWindowMenu {
        trigger_control_id: "WorkbenchRunMode",
        menu_control_id: "WorkbenchRunModeMenu",
        action_ids: &["workbench.run_mode.menu.open"],
    },
    ToolbarWindowMenu {
        trigger_control_id: "WorkbenchLayoutGrid",
        menu_control_id: "WorkbenchLayoutMenu",
        action_ids: &["workbench.layout.menu.open"],
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
