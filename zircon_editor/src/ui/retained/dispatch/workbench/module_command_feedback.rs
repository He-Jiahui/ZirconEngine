use zircon_runtime_interface::ui::component::UiValue;

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn apply_workbench_module_command_feedback(
        &mut self,
        action_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let active_module = self.active_workbench_feedback_module();
        let Some(feedback) = module_command_feedback(action_id, active_module) else {
            return Ok(());
        };

        self.mutate_control_property(
            "WorkbenchStatusReady",
            "text",
            UiValue::String(feedback.status_text.to_string()),
        )?;
        self.mutate_control_property(
            "WorkbenchStatusMessages",
            "text",
            UiValue::String(feedback.message_count.to_string()),
        )?;
        if let Some(output) = feedback.output {
            self.mutate_control_property(
                output.control_id,
                output.property,
                UiValue::String(output.text.to_string()),
            )?;
        }
        Ok(())
    }

    fn active_workbench_feedback_module(&self) -> WorkbenchFeedbackModule {
        WorkbenchFeedbackModule::from_selected_tab(|control_id| {
            self.control_selected_or_checked(control_id)
        })
    }

    fn control_selected_or_checked(&self, control_id: &str) -> bool {
        let Some(node_id) = self.control_node_id(control_id) else {
            return false;
        };
        self.template_surface
            .surface
            .tree
            .nodes
            .get(&node_id)
            .and_then(|node| node.template_metadata.as_ref())
            .is_some_and(|metadata| {
                metadata
                    .attributes
                    .get("selected")
                    .and_then(toml::Value::as_bool)
                    .unwrap_or(false)
                    || metadata
                        .attributes
                        .get("checked")
                        .and_then(toml::Value::as_bool)
                        .unwrap_or(false)
            })
    }
}

#[cfg(test)]
#[path = "../tests/module_command_feedback_perf.rs"]
mod optimization_batch_20260830bq_editor_tests;

struct ModuleCommandFeedback {
    status_text: &'static str,
    message_count: &'static str,
    output: Option<ModuleOutputFeedback>,
}

struct ModuleOutputFeedback {
    control_id: &'static str,
    property: &'static str,
    text: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WorkbenchFeedbackModule {
    Scene,
    Effect,
    Ability,
    Tags,
    Perception,
    Material,
    Behavior,
    Render,
    Assets,
    Vfx,
    Hud,
}

impl WorkbenchFeedbackModule {
    fn from_selected_tab(is_selected: impl Fn(&str) -> bool) -> Self {
        for (control_id, module) in [
            ("WorkbenchModuleScene", Self::Scene),
            ("WorkbenchModuleEffect", Self::Effect),
            ("WorkbenchModuleAbility", Self::Ability),
            ("WorkbenchModuleTags", Self::Tags),
            ("WorkbenchModulePerception", Self::Perception),
            ("WorkbenchModuleMaterial", Self::Material),
            ("WorkbenchModuleBehavior", Self::Behavior),
            ("WorkbenchModuleRender", Self::Render),
            ("WorkbenchModuleAssets", Self::Assets),
            ("WorkbenchModuleVfx", Self::Vfx),
            ("WorkbenchModuleHud", Self::Hud),
        ] {
            if is_selected(control_id) {
                return module;
            }
        }
        Self::Effect
    }

    fn output(self, text: &'static str) -> Option<ModuleOutputFeedback> {
        match self {
            Self::Scene => None,
            Self::Effect => Some(output("WorkbenchEffectOutputRow", "text", text)),
            Self::Ability => Some(output("WorkbenchAbilityOutputRow", "value_text", text)),
            Self::Tags => Some(output("WorkbenchTagsValidationRow", "value_text", text)),
            Self::Perception => Some(output("WorkbenchPerceptionEventRow", "value_text", text)),
            Self::Material => Some(output("WorkbenchMaterialOutputRow", "text", text)),
            Self::Behavior => Some(output("WorkbenchBehaviorOutputRow", "text", text)),
            Self::Render => Some(output("WorkbenchRenderCaptureRow", "value_text", text)),
            Self::Assets => Some(output("WorkbenchAssetsOutputRow", "text", text)),
            Self::Vfx => Some(output("WorkbenchVfxOutputRow", "text", text)),
            Self::Hud => Some(output("WorkbenchHudValidationRow", "value_text", text)),
        }
    }

    fn save_status(self) -> &'static str {
        "Not implemented (preview)"
    }

    fn save_output(self) -> &'static str {
        "Not implemented (preview)"
    }

    fn compile_status(self) -> &'static str {
        "Not implemented (preview)"
    }

    fn compile_output(self) -> &'static str {
        "Not implemented (preview)"
    }

    fn diff_status(self) -> &'static str {
        "Not implemented (preview)"
    }

    fn diff_output(self) -> &'static str {
        "Not implemented (preview)"
    }

    fn simulate_status(self) -> &'static str {
        "Not implemented (preview)"
    }

    fn simulate_output(self) -> &'static str {
        "Not implemented (preview)"
    }
}

fn module_command_feedback(
    action_id: &str,
    active_module: WorkbenchFeedbackModule,
) -> Option<ModuleCommandFeedback> {
    let feedback = match action_id {
        "workbench.module.save.invoke" => module_feedback(
            active_module.save_status(),
            active_module.output(active_module.save_output()),
        ),
        "workbench.module.browse.invoke" => ModuleCommandFeedback {
            status_text: "Preview workspace selected",
            message_count: "1 Message",
            output: Some(output(
                "WorkbenchAssetsOutputRow",
                "text",
                "Not implemented (preview)",
            )),
        },
        "workbench.module.compile.invoke" => module_feedback(
            active_module.compile_status(),
            active_module.output(active_module.compile_output()),
        ),
        "workbench.module.diff.invoke" => module_feedback(
            active_module.diff_status(),
            active_module.output(active_module.diff_output()),
        ),
        "workbench.module.simulate.invoke" => module_feedback(
            active_module.simulate_status(),
            active_module.output(active_module.simulate_output()),
        ),
        _ => return None,
    };
    Some(feedback)
}

fn module_feedback(
    status_text: &'static str,
    output: Option<ModuleOutputFeedback>,
) -> ModuleCommandFeedback {
    ModuleCommandFeedback {
        status_text,
        message_count: "1 Message",
        output,
    }
}

fn output(
    control_id: &'static str,
    property: &'static str,
    text: &'static str,
) -> ModuleOutputFeedback {
    ModuleOutputFeedback {
        control_id,
        property,
        text,
    }
}
