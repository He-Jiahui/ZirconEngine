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
mod optimization_batch_20260830bq_editor_tests {
    use std::collections::HashMap;
    use std::time::Instant;

    use super::WorkbenchFeedbackModule;

    const SAMPLE_PAIRS: usize = 17;
    const PROBES_PER_SAMPLE: usize = 100_000;
    const MODULE_IDS: [&str; 11] = [
        "WorkbenchModuleScene",
        "WorkbenchModuleEffect",
        "WorkbenchModuleAbility",
        "WorkbenchModuleTags",
        "WorkbenchModulePerception",
        "WorkbenchModuleMaterial",
        "WorkbenchModuleBehavior",
        "WorkbenchModuleRender",
        "WorkbenchModuleAssets",
        "WorkbenchModuleVfx",
        "WorkbenchModuleHud",
    ];

    #[test]
    fn selected_tab_resolution_preserves_order_and_default() {
        assert_eq!(
            WorkbenchFeedbackModule::from_selected_tab(|id| id == "WorkbenchModuleRender"),
            WorkbenchFeedbackModule::Render
        );
        assert_eq!(
            WorkbenchFeedbackModule::from_selected_tab(|_| false),
            WorkbenchFeedbackModule::Effect
        );
    }

    #[test]
    fn selected_tab_resolution_uses_one_node_lookup_for_both_flags() {
        let source = include_str!("module_command_feedback.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(implementation.contains("fn control_selected_or_checked"));
        assert!(implementation.contains(".nodes"));
        assert!(implementation.contains(".get(&node_id)"));
        assert!(!implementation.contains(
            "self.control_bool(control_id, \"selected\") || self.control_bool(control_id, \"checked\")"
        ));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830bq_editor_workbench_module_lookup_p95() {
        let nodes = MODULE_IDS
            .iter()
            .enumerate()
            .map(|(index, id)| (*id, [index == 7, index == 9]))
            .collect::<HashMap<_, _>>();
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(&nodes, false));
                optimized.push(measure(&nodes, true));
            } else {
                optimized.push(measure(&nodes, true));
                legacy.push(measure(&nodes, false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "EDITOR315_WORKBENCH_MODULE_LOOKUP_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} probes_per_sample={PROBES_PER_SAMPLE} controls={} legacy_node_lookups_per_probe=2 optimized_node_lookups_per_probe=1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            MODULE_IDS.len(),
            sample_csv(&legacy),
            sample_csv(&optimized),
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(nodes: &HashMap<&'static str, [bool; 2]>, optimized: bool) -> u128 {
        let started = Instant::now();
        let mut selected_count = 0usize;
        for _ in 0..PROBES_PER_SAMPLE {
            for id in MODULE_IDS {
                let selected = if optimized {
                    nodes.get(id).is_some_and(|flags| flags[0] || flags[1])
                } else {
                    nodes.get(id).is_some_and(|flags| flags[0])
                        || nodes.get(id).is_some_and(|flags| flags[1])
                };
                selected_count += selected as usize;
            }
        }
        std::hint::black_box(selected_count);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn sample_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
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
            Self::Perception => "Perception simulation tick",
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
