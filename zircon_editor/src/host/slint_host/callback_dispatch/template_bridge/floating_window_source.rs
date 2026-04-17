use thiserror::Error;
use zircon_ui::{UiFrame, UiSize, UiSurface};

use crate::host::slint_host::callback_dispatch::constants::BUILTIN_FLOATING_WINDOW_SOURCE_DOCUMENT_ID;
use crate::host::template_runtime::{EditorUiHostRuntime, EditorUiHostRuntimeError};

const FLOATING_WINDOW_CENTER_BAND_CONTROL_ID: &str = "FloatingWindowCenterBandRoot";
const FLOATING_WINDOW_DOCUMENT_CONTROL_ID: &str = "FloatingWindowDocumentRoot";

#[derive(Debug, Error)]
pub(crate) enum BuiltinFloatingWindowSourceTemplateBridgeError {
    #[error(transparent)]
    HostRuntime(#[from] EditorUiHostRuntimeError),
    #[error(transparent)]
    Layout(#[from] zircon_ui::UiTreeError),
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct BuiltinFloatingWindowSourceFrames {
    pub document_frame: Option<UiFrame>,
    pub center_band_frame: Option<UiFrame>,
}

pub(crate) struct BuiltinFloatingWindowSourceTemplateBridge {
    runtime: EditorUiHostRuntime,
    surface: UiSurface,
}

impl BuiltinFloatingWindowSourceTemplateBridge {
    pub(crate) fn new(
        shell_size: UiSize,
    ) -> Result<Self, BuiltinFloatingWindowSourceTemplateBridgeError> {
        let mut runtime = EditorUiHostRuntime::default();
        runtime.load_builtin_workbench_shell()?;
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
        BuiltinFloatingWindowSourceFrames {
            document_frame: surface_control_frame(
                &self.surface,
                FLOATING_WINDOW_DOCUMENT_CONTROL_ID,
            ),
            center_band_frame: surface_control_frame(
                &self.surface,
                FLOATING_WINDOW_CENTER_BAND_CONTROL_ID,
            ),
        }
    }
}

fn build_builtin_floating_window_source_surface(
    runtime: &EditorUiHostRuntime,
    shell_size: UiSize,
) -> Result<UiSurface, BuiltinFloatingWindowSourceTemplateBridgeError> {
    let mut surface = runtime.build_shared_surface(BUILTIN_FLOATING_WINDOW_SOURCE_DOCUMENT_ID)?;
    surface.compute_layout(shell_size)?;
    Ok(surface)
}

fn surface_control_frame(surface: &UiSurface, control_id: &str) -> Option<UiFrame> {
    surface.tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
            .filter(|candidate| *candidate == control_id)
            .map(|_| node.layout_cache.frame)
    })
}
