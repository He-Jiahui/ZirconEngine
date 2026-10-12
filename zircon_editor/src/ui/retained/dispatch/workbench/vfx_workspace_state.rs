use zircon_runtime_interface::ui::component::UiValue;

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const VFX_CONTEXT_ROWS: &[&str] = &["WorkbenchVfxEmitterRow", "WorkbenchVfxCurveRow"];
const VFX_PARAMETER_ROWS: &[&str] = &[
    "WorkbenchVfxSpawnRow",
    "WorkbenchVfxLifetimeRow",
    "WorkbenchVfxMaterialRow",
];
static VFX_CONTEXT_PROFILES: &[VfxContextProfile] = &[
    VfxContextProfile {
        action_id: "workbench.module.vfx.emitter_row.select",
        row_control_id: "WorkbenchVfxEmitterRow",
        label: "Emitter: Sparks",
    },
    VfxContextProfile {
        action_id: "workbench.module.vfx.curve_row.select",
        row_control_id: "WorkbenchVfxCurveRow",
        label: "Curve: Spawn Rate",
    },
];
static VFX_PARAMETER_PROFILES: &[VfxParameterProfile] = &[
    VfxParameterProfile {
        action_id: "workbench.module.vfx.spawn_row.select",
        row_control_id: "WorkbenchVfxSpawnRow",
        label: "Spawn Rate",
        value: "280 / sec",
    },
    VfxParameterProfile {
        action_id: "workbench.module.vfx.lifetime_row.select",
        row_control_id: "WorkbenchVfxLifetimeRow",
        label: "Lifetime",
        value: "0.65 s",
    },
    VfxParameterProfile {
        action_id: "workbench.module.vfx.material_row.select",
        row_control_id: "WorkbenchVfxMaterialRow",
        label: "Material",
        value: "M_Bolt_01",
    },
];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn initialize_vfx_workspace_state(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.project_vfx_context(&VFX_CONTEXT_PROFILES[0])?;
        self.project_vfx_parameter(&VFX_PARAMETER_PROFILES[0])
    }

    pub(super) fn apply_vfx_workspace_action(
        &mut self,
        action_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if action_id == "workbench.module.vfx.simulate.invoke" {
            self.apply_vfx_simulation_feedback()?;
            return Ok(true);
        }
        if let Some(profile) = VFX_CONTEXT_PROFILES
            .iter()
            .find(|profile| profile.action_id == action_id)
        {
            self.project_vfx_context(profile)?;
            return Ok(true);
        }
        if let Some(profile) = VFX_PARAMETER_PROFILES
            .iter()
            .find(|profile| profile.action_id == action_id)
        {
            self.project_vfx_parameter(profile)?;
            return Ok(true);
        }
        Ok(false)
    }

    fn project_vfx_context(
        &mut self,
        profile: &VfxContextProfile,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.select_exclusive_selected(VFX_CONTEXT_ROWS, profile.row_control_id)?;
        self.set_vfx_string(
            "WorkbenchVfxCenterTitle",
            "text",
            format!("P_Bolt_01 / {}", profile.label),
        )?;
        self.set_vfx_string(
            "WorkbenchVfxOutputRow",
            "text",
            format!("Context: {} selected", profile.label),
        )
    }

    fn project_vfx_parameter(
        &mut self,
        profile: &VfxParameterProfile,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.select_exclusive_selected(VFX_PARAMETER_ROWS, profile.row_control_id)?;
        self.set_vfx_string(
            "WorkbenchVfxOutputRow",
            "text",
            format!("Parameter: {}   {}", profile.label, profile.value),
        )
    }

    fn apply_vfx_simulation_feedback(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let context = VFX_CONTEXT_PROFILES
            .iter()
            .find(|profile| self.control_bool(profile.row_control_id, "selected"))
            .unwrap_or(&VFX_CONTEXT_PROFILES[0]);
        let parameter = VFX_PARAMETER_PROFILES
            .iter()
            .find(|profile| self.control_bool(profile.row_control_id, "selected"))
            .unwrap_or(&VFX_PARAMETER_PROFILES[0]);
        let system = self
            .control_string("WorkbenchVfxSystemField", "value")
            .unwrap_or_default();
        let bounds = self
            .control_string("WorkbenchVfxBoundsField", "value")
            .unwrap_or_default();
        let sort = self
            .control_string("WorkbenchVfxSortField", "value")
            .unwrap_or_default();
        self.set_vfx_string("WorkbenchStatusReady", "text", "VFX simulation running")?;
        self.set_vfx_string("WorkbenchStatusMessages", "text", "1 Message")?;
        self.set_vfx_string(
            "WorkbenchVfxCenterTitle",
            "text",
            format!("{} / {}", system, context.label),
        )?;
        self.set_vfx_string(
            "WorkbenchVfxOutputRow",
            "text",
            format!(
                "Simulation: {} / {}   {} {}   60 fps",
                bounds, sort, parameter.label, parameter.value
            ),
        )
    }

    fn set_vfx_string(
        &mut self,
        control_id: &str,
        property: &str,
        value: impl Into<String>,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.mutate_control_property(control_id, property, UiValue::String(value.into()))
    }
}

struct VfxContextProfile {
    action_id: &'static str,
    row_control_id: &'static str,
    label: &'static str,
}

struct VfxParameterProfile {
    action_id: &'static str,
    row_control_id: &'static str,
    label: &'static str,
    value: &'static str,
}

#[cfg(test)]
#[path = "tests/vfx_workspace_state.rs"]
mod tests;
