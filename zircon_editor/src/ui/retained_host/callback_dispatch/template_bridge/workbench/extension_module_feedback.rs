use zircon_runtime_interface::ui::component::UiValue;

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

mod data_production;
mod gameplay_state;
mod live_input_summary;
mod online_sessions;
mod runtime_state;
mod simulation_physics;
mod ui_diagnostics;

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn apply_workbench_extension_module_command_feedback(
        &mut self,
        action_id: &str,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let Some(feedback) = extension_module_feedback(action_id) else {
            return Ok(());
        };
        let output_text = live_input_summary::for_command(self, action_id)
            .unwrap_or_else(|| feedback.output_text.to_string());

        self.mutate_control_property(
            "WorkbenchStatusReady",
            "text",
            UiValue::String(feedback.status_text.to_string()),
        )?;
        self.mutate_control_property(
            "WorkbenchStatusMessages",
            "text",
            UiValue::String("1 Message".to_string()),
        )?;
        self.mutate_control_property(
            feedback.output_control_id,
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
            status_text: "Shader editor opened",
            output_text: "Native extension workspace opened for lighting.wgsl",
        },
        "workbench.extension.shader_editor.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionShaderOutputRow",
            status_text: "Shader preview queued",
            output_text: "Preview queued   fragment stage   lighting.wgsl",
        },
        "workbench.extension.shader_editor.compile.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionShaderOutputRow",
            status_text: "Shader compile queued",
            output_text: "Compile queued   3 warnings   0 errors",
        },
        "workbench.extension.shader_editor.source_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionShaderOutputRow",
            status_text: "Shader source selected",
            output_text: "Source selected   lighting.wgsl",
        },
        "workbench.extension.shader_editor.fragment_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionShaderOutputRow",
            status_text: "Shader fragment stage selected",
            output_text: "Fragment stage selected   fs_main   GBuffer",
        },
        "workbench.extension.lighting_bake.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLightingBakeOutputRow",
            status_text: "Lighting bake opened",
            output_text: "Native extension workspace opened for City_Block_A",
        },
        "workbench.extension.lighting_bake.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLightingBakeOutputRow",
            status_text: "Lighting bake preview queued",
            output_text: "Preview queued   City_Block_A   production quality",
        },
        "workbench.extension.lighting_bake.bake.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLightingBakeOutputRow",
            status_text: "Lighting bake queued",
            output_text: "Bake queued   87 lightmaps   estimate 02:30",
        },
        "workbench.extension.lighting_bake.lightmap_uv_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionLightingBakeOutputRow",
                status_text: "Lighting bake lightmap row selected",
                output_text: "Selected Lightmap UV   87 assets   4 warnings",
            }
        }
        "workbench.extension.lighting_bake.bleed_warning_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionLightingBakeOutputRow",
                status_text: "Lighting bake warning selected",
                output_text: "Selected Bleed Warning   Interior_Lab   6 texels",
            }
        }
        "workbench.extension.post_process.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPostProcessOutputRow",
            status_text: "Post process opened",
            output_text: "Native extension workspace opened for PPV_CityGlobal",
        },
        "workbench.extension.post_process.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPostProcessOutputRow",
            status_text: "Post process preview queued",
            output_text: "Preview queued   Cinematic profile   weighted blend",
        },
        "workbench.extension.post_process.apply.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPostProcessOutputRow",
            status_text: "Post process apply queued",
            output_text: "Apply queued   Global Stack   1 warning",
        },
        "workbench.extension.post_process.tonemap_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPostProcessOutputRow",
            status_text: "Post process tonemap selected",
            output_text: "Selected Tonemap   Filmic   exposure +0.4",
        },
        "workbench.extension.post_process.exposure_warning_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionPostProcessOutputRow",
                status_text: "Post process warning selected",
                output_text: "Selected Exposure Warning   Interior volume   EV +2.1",
            }
        }
        "workbench.extension.sequencer.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSequencerOutputRow",
            status_text: "Sequencer opened",
            output_text: "Native extension workspace opened for SEQ_Intro",
        },
        "workbench.extension.sequencer.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSequencerOutputRow",
            status_text: "Sequencer preview queued",
            output_text: "Preview queued   SEQ_Intro   24 fps",
        },
        "workbench.extension.sequencer.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSequencerOutputRow",
            status_text: "Sequencer validation queued",
            output_text: "Validation queued   12 shots   1 gap",
        },
        "workbench.extension.sequencer.hero_transform_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionSequencerOutputRow",
                status_text: "Sequencer hero track selected",
                output_text: "Selected Hero Transform   0180-0620   keyed",
            }
        }
        "workbench.extension.sequencer.event_cue_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionSequencerOutputRow",
            status_text: "Sequencer event cue selected",
            output_text: "Selected Event Cues   0520-0860   Warning",
        },
        "workbench.extension.montage_editor.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMontageEditorOutputRow",
            status_text: "Montage editor opened",
            output_text: "Native extension workspace opened for AM_DashAttack",
        },
        "workbench.extension.montage_editor.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMontageEditorOutputRow",
            status_text: "Montage preview queued",
            output_text: "Preview queued   AM_DashAttack   UpperBody slot",
        },
        "workbench.extension.montage_editor.apply.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMontageEditorOutputRow",
            status_text: "Montage apply queued",
            output_text: "Apply queued   4 sections   3 notifies",
        },
        "workbench.extension.montage_editor.intro_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMontageEditorOutputRow",
            status_text: "Montage intro selected",
            output_text: "Selected Intro Section   0.00-0.38s   UpperBody",
        },
        "workbench.extension.montage_editor.root_motion_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionMontageEditorOutputRow",
                status_text: "Montage root motion selected",
                output_text: "Selected Root Motion   extracted   2.8m forward",
            }
        }
        "workbench.extension.blend_space.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionBlendSpaceOutputRow",
            status_text: "Blend space opened",
            output_text: "Native extension workspace opened for BS_Locomotion",
        },
        "workbench.extension.blend_space.validation.filter_all" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionBlendSpaceOutputRow",
            status_text: "Validation filter: All",
            output_text: "Showing all validation diagnostics   3 info   1 warning   0 errors",
        },
        "workbench.extension.blend_space.validation.filter_errors" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionBlendSpaceOutputRow",
            status_text: "Validation filter: Errors",
            output_text: "Showing validation errors   0 results",
        },
        "workbench.extension.blend_space.validation.filter_warnings" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionBlendSpaceOutputRow",
            status_text: "Validation filter: Warnings",
            output_text: "Showing validation warnings   1 result",
        },
        "workbench.extension.blend_space.validation.filter_infos" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionBlendSpaceOutputRow",
            status_text: "Validation filter: Infos",
            output_text: "Showing validation info   3 results",
        },
        "workbench.extension.blend_space.validation.clear" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionBlendSpaceOutputRow",
            status_text: "Validation log cleared",
            output_text: "Validation diagnostics cleared   0 results",
        },
        "workbench.extension.pose_library.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPoseLibraryOutputRow",
            status_text: "Pose library opened",
            output_text: "Native extension workspace opened for PL_Combat",
        },
        "workbench.extension.pose_library.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPoseLibraryOutputRow",
            status_text: "Pose library preview queued",
            output_text: "Preview queued   PL_Combat   Combat.Ready tag",
        },
        "workbench.extension.pose_library.apply.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPoseLibraryOutputRow",
            status_text: "Pose library apply queued",
            output_text: "Apply queued   42 poses   6 tags",
        },
        "workbench.extension.pose_library.idle_pose_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPoseLibraryOutputRow",
            status_text: "Pose library idle pose selected",
            output_text: "Selected Idle Ready   Full body   Combat",
        },
        "workbench.extension.pose_library.mirror_pose_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionPoseLibraryOutputRow",
                status_text: "Pose library mirror candidate selected",
                output_text: "Selected Mirror Candidate   left hand pair missing",
            }
        }
        "workbench.extension.retarget.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionRetargetOutputRow",
            status_text: "Retarget opened",
            output_text: "Native extension workspace opened for SK_Mannequin",
        },
        "workbench.extension.retarget.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionRetargetOutputRow",
            status_text: "Retarget preview queued",
            output_text: "Preview queued   SK_Mannequin -> SK_Robot   Full Body IK",
        },
        "workbench.extension.retarget.apply.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionRetargetOutputRow",
            status_text: "Retarget apply queued",
            output_text: "Apply queued   4 chains   1 foot lock warning",
        },
        "workbench.extension.retarget.root_chain_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionRetargetOutputRow",
            status_text: "Retarget root chain selected",
            output_text: "Selected Root Chain   pelvis -> root   locked",
        },
        "workbench.extension.retarget.leg_chain_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionRetargetOutputRow",
            status_text: "Retarget leg chain selected",
            output_text: "Selected Leg Chain   thigh..ball   foot lock warning",
        },
        "workbench.extension.control_rig.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionControlRigOutputRow",
            status_text: "Control rig opened",
            output_text: "Native extension workspace opened for CR_Hero",
        },
        "workbench.extension.control_rig.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionControlRigOutputRow",
            status_text: "Control rig preview queued",
            output_text: "Preview queued   CR_Hero   Hand_IK_L selected",
        },
        "workbench.extension.control_rig.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionControlRigOutputRow",
            status_text: "Control rig validation queued",
            output_text: "Validation queued   64 controls   1 warning",
        },
        "workbench.extension.control_rig.spine_control_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionControlRigOutputRow",
                status_text: "Control rig spine selected",
                output_text: "Selected Spine_CTRL   FK   world space",
            }
        }
        "workbench.extension.control_rig.hand_ik_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionControlRigOutputRow",
            status_text: "Control rig hand IK selected",
            output_text: "Selected Hand_IK_L   space switch keyed",
        },
        "workbench.extension.motion_matching.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMotionMatchingOutputRow",
            status_text: "Motion matching opened",
            output_text: "Native extension workspace opened for MM_Locomotion",
        },
        "workbench.extension.motion_matching.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMotionMatchingOutputRow",
            status_text: "Motion matching preview queued",
            output_text: "Preview queued   MM_Locomotion   trajectory 0.8s",
        },
        "workbench.extension.motion_matching.rebuild.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMotionMatchingOutputRow",
            status_text: "Motion matching rebuild queued",
            output_text: "Rebuild queued   184 clips   1 warning",
        },
        "workbench.extension.motion_matching.idle_clip_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionMotionMatchingOutputRow",
                status_text: "Motion matching idle clip selected",
                output_text: "Selected Idle_Breath   cost 0.04   relaxed stance",
            }
        }
        "workbench.extension.motion_matching.pivot_clip_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionMotionMatchingOutputRow",
                status_text: "Motion matching pivot clip selected",
                output_text: "Selected Pivot_Left_90   turn bias warning",
            }
        }
        "workbench.extension.animation_compression.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionAnimationCompressionOutputRow",
            status_text: "Animation compression opened",
            output_text: "Native extension workspace opened for AC_Locomotion",
        },
        "workbench.extension.animation_compression.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionAnimationCompressionOutputRow",
            status_text: "Animation compression preview queued",
            output_text: "Preview queued   AC_Locomotion   tolerance 0.5 cm",
        },
        "workbench.extension.animation_compression.compress.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionAnimationCompressionOutputRow",
            status_text: "Animation compression queued",
            output_text: "Compress queued   312 clips   3 warnings",
        },
        "workbench.extension.animation_compression.run_clip_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionAnimationCompressionOutputRow",
                status_text: "Animation compression run clip selected",
                output_text: "Selected Run_Fwd   ratio 8.2:1   max err 0.18 cm",
            }
        }
        "workbench.extension.animation_compression.error_clip_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionAnimationCompressionOutputRow",
                status_text: "Animation compression warning selected",
                output_text: "Selected Turn_180   ratio 10.5:1   max err 1.8 cm",
            }
        }
        "workbench.extension.terrain_editor.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionTerrainEditorOutputRow",
            status_text: "Terrain editor opened",
            output_text: "Native extension workspace opened for Summit Valley",
        },
        "workbench.extension.terrain_editor.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionTerrainEditorOutputRow",
            status_text: "Terrain preview queued",
            output_text: "Preview queued   Summit Valley   brush radius 512",
        },
        "workbench.extension.terrain_editor.build.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionTerrainEditorOutputRow",
            status_text: "Terrain build queued",
            output_text: "Build queued   64 cells   2 warnings",
        },
        "workbench.extension.terrain_editor.cell_a_1208_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionTerrainEditorOutputRow",
                status_text: "Terrain cell selected",
                output_text: "Selected A12_08   Rock   Loaded",
            }
        }
        "workbench.extension.terrain_editor.streaming_cell_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionTerrainEditorOutputRow",
                status_text: "Terrain streaming cell selected",
                output_text: "Selected A13_08   Visible   High priority",
            }
        }
        "workbench.extension.foliage_editor.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionFoliageEditorOutputRow",
            status_text: "Foliage editor opened",
            output_text: "Native extension workspace opened for FOL_Forest",
        },
        "workbench.extension.foliage_editor.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionFoliageEditorOutputRow",
            status_text: "Foliage density preview queued",
            output_text: "Preview queued   84K instances   density 0.72",
        },
        "workbench.extension.foliage_editor.build.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionFoliageEditorOutputRow",
            status_text: "Foliage cluster build queued",
            output_text: "Build queued   128 clusters   2 warnings",
        },
        "workbench.extension.foliage_editor.forest_a_13_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionFoliageEditorOutputRow",
                status_text: "Foliage cluster selected",
                output_text: "Selected Forest_A13   Fern   density 0.58",
            }
        }
        "workbench.extension.foliage_editor.cliff_01_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionFoliageEditorOutputRow",
            status_text: "Foliage validation row selected",
            output_text: "Selected Cliff_01   steep slope warning",
        },
        "workbench.extension.level_streaming.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLevelStreamingOutputRow",
            status_text: "Level streaming opened",
            output_text: "Native extension workspace opened for World_Main",
        },
        "workbench.extension.level_streaming.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLevelStreamingOutputRow",
            status_text: "Level streaming preview queued",
            output_text: "Preview queued   96 cells   player distance",
        },
        "workbench.extension.level_streaming.load.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLevelStreamingOutputRow",
            status_text: "Level streaming load queued",
            output_text: "Load queued   Cell_A12   async",
        },
        "workbench.extension.level_streaming.cell_a_13_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionLevelStreamingOutputRow",
                status_text: "Streaming cell selected",
                output_text: "Selected Cell_A13   queued   96 MB",
            }
        }
        "workbench.extension.level_streaming.cell_b_12_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionLevelStreamingOutputRow",
                status_text: "Streaming warning selected",
                output_text: "Selected Cell_B12   hidden   out of range",
            }
        }
        "workbench.extension.level_variant.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLevelVariantOutputRow",
            status_text: "Level variant opened",
            output_text: "Native extension workspace opened for Vehicle_Showcase",
        },
        "workbench.extension.level_variant.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLevelVariantOutputRow",
            status_text: "Level variant preview queued",
            output_text: "Preview queued   Variant_Red   18 overrides",
        },
        "workbench.extension.level_variant.apply.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLevelVariantOutputRow",
            status_text: "Level variant apply queued",
            output_text: "Apply queued   18 overrides   2 conflicts",
        },
        "workbench.extension.level_variant.car_body_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLevelVariantOutputRow",
            status_text: "Level variant override selected",
            output_text: "Selected CarBody   Material   M_RedPaint",
        },
        "workbench.extension.level_variant.door_l_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionLevelVariantOutputRow",
            status_text: "Level variant conflict selected",
            output_text: "Selected Door_L   Transform conflict",
        },
        "workbench.extension.prefab_editor.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPrefabEditorOutputRow",
            status_text: "Prefab editor opened",
            output_text: "Native extension workspace opened for PF_Chest",
        },
        "workbench.extension.prefab_editor.apply.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPrefabEditorOutputRow",
            status_text: "Prefab override apply queued",
            output_text: "Apply queued   Chest_04   6 overrides",
        },
        "workbench.extension.prefab_editor.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPrefabEditorOutputRow",
            status_text: "Prefab validation queued",
            output_text: "Validation queued   18 children   2 warnings",
        },
        "workbench.extension.prefab_editor.loot_socket_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionPrefabEditorOutputRow",
                status_text: "Prefab socket selected",
                output_text: "Selected LootSocket   LootDrop   Ready",
            }
        }
        "workbench.extension.prefab_editor.override_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPrefabEditorOutputRow",
            status_text: "Prefab override selected",
            output_text: "Selected Override_Open   Chest_04 warning",
        },
        "workbench.extension.scatter_editor.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionScatterEditorOutputRow",
            status_text: "Scatter editor opened",
            output_text: "Native extension workspace opened for SC_Forest",
        },
        "workbench.extension.scatter_editor.generate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionScatterEditorOutputRow",
            status_text: "Scatter generation queued",
            output_text: "Generate queued   SC_Forest   64K instances",
        },
        "workbench.extension.scatter_editor.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionScatterEditorOutputRow",
            status_text: "Scatter validation queued",
            output_text: "Validation queued   18 rules   1 conflict",
        },
        "workbench.extension.scatter_editor.slope_filter_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionScatterEditorOutputRow",
                status_text: "Scatter slope filter selected",
                output_text: "Selected Slope Filter   0-38 deg",
            }
        }
        "workbench.extension.scatter_editor.collision_test_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionScatterEditorOutputRow",
                status_text: "Scatter conflict selected",
                output_text: "Selected Collision Test   1 conflict",
            }
        }
        "workbench.extension.volume_editor.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionVolumeEditorOutputRow",
            status_text: "Volume editor opened",
            output_text: "Native extension workspace opened for VOL_DamageZone",
        },
        "workbench.extension.volume_editor.inspect.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionVolumeEditorOutputRow",
            status_text: "Volume overlap inspection queued",
            output_text: "Inspect queued   VOL_DamageZone   12 overlaps",
        },
        "workbench.extension.volume_editor.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionVolumeEditorOutputRow",
            status_text: "Volume validation queued",
            output_text: "Validation queued   24 volumes   1 warning",
        },
        "workbench.extension.volume_editor.player_overlap_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionVolumeEditorOutputRow",
                status_text: "Volume overlap selected",
                output_text: "Selected Player overlap   Pawn capsule ready",
            }
        }
        "workbench.extension.volume_editor.on_enter_event_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionVolumeEditorOutputRow",
                status_text: "Volume event selected",
                output_text: "Selected OnEnter event   Generate overlap warning",
            }
        }
        "workbench.extension.weather_editor.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionWeatherEditorOutputRow",
            status_text: "Weather editor opened",
            output_text: "Native extension workspace opened for Weather_Storm",
        },
        "workbench.extension.weather_editor.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionWeatherEditorOutputRow",
            status_text: "Weather preview queued",
            output_text: "Preview queued   Storm   Mountains region",
        },
        "workbench.extension.weather_editor.build.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionWeatherEditorOutputRow",
            status_text: "Weather build queued",
            output_text: "Build queued   8 layers   2 warnings",
        },
        "workbench.extension.weather_editor.rain_burst_timeline_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionWeatherEditorOutputRow",
                status_text: "Weather rain burst selected",
                output_text: "Selected Rain Burst   02:00-04:00   active",
            }
        }
        "workbench.extension.weather_editor.lightning_timeline_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionWeatherEditorOutputRow",
                status_text: "Weather lightning selected",
                output_text: "Selected Lightning   04:00-04:30   warning",
            }
        }
        "workbench.extension.particle_library.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionParticleLibraryOutputRow",
            status_text: "Particle library opened",
            output_text: "Native extension workspace opened for P_Sparks",
        },
        "workbench.extension.particle_library.simulate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionParticleLibraryOutputRow",
            status_text: "Particle simulation queued",
            output_text: "Simulation queued   P_Sparks   60 fps preview",
        },
        "workbench.extension.particle_library.compile.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionParticleLibraryOutputRow",
            status_text: "Particle compile queued",
            output_text: "Compile queued   4 emitters   bounds fixed",
        },
        "workbench.extension.particle_library.gpu_spark_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionParticleLibraryOutputRow",
            status_text: "Particle GPU spark selected",
            output_text: "Selected GPU Spark Burst   GPU   60 fps",
        },
        "workbench.extension.particle_library.archived_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionParticleLibraryOutputRow",
            status_text: "Particle warning row selected",
            output_text: "Selected Archived Smoke   Archived   Warning",
        },
        "workbench.extension.ui_asset_editor.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionUiAssetEditorOutputRow",
            status_text: "UI asset editor opened",
            output_text: "Native extension workspace opened for WBP_Inventory",
        },
        "workbench.extension.ui_asset_editor.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionUiAssetEditorOutputRow",
            status_text: "UI asset preview queued",
            output_text: "Preview queued   WBP_Inventory   desktop breakpoint",
        },
        "workbench.extension.ui_asset_editor.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionUiAssetEditorOutputRow",
            status_text: "UI asset validation queued",
            output_text: "Validation queued   42 widgets   3 issues",
        },
        "workbench.extension.ui_asset_editor.selected_button_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionUiAssetEditorOutputRow",
                status_text: "UI asset widget selected",
                output_text: "Selected Button_Equip   hover state   bound",
            }
        }
        "workbench.extension.ui_asset_editor.binding_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionUiAssetEditorOutputRow",
            status_text: "UI asset binding selected",
            output_text: "Selected Inventory.SelectedItem   binding OK",
        },
        "workbench.extension.ui_binding.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionUiBindingOutputRow",
            status_text: "UI binding opened",
            output_text: "Native extension workspace opened for Health.Value",
        },
        "workbench.extension.ui_binding.preview.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionUiBindingOutputRow",
            status_text: "UI binding preview queued",
            output_text: "Preview queued   Health.Value   WBP_HealthBar",
        },
        "workbench.extension.ui_binding.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionUiBindingOutputRow",
            status_text: "UI binding validation queued",
            output_text: "Validation queued   18 bindings   2 invalid",
        },
        "workbench.extension.ui_binding.health_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionUiBindingOutputRow",
            status_text: "UI binding field selected",
            output_text: "Selected Health.Value   Field   valid",
        },
        "workbench.extension.ui_binding.validation_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionUiBindingOutputRow",
            status_text: "UI binding warning selected",
            output_text: "Selected Validation   2 invalid   Warning",
        },
        "workbench.extension.icon_library.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionIconLibraryOutputRow",
            status_text: "Icon library opened",
            output_text: "Native extension workspace opened for icon-warning",
        },
        "workbench.extension.icon_library.find_usage.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionIconLibraryOutputRow",
            status_text: "Icon usage search queued",
            output_text: "Usage search queued   icon-warning   14 references",
        },
        "workbench.extension.icon_library.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionIconLibraryOutputRow",
            status_text: "Icon validation queued",
            output_text: "Validation queued   312 icons   4 missing",
        },
        "workbench.extension.icon_library.warning_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionIconLibraryOutputRow",
            status_text: "Icon row selected",
            output_text: "Selected icon-warning   System   14 refs",
        },
        "workbench.extension.icon_library.archived_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionIconLibraryOutputRow",
            status_text: "Icon warning row selected",
            output_text: "Selected icon-archive   Archived   Warning",
        },
        "workbench.extension.accessibility_audit.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionAccessibilityAuditOutputRow",
            status_text: "Accessibility audit opened",
            output_text: "Native extension workspace opened for Gameplay_HUD",
        },
        "workbench.extension.accessibility_audit.audit_screen.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionAccessibilityAuditOutputRow",
            status_text: "Accessibility audit queued",
            output_text: "Audit queued   Gameplay_HUD   9 issues",
        },
        "workbench.extension.accessibility_audit.preview_fix.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionAccessibilityAuditOutputRow",
            status_text: "Accessibility fix preview queued",
            output_text: "Preview queued   Contrast   AmmoText token",
        },
        "workbench.extension.accessibility_audit.contrast_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionAccessibilityAuditOutputRow",
                status_text: "Accessibility issue selected",
                output_text: "Selected Contrast   AmmoText   High",
            }
        }
        "workbench.extension.accessibility_audit.focus_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionAccessibilityAuditOutputRow",
                status_text: "Accessibility focus issue selected",
                output_text: "Selected Focus Order   InventoryGrid   Medium",
            }
        }
        "workbench.extension.menu_flow.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMenuFlowOutputRow",
            status_text: "Menu flow opened",
            output_text: "Native extension workspace opened for Screen_Start",
        },
        "workbench.extension.menu_flow.preview_flow.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMenuFlowOutputRow",
            status_text: "Menu flow preview queued",
            output_text: "Preview queued   Screen_Start   desktop breakpoint",
        },
        "workbench.extension.menu_flow.validate_focus.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMenuFlowOutputRow",
            status_text: "Menu focus validation queued",
            output_text: "Validation queued   64 focus rules   2 issues",
        },
        "workbench.extension.menu_flow.start_node_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMenuFlowOutputRow",
            status_text: "Menu flow node selected",
            output_text: "Selected Start   Screen   12,34",
        },
        "workbench.extension.menu_flow.exit_route_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionMenuFlowOutputRow",
            status_text: "Menu flow route selected",
            output_text: "Selected Exit   escape/back   warning",
        },
        "workbench.extension.font_atlas.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionFontAtlasOutputRow",
            status_text: "Font atlas opened",
            output_text: "Native extension workspace opened for Inter UI",
        },
        "workbench.extension.font_atlas.bake_atlas.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionFontAtlasOutputRow",
            status_text: "Font atlas bake queued",
            output_text: "Bake queued   4096 glyphs   4 pages",
        },
        "workbench.extension.font_atlas.inspect_glyph.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionFontAtlasOutputRow",
            status_text: "Font glyph inspection queued",
            output_text: "Inspect queued   Latin range   Page 0",
        },
        "workbench.extension.font_atlas.latin_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionFontAtlasOutputRow",
            status_text: "Font atlas range selected",
            output_text: "Selected Latin   512 glyphs   Page 0",
        },
        "workbench.extension.font_atlas.cjk_table_row.select" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionFontAtlasOutputRow",
            status_text: "Font atlas missing glyphs selected",
            output_text: "Selected CJK   12 missing   Page 2",
        },
        "workbench.extension.console_diagnostics.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionConsoleDiagnosticsOutputRow",
            status_text: "Console diagnostics opened",
            output_text: "Native extension workspace opened for Session_12_10",
        },
        "workbench.extension.console_diagnostics.filter_console.invoke" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionConsoleDiagnosticsOutputRow",
                status_text: "Console filter applied",
                output_text: "Filtered Renderer   Warnings+   texture|shader",
            }
        }
        "workbench.extension.console_diagnostics.clear_console.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionConsoleDiagnosticsOutputRow",
            status_text: "Console clear preview queued",
            output_text: "Clear queued   current filtered console buffer",
        },
        "workbench.extension.console_diagnostics.warning_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionConsoleDiagnosticsOutputRow",
                status_text: "Console warning selected",
                output_text: "Selected Renderer   Warning   Missing transient view",
            }
        }
        "workbench.extension.console_diagnostics.error_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionConsoleDiagnosticsOutputRow",
                status_text: "Console error selected",
                output_text: "Selected Runtime   Error   Null object path",
            }
        }
        _ => return None,
    };
    Some(feedback)
}
