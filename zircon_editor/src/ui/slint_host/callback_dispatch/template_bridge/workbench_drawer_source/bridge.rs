use zircon_runtime::ui::{
    layout::{UiFrame, UiSize},
    surface::UiSurface,
};

use crate::ui::template_runtime::EditorUiHostRuntime;
use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
use crate::ui::workbench::model::WorkbenchViewModel;

use super::error::BuiltinHostDrawerSourceTemplateBridgeError;
use super::layout::{
    build_builtin_host_drawer_source_surface, default_drawer_layout_inputs,
    drawer_layout_inputs_from_workbench_model,
};
use super::source_frames::{source_frames_from_surface, BuiltinHostDrawerSourceFrames};

pub(crate) struct BuiltinHostDrawerSourceTemplateBridge {
    runtime: EditorUiHostRuntime,
    surface: UiSurface,
}

impl BuiltinHostDrawerSourceTemplateBridge {
    pub(crate) fn new(
        shell_size: UiSize,
    ) -> Result<Self, BuiltinHostDrawerSourceTemplateBridgeError> {
        let mut runtime = EditorUiHostRuntime::default();
        runtime.load_builtin_host_templates()?;
        let surface = build_builtin_host_drawer_source_surface(
            &runtime,
            shell_size,
            default_drawer_layout_inputs(),
            WorkbenchChromeMetrics::default(),
        )?;
        Ok(Self { runtime, surface })
    }

    #[cfg(test)]
    pub(crate) fn recompute_layout(
        &mut self,
        shell_size: UiSize,
    ) -> Result<(), BuiltinHostDrawerSourceTemplateBridgeError> {
        self.surface = build_builtin_host_drawer_source_surface(
            &self.runtime,
            shell_size,
            default_drawer_layout_inputs(),
            WorkbenchChromeMetrics::default(),
        )?;
        Ok(())
    }

    pub(crate) fn recompute_layout_with_workbench_model(
        &mut self,
        shell_size: UiSize,
        model: &WorkbenchViewModel,
        metrics: &WorkbenchChromeMetrics,
    ) -> Result<(), BuiltinHostDrawerSourceTemplateBridgeError> {
        self.surface = build_builtin_host_drawer_source_surface(
            &self.runtime,
            shell_size,
            drawer_layout_inputs_from_workbench_model(model, metrics),
            *metrics,
        )?;
        Ok(())
    }

    pub(crate) fn control_frame(&self, control_id: &str) -> Option<UiFrame> {
        self.source_frames().control_frame(control_id)
    }

    pub(crate) fn source_frames(&self) -> BuiltinHostDrawerSourceFrames {
        source_frames_from_surface(&self.surface)
    }
}
