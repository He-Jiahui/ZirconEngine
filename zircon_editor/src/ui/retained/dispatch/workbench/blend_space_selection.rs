use zircon_runtime_interface::ui::component::UiValue;

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const ASSET_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionBlendSpaceIdleRunRow",
    "WorkbenchExtensionBlendSpaceStrafeRow",
    "WorkbenchExtensionBlendSpaceSprintRow",
];
const SAMPLE_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionBlendSpaceSampleRunForwardRow",
    "WorkbenchExtensionBlendSpaceSampleStrafeLeftRow",
    "WorkbenchExtensionBlendSpaceSampleStrafeRightRow",
    "WorkbenchExtensionBlendSpaceSampleIdleRow",
];
const WEIGHT_CONTROLS: &[(&str, &str)] = &[
    (
        "WorkbenchSampleWeightsRunForward",
        "WorkbenchSampleWeightsRunForwardValue",
    ),
    (
        "WorkbenchSampleWeightsRunLeft",
        "WorkbenchSampleWeightsRunLeftValue",
    ),
    (
        "WorkbenchSampleWeightsRunRight",
        "WorkbenchSampleWeightsRunRightValue",
    ),
    (
        "WorkbenchSampleWeightsIdle",
        "WorkbenchSampleWeightsIdleValue",
    ),
];
const SAMPLE_PROFILES: &[BlendSpaceSampleProfile] = &[
    BlendSpaceSampleProfile {
        action_id: "workbench.extension.blend_space.run_sample_table_row.select",
        row_control_id: "WorkbenchExtensionBlendSpaceSampleRunForwardRow",
        sample_name: "Run_Fwd",
        direction: "0.0",
        speed: "600.0",
        sample_position: "0, 600",
        rate_scale: "1.00",
        weights: [1.0, 0.0, 0.0, 0.0],
    },
    BlendSpaceSampleProfile {
        action_id: "workbench.extension.blend_space.walk_sample_table_row.select",
        row_control_id: "WorkbenchExtensionBlendSpaceSampleStrafeLeftRow",
        sample_name: "Strafe_L",
        direction: "-90.0",
        speed: "240.0",
        sample_position: "-90, 240",
        rate_scale: "0.34",
        weights: [0.0, 1.0, 0.0, 0.0],
    },
    BlendSpaceSampleProfile {
        action_id: "workbench.extension.blend_space.diagonal_sample_table_row.select",
        row_control_id: "WorkbenchExtensionBlendSpaceSampleStrafeRightRow",
        sample_name: "Strafe_R",
        direction: "90.0",
        speed: "240.0",
        sample_position: "90, 240",
        rate_scale: "0.32",
        weights: [0.0, 0.0, 1.0, 0.0],
    },
    BlendSpaceSampleProfile {
        action_id: "workbench.extension.blend_space.idle_sample_table_row.select",
        row_control_id: "WorkbenchExtensionBlendSpaceSampleIdleRow",
        sample_name: "Idle",
        direction: "0.0",
        speed: "0.0",
        sample_position: "0, 0",
        rate_scale: "0.18",
        weights: [0.0, 0.0, 0.0, 1.0],
    },
];

