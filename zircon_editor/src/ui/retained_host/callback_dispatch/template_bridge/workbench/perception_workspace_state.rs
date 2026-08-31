use zircon_runtime_interface::ui::component::UiValue;

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const PERCEPTION_AGENT_ROWS: &[&str] = &[
    "WorkbenchPerceptionGuardRow",
    "WorkbenchPerceptionSniperRow",
];
const PERCEPTION_MAP_ROWS: &[&str] = &[
    "WorkbenchPerceptionSightConeRow",
    "WorkbenchPerceptionHearingPulseRow",
    "WorkbenchPerceptionStimulusRow",
];
const PERCEPTION_MAP_PROFILES: &[PerceptionMapProfile] = &[
    PerceptionMapProfile {
        action_id: "workbench.module.perception.sight_cone.select",
        row_control_id: "WorkbenchPerceptionSightConeRow",
        label: "Sight Cone",
    },
    PerceptionMapProfile {
        action_id: "workbench.module.perception.hearing_pulse.select",
        row_control_id: "WorkbenchPerceptionHearingPulseRow",
        label: "Hearing Pulse",
    },
    PerceptionMapProfile {
        action_id: "workbench.module.perception.stimulus.select",
        row_control_id: "WorkbenchPerceptionStimulusRow",
        label: "Stimulus",
    },
];
const PERCEPTION_PROFILES: &[PerceptionAgentProfile] = &[
    PerceptionAgentProfile {
        action_id: "workbench.module.perception.guard.select",
        row_control_id: "WorkbenchPerceptionGuardRow",
        agent_name: "AI_Guard_01",
        title: "AI_Guard_01 Perception",
        sight: "AI_Guard_01   74 deg   target visible",
        hearing: "Noise_Maker_BP   1200 cm",
        stimulus: "Noise_Maker_BP   Hearing   Purple",
        event: "AI_Guard_01   Hearing stimulus   00:11.8",
        config_value: "Guard_Perception",
        config_text: "Guard Perception",
        line_of_sight: "On",
        team_filter: "All",
        simulation_time: "00:12.4",
    },
    PerceptionAgentProfile {
        action_id: "workbench.module.perception.sniper.select",
        row_control_id: "WorkbenchPerceptionSniperRow",
        agent_name: "Sniper_Perception",
        title: "Sniper_Perception Map",
        sight: "Sniper_Perception   38 deg   target occluded",
        hearing: "DistantShot   2400 cm",
        stimulus: "Player_01   Sight   Red",
        event: "Sniper_Perception   Sight stimulus   00:07.2",
        config_value: "Sniper_Perception",
        config_text: "Sniper Perception",
        line_of_sight: "On",
        team_filter: "Enemies",
        simulation_time: "00:08.0",
    },
];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn initialize_perception_workspace_state(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.project_perception_agent(&PERCEPTION_PROFILES[0])?;
        self.project_perception_map(&PERCEPTION_MAP_PROFILES[0])
    }

    pub(super) fn apply_perception_workspace_action(
        &mut self,
        action_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if action_id == "workbench.module.perception.simulate.invoke" {
            self.apply_perception_simulation_feedback()?;
            return Ok(true);
        }
        if let Some(profile) = PERCEPTION_MAP_PROFILES
            .iter()
            .find(|profile| profile.action_id == action_id)
        {
            self.project_perception_map(profile)?;
            return Ok(true);
        }
        let Some(profile) = PERCEPTION_PROFILES
            .iter()
            .find(|profile| profile.action_id == action_id)
        else {
            return Ok(false);
        };
        self.project_perception_agent(profile)?;
        Ok(true)
    }

    fn project_perception_agent(
        &mut self,
        profile: &PerceptionAgentProfile,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.select_exclusive_selected(PERCEPTION_AGENT_ROWS, profile.row_control_id)?;
        for (control_id, property, value) in [
            ("WorkbenchPerceptionCenterTitle", "text", profile.title),
            (
                "WorkbenchPerceptionSightConeRow",
                "value_text",
                profile.sight,
            ),
            (
                "WorkbenchPerceptionHearingPulseRow",
                "value_text",
                profile.hearing,
            ),
            (
                "WorkbenchPerceptionStimulusRow",
                "value_text",
                profile.stimulus,
            ),
            ("WorkbenchPerceptionEventRow", "value_text", profile.event),
            (
                "WorkbenchPerceptionConfigDropdown",
                "value",
                profile.config_value,
            ),
            (
                "WorkbenchPerceptionConfigDropdown",
                "value_text",
                profile.config_text,
            ),
            (
                "WorkbenchPerceptionLosField",
                "value",
                profile.line_of_sight,
            ),
            ("WorkbenchPerceptionTeamField", "value", profile.team_filter),
        ] {
            self.set_perception_string(control_id, property, value)?;
        }
        Ok(())
    }

    fn project_perception_map(
        &mut self,
        map: &PerceptionMapProfile,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.select_exclusive_selected(PERCEPTION_MAP_ROWS, map.row_control_id)?;
        let agent = PERCEPTION_PROFILES
            .iter()
            .find(|profile| self.control_bool(profile.row_control_id, "selected"))
            .unwrap_or(&PERCEPTION_PROFILES[0]);
        self.set_perception_string(
            "WorkbenchPerceptionEventRow",
            "value_text",
            format!("{}   inspecting {}", agent.agent_name, map.label),
        )
    }

    fn apply_perception_simulation_feedback(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let profile = PERCEPTION_PROFILES
            .iter()
            .find(|profile| self.control_bool(profile.row_control_id, "selected"))
            .unwrap_or(&PERCEPTION_PROFILES[0]);
        let config = self
            .control_string("WorkbenchPerceptionConfigDropdown", "value_text")
            .unwrap_or_default();
        let line_of_sight = self
            .control_string("WorkbenchPerceptionLosField", "value")
            .unwrap_or_default();
        let team = self
            .control_string("WorkbenchPerceptionTeamField", "value")
            .unwrap_or_default();
        self.set_perception_string(
            "WorkbenchStatusReady",
            "text",
            "Perception simulation running",
        )?;
        self.set_perception_string("WorkbenchStatusMessages", "text", "1 Message")?;
        self.set_perception_string(
            "WorkbenchPerceptionCenterTitle",
            "text",
            format!("{} / {}", profile.agent_name, config),
        )?;
        self.set_perception_string(
            "WorkbenchPerceptionEventRow",
            "value_text",
            format!(
                "LOS {} / {}   {}",
                line_of_sight, team, profile.simulation_time
            ),
        )
    }

    fn set_perception_string(
        &mut self,
        control_id: &str,
        property: &str,
        value: impl Into<String>,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.mutate_control_property(control_id, property, UiValue::String(value.into()))
    }
}

