use zircon_runtime_interface::ui::component::UiValue;

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const RENDER_PASS_ROWS: &[&str] = &[
    "WorkbenchRenderLightingPassRow",
    "WorkbenchRenderBloomPassRow",
];
const RENDER_GRAPH_ROWS: &[&str] = &[
    "WorkbenchRenderFrameStartRow",
    "WorkbenchRenderLightingNodeRow",
    "WorkbenchRenderSceneColorRow",
];
const RENDER_PLATFORM_DROPDOWN: &str = "WorkbenchRenderPlatformDropdown";
static RENDER_PASS_PROFILES: &[RenderPassProfile] = &[
    RenderPassProfile {
        action_id: "workbench.module.render.lighting_pass.select",
        row_control_id: "WorkbenchRenderLightingPassRow",
        label: "Lighting Pass",
        gpu_time: "1.84 ms",
    },
    RenderPassProfile {
        action_id: "workbench.module.render.bloom_pass.select",
        row_control_id: "WorkbenchRenderBloomPassRow",
        label: "Bloom Pass",
        gpu_time: "0.82 ms",
    },
];
static RENDER_GRAPH_PROFILES: &[RenderGraphProfile] = &[
    RenderGraphProfile {
        action_id: "workbench.module.render.frame_start.select",
        row_control_id: "WorkbenchRenderFrameStartRow",
        label: "Frame Start",
        summary: "Frame 1234 captured   0.000 ms",
    },
    RenderGraphProfile {
        action_id: "workbench.module.render.lighting_node.select",
        row_control_id: "WorkbenchRenderLightingNodeRow",
        label: "Lighting",
        summary: "SceneColor -> BloomInput   1.84 ms",
    },
    RenderGraphProfile {
        action_id: "workbench.module.render.scene_color.select",
        row_control_id: "WorkbenchRenderSceneColorRow",
        label: "SceneColor",
        summary: "R11G11B10_FLOAT   Read",
    },
];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn initialize_render_workspace_state(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.set_render_string(RENDER_PLATFORM_DROPDOWN, "value", "windows_dx12")?;
        self.set_render_string(RENDER_PLATFORM_DROPDOWN, "value_text", "Windows DX12")?;
        self.project_render_pass(&RENDER_PASS_PROFILES[0])?;
        self.project_render_graph(&RENDER_GRAPH_PROFILES[0])
    }

    pub(super) fn apply_render_workspace_action(
        &mut self,
        action_id: &str,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        if action_id == "workbench.module.render.compile.invoke" {
            self.apply_render_compile_feedback()?;
            return Ok(true);
        }
        if let Some(profile) = RENDER_PASS_PROFILES
            .iter()
            .find(|profile| profile.action_id == action_id)
        {
            self.project_render_pass(profile)?;
            return Ok(true);
        }
        if let Some(profile) = RENDER_GRAPH_PROFILES
            .iter()
            .find(|profile| profile.action_id == action_id)
        {
            self.project_render_graph(profile)?;
            return Ok(true);
        }
        Ok(false)
    }

    fn project_render_pass(
        &mut self,
        profile: &RenderPassProfile,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.select_exclusive_selected(RENDER_PASS_ROWS, profile.row_control_id)?;
        let platform = self.render_platform_label();
        self.set_render_string(
            "WorkbenchRenderCenterTitle",
            "text",
            format!("Render Graph / {}", profile.label),
        )?;
        self.set_render_string(
            "WorkbenchRenderCaptureRow",
            "value_text",
            format!(
                "Pass: {} selected   {}   GPU {}",
                profile.label, platform, profile.gpu_time
            ),
        )
    }

    fn project_render_graph(
        &mut self,
        profile: &RenderGraphProfile,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.select_exclusive_selected(RENDER_GRAPH_ROWS, profile.row_control_id)?;
        self.set_render_string(
            "WorkbenchRenderCaptureRow",
            "value_text",
            format!("Selection: {}   {}", profile.label, profile.summary),
        )
    }

    fn apply_render_compile_feedback(
        &mut self,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let pass = RENDER_PASS_PROFILES
            .iter()
            .find(|profile| self.control_bool(profile.row_control_id, "selected"))
            .unwrap_or(&RENDER_PASS_PROFILES[0]);
        let graph = RENDER_GRAPH_PROFILES
            .iter()
            .find(|profile| self.control_bool(profile.row_control_id, "selected"))
            .unwrap_or(&RENDER_GRAPH_PROFILES[0]);
        let platform = self.render_platform_label();
        let pipeline = self
            .control_string("WorkbenchRenderPipelineField", "value")
            .unwrap_or_default();
        let frame = self
            .control_string("WorkbenchRenderFrameField", "value")
            .unwrap_or_default();
        self.set_render_string("WorkbenchStatusReady", "text", "Render graph compiled")?;
        self.set_render_string("WorkbenchStatusMessages", "text", "1 Message")?;
        self.set_render_string(
            "WorkbenchRenderCenterTitle",
            "text",
            format!("{} / {}", pipeline, pass.label),
        )?;
        self.set_render_string(
            "WorkbenchRenderCaptureRow",
            "value_text",
            format!("{}   frame {}   {} compiled", platform, frame, graph.label),
        )
    }

    fn render_platform_label(&self) -> String {
        self.control_string(RENDER_PLATFORM_DROPDOWN, "value_text")
            .filter(|label| !label.trim().is_empty())
            .unwrap_or_else(|| "Windows DX12".to_string())
    }

    fn set_render_string(
        &mut self,
        control_id: &str,
        property: &str,
        value: impl Into<String>,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.mutate_control_property(control_id, property, UiValue::String(value.into()))
    }
}