const ASSET_PROFILES: &[BlendSpaceAssetProfile] = &[
    BlendSpaceAssetProfile {
        action_id: "workbench.extension.blend_space.idle_run_row.select",
        row_control_id: "WorkbenchExtensionBlendSpaceIdleRunRow",
        asset_name: "BS_Idle_Run",
        default_sample_name: "Idle",
    },
    BlendSpaceAssetProfile {
        action_id: "workbench.extension.blend_space.strafe_row.select",
        row_control_id: "WorkbenchExtensionBlendSpaceStrafeRow",
        asset_name: "BS_Strafe_Grid",
        default_sample_name: "Strafe_L",
    },
    BlendSpaceAssetProfile {
        action_id: "workbench.extension.blend_space.sprint_row.select",
        row_control_id: "WorkbenchExtensionBlendSpaceSprintRow",
        asset_name: "BS_Sprint_Lean",
        default_sample_name: "Run_Fwd",
    },
];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    // 资产选中把单一资料投影到图表和详情，同时保留独立的播放进度状态。
    pub(super) fn apply_blend_space_asset_selection_action(
        &mut self,
        action_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        let profile = if let Some(profile) = ASSET_PROFILES
            .iter()
            .find(|profile| profile.action_id == action_id)
        {
            profile
        } else if matches!(
            action_id,
            "workbench.extension.blend_space.asset.edit"
                | "workbench.extension.blend_space.asset.commit"
        ) {
            let selected_asset = self
                .control_string("WorkbenchExtensionBlendSpaceAssetDropdown", "value")
                .unwrap_or_default();
            let Some(profile) = ASSET_PROFILES
                .iter()
                .find(|profile| profile.asset_name == selected_asset)
            else {
                return Ok(false);
            };
            profile
        } else {
            return Ok(false);
        };
        self.project_blend_space_asset_selection(profile)?;
        Ok(true)
    }

    pub(super) fn select_blend_space_asset_control(
        &mut self,
        control_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        let Some(profile) = ASSET_PROFILES
            .iter()
            .find(|profile| profile.row_control_id == control_id)
        else {
            return Ok(false);
        };
        self.project_blend_space_asset_selection(profile)?;
        Ok(true)
    }

    // 样本选择仅更新局部编辑上下文；后续命令反馈仍以当前资产和样本为目标。
    pub(super) fn apply_blend_space_sample_selection_action(
        &mut self,
        action_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        let Some(profile) = SAMPLE_PROFILES
            .iter()
            .find(|profile| profile.action_id == action_id)
        else {
            return Ok(false);
        };
        self.project_blend_space_sample_selection(profile)?;
        self.write_blend_space_feedback(
            "Blend space sample selected",
            format!(
                "Selected {}   Speed {}   Direction {}",
                profile.sample_name, profile.speed, profile.direction
            ),
        )?;
        Ok(true)
    }

    pub(super) fn apply_blend_space_contextual_command_feedback(
        &mut self,
        action_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        let asset_name = self
            .control_string("WorkbenchExtensionBlendSpaceAssetDropdown", "value")
            .unwrap_or_else(|| "BS_Locomotion".to_string());
        let (status_text, output_text) = match action_id {
            "workbench.extension.blend_space.preview.invoke" => {
                let sample_name = self
                    .control_string("WorkbenchExtensionBlendSpacePreviewAsset", "value")
                    .unwrap_or_else(|| "Selected sample".to_string());
                (
                    "Blend space preview queued",
                    format!("Preview queued   {asset_name}   {sample_name}"),
                )
            }
            "workbench.extension.blend_space.apply.invoke" => {
                let interpolation = self
                    .control_string("WorkbenchExtensionBlendSpaceInterpolationDropdown", "value")
                    .unwrap_or_else(|| "Triangulated".to_string());
                (
                    "Blend space apply queued",
                    format!("Apply queued   {asset_name}   8 samples   {interpolation}"),
                )
            }
            _ => return Ok(false),
        };
        self.write_blend_space_feedback(status_text, output_text)?;
        Ok(true)
    }

    fn project_blend_space_asset_selection(
        &mut self,
        profile: &BlendSpaceAssetProfile,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.select_exclusive_selected(ASSET_ROW_CONTROLS, profile.row_control_id)?;
        self.set_blend_space_string(
            "WorkbenchExtensionBlendSpaceAssetSummary",
            "text",
            format!("{}  |  8 samples", profile.asset_name),
        )?;
        for property in ["text", "value"] {
            self.set_blend_space_string(
                "WorkbenchExtensionBlendSpaceAssetDropdown",
                property,
                profile.asset_name,
            )?;
        }
        let sample_profile = SAMPLE_PROFILES
            .iter()
            .find(|sample| sample.sample_name == profile.default_sample_name)
            .expect("Blend Space asset profiles must reference a sample profile");
        self.project_blend_space_sample_selection(sample_profile)
    }

    fn project_blend_space_sample_selection(
        &mut self,
        profile: &BlendSpaceSampleProfile,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let status_suffix = self
            .control_string("WorkbenchExtensionBlendSpacePreviewStatus", "text")
            .and_then(|status| {
                status
                    .split_once("  |  ")
                    .map(|(_, suffix)| suffix.to_string())
            })
            .filter(|suffix| !suffix.is_empty())
            .unwrap_or_else(|| "Previewing".to_string());

        self.select_exclusive_selected(SAMPLE_ROW_CONTROLS, profile.row_control_id)?;
        self.set_blend_space_string(
            "WorkbenchExtensionBlendSpacePreviewAsset",
            "value",
            profile.sample_name,
        )?;
        self.set_blend_space_string(
            "WorkbenchExtensionBlendSpacePreviewStatus",
            "text",
            format!("{}  |  {status_suffix}", profile.sample_name),
        )?;
        self.set_blend_space_string(
            "WorkbenchExtensionBlendSpaceSamplePositionProperty",
            "value",
            profile.sample_position,
        )?;
        self.set_blend_space_string(
            "WorkbenchExtensionBlendSpaceSampleRateProperty",
            "value",
            profile.rate_scale,
        )?;
        self.set_blend_space_string(
            "WorkbenchExtensionBlendSpacePreviewTimeline",
            "track_label",
            profile.sample_name,
        )?;
        self.set_blend_space_string(
            "WorkbenchSampleWeightsDirectionValue",
            "text",
            profile.direction,
        )?;
        self.set_blend_space_string("WorkbenchSampleWeightsSpeedValue", "text", profile.speed)?;

        for ((progress_control, value_control), weight) in
            WEIGHT_CONTROLS.iter().zip(profile.weights)
        {
            self.mutate_control_property(
                progress_control,
                "value",
                UiValue::Float(weight * 100.0),
            )?;
            self.mutate_control_property(
                progress_control,
                "value_percent",
                UiValue::Float(weight),
            )?;
            self.set_blend_space_string(value_control, "text", format!("{weight:.2}"))?;
        }
        Ok(())
    }

    fn set_blend_space_string(
        &mut self,
        control_id: &str,
        property: &str,
        value: impl Into<String>,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.mutate_control_property(control_id, property, UiValue::String(value.into()))
    }

    fn write_blend_space_feedback(
        &mut self,
        status_text: impl Into<String>,
        output_text: impl Into<String>,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.set_blend_space_string("WorkbenchStatusReady", "text", status_text)?;
        self.set_blend_space_string("WorkbenchStatusMessages", "text", "1 Message")?;
        self.set_blend_space_string(
            "WorkbenchExtensionBlendSpaceOutputRow",
            "value_text",
            output_text,
        )
    }
}

struct BlendSpaceAssetProfile {
    action_id: &'static str,
    row_control_id: &'static str,
    asset_name: &'static str,
    default_sample_name: &'static str,
}

struct BlendSpaceSampleProfile {
    action_id: &'static str,
    row_control_id: &'static str,
    sample_name: &'static str,
    direction: &'static str,
    speed: &'static str,
    sample_position: &'static str,
    rate_scale: &'static str,
    weights: [f64; 4],
}

#[cfg(test)]
#[path = "tests/blend_space_selection.rs"]
mod tests;
