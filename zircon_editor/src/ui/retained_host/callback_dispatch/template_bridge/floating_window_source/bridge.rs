use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::layout::UiSize;
#[cfg(test)]
use zircon_runtime_interface::ui::surface::UiSurfaceRebuildDebugStats;

use crate::ui::template_runtime::EditorUiHostRuntime;

use super::error::BuiltinFloatingWindowSourceTemplateBridgeError;
use super::source_frames::{source_frames_from_surface, BuiltinFloatingWindowSourceFrames};
use super::surface::{
    build_builtin_floating_window_source_surface, rebuild_builtin_floating_window_source_surface,
};

pub(crate) struct BuiltinFloatingWindowSourceTemplateBridge {
    surface: UiSurface,
}

impl BuiltinFloatingWindowSourceTemplateBridge {
    pub(crate) fn new(
        shell_size: UiSize,
    ) -> Result<Self, BuiltinFloatingWindowSourceTemplateBridgeError> {
        let mut runtime = EditorUiHostRuntime::default();
        runtime.load_builtin_host_templates()?;
        let surface = build_builtin_floating_window_source_surface(&runtime, shell_size)?;
        Ok(Self { surface })
    }

    pub(crate) fn recompute_layout(
        &mut self,
        shell_size: UiSize,
    ) -> Result<(), BuiltinFloatingWindowSourceTemplateBridgeError> {
        rebuild_builtin_floating_window_source_surface(&mut self.surface, shell_size)?;
        Ok(())
    }

    pub(crate) fn source_frames(&self) -> BuiltinFloatingWindowSourceFrames {
        source_frames_from_surface(&self.surface)
    }

    #[cfg(test)]
    pub(crate) fn debug_surface_node_ids(&self) -> Vec<u64> {
        self.surface
            .tree
            .nodes
            .keys()
            .map(|node_id| node_id.0)
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn debug_render_command_count(&self) -> usize {
        self.surface.render_extract.list.commands.len()
    }

    #[cfg(test)]
    pub(crate) fn debug_last_rebuild_report(&self) -> UiSurfaceRebuildDebugStats {
        self.surface.surface_frame().last_rebuild
    }
}
