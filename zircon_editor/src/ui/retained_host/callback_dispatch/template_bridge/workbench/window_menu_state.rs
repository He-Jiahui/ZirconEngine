use zircon_runtime_interface::ui::{component::UiValue, layout::UiSize};

use crate::ui::retained_host::host_contract::{current_host_metrics, menu_popup_text_width};
use crate::ui::retained_host::menu_popup_contract::{
    content_measured_structured_menu_popup_width, menu_popup_content_height,
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
        let Some(target) = workbench_menu_for_action(source_control_id, action_id) else {
            return Ok(false);
        };
        let open = !self.control_bool(target.menu_control_id, "popup_open");

        for menu in WORKBENCH_WINDOW_MENUS {
            self.set_workbench_window_menu_open(menu, open && menu == target)?;
        }
        Ok(true)
    }

    pub(super) fn close_workbench_window_menu_control(
        &mut self,
        menu_control_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        let Some(menu) = WORKBENCH_WINDOW_MENUS
            .iter()
            .find(|menu| menu.menu_control_id == menu_control_id)
        else {
            return Ok(false);
        };
        self.set_workbench_window_menu_open(menu, false)?;
        Ok(true)
    }

    fn set_workbench_window_menu_open(
        &mut self,
        menu: &WorkbenchWindowMenu,
        open: bool,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        if open && menu.menu_control_id == WORKBENCH_MODULE_OVERFLOW_MENU_CONTROL_ID {
            self.refresh_workbench_module_overflow_menu_items()?;
        }
        if open {
            self.apply_workbench_window_menu_extent(menu.menu_control_id)?;
        }
        self.set_control_active(menu.trigger_control_id, open)?;
        self.set_visible(menu.menu_control_id, open)?;
        self.set_selected(menu.menu_control_id, open)?;
        self.mutate_control_property(menu.menu_control_id, "popup_open", UiValue::Bool(open))?;
        self.mutate_control_property(menu.menu_control_id, "focused", UiValue::Bool(open))?;
        Ok(())
    }

    fn apply_workbench_window_menu_extent(
        &mut self,
        menu_control_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let scale_factor = normalized_scale_factor(self.presentation_scale_factor);
        let logical_shell_width = (self.mount_frame.width / scale_factor).max(1.0);
        let logical_shell_height = (self.mount_frame.height / scale_factor).max(1.0);
        let menu_items = self.control_string_array(menu_control_id, "menu_items");
        let metrics = current_host_metrics();
        let trailing_adornment_reserve =
            (metrics.font_large + metrics.gap_m * 2.0 - metrics.input_pad[1]).max(0.0)
                / scale_factor;
        let fallback_width = self
            .control_float(menu_control_id, "layout_min_width")
            .or_else(|| {
                self.control_node_id(menu_control_id)
                    .and_then(|node_id| self.template_surface.surface.tree.node(node_id))
                    .map(|node| node.constraints.width.preferred)
            })
            .unwrap_or(1.0);
        let width = content_measured_structured_menu_popup_width(
            fallback_width,
            logical_shell_width,
            menu_items.iter().map(String::as_str),
            trailing_adornment_reserve,
            |text| menu_popup_text_width(text) / scale_factor,
        );
        let height = menu_popup_content_height(menu_items.len())
            .min(logical_shell_height)
            .max(1.0);
        self.set_fixed_control_extent(menu_control_id, UiSize::new(width, height))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct WorkbenchWindowMenu {
    trigger_control_id: &'static str,
    menu_control_id: &'static str,
    action_ids: &'static [&'static str],
}

const WORKBENCH_WINDOW_MENUS: &[WorkbenchWindowMenu] = &[
    WorkbenchWindowMenu {
        trigger_control_id: "WorkbenchToolbarMenu",
        menu_control_id: "WorkbenchToolbarMainMenu",
        action_ids: &["workbench.menu.main.open"],
    },
    WorkbenchWindowMenu {
        trigger_control_id: "WorkbenchRunMode",
        menu_control_id: "WorkbenchRunModeMenu",
        action_ids: &["workbench.run_mode.menu.open"],
    },
    WorkbenchWindowMenu {
        trigger_control_id: "WorkbenchLayoutGrid",
        menu_control_id: "WorkbenchLayoutMenu",
        action_ids: &["workbench.layout.menu.open"],
    },
    WorkbenchWindowMenu {
        trigger_control_id: WORKBENCH_MODULE_OVERFLOW_TRIGGER_CONTROL_ID,
        menu_control_id: WORKBENCH_MODULE_OVERFLOW_MENU_CONTROL_ID,
        action_ids: &["workbench.module.more.open"],
    },
    WorkbenchWindowMenu {
        trigger_control_id: "WorkbenchAssetsWorldTools",
        menu_control_id: "WorkbenchAssetsWorldToolsMenu",
        action_ids: &["workbench.module.assets.world_tools.open"],
    },
    WorkbenchWindowMenu {
        trigger_control_id: "WorkbenchAssetsGameplayTools",
        menu_control_id: "WorkbenchAssetsGameplayToolsMenu",
        action_ids: &["workbench.module.assets.gameplay_tools.open"],
    },
    WorkbenchWindowMenu {
        trigger_control_id: "WorkbenchAssetsProductionTools",
        menu_control_id: "WorkbenchAssetsProductionToolsMenu",
        action_ids: &["workbench.module.assets.production_tools.open"],
    },
    WorkbenchWindowMenu {
        trigger_control_id: "WorkbenchAbilityAnimationTools",
        menu_control_id: "WorkbenchAbilityAnimationToolsMenu",
        action_ids: &["workbench.module.ability.animation_tools.open"],
    },
    WorkbenchWindowMenu {
        trigger_control_id: "WorkbenchRenderTools",
        menu_control_id: "WorkbenchRenderToolsMenu",
        action_ids: &["workbench.module.render.tools.open"],
    },
    WorkbenchWindowMenu {
        trigger_control_id: "WorkbenchHudTools",
        menu_control_id: "WorkbenchHudToolsMenu",
        action_ids: &["workbench.module.hud.tools.open"],
    },
];

fn workbench_menu_for_action(
    source_control_id: &str,
    action_id: &str,
) -> Option<&'static WorkbenchWindowMenu> {
    WORKBENCH_WINDOW_MENUS.iter().find(|menu| {
        menu.trigger_control_id == source_control_id
            || menu
                .action_ids
                .iter()
                .any(|candidate| *candidate == action_id)
    })
}

fn normalized_scale_factor(scale_factor: f32) -> f32 {
    if scale_factor.is_finite() && scale_factor > f32::EPSILON {
        scale_factor
    } else {
        1.0
    }
}
