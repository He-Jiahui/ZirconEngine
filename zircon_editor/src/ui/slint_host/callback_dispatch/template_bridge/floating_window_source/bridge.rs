use zircon_runtime::ui::{layout::UiSize, surface::UiSurface};

use crate::ui::template_runtime::EditorUiHostRuntime;

use super::error::BuiltinFloatingWindowSourceTemplateBridgeError;
use super::source_frames::{source_frames_from_surface, BuiltinFloatingWindowSourceFrames};
use super::surface::build_builtin_floating_window_source_surface;

pub(crate) struct BuiltinFloatingWindowSourceTemplateBridge {
    runtime: EditorUiHostRuntime,
    surface: UiSurface,
}

impl BuiltinFloatingWindowSourceTemplateBridge {
    pub(crate) fn new(
        shell_size: UiSize,
    ) -> Result<Self, BuiltinFloatingWindowSourceTemplateBridgeError> {
        let mut runtime = EditorUiHostRuntime::default();
        runtime.load_builtin_host_templates()?;
        let surface = build_builtin_floating_window_source_surface(&runtime, shell_size)?;
        Ok(Self { runtime, surface })
    }

    pub(crate) fn recompute_layout(
        &mut self,
        shell_size: UiSize,
    ) -> Result<(), BuiltinFloatingWindowSourceTemplateBridgeError> {
        self.surface = build_builtin_floating_window_source_surface(&self.runtime, shell_size)?;
        Ok(())
    }

    pub(crate) fn source_frames(&self) -> BuiltinFloatingWindowSourceFrames {
        source_frames_from_surface(&self.surface)
    }
}
