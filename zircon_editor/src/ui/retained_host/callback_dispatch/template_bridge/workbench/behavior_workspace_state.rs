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
#[path = "tests/behavior_workspace_state.rs"]
mod tests;
