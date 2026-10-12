use zircon_runtime_interface::ui::component::UiValue;

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const HUD_WIDGET_ROWS: &[&str] = &["WorkbenchHudWidgetTextRow", "WorkbenchHudWidgetButtonRow"];
const HUD_CANVAS_ROWS: &[&str] = &[
    "WorkbenchHudMinimapRow",
    "WorkbenchHudAmmoPanelRow",
    "WorkbenchHudBindingRow",
];
const HUD_SCREEN_DROPDOWN: &str = "WorkbenchHudScreenDropdown";
static HUD_WIDGET_PROFILES: &[HudWidgetProfile] = &[
    HudWidgetProfile {
        action_id: "workbench.module.hud.widget_text.select",
        row_control_id: "WorkbenchHudWidgetTextRow",
        label: "MatchTimer",
        validation: "localization warning",
    },
    HudWidgetProfile {
        action_id: "workbench.module.hud.widget_button.select",
        row_control_id: "WorkbenchHudWidgetButtonRow",
        label: "WeaponPanel",
        validation: "bindings valid",
    },
];
static HUD_CANVAS_PROFILES: &[HudCanvasProfile] = &[
    HudCanvasProfile {
        action_id: "workbench.module.hud.minimap.select",
        row_control_id: "WorkbenchHudMinimapRow",
        label: "Minimap",
        summary: "Anchor Top Right   240 x 180",
    },
    HudCanvasProfile {
        action_id: "workbench.module.hud.ammo_panel.select",
        row_control_id: "WorkbenchHudAmmoPanelRow",
        label: "AmmoPanel",
        summary: "Anchor Bottom Right   bound",
    },
    HudCanvasProfile {
        action_id: "workbench.module.hud.binding_ammo.select",
        row_control_id: "WorkbenchHudBindingRow",
        label: "Ammo_Clip",
        summary: "GetCurrentAmmo   OK",
    },
];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn initialize_hud_workspace_state(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.set_hud_string(HUD_SCREEN_DROPDOWN, "value", "gameplay_hud")?;
        self.set_hud_string(HUD_SCREEN_DROPDOWN, "value_text", "Gameplay HUD")?;
        self.project_hud_widget(&HUD_WIDGET_PROFILES[0])?;
        self.project_hud_canvas(&HUD_CANVAS_PROFILES[0])
    }

    pub(super) fn apply_hud_workspace_action(
        &mut self,
        action_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if action_id == "workbench.module.hud.preview.invoke" {
            self.apply_hud_preview_feedback()?;
            return Ok(true);
        }
        if let Some(profile) = HUD_WIDGET_PROFILES
            .iter()
            .find(|profile| profile.action_id == action_id)
        {
            self.project_hud_widget(profile)?;
            return Ok(true);
        }
        if let Some(profile) = HUD_CANVAS_PROFILES
            .iter()
            .find(|profile| profile.action_id == action_id)
        {
            self.project_hud_canvas(profile)?;
            return Ok(true);
        }
        Ok(false)
    }

    fn project_hud_widget(
        &mut self,
        profile: &HudWidgetProfile,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.select_exclusive_selected(HUD_WIDGET_ROWS, profile.row_control_id)?;
        let screen = self.hud_screen_label();
        self.set_hud_string(
            "WorkbenchHudCenterTitle",
            "text",
            format!("{} / {}", screen, profile.label),
        )?;
        self.set_hud_string(
            "WorkbenchHudValidationRow",
            "value_text",
            format!("Selection: {}   {}", profile.label, profile.validation),
        )
    }

    fn project_hud_canvas(
        &mut self,
        profile: &HudCanvasProfile,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.select_exclusive_selected(HUD_CANVAS_ROWS, profile.row_control_id)?;
        self.set_hud_string(
            "WorkbenchHudValidationRow",
            "value_text",
            format!("Canvas: {}   {}", profile.label, profile.summary),
        )
    }

    fn apply_hud_preview_feedback(&mut self) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let widget = HUD_WIDGET_PROFILES
            .iter()
            .find(|profile| self.control_bool(profile.row_control_id, "selected"))
            .unwrap_or(&HUD_WIDGET_PROFILES[0]);
        let canvas = HUD_CANVAS_PROFILES
            .iter()
            .find(|profile| self.control_bool(profile.row_control_id, "selected"))
            .unwrap_or(&HUD_CANVAS_PROFILES[0]);
        let screen = self.hud_screen_label();
        let dpi = self
            .control_string("WorkbenchHudDpiField", "value")
            .unwrap_or_default();
        let locale = self
            .control_string("WorkbenchHudLocaleField", "value")
            .unwrap_or_default();
        self.set_hud_string("WorkbenchStatusReady", "text", "HUD preview refreshed")?;
        self.set_hud_string("WorkbenchStatusMessages", "text", "1 Message")?;
        self.set_hud_string(
            "WorkbenchHudCenterTitle",
            "text",
            format!("{} / {}", screen, widget.label),
        )?;
        self.set_hud_string(
            "WorkbenchHudValidationRow",
            "value_text",
            format!(
                "Preview: {}   {}x / {}   {}",
                canvas.label, dpi, locale, widget.validation
            ),
        )
    }

    fn hud_screen_label(&self) -> String {
        self.control_string(HUD_SCREEN_DROPDOWN, "value_text")
            .filter(|label| !label.trim().is_empty())
            .unwrap_or_else(|| "Gameplay HUD".to_string())
    }

    fn set_hud_string(
        &mut self,
        control_id: &str,
        property: &str,
        value: impl Into<String>,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.mutate_control_property(control_id, property, UiValue::String(value.into()))
    }
}

struct HudWidgetProfile {
    action_id: &'static str,
    row_control_id: &'static str,
    label: &'static str,
    validation: &'static str,
}

struct HudCanvasProfile {
    action_id: &'static str,
    row_control_id: &'static str,
    label: &'static str,
    summary: &'static str,
}

#[cfg(test)]
#[path = "tests/hud_workspace_state.rs"]
mod tests;
