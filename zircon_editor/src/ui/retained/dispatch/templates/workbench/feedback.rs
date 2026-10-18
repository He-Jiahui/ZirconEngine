use zircon_runtime_interface::ui::component::UiValue;

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

#[path = "../../workbench/feedback/data_production.rs"]
mod data_production;
#[path = "../../workbench/feedback/gameplay_state.rs"]
mod gameplay_state;
#[path = "../../workbench/feedback/live_input_summary.rs"]
mod live_input_summary;
#[path = "../../workbench/feedback/online_sessions.rs"]
mod online_sessions;
#[path = "../../workbench/feedback/runtime_state.rs"]
mod runtime_state;
#[path = "../../workbench/feedback/simulation_physics.rs"]
mod simulation_physics;
#[path = "../../workbench/feedback/ui_diagnostics.rs"]
mod ui_diagnostics;

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn apply_workbench_extension_module_command_feedback(
        &mut self,
        action_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let (output_control_id, status_text, output_text) =
            if action_id == "workbench.extension.icon_library.find_usage.invoke" {
                (
                    "WorkbenchExtensionIconLibraryOutputRow",
                    self.i18n
                        .translate("workbench.extension.icon_library.find_usage.unavailable")
                        .to_string(),
                    self.i18n
                        .translate("workbench.extension.icon_library.find_usage.no_scanner")
                        .to_string(),
                )
            } else {
                let Some(feedback) = extension_module_feedback(action_id) else {
                    return Ok(());
                };
                let output_text = live_input_summary::for_command(self, action_id)
                    .map(|inputs| format!("Not implemented (preview) | {inputs}"))
                    .unwrap_or_else(|| feedback.output_text.to_string());
                (
                    feedback.output_control_id,
                    feedback.status_text.to_string(),
                    output_text,
                )
            };

        self.mutate_control_property("WorkbenchStatusReady", "text", UiValue::String(status_text))?;
        self.mutate_control_property(
            "WorkbenchStatusMessages",
            "text",
            UiValue::String("1 Message".to_string()),
        )?;
        self.mutate_control_property(
            output_control_id,
            "value_text",
            UiValue::String(output_text),
        )?;
        Ok(())
    }
}

struct ExtensionModuleFeedback {
    output_control_id: &'static str,
    status_text: &'static str,
    output_text: &'static str,
}

