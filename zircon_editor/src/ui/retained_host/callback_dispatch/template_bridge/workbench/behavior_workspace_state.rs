use zircon_runtime_interface::ui::component::UiValue;

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const BEHAVIOR_TREE_ROWS: &[&str] = &["WorkbenchBehaviorSelectorRow", "WorkbenchBehaviorAttackRow"];
const BEHAVIOR_GRAPH_ROWS: &[&str] = &[
    "WorkbenchBehaviorNodeRow01",
    "WorkbenchBehaviorNodeRow02",
    "WorkbenchBehaviorNodeRow03",
];
static BEHAVIOR_PROFILES: &[BehaviorNodeProfile] = &[
    BehaviorNodeProfile {
        action_ids: &[
            "workbench.module.behavior.selector_row.select",
            "workbench.module.behavior.node_selector.select",
        ],
        tree_control_id: "WorkbenchBehaviorSelectorRow",
        graph_control_id: "WorkbenchBehaviorNodeRow01",
        title: "BT_Enemy / Combat Root",
        blackboard: "BB_Enemy",
        ai_controller: "AIController_Enemy",
        preview_state: "Running",
        trace: "Runtime Trace: Combat Root selector running",
    },
    BehaviorNodeProfile {
        action_ids: &[
            "workbench.module.behavior.attack_row.select",
            "workbench.module.behavior.node_attack.select",
        ],
        tree_control_id: "WorkbenchBehaviorAttackRow",
        graph_control_id: "WorkbenchBehaviorNodeRow02",
        title: "BT_Enemy / Attack Target",
        blackboard: "BB_Combat",
        ai_controller: "AIController_CombatEnemy",
        preview_state: "Executing",
        trace: "Runtime Trace: Attack Target task executing",
    },
    BehaviorNodeProfile {
        action_ids: &["workbench.module.behavior.node_cooldown.select"],
        tree_control_id: "WorkbenchBehaviorAttackRow",
        graph_control_id: "WorkbenchBehaviorNodeRow03",
        title: "BT_Enemy / Cooldown",
        blackboard: "BB_Combat",
        ai_controller: "AIController_CombatEnemy",
        preview_state: "Cooldown 0.8 s",
        trace: "Runtime Trace: Cooldown decorator active (0.8 s)",
    },
];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn initialize_behavior_workspace_state(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.project_behavior_profile(&BEHAVIOR_PROFILES[0])
    }

    pub(super) fn apply_behavior_workspace_action(
        &mut self,
        action_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if action_id == "workbench.module.behavior.validate.invoke" {
            self.apply_behavior_validation_feedback()?;
            return Ok(true);
        }
        let Some(profile) = BEHAVIOR_PROFILES
            .iter()
            .find(|profile| profile.action_ids.contains(&action_id))
        else {
            return Ok(false);
        };
        self.project_behavior_profile(profile)?;
        Ok(true)
    }

    fn project_behavior_profile(
        &mut self,
        profile: &BehaviorNodeProfile,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.select_exclusive_selected(BEHAVIOR_TREE_ROWS, profile.tree_control_id)?;
        self.select_exclusive_selected(BEHAVIOR_GRAPH_ROWS, profile.graph_control_id)?;
        for (control_id, property, value) in [
            ("WorkbenchBehaviorCenterTitle", "text", profile.title),
            (
                "WorkbenchBehaviorBlackboardField",
                "value",
                profile.blackboard,
            ),
            ("WorkbenchBehaviorAiField", "value", profile.ai_controller),
            (
                "WorkbenchBehaviorStateField",
                "value",
                profile.preview_state,
            ),
            ("WorkbenchBehaviorOutputRow", "text", profile.trace),
        ] {
            self.set_behavior_string(control_id, property, value)?;
        }
        Ok(())
    }

    fn apply_behavior_validation_feedback(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let blackboard = self
            .control_string("WorkbenchBehaviorBlackboardField", "value")
            .unwrap_or_default();
        let controller = self
            .control_string("WorkbenchBehaviorAiField", "value")
            .unwrap_or_default();
        let state = self
            .control_string("WorkbenchBehaviorStateField", "value")
            .unwrap_or_default();
        self.set_behavior_string("WorkbenchStatusReady", "text", "Behavior tree validated")?;
        self.set_behavior_string("WorkbenchStatusMessages", "text", "1 Message")?;
        self.set_behavior_string(
            "WorkbenchBehaviorOutputRow",
            "text",
            format!("Validated {blackboard} / {controller}   {state}"),
        )
    }

    fn set_behavior_string(
        &mut self,
        control_id: &str,
        property: &str,
        value: impl Into<String>,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.mutate_control_property(control_id, property, UiValue::String(value.into()))
    }
}

struct BehaviorNodeProfile {
    action_ids: &'static [&'static str],
    tree_control_id: &'static str,
    graph_control_id: &'static str,
    title: &'static str,
    blackboard: &'static str,
    ai_controller: &'static str,
    preview_state: &'static str,
    trace: &'static str,
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue, layout::UiSize};

    use super::*;

    #[test]
    fn tree_graph_details_and_validation_share_one_behavior_profile() {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
                .expect("workbench bridge should build");

        assert!(bridge.control_bool("WorkbenchBehaviorSelectorRow", "selected"));
        assert!(bridge.control_bool("WorkbenchBehaviorNodeRow01", "selected"));

        bridge
            .dispatch_control_state("WorkbenchBehaviorAttackRow", UiEventKind::Click)
            .expect("attack task should dispatch")
            .expect("attack task should bind");
        assert!(bridge.control_bool("WorkbenchBehaviorAttackRow", "selected"));
        assert!(bridge.control_bool("WorkbenchBehaviorNodeRow02", "selected"));
        assert_eq!(
            Some("BT_Enemy / Attack Target".to_string()),
            bridge.control_string("WorkbenchBehaviorCenterTitle", "text")
        );
        assert_eq!(
            Some("Executing".to_string()),
            bridge.control_string("WorkbenchBehaviorStateField", "value")
        );

        bridge
            .dispatch_control_state("WorkbenchBehaviorNodeRow03", UiEventKind::Click)
            .expect("cooldown decorator should dispatch")
            .expect("cooldown decorator should bind");
        assert!(bridge.control_bool("WorkbenchBehaviorAttackRow", "selected"));
        assert!(bridge.control_bool("WorkbenchBehaviorNodeRow03", "selected"));
        assert_eq!(
            Some("Cooldown 0.8 s".to_string()),
            bridge.control_string("WorkbenchBehaviorStateField", "value")
        );
        for (control_id, value) in [
            ("WorkbenchBehaviorBlackboardField", "BB_Custom"),
            ("WorkbenchBehaviorAiField", "AIController_Custom"),
            ("WorkbenchBehaviorStateField", "Paused"),
        ] {
            bridge
                .mutate_control_property(control_id, "value", UiValue::String(value.to_string()))
                .expect("behavior detail should edit");
        }

        bridge
            .dispatch_control_state("WorkbenchBehaviorValidateButton", UiEventKind::Click)
            .expect("behavior validation should dispatch")
            .expect("behavior validation should bind");
        assert_eq!(
            Some("Validated BB_Custom / AIController_Custom   Paused".to_string()),
            bridge.control_string("WorkbenchBehaviorOutputRow", "text")
        );
    }
}
