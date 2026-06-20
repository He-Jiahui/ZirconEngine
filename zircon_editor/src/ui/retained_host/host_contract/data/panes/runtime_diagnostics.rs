use crate::ui::retained_host::primitives::{ModelRc, SharedString};
use zircon_runtime_interface::ui::surface::UiDebugOverlayPrimitiveKind;

use super::super::{FrameRect, TemplatePaneNodeData};

#[derive(Clone)]
pub(crate) struct UiDebugOverlayPrimitiveData {
    pub kind: UiDebugOverlayPrimitiveKind,
    pub node_id: SharedString,
    pub frame: FrameRect,
    pub label: SharedString,
    pub severity: SharedString,
}

impl Default for UiDebugOverlayPrimitiveData {
    fn default() -> Self {
        Self {
            kind: UiDebugOverlayPrimitiveKind::SelectedFrame,
            node_id: SharedString::default(),
            frame: FrameRect::default(),
            label: SharedString::default(),
            severity: SharedString::default(),
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct RuntimeDiagnosticsPaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
    pub overlay_primitives: ModelRc<UiDebugOverlayPrimitiveData>,
    pub preserve_payload_debug_reflector: bool,
}