fn extension_module_feedback(action_id: &str) -> Option<ExtensionModuleFeedback> {
    if let Some(feedback) = gameplay_state::feedback(action_id)
        .or_else(|| simulation_physics::feedback(action_id))
        .or_else(|| online_sessions::feedback(action_id))
        .or_else(|| runtime_state::feedback(action_id))
        .or_else(|| ui_diagnostics::feedback(action_id))
        .or_else(|| data_production::feedback(action_id))
    {
        return Some(feedback);
    }

    let feedback = match action_id {
        "workbench.extension.shader_editor.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionShaderOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.shader_editor.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionShaderOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.shader_editor.compile.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionShaderOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.shader_editor.source_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionShaderOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.shader_editor.fragment_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionShaderOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.lighting_bake.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLightingBakeOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.lighting_bake.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLightingBakeOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.lighting_bake.bake.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLightingBakeOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.lighting_bake.lightmap_uv_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionLightingBakeOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.lighting_bake.bleed_warning_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionLightingBakeOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.post_process.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPostProcessOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.post_process.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPostProcessOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.post_process.apply.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPostProcessOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.post_process.tonemap_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPostProcessOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.post_process.exposure_warning_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionPostProcessOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.sequencer.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSequencerOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.sequencer.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSequencerOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.sequencer.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSequencerOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.sequencer.hero_transform_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionSequencerOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.sequencer.event_cue_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSequencerOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.montage_editor.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMontageEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.montage_editor.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMontageEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.montage_editor.apply.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMontageEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.montage_editor.intro_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMontageEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.montage_editor.root_motion_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionMontageEditorOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.blend_space.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionBlendSpaceOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.blend_space.validation.filter_all" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionBlendSpaceOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.blend_space.validation.filter_errors" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionBlendSpaceOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.blend_space.validation.filter_warnings" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionBlendSpaceOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.blend_space.validation.filter_infos" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionBlendSpaceOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.blend_space.validation.clear" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionBlendSpaceOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.pose_library.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPoseLibraryOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.pose_library.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPoseLibraryOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.pose_library.apply.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPoseLibraryOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.pose_library.idle_pose_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPoseLibraryOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.pose_library.mirror_pose_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionPoseLibraryOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.retarget.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionRetargetOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.retarget.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionRetargetOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.retarget.apply.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionRetargetOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.retarget.root_chain_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionRetargetOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.retarget.leg_chain_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionRetargetOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.control_rig.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionControlRigOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.control_rig.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionControlRigOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.control_rig.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionControlRigOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.control_rig.spine_control_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionControlRigOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.control_rig.hand_ik_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionControlRigOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.motion_matching.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMotionMatchingOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.motion_matching.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMotionMatchingOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.motion_matching.rebuild.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMotionMatchingOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.motion_matching.idle_clip_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionMotionMatchingOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.motion_matching.pivot_clip_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionMotionMatchingOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.animation_compression.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionAnimationCompressionOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.animation_compression.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionAnimationCompressionOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.animation_compression.compress.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionAnimationCompressionOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.animation_compression.run_clip_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionAnimationCompressionOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.animation_compression.error_clip_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionAnimationCompressionOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.terrain_editor.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionTerrainEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.terrain_editor.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionTerrainEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.terrain_editor.build.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionTerrainEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.terrain_editor.cell_a_1208_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionTerrainEditorOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.terrain_editor.streaming_cell_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionTerrainEditorOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.foliage_editor.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionFoliageEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.foliage_editor.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionFoliageEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.foliage_editor.build.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionFoliageEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.foliage_editor.forest_a_13_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionFoliageEditorOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.foliage_editor.cliff_01_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionFoliageEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.level_streaming.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLevelStreamingOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.level_streaming.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLevelStreamingOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.level_streaming.load.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLevelStreamingOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.level_streaming.cell_a_13_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionLevelStreamingOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.level_streaming.cell_b_12_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionLevelStreamingOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.level_variant.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLevelVariantOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.level_variant.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLevelVariantOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.level_variant.apply.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLevelVariantOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.level_variant.car_body_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLevelVariantOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.level_variant.door_l_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLevelVariantOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.prefab_editor.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPrefabEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.prefab_editor.apply.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPrefabEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.prefab_editor.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPrefabEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.prefab_editor.loot_socket_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionPrefabEditorOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.prefab_editor.override_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPrefabEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.scatter_editor.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionScatterEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.scatter_editor.generate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionScatterEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.scatter_editor.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionScatterEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.scatter_editor.slope_filter_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionScatterEditorOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.scatter_editor.collision_test_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionScatterEditorOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.volume_editor.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionVolumeEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.volume_editor.inspect.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionVolumeEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.volume_editor.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionVolumeEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.volume_editor.player_overlap_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionVolumeEditorOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.volume_editor.on_enter_event_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionVolumeEditorOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.weather_editor.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionWeatherEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.weather_editor.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionWeatherEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.weather_editor.build.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionWeatherEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.weather_editor.rain_burst_timeline_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionWeatherEditorOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.weather_editor.lightning_timeline_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionWeatherEditorOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.particle_library.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionParticleLibraryOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.particle_library.simulate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionParticleLibraryOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.particle_library.compile.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionParticleLibraryOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.particle_library.gpu_spark_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionParticleLibraryOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.particle_library.archived_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionParticleLibraryOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.ui_asset_editor.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionUiAssetEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.ui_asset_editor.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionUiAssetEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.ui_asset_editor.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionUiAssetEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.ui_asset_editor.selected_button_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionUiAssetEditorOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.ui_asset_editor.binding_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionUiAssetEditorOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.ui_binding.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionUiBindingOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.ui_binding.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionUiBindingOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.ui_binding.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionUiBindingOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.ui_binding.health_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionUiBindingOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.ui_binding.validation_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionUiBindingOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.icon_library.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionIconLibraryOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.icon_library.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionIconLibraryOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.icon_library.warning_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionIconLibraryOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.icon_library.archived_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionIconLibraryOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.accessibility_audit.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionAccessibilityAuditOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.accessibility_audit.audit_screen.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionAccessibilityAuditOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.accessibility_audit.preview_fix.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionAccessibilityAuditOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.accessibility_audit.contrast_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionAccessibilityAuditOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.accessibility_audit.focus_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionAccessibilityAuditOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.menu_flow.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMenuFlowOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.menu_flow.preview_flow.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMenuFlowOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.menu_flow.validate_focus.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMenuFlowOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.menu_flow.start_node_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMenuFlowOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.menu_flow.exit_route_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMenuFlowOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.font_atlas.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionFontAtlasOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.font_atlas.bake_atlas.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionFontAtlasOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.font_atlas.inspect_glyph.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionFontAtlasOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.font_atlas.latin_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionFontAtlasOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.font_atlas.cjk_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionFontAtlasOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.console_diagnostics.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionConsoleDiagnosticsOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.console_diagnostics.filter_console.invoke" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionConsoleDiagnosticsOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.console_diagnostics.clear_console.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionConsoleDiagnosticsOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.console_diagnostics.warning_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionConsoleDiagnosticsOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.console_diagnostics.error_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionConsoleDiagnosticsOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        _ => return None,
    };
    Some(feedback)
}

#[cfg(test)]
#[path = "tests/feedback.rs"]
mod tests;
