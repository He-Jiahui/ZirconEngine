use zircon_runtime_interface::ui::component::UiValue;

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const ABILITY_TASK_ROWS: &[&str] = &[
    "WorkbenchAbilityTaskActivateRow",
    "WorkbenchAbilityTaskCostRow",
    "WorkbenchAbilityAssetRow",
];
const ABILITY_GRAPH_ROWS: &[&str] = &[
    "WorkbenchAbilityPhaseActivateRow",
    "WorkbenchAbilityPhaseCostRow",
    "WorkbenchAbilityGraphRow",
];
static ABILITY_PROFILES: &[AbilitySelectionProfile] = &[
    AbilitySelectionProfile {
        action_ids: &[
            "workbench.module.ability.task_activate.select",
            "workbench.module.ability.phase_activate.select",
        ],
        task_control_id: "WorkbenchAbilityTaskActivateRow",
        graph_control_id: "WorkbenchAbilityPhaseActivateRow",
        title: "GA_DashAttack / Activation",
        timeline: "Activation ready   Server   GA_DashAttack",
        playtest_scope: "activation phase",
    },
    AbilitySelectionProfile {
        action_ids: &[
            "workbench.module.ability.task_cost.select",
            "workbench.module.ability.phase_cost.select",
        ],
        task_control_id: "WorkbenchAbilityTaskCostRow",
        graph_control_id: "WorkbenchAbilityPhaseCostRow",
        title: "GA_DashAttack / Cost",
        timeline: "Cost ready   Predicted   GE_DashAttack_Cost",
        playtest_scope: "cost phase",
    },
    AbilitySelectionProfile {
        action_ids: &[
            "workbench.module.ability.asset_dash.select",
            "workbench.module.ability.graph_select.select",
        ],
        task_control_id: "WorkbenchAbilityAssetRow",
        graph_control_id: "WorkbenchAbilityGraphRow",
        title: "GA_DashAttack / Full Graph",
        timeline: "Full graph ready   1.22 s   GA_DashAttack",
        playtest_scope: "full graph",
    },
];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn initialize_ability_workspace_state(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.initialize_ability_properties()?;
        self.project_ability_profile(&ABILITY_PROFILES[0])
    }

    fn initialize_ability_properties(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        for (control_id, property, value) in [
            ("WorkbenchAbilityNameField", "value", "GA_DashAttack"),
            (
                "WorkbenchAbilityNetPolicyDropdown",
                "value",
                "server_initiated",
            ),
            (
                "WorkbenchAbilityNetPolicyDropdown",
                "value_text",
                "Server Initiated",
            ),
            ("WorkbenchAbilityCooldownField", "value", "4.00s"),
        ] {
            self.set_ability_string(control_id, property, value)?;
        }
        Ok(())
    }

    pub(super) fn apply_ability_workspace_action(
        &mut self,
        action_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if action_id == "workbench.module.ability.playtest.invoke" {
            self.apply_ability_playtest_feedback()?;
            return Ok(true);
        }
        let Some(profile) = ABILITY_PROFILES
            .iter()
            .find(|profile| profile.action_ids.contains(&action_id))
        else {
            return Ok(false);
        };
        self.project_ability_profile(profile)?;
        Ok(true)
    }

    fn project_ability_profile(
        &mut self,
        profile: &AbilitySelectionProfile,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.select_exclusive_selected(ABILITY_TASK_ROWS, profile.task_control_id)?;
        self.select_exclusive_selected(ABILITY_GRAPH_ROWS, profile.graph_control_id)?;
        for (control_id, property, value) in [
            ("WorkbenchAbilityCenterTitle", "text", profile.title),
            ("WorkbenchAbilityOutputRow", "value_text", profile.timeline),
        ] {
            self.set_ability_string(control_id, property, value)?;
        }
        Ok(())
    }

    fn apply_ability_playtest_feedback(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let profile = ABILITY_PROFILES
            .iter()
            .find(|profile| self.control_bool(profile.graph_control_id, "selected"))
            .unwrap_or(&ABILITY_PROFILES[0]);
        let ability_name = self
            .control_string("WorkbenchAbilityNameField", "value")
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "GA_DashAttack".to_string());
        let net_policy = self
            .control_string("WorkbenchAbilityNetPolicyDropdown", "value_text")
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "Server Initiated".to_string());
        let cooldown = self
            .control_string("WorkbenchAbilityCooldownField", "value")
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "4.00s".to_string());
        self.set_ability_string("WorkbenchStatusReady", "text", "Ability playtest queued")?;
        self.set_ability_string("WorkbenchStatusMessages", "text", "1 Message")?;
        self.set_ability_string(
            "WorkbenchAbilityOutputRow",
            "value_text",
            format!(
                "Playtest queued   {}   {}   {}   cooldown {}",
                net_policy, profile.playtest_scope, ability_name, cooldown
            ),
        )
    }

    fn set_ability_string(
        &mut self,
        control_id: &str,
        property: &str,
        value: impl Into<String>,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.mutate_control_property(control_id, property, UiValue::String(value.into()))
    }
}

struct AbilitySelectionProfile {
    action_ids: &'static [&'static str],
    task_control_id: &'static str,
    graph_control_id: &'static str,
    title: &'static str,
    timeline: &'static str,
    playtest_scope: &'static str,
}

#[cfg(test)]
#[path = "tests/ability_workspace_state.rs"]
mod tests;