struct PerceptionAgentProfile {
    action_id: &'static str,
    row_control_id: &'static str,
    agent_name: &'static str,
    title: &'static str,
    sight: &'static str,
    hearing: &'static str,
    stimulus: &'static str,
    event: &'static str,
    config_value: &'static str,
    config_text: &'static str,
    line_of_sight: &'static str,
    team_filter: &'static str,
    simulation_time: &'static str,
}

struct PerceptionMapProfile {
    action_id: &'static str,
    row_control_id: &'static str,
    label: &'static str,
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue, layout::UiSize};

    use super::*;

    #[test]
    fn agent_selection_and_simulation_share_one_profile_projection() {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
                .expect("workbench bridge should build");

        assert!(bridge.control_bool("WorkbenchPerceptionGuardRow", "selected"));
        assert_eq!(
            Some("AI_Guard_01 Perception".to_string()),
            bridge.control_string("WorkbenchPerceptionCenterTitle", "text")
        );

        bridge
            .dispatch_control_state("WorkbenchPerceptionSniperRow", UiEventKind::Click)
            .expect("sniper profile should dispatch")
            .expect("sniper profile should bind");
        assert_eq!(
            Some("Sniper_Perception Map".to_string()),
            bridge.control_string("WorkbenchPerceptionCenterTitle", "text")
        );
        assert_eq!(
            Some("Sniper Perception".to_string()),
            bridge.control_string("WorkbenchPerceptionConfigDropdown", "value_text")
        );
        assert_eq!(
            Some("Enemies".to_string()),
            bridge.control_string("WorkbenchPerceptionTeamField", "value")
        );

        bridge
            .dispatch_control_state("WorkbenchPerceptionHearingPulseRow", UiEventKind::Click)
            .expect("hearing map item should dispatch")
            .expect("hearing map item should bind");
        assert!(bridge.control_bool("WorkbenchPerceptionSniperRow", "selected"));
        assert!(bridge.control_bool("WorkbenchPerceptionHearingPulseRow", "selected"));
        assert_eq!(
            Some("Sniper_Perception   inspecting Hearing Pulse".to_string()),
            bridge.control_string("WorkbenchPerceptionEventRow", "value_text")
        );
        assert!(bridge
            .select_dropdown_option("WorkbenchPerceptionConfigDropdown", "Guard_Perception")
            .expect("perception config should select"));
        for (control_id, value) in [
            ("WorkbenchPerceptionLosField", "Off"),
            ("WorkbenchPerceptionTeamField", "Friendlies"),
        ] {
            bridge
                .mutate_control_property(control_id, "value", UiValue::String(value.to_string()))
                .expect("perception configuration should edit");
        }

        bridge
            .dispatch_control_state("WorkbenchPerceptionSimulateButton", UiEventKind::Click)
            .expect("simulation should dispatch")
            .expect("simulation should bind");
        assert_eq!(
            Some("LOS Off / Friendlies   00:08.0".to_string()),
            bridge.control_string("WorkbenchPerceptionEventRow", "value_text")
        );
        assert_eq!(
            Some("Sniper_Perception / Guard Perception".to_string()),
            bridge.control_string("WorkbenchPerceptionCenterTitle", "text")
        );
    }
}
