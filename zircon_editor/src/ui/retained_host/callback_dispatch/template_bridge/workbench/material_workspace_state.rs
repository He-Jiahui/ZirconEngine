use zircon_runtime_interface::ui::component::UiValue;

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const MATERIAL_PARAMETER_ROWS: &[&str] = &[
    "WorkbenchMaterialBaseColorRow",
    "WorkbenchMaterialRoughnessRow",
    "WorkbenchMaterialNormalRow",
];
const MATERIAL_GRAPH_ROWS: &[&str] = &[
    "WorkbenchMaterialNodeRow01",
    "WorkbenchMaterialNodeRow02",
    "WorkbenchMaterialNodeRow03",
];
const MATERIAL_DOMAIN_DROPDOWN: &str = "WorkbenchMaterialDomainDropdown";
const MATERIAL_BLEND_DROPDOWN: &str = "WorkbenchMaterialBlendDropdown";
static MATERIAL_PROFILES: &[MaterialSelectionProfile] = &[
    MaterialSelectionProfile {
        action_ids: &[
            "workbench.module.material.base_color_row.select",
            "workbench.module.material.node_albedo.select",
        ],
        parameter_control_id: "WorkbenchMaterialBaseColorRow",
        graph_control_id: "WorkbenchMaterialNodeRow01",
        label: "Base Color",
        node_summary: "Texture Sample T_Rock_A / RGBA",
    },
    MaterialSelectionProfile {
        action_ids: &[
            "workbench.module.material.roughness_row.select",
            "workbench.module.material.node_roughness.select",
        ],
        parameter_control_id: "WorkbenchMaterialRoughnessRow",
        graph_control_id: "WorkbenchMaterialNodeRow02",
        label: "Roughness",
        node_summary: "Multiply / 0.72 float",
    },
    MaterialSelectionProfile {
        action_ids: &[
            "workbench.module.material.normal_row.select",
            "workbench.module.material.node_normal.select",
        ],
        parameter_control_id: "WorkbenchMaterialNormalRow",
        graph_control_id: "WorkbenchMaterialNodeRow03",
        label: "Normal",
        node_summary: "Texture Sample T_Rock_N / tangent RGB",
    },
];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn initialize_material_workspace_state(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.project_material_profile(&MATERIAL_PROFILES[0])
    }

    pub(super) fn apply_material_workspace_action(
        &mut self,
        action_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if action_id == "workbench.module.material.compile.invoke" {
            self.apply_material_compile_feedback()?;
            return Ok(true);
        }
        let Some(profile) = MATERIAL_PROFILES
            .iter()
            .find(|profile| profile.action_ids.contains(&action_id))
        else {
            return Ok(false);
        };
        self.project_material_profile(profile)?;
        Ok(true)
    }

    fn project_material_profile(
        &mut self,
        profile: &MaterialSelectionProfile,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.select_exclusive_selected(MATERIAL_PARAMETER_ROWS, profile.parameter_control_id)?;
        self.select_exclusive_selected(MATERIAL_GRAPH_ROWS, profile.graph_control_id)?;
        self.set_material_string(
            "WorkbenchMaterialCenterTitle",
            "text",
            format!("M_Rock_Cliff / {}", profile.label),
        )?;
        self.set_material_string(
            "WorkbenchMaterialOutputRow",
            "text",
            format!("Selection: {}   {}", profile.label, profile.node_summary),
        )
    }

    fn apply_material_compile_feedback(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let profile = MATERIAL_PROFILES
            .iter()
            .find(|profile| self.control_bool(profile.graph_control_id, "selected"))
            .unwrap_or(&MATERIAL_PROFILES[0]);
        let domain = self.material_dropdown_label(MATERIAL_DOMAIN_DROPDOWN, "Surface");
        let blend = self.material_dropdown_label(MATERIAL_BLEND_DROPDOWN, "Opaque");
        let preview = self
            .control_string("WorkbenchMaterialPreviewField", "value")
            .unwrap_or_default();
        self.set_material_string("WorkbenchStatusReady", "text", "Material compile complete")?;
        self.set_material_string("WorkbenchStatusMessages", "text", "1 Message")?;
        self.set_material_string(
            "WorkbenchMaterialCenterTitle",
            "text",
            format!("M_Rock_Cliff / {preview}"),
        )?;
        self.set_material_string(
            "WorkbenchMaterialOutputRow",
            "text",
            format!(
                "{} compiled   {} / {}   2 warnings",
                profile.label, domain, blend
            ),
        )
    }

    fn material_dropdown_label(&self, control_id: &str, fallback: &str) -> String {
        self.control_string(control_id, "value_text")
            .filter(|label| !label.trim().is_empty())
            .unwrap_or_else(|| fallback.to_string())
    }

    fn set_material_string(
        &mut self,
        control_id: &str,
        property: &str,
        value: impl Into<String>,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.mutate_control_property(control_id, property, UiValue::String(value.into()))
    }
}

struct MaterialSelectionProfile {
    action_ids: &'static [&'static str],
    parameter_control_id: &'static str,
    graph_control_id: &'static str,
    label: &'static str,
    node_summary: &'static str,
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue, layout::UiSize};

    use super::*;

    #[test]
    fn parameter_graph_and_compile_share_one_material_profile() {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
                .expect("workbench bridge should build");

        assert!(bridge.control_bool("WorkbenchMaterialBaseColorRow", "selected"));
        assert!(bridge.control_bool("WorkbenchMaterialNodeRow01", "selected"));

        assert!(bridge
            .select_dropdown_option(MATERIAL_DOMAIN_DROPDOWN, "post_process")
            .expect("material domain should select"));
        assert!(bridge
            .select_dropdown_option(MATERIAL_BLEND_DROPDOWN, "masked")
            .expect("material blend should select"));
        bridge
            .mutate_control_property(
                "WorkbenchMaterialPreviewField",
                "value",
                UiValue::String("Plane".to_string()),
            )
            .expect("material preview should edit");

        bridge
            .dispatch_control_state("WorkbenchMaterialNodeRow02", UiEventKind::Click)
            .expect("roughness node should dispatch")
            .expect("roughness node should bind");
        assert!(bridge.control_bool("WorkbenchMaterialRoughnessRow", "selected"));
        assert!(bridge.control_bool("WorkbenchMaterialNodeRow02", "selected"));
        assert_eq!(
            Some("M_Rock_Cliff / Roughness".to_string()),
            bridge.control_string("WorkbenchMaterialCenterTitle", "text")
        );

        bridge
            .dispatch_control_state("WorkbenchMaterialCompileButton", UiEventKind::Click)
            .expect("material compile should dispatch")
            .expect("material compile should bind");
        assert_eq!(
            Some("Roughness compiled   Post Process / Masked   2 warnings".to_string()),
            bridge.control_string("WorkbenchMaterialOutputRow", "text")
        );
        assert_eq!(
            Some("M_Rock_Cliff / Plane".to_string()),
            bridge.control_string("WorkbenchMaterialCenterTitle", "text")
        );
    }
}
