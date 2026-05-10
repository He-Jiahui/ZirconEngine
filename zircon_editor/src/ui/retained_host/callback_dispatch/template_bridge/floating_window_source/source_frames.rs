use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::layout::UiFrame;

const FLOATING_WINDOW_CENTER_BAND_CONTROL_ID: &str = "FloatingWindowCenterBandRoot";
const FLOATING_WINDOW_DOCUMENT_CONTROL_ID: &str = "FloatingWindowDocumentRoot";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct BuiltinFloatingWindowSourceFrames {
    pub document_frame: Option<UiFrame>,
    pub center_band_frame: Option<UiFrame>,
}

pub(super) fn source_frames_from_surface(surface: &UiSurface) -> BuiltinFloatingWindowSourceFrames {
    BuiltinFloatingWindowSourceFrames {
        document_frame: surface_control_frame(surface, FLOATING_WINDOW_DOCUMENT_CONTROL_ID),
        center_band_frame: surface_control_frame(surface, FLOATING_WINDOW_CENTER_BAND_CONTROL_ID),
    }
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