struct RenderPassProfile {
    action_id: &'static str,
    row_control_id: &'static str,
    label: &'static str,
    gpu_time: &'static str,
}

struct RenderGraphProfile {
    action_id: &'static str,
    row_control_id: &'static str,
    label: &'static str,
    summary: &'static str,
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue, layout::UiSize};

    use super::*;

    #[test]
    fn pass_graph_and_compile_keep_distinct_state_domains() {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
                .expect("workbench bridge should build");

        assert!(bridge.control_bool("WorkbenchRenderLightingPassRow", "selected"));
        assert!(bridge.control_bool("WorkbenchRenderFrameStartRow", "selected"));

        assert!(bridge
            .select_dropdown_option(RENDER_PLATFORM_DROPDOWN, "vulkan")
            .expect("render platform should select"));
        for (control_id, value) in [
            ("WorkbenchRenderPipelineField", "Cinematic.rp"),
            ("WorkbenchRenderFrameField", "2048"),
        ] {
            bridge
                .mutate_control_property(control_id, "value", UiValue::String(value.to_string()))
                .expect("render property should edit");
        }

        bridge
            .dispatch_control_state("WorkbenchRenderBloomPassRow", UiEventKind::Click)
            .expect("bloom pass should dispatch")
            .expect("bloom pass should bind");
        bridge
            .dispatch_control_state("WorkbenchRenderLightingNodeRow", UiEventKind::Click)
            .expect("lighting graph node should dispatch")
            .expect("lighting graph node should bind");
        assert!(bridge.control_bool("WorkbenchRenderBloomPassRow", "selected"));
        assert!(bridge.control_bool("WorkbenchRenderLightingNodeRow", "selected"));

        bridge
            .dispatch_control_state("WorkbenchRenderCompileButton", UiEventKind::Click)
            .expect("render compile should dispatch")
            .expect("render compile should bind");
        assert_eq!(
            Some("Vulkan   frame 2048   Lighting compiled".to_string()),
            bridge.control_string("WorkbenchRenderCaptureRow", "value_text")
        );
        assert_eq!(
            Some("Cinematic.rp / Bloom Pass".to_string()),
            bridge.control_string("WorkbenchRenderCenterTitle", "text")
        );
    }
}
