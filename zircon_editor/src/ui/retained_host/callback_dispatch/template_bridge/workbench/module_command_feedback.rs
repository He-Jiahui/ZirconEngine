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
            self.control_bool(control_id, "selected") || self.control_bool(control_id, "checked")
        })
    }
}

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

#[derive(Clone, Copy)]
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
        match self {
            Self::Scene => "Scene saved",
            Self::Effect => "Gameplay effect saved",
            Self::Ability => "Ability saved",
            Self::Tags => "Gameplay tags saved",
            Self::Perception => "Perception profile saved",
            Self::Material => "Material saved",
            Self::Behavior => "Behavior tree saved",
            Self::Render => "Render pipeline saved",
            Self::Assets => "Asset selection saved",
            Self::Vfx => "VFX system saved",
            Self::Hud => "HUD layout saved",
        }
    }

    fn save_output(self) -> &'static str {
        match self {
            Self::Scene => "Save: scene sample persisted",
            Self::Effect => "Save: GE_HealthRegen sample persisted",
            Self::Ability => "Save: GA_DashAttack sample persisted",
            Self::Tags => "Save: gameplay tag registry persisted",
            Self::Perception => "Save: AI perception profile persisted",
            Self::Material => "Save: M_Rock_Cliff preview persisted",
            Self::Behavior => "Save: BT_Enemy sample persisted",
            Self::Render => "Save: Forward Plus renderer profile persisted",
            Self::Assets => "Save: asset browser filters persisted",
            Self::Vfx => "Save: P_Bolt_01 system persisted",
            Self::Hud => "Save: Gameplay_HUD layout persisted",
        }
    }

    fn compile_status(self) -> &'static str {
        match self {
            Self::Scene => "Scene command previewed",
            Self::Effect => "Gameplay effect compile queued",
            Self::Ability => "Ability compile queued",
            Self::Tags => "Gameplay tags compile queued",
            Self::Perception => "Perception query compile queued",
            Self::Material => "Material compile queued",
            Self::Behavior => "Behavior tree compile queued",
            Self::Render => "Render graph compile queued",
            Self::Assets => "Asset cook queued",
            Self::Vfx => "VFX compile queued",
            Self::Hud => "HUD compile queued",
        }
    }

    fn compile_output(self) -> &'static str {
        match self {
            Self::Scene => "Compile: scene has no module output surface",
            Self::Effect => "Compile Log: GE_HealthRegen compile queued",
            Self::Ability => "Compile Log: GA_DashAttack compile queued",
            Self::Tags => "Compile Log: gameplay tag table queued",
            Self::Perception => "Compile Log: sight query compile queued",
            Self::Material => "Shader Output: material compile queued",
            Self::Behavior => "Runtime Trace: behavior tree compile queued",
            Self::Render => "Compile Output: render graph compile queued",
            Self::Assets => "Cook: asset build graph queued",
            Self::Vfx => "Compile Output: E_Bolt compile queued",
            Self::Hud => "Compile Log: widget tree compile queued",
        }
    }

    fn diff_status(self) -> &'static str {
        match self {
            Self::Scene => "Scene diff prepared",
            Self::Effect => "Gameplay effect diff prepared",
            Self::Ability => "Ability diff prepared",
            Self::Tags => "Gameplay tags diff prepared",
            Self::Perception => "Perception diff prepared",
            Self::Material => "Material diff prepared",
            Self::Behavior => "Behavior tree diff prepared",
            Self::Render => "Render pipeline diff prepared",
            Self::Assets => "Asset diff prepared",
            Self::Vfx => "VFX diff prepared",
            Self::Hud => "HUD diff prepared",
        }
    }

    fn diff_output(self) -> &'static str {
        match self {
            Self::Scene => "Diff: scene preview state compared",
            Self::Effect => "Diff: gameplay effect modifier changes compared",
            Self::Ability => "Diff: ability graph changes compared",
            Self::Tags => "Diff: tag redirect changes compared",
            Self::Perception => "Diff: perception filter changes compared",
            Self::Material => "Diff: material graph changes compared",
            Self::Behavior => "Diff: behavior tree branch changes compared",
            Self::Render => "Diff: render pipeline pass changes compared",
            Self::Assets => "Diff: asset metadata changes compared",
            Self::Vfx => "Diff: emitter stack changes compared",
            Self::Hud => "Diff: HUD widget hierarchy changes compared",
        }
    }

    fn simulate_status(self) -> &'static str {
        match self {
            Self::Scene => "Scene preview running",
            Self::Effect => "Gameplay effect simulation running",
            Self::Ability => "Ability simulation running",
            Self::Tags => "Gameplay tag validation running",
            Self::Perception => "Perception simulation running",
            Self::Material => "Material preview running",
            Self::Behavior => "Behavior tree simulation running",
            Self::Render => "Render frame preview running",
            Self::Assets => "Asset validation running",
            Self::Vfx => "VFX simulation running",
            Self::Hud => "HUD preview running",
        }
    }

    fn simulate_output(self) -> &'static str {
        match self {
            Self::Scene => "Preview: scene viewport sample running",
            Self::Effect => "Simulation Output: gameplay effect preview running",
            Self::Ability => "Simulation: ability playtest preview running",
            Self::Tags => "Validation: gameplay tag references scanning",
            Self::Perception => "AI_Guard_01   simulation tick   00:12.4",
            Self::Material => "Preview Variants: material preview refreshed",
            Self::Behavior => "Runtime Trace: behavior tree preview running",
            Self::Render => "Frame Preview: render graph frame queued",
            Self::Assets => "Validation: asset scan queued",
            Self::Vfx => "Simulation: preview running at 60 fps",
            Self::Hud => "Preview refreshed   localization warning remains",
        }
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
            status_text: "Asset browser focused",
            message_count: "1 Message",
            output: Some(output(
                "WorkbenchAssetsOutputRow",
                "text",
                "Browse: focused Content/Environment/Forest",
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
        "workbench.module.effect.apply.invoke" => ModuleCommandFeedback {
            status_text: "Gameplay effect applied",
            message_count: "1 Message",
            output: Some(output(
                "WorkbenchEffectOutputRow",
                "text",
                "Simulation Output: applied +50 health preview",
            )),
        },
        "workbench.module.material.compile.invoke" => ModuleCommandFeedback {
            status_text: "Material compiled",
            message_count: "1 Message",
            output: Some(output(
                "WorkbenchMaterialOutputRow",
                "text",
                "Shader Output: compile complete, 2 warnings",
            )),
        },
        "workbench.module.behavior.validate.invoke" => ModuleCommandFeedback {
            status_text: "Behavior tree validated",
            message_count: "1 Message",
            output: Some(output(
                "WorkbenchBehaviorOutputRow",
                "text",
                "Validation: selector branch is reachable",
            )),
        },
        "workbench.module.assets.import.invoke" => ModuleCommandFeedback {
            status_text: "Asset import queued",
            message_count: "1 Message",
            output: Some(output(
                "WorkbenchAssetsOutputRow",
                "text",
                "Import: queued SM_Tree_Oak_01 and dependencies",
            )),
        },
        "workbench.module.vfx.simulate.invoke" => ModuleCommandFeedback {
            status_text: "VFX simulation running",
            message_count: "1 Message",
            output: Some(output(
                "WorkbenchVfxOutputRow",
                "text",
                "Compile Output: simulation running, no errors",
            )),
        },
        "workbench.module.ability.playtest.invoke" => ModuleCommandFeedback {
            status_text: "Ability playtest queued",
            message_count: "1 Message",
            output: Some(output(
                "WorkbenchAbilityOutputRow",
                "value_text",
                "Playtest queued   predicted activation   GA_DashAttack",
            )),
        },
        "workbench.module.tags.add.invoke" => ModuleCommandFeedback {
            status_text: "Tag add dialog prepared",
            message_count: "1 Message",
            output: Some(output(
                "WorkbenchTagsValidationRow",
                "value_text",
                "Add Tag   pending registry update",
            )),
        },
        "workbench.module.tags.rename.invoke" => ModuleCommandFeedback {
            status_text: "Tag rename prepared",
            message_count: "1 Message",
            output: Some(output(
                "WorkbenchTagsValidationRow",
                "value_text",
                "Rename Tag   pending redirect update",
            )),
        },
        "workbench.module.perception.simulate.invoke" => ModuleCommandFeedback {
            status_text: "Perception simulation running",
            message_count: "1 Message",
            output: Some(output(
                "WorkbenchPerceptionEventRow",
                "value_text",
                "AI_Guard_01   simulation tick   00:12.4",
            )),
        },
        "workbench.module.render.compile.invoke" => ModuleCommandFeedback {
            status_text: "Render graph compiled",
            message_count: "1 Message",
            output: Some(output(
                "WorkbenchRenderCaptureRow",
                "value_text",
                "Windows DX12   30 fps   GPU 6.24 ms   compiled",
            )),
        },
        "workbench.module.hud.preview.invoke" => ModuleCommandFeedback {
            status_text: "HUD preview refreshed",
            message_count: "1 Message",
            output: Some(output(
                "WorkbenchHudValidationRow",
                "value_text",
                "Preview refreshed   localization warning remains",
            )),
        },
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
